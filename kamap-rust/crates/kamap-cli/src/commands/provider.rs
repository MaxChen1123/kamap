use anyhow::Result;
use clap::{Args, Subcommand};

use super::load_config;

#[derive(Args)]
pub struct ProviderArgs {
    #[command(subcommand)]
    pub command: ProviderCommands,

    /// Path to config file
    #[arg(long, global = true)]
    pub config: Option<String>,
}

#[derive(Subcommand)]
pub enum ProviderCommands {
    /// List all providers (builtin + configured)
    List(ProviderListArgs),
    /// Show provider info
    Info(ProviderInfoArgs),
}

#[derive(Args)]
pub struct ProviderListArgs {
    #[arg(long, short, default_value = "text")]
    pub output: String,
}

#[derive(Args)]
pub struct ProviderInfoArgs {
    /// Provider name
    #[arg(long)]
    pub name: String,
    #[arg(long, short, default_value = "text")]
    pub output: String,
}

/// 内置 provider 名称列表
const BUILTIN_PROVIDERS: &[&str] = &["localfs", "sqlite"];

pub fn run(args: ProviderArgs) -> Result<()> {
    match args.command {
        ProviderCommands::List(a) => run_list(a, args.config.as_deref()),
        ProviderCommands::Info(a) => run_info(a, args.config.as_deref()),
    }
}

fn run_list(args: ProviderListArgs, config_path: Option<&str>) -> Result<()> {
    let cm = load_config(config_path)?;
    let config = cm.config();

    // 收集所有 provider：内置 + 配置中定义的
    let mut providers: Vec<serde_json::Value> = Vec::new();

    for &name in BUILTIN_PROVIDERS {
        let custom = config.providers.iter().find(|p| p.name == name);
        providers.push(serde_json::json!({
            "name": name,
            "type": "builtin",
            "has_custom_template": custom.and_then(|c| c.prompt_template.as_ref()).is_some(),
        }));
    }

    for p in &config.providers {
        if !BUILTIN_PROVIDERS.contains(&p.name.as_str()) {
            providers.push(serde_json::json!({
                "name": p.name,
                "type": "custom",
                "has_custom_template": p.prompt_template.is_some(),
            }));
        }
    }

    if args.output == "json" {
        println!("{}", serde_json::to_string_pretty(&providers)?);
    } else {
        println!("Providers ({}):\n", providers.len());
        for p in &providers {
            let ptype = p["type"].as_str().unwrap_or("?");
            let name = p["name"].as_str().unwrap_or("?");
            let has_template = p["has_custom_template"].as_bool().unwrap_or(false);
            let template_tag = if has_template { " (custom template)" } else { "" };
            println!("  {} [{}]{}", name, ptype, template_tag);
        }
    }

    Ok(())
}

fn run_info(args: ProviderInfoArgs, config_path: Option<&str>) -> Result<()> {
    let cm = load_config(config_path)?;
    let config = cm.config();

    let is_builtin = BUILTIN_PROVIDERS.contains(&args.name.as_str());
    let custom_def = config.providers.iter().find(|p| p.name == args.name);

    if !is_builtin && custom_def.is_none() {
        anyhow::bail!("Provider '{}' not found", args.name);
    }

    let provider_type = if is_builtin { "builtin" } else { "custom" };
    let template = custom_def.and_then(|d| d.prompt_template.as_ref());

    // 统计使用此 provider 的资产数
    let asset_count = config.assets.iter().filter(|a| a.provider == args.name).count();

    if args.output == "json" {
        println!(
            "{}",
            serde_json::json!({
                "name": args.name,
                "type": provider_type,
                "prompt_template": template,
                "asset_count": asset_count,
            })
        );
    } else {
        println!("Provider: {}", args.name);
        println!("  Type: {}", provider_type);
        println!("  Assets using this provider: {}", asset_count);
        if let Some(tmpl) = template {
            println!("  Prompt template:");
            for line in tmpl.lines() {
                println!("    {}", line);
            }
        } else if is_builtin {
            println!("  Prompt: (builtin default)");
        } else {
            println!("  Prompt template: (none)");
        }
    }

    Ok(())
}
