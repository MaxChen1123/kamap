use anyhow::Result;
use clap::Args;

use kamap_core::analyzer::ImpactAnalyzer;
use kamap_core::git::DiffAnalyzer;
use kamap_core::mapping::MappingEngine;
use kamap_core::output::{format_impact_json, format_impact_text, OutputMode};

use super::{load_config, workspace_root};

#[derive(Args)]
pub struct ScanArgs {
    /// Base Git ref (default: origin/main)
    #[arg(long, default_value = "origin/main")]
    pub base: String,

    /// Head Git ref (default: HEAD)
    #[arg(long, default_value = "HEAD")]
    pub head: String,

    /// Output format: text, json
    #[arg(long, short, default_value = "text")]
    pub output: String,

    /// Path to config file
    #[arg(long)]
    pub config: Option<String>,
}

pub fn run(args: ScanArgs) -> Result<()> {
    let cm = load_config(args.config.as_deref())?;
    let workspace = workspace_root(cm.path());
    let config = cm.config();
    let output_mode = OutputMode::from_str(&args.output);

    // 1. Git diff 分析
    let diff_result = DiffAnalyzer::analyze(&workspace, &args.base, &args.head)?;

    // 2. 构建映射引擎并匹配
    let engine = MappingEngine::build(config, &workspace)?;
    let hits = engine.resolve(&diff_result.changes);

    // 3. 影响分析
    let report = ImpactAnalyzer::analyze(
        hits,
        config,
        &diff_result.base_ref,
        &diff_result.head_ref,
        diff_result.changes.len(),
    )?;

    // 4. 输出
    match output_mode {
        OutputMode::Json => println!("{}", format_impact_json(&report)),
        OutputMode::Text => print!("{}", format_impact_text(&report)),
    }

    Ok(())
}
