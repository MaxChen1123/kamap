use anyhow::Result;
use kamap_core::builder::MappingCandidate;
use kamap_core::models::{AssetDef, AssetMeta, HealthStatus, SegmentInfo};
use kamap_core::plugin::protocol::{AssetPlugin, Capabilities, Validation};

/// SQLite 资产插件
pub struct SqlitePlugin;

impl SqlitePlugin {
    pub fn new() -> Self {
        Self
    }
}

impl Default for SqlitePlugin {
    fn default() -> Self {
        Self::new()
    }
}

impl AssetPlugin for SqlitePlugin {
    fn provider(&self) -> &str {
        "sqlite"
    }

    fn asset_types(&self) -> Vec<String> {
        vec!["sqlite-db".to_string()]
    }

    fn capabilities(&self) -> Capabilities {
        Capabilities {
            can_resolve_segment: true,
            can_read_content: false,
            can_discover_mappings: false,
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
                message: Some("SQLite target path is empty".to_string()),
            });
        }
        Ok(Validation {
            valid: true,
            message: None,
        })
    }

    fn resolve_segment(
        &self,
        _asset: &AssetDef,
        segment: &serde_json::Value,
    ) -> Result<Option<SegmentInfo>> {
        if let Some(table) = segment.get("table").and_then(|v| v.as_str()) {
            return Ok(Some(SegmentInfo {
                label: format!("Table: {}", table),
                detail: Some(segment.clone()),
            }));
        }
        Ok(None)
    }

    fn health_check(&self, asset: &AssetDef) -> Result<HealthStatus> {
        let path = std::path::Path::new(&asset.target);
        if path.exists() {
            // 尝试打开验证
            match rusqlite::Connection::open(path) {
                Ok(_) => Ok(HealthStatus::Healthy),
                Err(_) => Ok(HealthStatus::Unhealthy),
            }
        } else {
            Ok(HealthStatus::Unhealthy)
        }
    }

    fn get_meta(&self, asset: &AssetDef) -> Result<Option<AssetMeta>> {
        let path = std::path::Path::new(&asset.target);
        if !path.exists() {
            return Ok(None);
        }

        let conn = rusqlite::Connection::open(path)?;
        let mut stmt = conn.prepare("SELECT name FROM sqlite_master WHERE type='table'")?;
        let tables: Vec<String> = stmt
            .query_map([], |row| row.get::<_, String>(0))?
            .filter_map(|r| r.ok())
            .collect();

        let mut extra = std::collections::HashMap::new();
        extra.insert(
            "tables".to_string(),
            serde_json::Value::Array(tables.into_iter().map(serde_json::Value::String).collect()),
        );

        Ok(Some(AssetMeta {
            title: path.file_stem().map(|s| s.to_string_lossy().to_string()),
            last_modified: None,
            owner: None,
            extra,
        }))
    }

    fn discover_mappings(&self, _asset: &AssetDef) -> Result<Vec<MappingCandidate>> {
        Ok(vec![])
    }
}
