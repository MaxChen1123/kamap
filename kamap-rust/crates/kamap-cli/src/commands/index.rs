use anyhow::Result;
use clap::{Args, Subcommand};

use kamap_core::storage::IndexStore;

use super::{load_config, workspace_root};

#[derive(Args)]
pub struct IndexArgs {
    #[command(subcommand)]
    pub command: IndexCommands,

    #[arg(long, global = true)]
    pub config: Option<String>,
}

#[derive(Subcommand)]
pub enum IndexCommands {
    /// Build or rebuild the runtime index
    Build(IndexBuildArgs),
    /// Show index statistics
    Stats(IndexStatsArgs),
}

#[derive(Args)]
pub struct IndexBuildArgs {
    #[arg(long, short, default_value = "text")]
    pub output: String,
}

#[derive(Args)]
pub struct IndexStatsArgs {
    #[arg(long, short, default_value = "text")]
    pub output: String,
}

pub fn run(args: IndexArgs) -> Result<()> {
    match args.command {
        IndexCommands::Build(a) => run_build(a, args.config.as_deref()),
        IndexCommands::Stats(a) => run_stats(a, args.config.as_deref()),
    }
}

fn run_build(args: IndexBuildArgs, config_path: Option<&str>) -> Result<()> {
    let cm = load_config(config_path)?;
    let workspace = workspace_root(cm.path());
    let db_path = workspace.join(".kamap").join("index.db");

    let store = IndexStore::open(&db_path)?;
    store.rebuild(cm.config())?;

    let stats = store.stats()?;

    if args.output == "json" {
        println!(
            "{}",
            serde_json::json!({
                "status": "built",
                "stats": stats,
            })
        );
    } else {
        println!("✅ Index built at .kamap/index.db");
        println!("   File entries:  {}", stats.file_entries);
        println!("   Range entries: {}", stats.range_entries);
        println!("   Assets:        {}", stats.asset_entries);
    }

    Ok(())
}

fn run_stats(args: IndexStatsArgs, config_path: Option<&str>) -> Result<()> {
    let cm = load_config(config_path)?;
    let workspace = workspace_root(cm.path());
    let db_path = workspace.join(".kamap").join("index.db");

    if !db_path.exists() {
        if args.output == "json" {
            println!(
                "{}",
                serde_json::json!({"error": "Index not found. Run `kamap index build` first."})
            );
        } else {
            println!("⚠️  Index not found. Run `kamap index build` first.");
        }
        return Ok(());
    }

    let store = IndexStore::open(&db_path)?;
    let stats = store.stats()?;

    if args.output == "json" {
        println!("{}", serde_json::to_string_pretty(&stats)?);
    } else {
        println!("Index statistics:");
        println!("   File entries:  {}", stats.file_entries);
        println!("   Range entries: {}", stats.range_entries);
        println!("   Assets:        {}", stats.asset_entries);
    }

    Ok(())
}
