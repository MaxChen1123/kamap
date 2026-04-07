use anyhow::Result;
use clap::{Args, Subcommand};

use super::build_plugin_registry;

#[derive(Args)]
pub struct PluginArgs {
    #[command(subcommand)]
    pub command: PluginCommands,
}

#[derive(Subcommand)]
pub enum PluginCommands {
    /// List all registered plugins
    List(PluginListArgs),
    /// Show plugin info
    Info(PluginInfoArgs),
}

#[derive(Args)]
pub struct PluginListArgs {
    #[arg(long, short, default_value = "text")]
    pub output: String,
}

#[derive(Args)]
pub struct PluginInfoArgs {
    /// Plugin name
    #[arg(long)]
    pub name: String,
    #[arg(long, short, default_value = "text")]
    pub output: String,
}

pub fn run(args: PluginArgs) -> Result<()> {
    match args.command {
        PluginCommands::List(a) => run_list(a),
        PluginCommands::Info(a) => run_info(a),
    }
}

fn run_list(args: PluginListArgs) -> Result<()> {
    let registry = build_plugin_registry();
    let plugins = registry.list();

    if args.output == "json" {
        let infos: Vec<_> = plugins
            .iter()
            .map(|name| {
                let plugin = registry.get(name).unwrap();
                serde_json::json!({
                    "name": name,
                    "types": plugin.asset_types(),
                    "capabilities": {
                        "resolve_segment": plugin.capabilities().can_resolve_segment,
                        "read_content": plugin.capabilities().can_read_content,
                        "discover_mappings": plugin.capabilities().can_discover_mappings,
                        "health_check": plugin.capabilities().can_health_check,
                        "get_meta": plugin.capabilities().can_get_meta,
                    }
                })
            })
            .collect();
        println!("{}", serde_json::to_string_pretty(&infos)?);
    } else {
        println!("Registered plugins ({}):\n", plugins.len());
        for name in &plugins {
            let plugin = registry.get(name).unwrap();
            println!("  {} — types: {:?}", name, plugin.asset_types());
        }
    }

    Ok(())
}

fn run_info(args: PluginInfoArgs) -> Result<()> {
    let registry = build_plugin_registry();
    let plugin = registry
        .get(&args.name)
        .ok_or_else(|| anyhow::anyhow!("Plugin '{}' not found", args.name))?;

    let caps = plugin.capabilities();

    if args.output == "json" {
        println!(
            "{}",
            serde_json::json!({
                "name": args.name,
                "types": plugin.asset_types(),
                "capabilities": {
                    "resolve_segment": caps.can_resolve_segment,
                    "read_content": caps.can_read_content,
                    "discover_mappings": caps.can_discover_mappings,
                    "health_check": caps.can_health_check,
                    "get_meta": caps.can_get_meta,
                }
            })
        );
    } else {
        println!("Plugin: {}", args.name);
        println!("  Types: {:?}", plugin.asset_types());
        println!("  Capabilities:");
        println!("    Resolve segment:  {}", caps.can_resolve_segment);
        println!("    Read content:     {}", caps.can_read_content);
        println!("    Discover mappings: {}", caps.can_discover_mappings);
        println!("    Health check:     {}", caps.can_health_check);
        println!("    Get meta:         {}", caps.can_get_meta);
    }

    Ok(())
}
