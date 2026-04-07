pub mod markdown;

use anyhow::Result;
use kamap_core::builder::MappingCandidate;
use kamap_core::models::{AssetDef, AssetMeta, HealthStatus, SegmentInfo};
use kamap_core::plugin::protocol::{AssetPlugin, Capabilities, Validation};

/// 本地文件系统插件
pub struct LocalFsPlugin;

impl LocalFsPlugin {
    pub fn new() -> Self {
        Self
    }
}

impl Default for LocalFsPlugin {
    fn default() -> Self {
        Self::new()
    }
}

impl AssetPlugin for LocalFsPlugin {
    fn provider(&self) -> &str {
        "localfs"
    }

    fn asset_types(&self) -> Vec<String> {
        vec![
            "markdown".to_string(),
            "text".to_string(),
            "config".to_string(),
        ]
    }

    fn capabilities(&self) -> Capabilities {
        Capabilities {
            can_resolve_segment: true,
            can_read_content: true,
            can_discover_mappings: true,
            can_health_check: true,
            can_get_meta: true,
        }
    }

    fn init(&mut self, _config: &serde_json::Value) -> Result<()> {
        Ok(())
    }

    fn validate_asset(&self, asset: &AssetDef) -> Result<Validation> {
        if asset.target.is_empty() {
            return Ok(Validation {
                valid: false,
                message: Some("Target path is empty".to_string()),
            });
        }
        Ok(Validation {
            valid: true,
            message: None,
        })
    }

    fn resolve_segment(
        &self,
        asset: &AssetDef,
        segment: &serde_json::Value,
    ) -> Result<Option<SegmentInfo>> {
        if asset.asset_type == "markdown" {
            if let Some(heading) = segment.get("heading").and_then(|v| v.as_str()) {
                return Ok(Some(SegmentInfo {
                    label: format!("## {}", heading),
                    detail: Some(segment.clone()),
                }));
            }
        }
        Ok(None)
    }

    fn get_meta(&self, asset: &AssetDef) -> Result<Option<AssetMeta>> {
        let path = std::path::Path::new(&asset.target);
        if path.exists() {
            let metadata = std::fs::metadata(path)?;
            let modified = metadata
                .modified()
                .ok()
                .map(|t| {
                    let datetime: chrono::DateTime<chrono::Utc> = t.into();
                    datetime.to_rfc3339()
                });
            Ok(Some(AssetMeta {
                title: path
                    .file_stem()
                    .map(|s| s.to_string_lossy().to_string()),
                last_modified: modified,
                owner: None,
                extra: Default::default(),
            }))
        } else {
            Ok(None)
        }
    }

    fn read_content(
        &self,
        asset: &AssetDef,
        _segment: Option<&serde_json::Value>,
    ) -> Result<Option<String>> {
        let path = std::path::Path::new(&asset.target);
        if path.exists() {
            Ok(Some(std::fs::read_to_string(path)?))
        } else {
            Ok(None)
        }
    }

    fn discover_mappings(&self, _asset: &AssetDef) -> Result<Vec<MappingCandidate>> {
        // frontmatter 解析由 builder 模块处理
        Ok(vec![])
    }

    fn health_check(&self, asset: &AssetDef) -> Result<HealthStatus> {
        let path = std::path::Path::new(&asset.target);
        if path.exists() {
            Ok(HealthStatus::Healthy)
        } else {
            Ok(HealthStatus::Unhealthy)
        }
    }
}
