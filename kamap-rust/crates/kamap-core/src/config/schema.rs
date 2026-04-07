use serde::{Deserialize, Serialize};

use crate::models::{AssetDef, MappingDef};

/// 插件定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginDef {
    pub name: String,
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub config: Option<serde_json::Value>,
}

/// 策略定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyDef {
    #[serde(rename = "match")]
    pub match_rule: PolicyMatch,
    pub severity: String,
}

/// 策略匹配规则
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyMatch {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub asset_priority: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub provider: Option<String>,
}

/// 映射发现配置
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DiscoveryConfig {
    #[serde(default)]
    pub annotation: AnnotationConfig,
    #[serde(default)]
    pub frontmatter: FrontmatterConfig,
    #[serde(default)]
    pub naming: NamingConfig,
}

/// 注释扫描配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnnotationConfig {
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default = "default_marker")]
    pub marker: String,
}

impl Default for AnnotationConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            marker: "@kamap".to_string(),
        }
    }
}

/// Frontmatter 解析配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrontmatterConfig {
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default = "default_frontmatter_key")]
    pub key: String,
}

impl Default for FrontmatterConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            key: "kamap".to_string(),
        }
    }
}

/// 命名约定配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NamingConfig {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub rules: Vec<NamingRule>,
}

impl Default for NamingConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            rules: vec![],
        }
    }
}

/// 命名规则
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NamingRule {
    pub source: String,
    pub asset_pattern: String,
}

/// 项目配置（顶层结构）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectConfig {
    #[serde(default = "default_version")]
    pub version: String,
    #[serde(default)]
    pub plugins: Vec<PluginDef>,
    #[serde(default)]
    pub assets: Vec<AssetDef>,
    #[serde(default)]
    pub mappings: Vec<MappingDef>,
    #[serde(default)]
    pub policies: Vec<PolicyDef>,
    #[serde(default)]
    pub discovery: DiscoveryConfig,
}

impl Default for ProjectConfig {
    fn default() -> Self {
        Self {
            version: "1".to_string(),
            plugins: vec![PluginDef {
                name: "localfs".to_string(),
                enabled: true,
                config: None,
            }],
            assets: vec![],
            mappings: vec![],
            policies: vec![],
            discovery: DiscoveryConfig::default(),
        }
    }
}

fn default_true() -> bool {
    true
}

fn default_version() -> String {
    "1".to_string()
}

fn default_marker() -> String {
    "@kamap".to_string()
}

fn default_frontmatter_key() -> String {
    "kamap".to_string()
}
