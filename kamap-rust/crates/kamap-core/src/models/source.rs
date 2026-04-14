use serde::{Deserialize, Serialize};

/// 源代码定位器
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceLocator {
    /// 文件路径，支持 glob 模式
    pub path: String,
    /// 可选行范围 [start, end]（静态，不推荐，anchor 优先）
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub lines: Option<[u32; 2]>,
    /// 语义锚点：用于在文件中定位代码块的文本特征（如 "fn login"、"class AuthService"）
    /// scan 时会在当前版本文件中动态解析出实际行范围，避免行号漂移问题。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub anchor: Option<String>,
    /// 锚点上下文：用于消歧的外层作用域文本（如 "impl Token"）
    /// 当文件中存在多个同名 anchor 时，先定位 anchor_context 块，再在其中查找 anchor。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub anchor_context: Option<String>,
}

/// Git 变更类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ChangeType {
    Added,
    Modified,
    Deleted,
    Renamed {
        old_path: String,
    },
}

/// 变更行范围
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HunkRange {
    pub start_line: u32,
    pub end_line: u32,
    /// 此 hunk 中新增的行数
    #[serde(default)]
    pub additions: u32,
    /// 此 hunk 中删除的行数
    #[serde(default)]
    pub deletions: u32,
}

/// 单个文件的变更
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangeEntry {
    /// 文件路径
    pub path: String,
    /// 变更类型
    pub change_type: ChangeType,
    /// 变更行范围列表
    pub hunks: Vec<HunkRange>,
}

/// Git diff 分析结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffResult {
    pub base_ref: String,
    pub head_ref: String,
    pub changes: Vec<ChangeEntry>,
}
