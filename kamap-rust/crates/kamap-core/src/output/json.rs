use crate::models::ImpactReport;

/// 将影响报告输出为 JSON 字符串
pub fn format_impact_json(report: &ImpactReport) -> String {
    serde_json::to_string_pretty(report).unwrap_or_else(|e| {
        format!("{{\"error\": \"Failed to serialize: {}\"}}", e)
    })
}

/// 结构化错误输出
pub fn format_error_json(code: &str, message: &str) -> String {
    serde_json::json!({
        "error": {
            "code": code,
            "message": message
        }
    })
    .to_string()
}
