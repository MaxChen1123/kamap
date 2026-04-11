use std::path::Path;

use anyhow::Result;
use globset::{Glob, GlobMatcher};

use crate::models::MappingDef;

/// 映射索引中的一条文件级匹配记录
#[derive(Debug, Clone)]
pub struct FileIndexEntry {
    pub mapping_id: String,
    pub asset_id: String,
    pub matcher: GlobMatcher,
    pub lines: Option<[u32; 2]>,
    pub anchor: Option<String>,
    pub anchor_context: Option<String>,
    pub segment: Option<serde_json::Value>,
}

/// 映射索引（内存级）
pub struct MappingIndex {
    pub entries: Vec<FileIndexEntry>,
}

impl MappingIndex {
    /// 从映射定义列表构建索引
    pub fn build(mappings: &[MappingDef], _workspace: &Path) -> Result<Self> {
        let mut entries = Vec::new();

        for mapping in mappings {
            let glob = Glob::new(&mapping.source.path)
                .map_err(|e| anyhow::anyhow!("Invalid glob '{}': {}", mapping.source.path, e))?;
            let matcher = glob.compile_matcher();

            entries.push(FileIndexEntry {
                mapping_id: mapping.id.clone(),
                asset_id: mapping.asset.clone(),
                matcher,
                lines: mapping.source.lines,
                anchor: mapping.source.anchor.clone(),
                anchor_context: mapping.source.anchor_context.clone(),
                segment: mapping.segment.clone(),
            });
        }

        Ok(Self { entries })
    }
}
