use serde::{Deserialize, Serialize};

use crate::models::{AssetDef, MappingDef};

/// Provider 定义（v2 新增）
///
/// Provider 定义了 kamap 在检测到影响时如何生成操作指引。
/// - 内置 provider（localfs、sqlite）无需 prompt_template，有默认 prompt
/// - 自定义 provider（iwiki、notion 等）通过 prompt_template 定义操作指引模板
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderDef {
    pub name: String,
    /// prompt 模板，支持 {{asset.id}}、{{asset.target}}、{{asset.meta.xxx}}、
    /// {{source.path}}、{{reason}}、{{action}}、{{mapping_id}} 等变量。
    /// 内置 provider 可省略此字段。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub prompt_template: Option<String>,
}

/// 插件定义（v1 兼容，deprecated）
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
    /// Provider 定义列表（v2 新增，定义操作指引模板）
    #[serde(default)]
    pub providers: Vec<ProviderDef>,
    /// 插件定义列表（v1 兼容，deprecated，保留以兼容旧配置）
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
            providers: vec![],
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
