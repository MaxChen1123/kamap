use serde::{Deserialize, Serialize};

/// 源代码定位器
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceLocator {
    /// 文件路径，支持 glob 模式
    pub path: String,
    /// 可选行范围 [start, end]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub lines: Option<[u32; 2]>,
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
