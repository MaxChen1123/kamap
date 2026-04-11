use std::path::Path;

use anyhow::Result;

use crate::config::ProjectConfig;
use crate::models::SourceLocator;

use super::{CandidateOrigin, DiscoveryOptions, DiscoveryStrategy, MappingCandidate};

/// 解析 Markdown 文档中的 frontmatter 声明
pub struct FrontmatterParser {
    key: String,
}

impl FrontmatterParser {
    pub fn new(key: &str) -> Self {
        Self {
            key: key.to_string(),
        }
    }

    fn parse_file(&self, file_path: &Path, rel_path: &str) -> Result<Vec<MappingCandidate>> {
        let content = std::fs::read_to_string(file_path)?;
        let mut candidates = Vec::new();

        // 检查 frontmatter (--- ... ---)
        if !content.starts_with("---") {
            return Ok(candidates);
        }

        let end = content[3..].find("---");
        if end.is_none() {
            return Ok(candidates);
        }
        let frontmatter_str = &content[3..3 + end.unwrap()];

        // 解析 YAML frontmatter
        let frontmatter: serde_yaml::Value = match serde_yaml::from_str(frontmatter_str) {
            Ok(v) => v,
            Err(_) => return Ok(candidates),
        };

        // 查找 kamap 键
        if let Some(kamap) = frontmatter.get(&self.key) {
            if let Some(relates_to) = kamap.get("relates-to") {
                if let Some(items) = relates_to.as_sequence() {
                    for item in items {
                        if let Some(candidate) = parse_frontmatter_item(item, rel_path) {
                            candidates.push(candidate);
                        }
                    }
                }
            }
        }

        Ok(candidates)
    }
}

impl DiscoveryStrategy for FrontmatterParser {
    fn name(&self) -> &str {
        "frontmatter"
    }

    fn discover(
        &self,
        workspace: &Path,
        _config: &ProjectConfig,
        _opts: &DiscoveryOptions,
    ) -> Result<Vec<MappingCandidate>> {
        let mut candidates = Vec::new();
        scan_markdown_files(self, workspace, workspace, &mut candidates)?;
        Ok(candidates)
    }
}

fn scan_markdown_files(
    parser: &FrontmatterParser,
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

        if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
            if name.starts_with('.') || name == "target" || name == "node_modules" {
                continue;
            }
        }

        if path.is_dir() {
            scan_markdown_files(parser, &path, workspace, candidates)?;
        } else if path.extension().and_then(|e| e.to_str()) == Some("md") {
            let rel_path = crate::path_util::to_forward_slash(
                path.strip_prefix(workspace).unwrap_or(&path),
            );
            if let Ok(mut file_candidates) = parser.parse_file(&path, &rel_path) {
                candidates.append(&mut file_candidates);
            }
        }
    }

    Ok(())
}

fn parse_frontmatter_item(
    item: &serde_yaml::Value,
    doc_path: &str,
) -> Option<MappingCandidate> {
    let path = item.get("path")?.as_str()?.to_string();
    let reason = item
        .get("reason")
        .and_then(|v| v.as_str())
        .unwrap_or("Declared in frontmatter")
        .to_string();

    let lines = item.get("lines").and_then(|v| {
        let s = v.as_str()?;
        let parts: Vec<&str> = s.split('-').collect();
        if parts.len() == 2 {
            let start: u32 = parts[0].trim().parse().ok()?;
            let end: u32 = parts[1].trim().parse().ok()?;
            Some([start, end])
        } else {
            None
        }
    });

    let segment = item.get("segment").map(|v| {
        serde_json::to_value(v).unwrap_or(serde_json::Value::Null)
    });

    Some(MappingCandidate {
        source: SourceLocator { path, lines, anchor: None, anchor_context: None },
        asset_id: doc_path.to_string(), // 反向映射：文档自己是资产
        reason,
        confidence: 0.85,
        origin: CandidateOrigin::AssetFrontmatter,
        segment,
    })
}
