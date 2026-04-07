use std::path::Path;

use anyhow::Result;

use crate::config::ProjectConfig;
use crate::models::SourceLocator;

use super::{CandidateOrigin, DiscoveryOptions, DiscoveryStrategy, MappingCandidate};

/// 扫描代码中的 @kamap 注释
pub struct AnnotationScanner {
    marker: String,
}

impl AnnotationScanner {
    pub fn new(marker: &str) -> Self {
        Self {
            marker: marker.to_string(),
        }
    }

    fn scan_file(&self, file_path: &Path, rel_path: &str) -> Result<Vec<MappingCandidate>> {
        let content = std::fs::read_to_string(file_path)?;
        let mut candidates = Vec::new();

        for (line_num, line) in content.lines().enumerate() {
            let trimmed = line.trim();

            // 查找 @kamap 标记
            let marker_prefix = &self.marker;
            let search_patterns = [
                format!("// {}", marker_prefix),
                format!("# {}", marker_prefix),
                format!("/* {}", marker_prefix),
                format!("-- {}", marker_prefix),
            ];

            let annotation_content = search_patterns.iter().find_map(|p| {
                if trimmed.starts_with(p.as_str()) {
                    Some(trimmed[p.len()..].trim().to_string())
                } else {
                    None
                }
            });

            if let Some(content) = annotation_content {
                if let Some(candidate) = parse_annotation(&content, rel_path, line_num as u32 + 1) {
                    candidates.push(candidate);
                }
            }
        }

        Ok(candidates)
    }
}

impl DiscoveryStrategy for AnnotationScanner {
    fn name(&self) -> &str {
        "annotation"
    }

    fn discover(
        &self,
        workspace: &Path,
        _config: &ProjectConfig,
        _opts: &DiscoveryOptions,
    ) -> Result<Vec<MappingCandidate>> {
        let mut candidates = Vec::new();
        scan_dir_for_annotations(self, workspace, workspace, &mut candidates)?;
        Ok(candidates)
    }
}

fn scan_dir_for_annotations(
    scanner: &AnnotationScanner,
    dir: &Path,
    workspace: &Path,
    candidates: &mut Vec<MappingCandidate>,
) -> Result<()> {
    if !dir.is_dir() {
        return Ok(());
    }

    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();

        // 跳过隐藏目录和 target/node_modules
        if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
            if name.starts_with('.') || name == "target" || name == "node_modules" {
                continue;
            }
        }

        if path.is_dir() {
            scan_dir_for_annotations(scanner, &path, workspace, candidates)?;
        } else if is_code_file(&path) {
            let rel_path = path
                .strip_prefix(workspace)
                .unwrap_or(&path)
                .to_string_lossy()
                .to_string();
            if let Ok(mut file_candidates) = scanner.scan_file(&path, &rel_path) {
                candidates.append(&mut file_candidates);
            }
        }
    }

    Ok(())
}

fn is_code_file(path: &Path) -> bool {
    matches!(
        path.extension().and_then(|e| e.to_str()),
        Some("rs" | "ts" | "tsx" | "js" | "jsx" | "py" | "go" | "java" | "rb" | "c" | "cpp" | "h")
    )
}

/// 解析注释内容: asset:xxx segment:{...} reason:"..."
fn parse_annotation(content: &str, file_path: &str, line: u32) -> Option<MappingCandidate> {
    let mut asset_id = None;
    let mut reason = None;
    let mut segment = None;

    // 简单的 key:value 解析
    let mut remaining = content.to_string();

    // 提取 asset:xxx
    if let Some(pos) = remaining.find("asset:") {
        let after = &remaining[pos + 6..];
        let end = after.find(' ').unwrap_or(after.len());
        asset_id = Some(after[..end].to_string());
        remaining = format!("{}{}", &remaining[..pos], &remaining[pos + 6 + end..]);
    }

    // 提取 reason:"xxx"
    if let Some(pos) = remaining.find("reason:") {
        let after = &remaining[pos + 7..];
        if after.starts_with('"') {
            if let Some(end) = after[1..].find('"') {
                reason = Some(after[1..end + 1].to_string());
            }
        } else {
            let end = after.find(' ').unwrap_or(after.len());
            reason = Some(after[..end].to_string());
        }
    }

    // 提取 segment:{...}
    if let Some(pos) = remaining.find("segment:") {
        let after = &remaining[pos + 8..];
        if after.starts_with('{') {
            // 找到匹配的 }
            let mut depth = 0;
            let mut end = 0;
            for (i, ch) in after.chars().enumerate() {
                if ch == '{' {
                    depth += 1;
                } else if ch == '}' {
                    depth -= 1;
                    if depth == 0 {
                        end = i + 1;
                        break;
                    }
                }
            }
            if end > 0 {
                if let Ok(value) = serde_json::from_str(&after[..end]) {
                    segment = Some(value);
                }
            }
        }
    }

    let asset_id = asset_id?;

    Some(MappingCandidate {
        source: SourceLocator {
            path: file_path.to_string(),
            lines: Some([line, line + 10]), // 估算范围
        },
        asset_id,
        reason: reason.unwrap_or_else(|| format!("@kamap annotation at {}:{}", file_path, line)),
        confidence: 0.9,
        origin: CandidateOrigin::CodeAnnotation,
        segment,
    })
}
