use anyhow::Result;

use crate::builder::MappingCandidate;
use crate::models::{AssetDef, AssetMeta, HealthStatus, SegmentInfo};

/// 插件能力声明
#[derive(Debug, Clone, Default)]
pub struct Capabilities {
    pub can_resolve_segment: bool,
    pub can_read_content: bool,
    pub can_discover_mappings: bool,
    pub can_health_check: bool,
    pub can_get_meta: bool,
}

/// 校验结果
#[derive(Debug, Clone)]
pub struct Validation {
    pub valid: bool,
    pub message: Option<String>,
}

/// 资产插件 trait
pub trait AssetPlugin: Send + Sync {
    /// 唯一标识
    fn provider(&self) -> &str;

    /// 支持的资产类型
    fn asset_types(&self) -> Vec<String>;

    /// 能力声明
    fn capabilities(&self) -> Capabilities;

    /// 初始化
    fn init(&mut self, config: &serde_json::Value) -> Result<()>;

    // === 必选 ===

    /// 校验资产定义是否有效
    fn validate_asset(&self, asset: &AssetDef) -> Result<Validation>;

    // === 可选（有默认空实现）===

    /// 解析片段信息
    fn resolve_segment(
        &self,
        _asset: &AssetDef,
        _segment: &serde_json::Value,
    ) -> Result<Option<SegmentInfo>> {
        Ok(None)
    }

    /// 获取资产元信息
    fn get_meta(&self, _asset: &AssetDef) -> Result<Option<AssetMeta>> {
        Ok(None)
    }

    /// 读取资产内容
    fn read_content(
        &self,
        _asset: &AssetDef,
        _segment: Option<&serde_json::Value>,
    ) -> Result<Option<String>> {
        Ok(None)
    }

    /// 从资产中发现映射
    fn discover_mappings(&self, _asset: &AssetDef) -> Result<Vec<MappingCandidate>> {
        Ok(vec![])
    }

    /// 健康检查
    fn health_check(&self, _asset: &AssetDef) -> Result<HealthStatus> {
        Ok(HealthStatus::Unknown)
    }
}
