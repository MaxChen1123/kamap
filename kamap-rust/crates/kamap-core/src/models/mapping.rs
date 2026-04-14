use serde::{Deserialize, Serialize};

use super::source::{HunkRange, SourceLocator};

/// 推荐动作
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum Action {
    Review,
    Update,
    Verify,
    Acknowledge,
    Custom(String),
}

impl Default for Action {
    fn default() -> Self {
        Action::Review
    }
}

/// 映射定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MappingDef {
    /// 映射唯一标识，自动生成
    #[serde(default = "generate_mapping_id")]
    pub id: String,
    /// 源代码定位
    pub source: SourceLocator,
    /// 关联的资产 ID
    pub asset: String,
    /// 资产片段信息（由插件解释）
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub segment: Option<serde_json::Value>,
    /// 映射原因
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
    /// 推荐动作
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub action: Option<Action>,
    /// 置信度
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub confidence: Option<f32>,
    /// 映射元数据（来源信息）
    #[serde(default, skip_serializing_if = "Option::is_none", rename = "_meta")]
    pub meta: Option<MappingMeta>,
}

/// 映射来源元信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MappingMeta {
    /// 来源: "manual" / "ai-generated" / "annotation" / "frontmatter"
    pub origin: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub added_at: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub confidence: Option<f32>,
}

/// 映射过滤条件
#[derive(Debug, Clone, Default)]
pub struct MappingFilter {
    pub asset_id: Option<String>,
    pub source_path: Option<String>,
}

/// 映射更新
#[derive(Debug, Clone, Default)]
pub struct MappingUpdate {
    pub reason: Option<String>,
    pub action: Option<Action>,
    pub confidence: Option<f32>,
    pub segment: Option<serde_json::Value>,
}

/// 变更行数统计
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ChangedLines {
    /// 新增行数
    pub additions: u32,
    /// 删除行数
    pub deletions: u32,
}

impl ChangedLines {
    pub fn total(&self) -> u32 {
        self.additions + self.deletions
    }
}

/// 映射匹配命中
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MappingHit {
    pub mapping_id: String,
    pub source_match: SourceMatch,
    pub asset_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub segment: Option<serde_json::Value>,
    pub hit_type: HitType,
    /// 触发此命中的 Git 变更类型
    pub change_type: super::source::ChangeType,
    /// 触发此命中的变更行数统计
    #[serde(default)]
    pub changed_lines: ChangedLines,
}

/// 命中类型
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HitType {
    FileMatch { pattern: String },
    RangeOverlap { defined_range: [u32; 2], change_hunk: HunkRange },
}

/// 源匹配信息
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SourceMatch {
    WholeFile { path: String },
    LineRange { path: String, matched_hunks: Vec<HunkRange> },
}

/// 批量操作结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchResult {
    pub added: Vec<String>,
    pub skipped: Vec<(usize, String)>,
    pub errors: Vec<(usize, String)>,
}

/// 合并策略
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MergeStrategy {
    Append,
    Merge,
    Replace,
}

fn generate_mapping_id() -> String {
    format!("map_{}", &uuid::Uuid::new_v4().to_string()[..8])
}
