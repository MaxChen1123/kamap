use std::collections::HashMap;
use std::path::Path;

use anyhow::{Context, Result};
use git2::{DiffOptions, Repository};

use crate::models::{ChangeEntry, ChangeType, DiffResult, HunkRange};

/// Git Diff 分析器
pub struct DiffAnalyzer;

impl DiffAnalyzer {
    /// 分析两个 Git ref 之间的变更
    ///
    /// 当 `head` 为 `"workdir"` 时，自动对比 base 与工作区（含 staged + unstaged）。
    pub fn analyze(repo_path: &Path, base: &str, head: &str) -> Result<DiffResult> {
        if head == "workdir" {
            return Self::analyze_base_to_workdir(repo_path, base);
        }

        let repo = Repository::open(repo_path)
            .with_context(|| format!("Failed to open git repo at {}", repo_path.display()))?;

        let base_obj = repo
            .revparse_single(base)
            .with_context(|| format!("Failed to resolve ref '{}'", base))?;
        let head_obj = repo
            .revparse_single(head)
            .with_context(|| format!("Failed to resolve ref '{}'", head))?;

        // 获取实际的 commit hash
        let base_commit_hash = base_obj.id().to_string();
        let head_commit_hash = head_obj.id().to_string();

        let base_tree = base_obj
            .peel_to_tree()
            .with_context(|| format!("Failed to peel '{}' to tree", base))?;
        let head_tree = head_obj
            .peel_to_tree()
            .with_context(|| format!("Failed to peel '{}' to tree", head))?;

        let mut opts = DiffOptions::new();
        opts.context_lines(0);

        let diff = repo
            .diff_tree_to_tree(Some(&base_tree), Some(&head_tree), Some(&mut opts))
            .with_context(|| "Failed to compute diff")?;

        let mut result = parse_diff(&diff)?;
        result.base_ref = base_commit_hash;
        result.head_ref = head_commit_hash;
        Ok(result)
    }

    /// 分析指定 base ref 到工作区的变更（含 staged + unstaged）
    fn analyze_base_to_workdir(repo_path: &Path, base: &str) -> Result<DiffResult> {
        let repo = Repository::open(repo_path)
            .with_context(|| format!("Failed to open git repo at {}", repo_path.display()))?;

        let base_obj = repo
            .revparse_single(base)
            .with_context(|| format!("Failed to resolve ref '{}'", base))?;
        let base_commit_hash = base_obj.id().to_string();

        // 用当前 HEAD commit hash 作为 head_ref，确保 ack 能正确关联到 commit
        let head_commit_hash = repo
            .head()
            .and_then(|h| h.peel_to_commit().map(|c| c.id().to_string()))
            .unwrap_or_else(|_| "workdir".to_string());

        let base_tree = base_obj
            .peel_to_tree()
            .with_context(|| format!("Failed to peel '{}' to tree", base))?;

        let mut opts = DiffOptions::new();
        opts.context_lines(0);

        let diff = repo
            .diff_tree_to_workdir_with_index(Some(&base_tree), Some(&mut opts))
            .with_context(|| "Failed to compute workdir diff")?;

        let mut result = parse_diff(&diff)?;
        result.base_ref = base_commit_hash;
        result.head_ref = head_commit_hash;
        Ok(result)
    }

    /// 获取 base..workdir 的完整变更文件路径集合
    ///
    /// 合并两段 diff：base..HEAD（已提交）+ HEAD..workdir（未提交）
    /// 返回所有变更文件的路径集合，用于检测资产文件是否被修改。
    pub fn changed_files_full(repo_path: &Path, base: &str) -> Result<std::collections::HashSet<String>> {
        let mut all_paths = std::collections::HashSet::new();

        // 1. base..HEAD 的已提交变更
        if let Ok(committed) = Self::analyze(repo_path, base, "HEAD") {
            for change in &committed.changes {
                all_paths.insert(change.path.clone());
            }
        }

        // 2. HEAD..workdir 的未提交变更（staged + unstaged）
        if let Ok(workdir) = Self::analyze_workdir(repo_path) {
            for change in &workdir.changes {
                all_paths.insert(change.path.clone());
            }
        }

        Ok(all_paths)
    }

    /// 分析工作区未提交的变更（与 HEAD 比较）
    pub fn analyze_workdir(repo_path: &Path) -> Result<DiffResult> {
        let repo = Repository::open(repo_path)
            .with_context(|| format!("Failed to open git repo at {}", repo_path.display()))?;

        let head = repo.head().with_context(|| "Failed to get HEAD")?;
        let head_commit_hash = head
            .peel_to_commit()
            .map(|c| c.id().to_string())
            .unwrap_or_else(|_| "HEAD".to_string());
        let head_tree = head
            .peel_to_tree()
            .with_context(|| "Failed to peel HEAD to tree")?;

        let mut opts = DiffOptions::new();
        opts.context_lines(0);

        let diff = repo
            .diff_tree_to_workdir_with_index(Some(&head_tree), Some(&mut opts))
            .with_context(|| "Failed to compute workdir diff")?;

        let mut result = parse_diff(&diff)?;
        result.base_ref = head_commit_hash.clone();
        result.head_ref = head_commit_hash;
        Ok(result)
    }
}

/// 使用 diff.print 来同时收集文件和 hunk 信息，避免多重可变借用
fn parse_diff(diff: &git2::Diff) -> Result<DiffResult> {
    // 先用 deltas 收集文件信息
    let mut entries: Vec<ChangeEntry> = Vec::new();
    let mut path_index: HashMap<String, usize> = HashMap::new();

    for (i, delta) in diff.deltas().enumerate() {
        let new_path = delta
            .new_file()
            .path()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_default();
        let old_path = delta
            .old_file()
            .path()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_default();

        let change_type = match delta.status() {
            git2::Delta::Added => ChangeType::Added,
            git2::Delta::Deleted => ChangeType::Deleted,
            git2::Delta::Modified => ChangeType::Modified,
            git2::Delta::Renamed => ChangeType::Renamed {
                old_path: old_path.clone(),
            },
            _ => ChangeType::Modified,
        };

        let path = if matches!(delta.status(), git2::Delta::Deleted) {
            old_path
        } else {
            new_path
        };

        path_index.insert(path.clone(), i);
        entries.push(ChangeEntry {
            path,
            change_type,
            hunks: Vec::new(),
        });
    }

    // 使用 diff.print 收集 hunk 信息
    diff.print(git2::DiffFormat::Patch, |delta, hunk, _line| {
        if let Some(hunk) = hunk {
            let path = delta
                .new_file()
                .path()
                .or_else(|| delta.old_file().path())
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_default();

            if let Some(&idx) = path_index.get(&path) {
                let entry = &mut entries[idx];
                let hunk_range = HunkRange {
                    start_line: hunk.new_start(),
                    end_line: hunk.new_start() + hunk.new_lines().max(1) - 1,
                    additions: hunk.new_lines(),
                    deletions: hunk.old_lines(),
                };
                // 避免重复添加相同的 hunk
                if !entry.hunks.iter().any(|h| {
                    h.start_line == hunk_range.start_line && h.end_line == hunk_range.end_line
                }) {
                    entry.hunks.push(hunk_range);
                }
            }
        }
        true
    })
    .with_context(|| "Failed to print diff")?;

    Ok(DiffResult {
        base_ref: String::new(),
        head_ref: String::new(),
        changes: entries,
    })
}
