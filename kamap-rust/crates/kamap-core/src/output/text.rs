use crate::models::{Action, ImpactReport, Severity, SourceMatch};

/// 将影响报告格式化为人类可读文本
pub fn format_impact_text(report: &ImpactReport) -> String {
    let mut out = String::new();

    out.push_str(&format!(
        "\n📊 kamap scan\n   Base: {} → Head: {} | {} files changed\n\n",
        report.meta.base, report.meta.head, report.meta.changes
    ));

    if report.impacts.is_empty() {
        out.push_str("   ✅ No impacted assets found.\n");
        return out;
    }

    for impact in &report.impacts {
        let icon = match impact.severity {
            Severity::Error => "🔴 [ERROR]",
            Severity::Warning => "🟡 [WARNING]",
            Severity::Info => "🔵 [INFO]",
        };

        out.push_str(&format!(
            "{} {} ({})\n",
            icon, impact.asset.id, impact.asset.target
        ));

        // Source 信息
        let source_str = match &impact.source {
            SourceMatch::WholeFile { path } => format!("   Source:  {} (modified)\n", path),
            SourceMatch::LineRange {
                path,
                matched_hunks,
            } => {
                let hunks: Vec<String> = matched_hunks
                    .iter()
                    .map(|h| format!("{}-{}", h.start_line, h.end_line))
                    .collect();
                format!("   Source:  {}:{} (modified)\n", path, hunks.join(","))
            }
        };
        out.push_str(&source_str);

        // Segment 信息
        if let Some(ref seg) = impact.segment {
            out.push_str(&format!("   Segment: {}\n", seg.label));
        }

        // Reason
        out.push_str(&format!("   Reason:  {}\n", impact.reason));

        // Action
        let action_str = format_action(&impact.suggested_action);
        out.push_str(&format!("   Action:  {}\n", action_str));

        out.push('\n');
    }

    // Summary
    let error_count = report
        .summary
        .by_severity
        .get("error")
        .copied()
        .unwrap_or(0);
    let warning_count = report
        .summary
        .by_severity
        .get("warning")
        .copied()
        .unwrap_or(0);

    out.push_str(&format!(
        "Summary: {} impacted assets ({} error, {} warning)\n",
        report.summary.total_impacts, error_count, warning_count
    ));

    out
}

fn format_action(action: &Action) -> String {
    match action {
        Action::Update => "⚡ UPDATE".to_string(),
        Action::Review => "👀 REVIEW".to_string(),
        Action::Verify => "🔍 VERIFY".to_string(),
        Action::Acknowledge => "✅ ACKNOWLEDGE".to_string(),
        Action::Custom(s) => format!("📌 {}", s.to_uppercase()),
    }
}
