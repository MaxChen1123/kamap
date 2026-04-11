use std::path::Path;

use anyhow::Result;

use crate::anchor::resolve_anchor;
use crate::config::ProjectConfig;
use crate::models::{ChangeEntry, HitType, HunkRange, MappingHit, SourceMatch};

use super::index::MappingIndex;

/// 映射匹配引擎
pub struct MappingEngine {
    index: MappingIndex,
}

impl MappingEngine {
    /// 从配置构建引擎
    pub fn build(config: &ProjectConfig, workspace: &Path) -> Result<Self> {
        let index = MappingIndex::build(&config.mappings, workspace)?;
        Ok(Self { index })
    }

    /// 给定变更列表，返回命中的映射
    ///
    /// `workspace` 用于读取文件以解析 anchor。
    pub fn resolve(&self, changes: &[ChangeEntry], workspace: &Path) -> Vec<MappingHit> {
        let mut hits = Vec::new();

        for change in changes {
            for entry in &self.index.entries {
                // 文件级匹配：路径精确匹配或 glob 匹配
                if !entry.matcher.is_match(&change.path) {
                    continue;
                }

                // 确定有效行范围：anchor 优先，其次 static lines
                let effective_range = self.resolve_effective_range(
                    entry.anchor.as_deref(),
                    entry.anchor_context.as_deref(),
                    entry.lines,
                    &change.path,
                    workspace,
                );

                if let Some(defined_range) = effective_range {
                    // 行范围匹配：检查 hunk 是否与定义的行范围重叠
                    let mut matched_hunks = Vec::new();
                    for hunk in &change.hunks {
                        if hunks_overlap(hunk, &defined_range) {
                            matched_hunks.push(hunk.clone());
                        }
                    }

                    if !matched_hunks.is_empty() {
                        hits.push(MappingHit {
                            mapping_id: entry.mapping_id.clone(),
                            source_match: SourceMatch::LineRange {
                                path: change.path.clone(),
                                matched_hunks,
                            },
                            asset_id: entry.asset_id.clone(),
                            segment: entry.segment.clone(),
                            hit_type: HitType::RangeOverlap {
                                defined_range,
                                change_hunk: change.hunks.first().cloned().unwrap_or(HunkRange {
                                    start_line: 0,
                                    end_line: 0,
                                }),
                            },
                        });
                    }
                } else {
                    // 文件级匹配（无行范围限制）
                    hits.push(MappingHit {
                        mapping_id: entry.mapping_id.clone(),
                        source_match: SourceMatch::WholeFile {
                            path: change.path.clone(),
                        },
                        asset_id: entry.asset_id.clone(),
                        segment: entry.segment.clone(),
                        hit_type: HitType::FileMatch {
                            pattern: entry.matcher.glob().to_string(),
                        },
                    });
                }
            }
        }

        hits
    }

    /// 解析有效行范围。
    ///
    /// 优先级：anchor > static lines > None（全文件）
    fn resolve_effective_range(
        &self,
        anchor: Option<&str>,
        anchor_context: Option<&str>,
        static_lines: Option<[u32; 2]>,
        file_path: &str,
        workspace: &Path,
    ) -> Option<[u32; 2]> {
        // 1. 尝试 anchor 解析
        if let Some(anchor_text) = anchor {
            let full_path = workspace.join(file_path);
            if let Ok(content) = std::fs::read_to_string(&full_path) {
                if let Some(result) = resolve_anchor(&content, anchor_text, anchor_context) {
                    return Some([result.start_line, result.end_line]);
                }
                // Anchor not found: log warning, fall through to static lines
                eprintln!(
                    "⚠️  Anchor '{}' not found in '{}', falling back to {}",
                    anchor_text,
                    file_path,
                    if static_lines.is_some() {
                        "static line range"
                    } else {
                        "whole-file match"
                    }
                );
            }
        }

        // 2. Fallback: static lines
        static_lines
    }
}

/// 判断变更 hunk 与映射定义行范围是否重叠
fn hunks_overlap(hunk: &HunkRange, range: &[u32; 2]) -> bool {
    hunk.start_line <= range[1] && range[0] <= hunk.end_line
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{ChangeType, MappingDef, MappingMeta, SourceLocator};

    fn make_config(mappings: Vec<MappingDef>) -> ProjectConfig {
        ProjectConfig {
            mappings,
            ..Default::default()
        }
    }

    #[test]
    fn test_file_level_match() {
        let config = make_config(vec![MappingDef {
            id: "m1".to_string(),
            source: SourceLocator {
                path: "src/auth/**/*.ts".to_string(),
                lines: None,
                anchor: None,
                anchor_context: None,
            },
            asset: "doc".to_string(),
            segment: None,
            reason: None,
            action: None,
            confidence: None,
            meta: None,
        }]);

        let engine = MappingEngine::build(&config, Path::new(".")).unwrap();
        let changes = vec![ChangeEntry {
            path: "src/auth/login.ts".to_string(),
            change_type: ChangeType::Modified,
            hunks: vec![HunkRange {
                start_line: 10,
                end_line: 20,
            }],
        }];

        let hits = engine.resolve(&changes, Path::new("."));
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].mapping_id, "m1");
    }

    #[test]
    fn test_range_overlap() {
        let config = make_config(vec![MappingDef {
            id: "m2".to_string(),
            source: SourceLocator {
                path: "src/auth/login.ts".to_string(),
                lines: Some([10, 45]),
                anchor: None,
                anchor_context: None,
            },
            asset: "doc".to_string(),
            segment: None,
            reason: None,
            action: None,
            confidence: None,
            meta: None,
        }]);

        let engine = MappingEngine::build(&config, Path::new(".")).unwrap();

        // 重叠的 hunk
        let changes = vec![ChangeEntry {
            path: "src/auth/login.ts".to_string(),
            change_type: ChangeType::Modified,
            hunks: vec![HunkRange {
                start_line: 30,
                end_line: 40,
            }],
        }];
        let hits = engine.resolve(&changes, Path::new("."));
        assert_eq!(hits.len(), 1);

        // 不重叠的 hunk
        let changes = vec![ChangeEntry {
            path: "src/auth/login.ts".to_string(),
            change_type: ChangeType::Modified,
            hunks: vec![HunkRange {
                start_line: 50,
                end_line: 60,
            }],
        }];
        let hits = engine.resolve(&changes, Path::new("."));
        assert_eq!(hits.len(), 0);
    }

    #[test]
    fn test_no_match() {
        let config = make_config(vec![MappingDef {
            id: "m3".to_string(),
            source: SourceLocator {
                path: "src/auth/**/*.ts".to_string(),
                lines: None,
                anchor: None,
                anchor_context: None,
            },
            asset: "doc".to_string(),
            segment: None,
            reason: None,
            action: None,
            confidence: None,
            meta: None,
        }]);

        let engine = MappingEngine::build(&config, Path::new(".")).unwrap();
        let changes = vec![ChangeEntry {
            path: "src/api/handler.ts".to_string(),
            change_type: ChangeType::Modified,
            hunks: vec![],
        }];
        let hits = engine.resolve(&changes, Path::new("."));
        assert_eq!(hits.len(), 0);
    }
}
