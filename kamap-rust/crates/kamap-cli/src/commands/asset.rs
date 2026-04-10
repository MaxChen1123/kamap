use std::collections::HashMap;

use anyhow::{Context, Result};
use clap::{Args, Subcommand};

use kamap_core::config::ConfigManager;
use kamap_core::models::AssetDef;

use super::{build_plugin_registry, find_config_dir, load_config, SHARED_CONFIG_NAME, LOCAL_CONFIG_NAME};

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
    /// Batch register multiple assets from JSON (stdin or file)
    AddBatch(AssetAddBatchArgs),
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
    #[arg(long, alias = "type", name = "asset-type")]
    pub asset_type: String,
    #[arg(long)]
    pub target: String,
    #[arg(long)]
    pub apply: bool,
    #[arg(long, short, default_value = "text")]
    pub output: String,
}

#[derive(Args)]
pub struct AssetAddBatchArgs {
    /// Read JSON from stdin
    #[arg(long)]
    pub stdin: bool,
    /// Read JSON from file
    #[arg(long)]
    pub file: Option<String>,
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
        AssetCommands::AddBatch(a) => run_add_batch(a, args.config.as_deref(), shared),
        AssetCommands::Remove(a) => run_remove(a, args.config.as_deref(), shared),
        AssetCommands::List(a) => run_list(a, args.config.as_deref()),
        AssetCommands::Check(a) => run_check(a, args.config.as_deref()),
    }
}

fn run_add(args: AssetAddArgs, config_path: Option<&str>, shared: bool) -> Result<()> {
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

    // 使用文件锁保护的原子操作：加载→添加→保存
    let (shared_path, local_path) = if let Some(path) = config_path {
        let p = std::path::PathBuf::from(path);
        (Some(p), None)
    } else {
        let dir = find_config_dir();
        let sp = dir.join(SHARED_CONFIG_NAME);
        let lp = dir.join(LOCAL_CONFIG_NAME);
        (
            if sp.exists() { Some(sp) } else { None },
            if lp.exists() { Some(lp) } else { None },
        )
    };

    ConfigManager::locked_modify(
        shared_path.as_deref(),
        local_path.as_deref(),
        shared,
        |cm| {
            cm.add_asset(asset)?;
            Ok(())
        },
    )?;

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

/// 批量添加 asset 的 JSON 输入格式
#[derive(serde::Deserialize)]
struct BatchAssetInput {
    assets: Vec<BatchAssetItem>,
}

#[derive(serde::Deserialize)]
struct BatchAssetItem {
    id: String,
    provider: String,
    #[serde(rename = "type")]
    asset_type: String,
    target: String,
}

fn run_add_batch(args: AssetAddBatchArgs, config_path: Option<&str>, shared: bool) -> Result<()> {
    // 读取 JSON 输入
    let input = if args.stdin {
        use std::io::Read;
        let mut buf = String::new();
        std::io::stdin().read_to_string(&mut buf)?;
        buf
    } else if let Some(ref file) = args.file {
        std::fs::read_to_string(file)
            .with_context(|| format!("Failed to read file: {}", file))?
    } else {
        anyhow::bail!("Must specify --stdin or --file");
    };

    let batch: BatchAssetInput = serde_json::from_str(&input)
        .with_context(|| "Failed to parse batch JSON. Expected: {\"assets\":[{\"id\":...,\"provider\":...,\"type\":...,\"target\":...}]}")?;

    if batch.assets.is_empty() {
        if args.output == "json" {
            println!("{}", serde_json::json!({"status": "empty", "added": [], "errors": []}));
        } else {
            println!("No assets to add.");
        }
        return Ok(());
    }

    let assets: Vec<AssetDef> = batch.assets.into_iter().map(|item| AssetDef {
        id: item.id,
        provider: item.provider,
        asset_type: item.asset_type,
        target: item.target,
        meta: HashMap::new(),
    }).collect();

    if !args.apply {
        if args.output == "json" {
            println!(
                "{}",
                serde_json::json!({
                    "status": "dry_run",
                    "count": assets.len(),
                    "assets": assets,
                    "message": "Use --apply to write."
                })
            );
        } else {
            println!("Dry run — {} assets would be added:", assets.len());
            for a in &assets {
                println!("  {} [{}:{}] → {}", a.id, a.provider, a.asset_type, a.target);
            }
            println!("\nUse --apply to write.");
        }
        return Ok(());
    }

    // 使用文件锁保护的原子操作：一次锁定内添加所有 asset
    let (shared_path, local_path) = if let Some(path) = config_path {
        let p = std::path::PathBuf::from(path);
        (Some(p), None)
    } else {
        let dir = find_config_dir();
        let sp = dir.join(SHARED_CONFIG_NAME);
        let lp = dir.join(LOCAL_CONFIG_NAME);
        (
            if sp.exists() { Some(sp) } else { None },
            if lp.exists() { Some(lp) } else { None },
        )
    };

    let mut added: Vec<String> = Vec::new();
    let mut errors: Vec<serde_json::Value> = Vec::new();

    ConfigManager::locked_modify(
        shared_path.as_deref(),
        local_path.as_deref(),
        shared,
        |cm| {
            for asset in assets {
                let id = asset.id.clone();
                match cm.add_asset(asset) {
                    Ok(()) => added.push(id),
                    Err(e) => errors.push(serde_json::json!({"id": id, "error": e.to_string()})),
                }
            }
            Ok(())
        },
    )?;

    if args.output == "json" {
        println!(
            "{}",
            serde_json::json!({
                "status": "done",
                "added": added,
                "errors": errors,
            })
        );
    } else {
        if !added.is_empty() {
            println!("✅ Added {} assets: {}", added.len(), added.join(", "));
        }
        if !errors.is_empty() {
            for e in &errors {
                println!("❌ {}: {}", e["id"].as_str().unwrap_or("?"), e["error"].as_str().unwrap_or("?"));
            }
        }
    }

    Ok(())
}

fn run_remove(args: AssetRemoveArgs, config_path: Option<&str>, shared: bool) -> Result<()> {
    let (shared_path, local_path) = if let Some(path) = config_path {
        let p = std::path::PathBuf::from(path);
        (Some(p), None)
    } else {
        let dir = find_config_dir();
        let sp = dir.join(SHARED_CONFIG_NAME);
        let lp = dir.join(LOCAL_CONFIG_NAME);
        (
            if sp.exists() { Some(sp) } else { None },
            if lp.exists() { Some(lp) } else { None },
        )
    };

    let mut removed = false;
    ConfigManager::locked_modify(
        shared_path.as_deref(),
        local_path.as_deref(),
        shared,
        |cm| {
            removed = cm.remove_asset(&args.id)?;
            Ok(())
        },
    )?;

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
