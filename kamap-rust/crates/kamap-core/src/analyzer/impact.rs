use std::collections::HashMap;

use anyhow::Result;

use crate::config::ProjectConfig;
use crate::models::*;

use super::policy::evaluate_severity;

/// 影响分析器
pub struct ImpactAnalyzer;

impl ImpactAnalyzer {
    /// 从映射命中和配置生成影响报告
    pub fn analyze(
        hits: Vec<MappingHit>,
        config: &ProjectConfig,
        base_ref: &str,
        head_ref: &str,
        total_changes: usize,
    ) -> Result<ImpactReport> {
        let mut impacts = Vec::new();

        for hit in &hits {
            // 查找资产定义
            let asset = config
                .assets
                .iter()
                .find(|a| a.id == hit.asset_id)
                .cloned()
                .unwrap_or(AssetDef {
                    id: hit.asset_id.clone(),
                    provider: "unknown".to_string(),
                    asset_type: "unknown".to_string(),
                    target: "unknown".to_string(),
                    meta: Default::default(),
                });

            // 查找映射定义
            let mapping = config.mappings.iter().find(|m| m.id == hit.mapping_id);

            let reason = mapping
                .and_then(|m| m.reason.clone())
                .unwrap_or_else(|| "Code change impacts this asset".to_string());

            let confidence = mapping.and_then(|m| m.confidence).unwrap_or(0.8);

            let suggested_action = mapping
                .and_then(|m| m.action.clone())
                .unwrap_or(Action::Review);

            let severity = evaluate_severity(&asset, config);

            // 生成 segment 信息
            let segment = hit.segment.as_ref().map(|s| SegmentInfo {
                label: segment_to_label(s),
                detail: Some(s.clone()),
            });

            impacts.push(Impact {
                asset,
                source: hit.source_match.clone(),
                mapping_id: hit.mapping_id.clone(),
                hit_type: hit.hit_type.clone(),
                reason,
                segment,
                confidence,
                suggested_action,
                severity,
            });
        }

        // 构建摘要
        let mut by_severity: HashMap<String, usize> = HashMap::new();
        let mut by_action: HashMap<String, usize> = HashMap::new();
        for impact in &impacts {
            *by_severity
                .entry(format!("{:?}", impact.severity).to_lowercase())
                .or_insert(0) += 1;
            *by_action
                .entry(format!("{:?}", impact.suggested_action).to_lowercase())
                .or_insert(0) += 1;
        }

        Ok(ImpactReport {
            meta: ScanMeta {
                base: base_ref.to_string(),
                head: head_ref.to_string(),
                changes: total_changes,
                impacts: impacts.len(),
            },
            summary: Summary {
                total_changes,
                total_impacts: impacts.len(),
                by_severity,
                by_action,
            },
            impacts,
        })
    }
}

fn segment_to_label(segment: &serde_json::Value) -> String {
    if let Some(heading) = segment.get("heading").and_then(|v| v.as_str()) {
        format!("## {}", heading)
    } else if let Some(table) = segment.get("table").and_then(|v| v.as_str()) {
        format!("Table: {}", table)
    } else if let Some(block_id) = segment.get("block_id").and_then(|v| v.as_str()) {
        format!("Block: {}", block_id)
    } else {
        segment.to_string()
    }
}
