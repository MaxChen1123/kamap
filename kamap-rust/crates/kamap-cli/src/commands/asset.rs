use std::collections::HashMap;

use anyhow::Result;
use clap::{Args, Subcommand};

use kamap_core::models::AssetDef;

use super::{build_plugin_registry, load_config};

#[derive(Args)]
pub struct AssetArgs {
    #[command(subcommand)]
    pub command: AssetCommands,

    #[arg(long, global = true)]
    pub config: Option<String>,

    /// Write to kamap.yaml (shared/team config) instead of .kamap.yaml (personal)
    #[arg(long, global = true)]
    pub shared: bool,
}

#[derive(Subcommand)]
pub enum AssetCommands {
    /// Register a new asset
    Add(AssetAddArgs),
    /// Remove an asset
    Remove(AssetRemoveArgs),
    /// List all assets
    List(AssetListArgs),
    /// Health check all assets
    Check(AssetCheckArgs),
}

#[derive(Args)]
pub struct AssetAddArgs {
    #[arg(long)]
    pub id: String,
    #[arg(long)]
    pub provider: String,
    #[arg(long, name = "type")]
    pub asset_type: String,
    #[arg(long)]
    pub target: String,
    #[arg(long)]
    pub apply: bool,
    #[arg(long, short, default_value = "text")]
    pub output: String,
}

#[derive(Args)]
pub struct AssetRemoveArgs {
    #[arg(long)]
    pub id: String,
    #[arg(long, short, default_value = "text")]
    pub output: String,
}

#[derive(Args)]
pub struct AssetListArgs {
    #[arg(long, short, default_value = "text")]
    pub output: String,
}

#[derive(Args)]
pub struct AssetCheckArgs {
    #[arg(long, short, default_value = "text")]
    pub output: String,
}

pub fn run(args: AssetArgs) -> Result<()> {
    let shared = args.shared;
    match args.command {
        AssetCommands::Add(a) => run_add(a, args.config.as_deref(), shared),
        AssetCommands::Remove(a) => run_remove(a, args.config.as_deref(), shared),
        AssetCommands::List(a) => run_list(a, args.config.as_deref()),
        AssetCommands::Check(a) => run_check(a, args.config.as_deref()),
    }
}

fn run_add(args: AssetAddArgs, config_path: Option<&str>, shared: bool) -> Result<()> {
    let mut cm = load_config(config_path)?;

    let asset = AssetDef {
        id: args.id.clone(),
        provider: args.provider.clone(),
        asset_type: args.asset_type.clone(),
        target: args.target.clone(),
        meta: HashMap::new(),
    };

    if !args.apply {
        if args.output == "json" {
            println!(
                "{}",
                serde_json::json!({
                    "status": "dry_run",
                    "asset": asset,
                    "message": "Use --apply to write."
                })
            );
        } else {
            println!("Dry run — asset would be added:");
            println!("  ID:       {}", asset.id);
            println!("  Provider: {}", asset.provider);
            println!("  Type:     {}", asset.asset_type);
            println!("  Target:   {}", asset.target);
            println!("\nUse --apply to write.");
        }
        return Ok(());
    }

    cm.add_asset(asset)?;
    cm.save_to(shared)?;

    if args.output == "json" {
        println!(
            "{}",
            serde_json::json!({"status": "added", "id": args.id})
        );
    } else {
        println!("✅ Asset '{}' added.", args.id);
    }

    Ok(())
}

fn run_remove(args: AssetRemoveArgs, config_path: Option<&str>, shared: bool) -> Result<()> {
    let mut cm = load_config(config_path)?;
    let removed = cm.remove_asset(&args.id)?;
    cm.save_to(shared)?;

    if args.output == "json" {
        println!(
            "{}",
            serde_json::json!({"status": if removed { "removed" } else { "not_found" }, "id": args.id})
        );
    } else if removed {
        println!("✅ Asset '{}' removed.", args.id);
    } else {
        println!("⚠️  Asset '{}' not found.", args.id);
    }

    Ok(())
}

fn run_list(args: AssetListArgs, config_path: Option<&str>) -> Result<()> {
    let cm = load_config(config_path)?;
    let assets = cm.list_assets(None);

    if args.output == "json" {
        println!("{}", serde_json::to_string_pretty(&assets)?);
    } else {
        if assets.is_empty() {
            println!("No assets registered.");
            return Ok(());
        }
        println!("Assets ({}):\n", assets.len());
        for a in &assets {
            println!("  {} [{}:{}] → {}", a.id, a.provider, a.asset_type, a.target);
        }
    }

    Ok(())
}

fn run_check(args: AssetCheckArgs, config_path: Option<&str>) -> Result<()> {
    let cm = load_config(config_path)?;
    let registry = build_plugin_registry();
    let assets = cm.list_assets(None);

    let mut results = Vec::new();

    for asset in &assets {
        let status = if let Some(plugin) = registry.get(&asset.provider) {
            match plugin.health_check(asset) {
                Ok(s) => format!("{:?}", s),
                Err(e) => format!("Error: {}", e),
            }
        } else {
            "Unknown (no plugin)".to_string()
        };

        results.push(serde_json::json!({
            "id": asset.id,
            "target": asset.target,
            "status": status,
        }));
    }

    if args.output == "json" {
        println!("{}", serde_json::to_string_pretty(&results)?);
    } else {
        for r in &results {
            let icon = match r["status"].as_str().unwrap_or("") {
                "Healthy" => "✅",
                "Unhealthy" => "❌",
                _ => "❓",
            };
            println!(
                "  {} {} ({})",
                icon,
                r["id"].as_str().unwrap_or(""),
                r["status"].as_str().unwrap_or("")
            );
        }
    }

    Ok(())
}
