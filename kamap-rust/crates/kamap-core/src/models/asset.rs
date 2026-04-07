use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 资产定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetDef {
    /// 资产唯一标识
    pub id: String,
    /// 插件提供者名称
    pub provider: String,
    /// 资产类型
    #[serde(rename = "type")]
    pub asset_type: String,
    /// 资产目标（路径/URL 等）
    pub target: String,
    /// 元信息
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub meta: HashMap<String, serde_json::Value>,
}

/// 资产过滤条件
#[derive(Debug, Clone, Default)]
pub struct AssetFilter {
    pub provider: Option<String>,
    pub asset_type: Option<String>,
}

/// 资产元信息（插件返回）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetMeta {
    pub title: Option<String>,
    pub last_modified: Option<String>,
    pub owner: Option<String>,
    pub extra: HashMap<String, serde_json::Value>,
}

/// 资产健康状态
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum HealthStatus {
    Healthy,
    Unhealthy,
    Unknown,
}
