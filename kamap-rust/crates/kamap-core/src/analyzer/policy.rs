use crate::config::schema::{PolicyDef, ProjectConfig};
use crate::models::{AssetDef, Severity};

/// 根据策略规则计算资产的严重程度
pub fn evaluate_severity(asset: &AssetDef, config: &ProjectConfig) -> Severity {
    for policy in &config.policies {
        if matches_policy(asset, policy) {
            return match policy.severity.as_str() {
                "error" => Severity::Error,
                "warning" => Severity::Warning,
                "info" => Severity::Info,
                _ => Severity::Warning,
            };
        }
    }
    // 默认 warning
    Severity::Warning
}

fn matches_policy(asset: &AssetDef, policy: &PolicyDef) -> bool {
    if let Some(ref priority) = policy.match_rule.asset_priority {
        if let Some(asset_priority) = asset.meta.get("priority") {
            if let Some(p) = asset_priority.as_str() {
                return p == priority;
            }
        }
        return false;
    }
    if let Some(ref provider) = policy.match_rule.provider {
        if &asset.provider != provider {
            return false;
        }
    }
    // 空 match = 匹配所有
    true
}
