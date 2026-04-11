use anyhow::Result;
use clap::Args;

use kamap_core::analyzer::ImpactAnalyzer;
use kamap_core::git::DiffAnalyzer;
use kamap_core::mapping::MappingEngine;
use kamap_core::output::{format_impact_json, format_impact_text, OutputMode};

use super::{load_config, workspace_root};

#[derive(Args)]
pub struct CheckArgs {
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

pub fn run(args: CheckArgs) -> Result<()> {
    let cm = load_config(args.config.as_deref())?;
    let workspace = workspace_root(cm.path());
    let config = cm.config();
    let output_mode = OutputMode::from_str(&args.output);

    let diff_result = DiffAnalyzer::analyze(&workspace, &args.base, &args.head)?;
    let engine = MappingEngine::build(config, &workspace)?;
    let hits = engine.resolve(&diff_result.changes, &workspace);
    let report = ImpactAnalyzer::analyze(
        hits,
        config,
        &diff_result.base_ref,
        &diff_result.head_ref,
        diff_result.changes.len(),
    )?;

    match output_mode {
        OutputMode::Json => println!("{}", format_impact_json(&report)),
        OutputMode::Text => print!("{}", format_impact_text(&report)),
    }

    // CI 模式：如果有 error 级别的影响，退出码非零
    let has_errors = report
        .summary
        .by_severity
        .get("error")
        .copied()
        .unwrap_or(0)
        > 0;

    if has_errors {
        std::process::exit(1);
    }

    Ok(())
}
