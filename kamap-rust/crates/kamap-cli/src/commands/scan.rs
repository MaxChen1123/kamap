use anyhow::Result;
use clap::{Args, Subcommand};

use kamap_core::ack::{ToAckEntry, ToAckStore};
use kamap_core::analyzer::ImpactAnalyzer;
use kamap_core::git::DiffAnalyzer;
use kamap_core::mapping::MappingEngine;
use kamap_core::models::{Action, ImpactReport, Severity, SourceMatch};
use kamap_core::output::OutputMode;

use super::{load_config, workspace_root};

#[derive(Args)]
pub struct ScanArgs {
    #[command(subcommand)]
    pub command: Option<ScanCommands>,

    /// Base Git ref (default: HEAD, i.e. latest commit)
    #[arg(long, default_value = "HEAD")]
    pub base: String,

    /// Head Git ref or "workdir" for uncommitted changes (default: workdir)
    #[arg(long, default_value = "workdir")]
    pub head: String,

    /// Output format: text, json
    #[arg(long, short, default_value = "text")]
    pub output: String,

    /// Path to config file
    #[arg(long)]
    pub config: Option<String>,
}

#[derive(Subcommand)]
pub enum ScanCommands {
    /// Mark impacts as acknowledged (document already synced)
    Ack(ScanAckArgs),
}

#[derive(Args)]
pub struct ScanAckArgs {
    /// Acknowledge all pending impacts
    #[arg(long)]
    pub all: bool,

    /// Acknowledge specific mapping IDs (comma-separated)
    #[arg(long, value_delimiter = ',')]
    pub ids: Vec<String>,

    /// Output format: text, json
    #[arg(long, short, default_value = "text")]
    pub output: String,

    /// Path to config file
    #[arg(long)]
    pub config: Option<String>,
}

pub fn run(args: ScanArgs) -> Result<()> {
    match args.command {
        Some(ScanCommands::Ack(ack_args)) => run_ack(ack_args),
        None => run_scan(args),
    }
}

fn run_scan(args: ScanArgs) -> Result<()> {
    let cm = load_config(args.config.as_deref())?;
    let workspace = workspace_root(cm.path());
    let config = cm.config();
    let output_mode = OutputMode::from_str(&args.output);

    // 1. Git diff
    let diff_result = DiffAnalyzer::analyze(&workspace, &args.base, &args.head)?;
    let head_commit = diff_result.head_ref.clone();

    // 2. 映射匹配
    let engine = MappingEngine::build(config, &workspace)?;
    let hits = engine.resolve(&diff_result.changes, &workspace);

    // 3. 影响分析
    let report = ImpactAnalyzer::analyze(
        hits,
        config,
        &diff_result.base_ref,
        &diff_result.head_ref,
        diff_result.changes.len(),
    )?;

    // 4. 读取已有的 to-ack.json，过滤掉已确认的 impact
    let ack_store = ToAckStore::open(&workspace)?;
    let mut pending_impacts: Vec<&kamap_core::models::Impact> = Vec::new();
    let mut acked_count = 0;

    for impact in &report.impacts {
        if ack_store.is_acked(&impact.mapping_id, &head_commit) {
            acked_count += 1;
        } else {
            pending_impacts.push(impact);
        }
    }

    // 5. 将未确认的 impact 写入 to-ack.json
    let to_ack_items: Vec<ToAckEntry> = report
        .impacts
        .iter()
        .map(|impact| {
            let acked = ack_store.is_acked(&impact.mapping_id, &head_commit);
            ToAckEntry {
                mapping_id: impact.mapping_id.clone(),
                asset_id: impact.asset.id.clone(),
                asset_target: impact.asset.target.clone(),
                source_path: source_path_str(&impact.source),
                reason: Some(impact.reason.clone()),
                action: format_action_tag(&impact.suggested_action),
                acked,
            }
        })
        .collect();

    let mut ack_store = ack_store; // rebind as mutable
    ack_store.write_scan_result(&head_commit, to_ack_items)?;

    // 6. 输出
    match output_mode {
        OutputMode::Json => {
            print_json_report(&report, &pending_impacts, acked_count, &head_commit);
        }
        OutputMode::Text => {
            print_text_report(&report, &pending_impacts, acked_count);
        }
    }

    Ok(())
}

// ─── JSON 输出（AI 友好） ───

fn print_json_report(
    report: &ImpactReport,
    pending: &[&kamap_core::models::Impact],
    acked_count: usize,
    head_commit: &str,
) {
    let pending_json: Vec<serde_json::Value> = pending
        .iter()
        .map(|i| {
            serde_json::json!({
                "mapping_id": i.mapping_id,
                "asset_id": i.asset.id,
                "asset_target": i.asset.target,
                "source": source_path_str(&i.source),
                "reason": i.reason,
                "action": format_action_tag(&i.suggested_action),
                "severity": format!("{:?}", i.severity).to_lowercase(),
            })
        })
        .collect();

    let output = serde_json::json!({
        "head_commit": head_commit,
        "total_changes": report.meta.changes,
        "total_impacts": report.impacts.len(),
        "pending": pending_json.len(),
        "acked": acked_count,
        "impacts": pending_json,
    });

    println!("{}", serde_json::to_string_pretty(&output).unwrap());
}

// ─── Text 输出（人类 + AI 两用） ───

fn print_text_report(
    report: &ImpactReport,
    pending: &[&kamap_core::models::Impact],
    acked_count: usize,
) {
    println!(
        "\n📊 kamap scan  ({} files changed, {} impacts, {} pending, {} acked)\n",
        report.meta.changes,
        report.impacts.len(),
        pending.len(),
        acked_count,
    );

    if pending.is_empty() {
        println!("  ✅ All clear — no pending impacts.\n");
        return;
    }

    for (i, impact) in pending.iter().enumerate() {
        let icon = match impact.severity {
            Severity::Error => "🔴",
            Severity::Warning => "🟡",
            Severity::Info => "🔵",
        };
        let action = format_action_tag(&impact.suggested_action);

        println!(
            "  {}  #{} [{}] {} → {}",
            icon,
            i + 1,
            action.to_uppercase(),
            source_path_str(&impact.source),
            impact.asset.target,
        );
        println!("      mapping: {}  asset: {}", impact.mapping_id, impact.asset.id);
        println!("      reason:  {}", impact.reason);
        println!();
    }

    println!("After syncing the documents, run:");
    println!("  kamap scan ack --all           # acknowledge all");
    println!("  kamap scan ack --ids <id,...>   # acknowledge specific ones");
}

// ─── scan ack ───

fn run_ack(args: ScanAckArgs) -> Result<()> {
    let cm = load_config(args.config.as_deref())?;
    let workspace = workspace_root(cm.path());

    let mut ack_store = ToAckStore::open(&workspace)?;

    if ack_store.data().is_none() {
        anyhow::bail!("No scan results found. Run `kamap scan` first.");
    }

    if args.all {
        let count = ack_store.ack_all()?;

        if args.output == "json" {
            println!("{}", serde_json::json!({"status": "ok", "acked": count}));
        } else {
            println!("✅ Acknowledged {} impact(s).", count);
        }
    } else if !args.ids.is_empty() {
        let (count, not_found) = ack_store.ack(&args.ids)?;

        if args.output == "json" {
            println!(
                "{}",
                serde_json::json!({
                    "status": "ok",
                    "acked": count,
                    "not_found": not_found,
                })
            );
        } else {
            println!("✅ Acknowledged {} impact(s).", count);
            for id in &not_found {
                println!("  ⚠️  '{}' not found in scan results.", id);
            }
        }
    } else {
        anyhow::bail!("Please specify --all or --ids <mapping_id,...>");
    }

    Ok(())
}

// ─── helpers ───

fn source_path_str(source: &SourceMatch) -> String {
    match source {
        SourceMatch::WholeFile { path } => path.clone(),
        SourceMatch::LineRange { path, matched_hunks } => {
            let hunks: Vec<String> = matched_hunks
                .iter()
                .map(|h| format!("{}-{}", h.start_line, h.end_line))
                .collect();
            format!("{}:{}", path, hunks.join(","))
        }
    }
}

fn format_action_tag(action: &Action) -> String {
    match action {
        Action::Update => "update".to_string(),
        Action::Review => "review".to_string(),
        Action::Verify => "verify".to_string(),
        Action::Acknowledge => "acknowledge".to_string(),
        Action::Custom(s) => s.clone(),
    }
}
