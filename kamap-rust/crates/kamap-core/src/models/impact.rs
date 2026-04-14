use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::asset::AssetDef;
use super::mapping::{Action, ChangedLines, HitType, SourceMatch};
use super::source::ChangeType;

/// 严重程度
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    Info,
    Warning,
    Error,
}

/// 片段信息（由插件解析后的结构化信息）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SegmentInfo {
    /// 人可读描述
    pub label: String,
    /// 结构化详情
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub detail: Option<serde_json::Value>,
}

/// 单个影响条目
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Impact {
    pub asset: AssetDef,
    pub source: SourceMatch,
    pub mapping_id: String,
    pub hit_type: HitType,
    /// 触发此影响的 Git 变更类型（added/modified/deleted/renamed）
    pub change_type: ChangeType,
    pub reason: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub segment: Option<SegmentInfo>,
    pub confidence: f32,
    pub suggested_action: Action,
    pub severity: Severity,
    /// 触发此影响的变更行数统计
    #[serde(default)]
    pub changed_lines: ChangedLines,
    /// Provider 生成的操作指引 prompt（v2 新增）
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub action_prompt: Option<String>,
}

/// 扫描元信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanMeta {
    pub base: String,
    pub head: String,
    pub changes: usize,
    pub impacts: usize,
}

/// 摘要
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Summary {
    pub total_changes: usize,
    pub total_impacts: usize,
    pub by_severity: HashMap<String, usize>,
    pub by_action: HashMap<String, usize>,
}

/// 影响分析报告
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImpactReport {
    pub meta: ScanMeta,
    pub impacts: Vec<Impact>,
    pub summary: Summary,
}
