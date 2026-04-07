use anyhow::Result;
use clap::Args;

use super::load_config;

#[derive(Args)]
pub struct ExplainArgs {
    /// Mapping ID to explain
    #[arg(long)]
    pub mapping: Option<String>,

    /// Asset ID to explain
    #[arg(long)]
    pub asset: Option<String>,

    /// Source file path to explain
    #[arg(long)]
    pub source: Option<String>,

    /// Output format: text, json
    #[arg(long, short, default_value = "text")]
    pub output: String,

    /// Path to config file
    #[arg(long)]
    pub config: Option<String>,
}

pub fn run(args: ExplainArgs) -> Result<()> {
    let cm = load_config(args.config.as_deref())?;
    let config = cm.config();

    if let Some(mapping_id) = &args.mapping {
        if let Some(mapping) = config.mappings.iter().find(|m| &m.id == mapping_id) {
            if args.output == "json" {
                println!("{}", serde_json::to_string_pretty(mapping)?);
            } else {
                println!("Mapping: {}", mapping.id);
                println!("  Source: {}", mapping.source.path);
                if let Some(ref lines) = mapping.source.lines {
                    println!("  Lines:  {}-{}", lines[0], lines[1]);
                }
                println!("  Asset:  {}", mapping.asset);
                if let Some(ref reason) = mapping.reason {
                    println!("  Reason: {}", reason);
                }
                if let Some(ref action) = mapping.action {
                    println!("  Action: {:?}", action);
                }
            }
        } else {
            anyhow::bail!("Mapping '{}' not found", mapping_id);
        }
        return Ok(());
    }

    if let Some(asset_id) = &args.asset {
        let asset = config
            .assets
            .iter()
            .find(|a| &a.id == asset_id)
            .ok_or_else(|| anyhow::anyhow!("Asset '{}' not found", asset_id))?;

        let related_mappings: Vec<_> = config
            .mappings
            .iter()
            .filter(|m| &m.asset == asset_id)
            .collect();

        if args.output == "json" {
            println!(
                "{}",
                serde_json::json!({
                    "asset": asset,
                    "mappings": related_mappings,
                })
            );
        } else {
            println!("Asset: {} ({})", asset.id, asset.target);
            println!("  Provider: {}", asset.provider);
            println!("  Type:     {}", asset.asset_type);
            println!("  Related mappings ({}):", related_mappings.len());
            for m in &related_mappings {
                println!(
                    "    - {} → {} {}",
                    m.source.path,
                    m.id,
                    m.reason.as_deref().unwrap_or("")
                );
            }
        }
        return Ok(());
    }

    if let Some(source_path) = &args.source {
        let related_mappings: Vec<_> = config
            .mappings
            .iter()
            .filter(|m| m.source.path == *source_path || m.source.path.contains(source_path.as_str()))
            .collect();

        if args.output == "json" {
            println!("{}", serde_json::to_string_pretty(&related_mappings)?);
        } else {
            println!("Source: {}", source_path);
            println!("  Related mappings ({}):", related_mappings.len());
            for m in &related_mappings {
                println!(
                    "    - {} → asset:{} {}",
                    m.id,
                    m.asset,
                    m.reason.as_deref().unwrap_or("")
                );
            }
        }
        return Ok(());
    }

    anyhow::bail!("Please specify --mapping, --asset, or --source to explain");
}
