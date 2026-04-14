use std::io::Read;

use anyhow::Result;
use clap::{Args, Subcommand};

use kamap_core::config::{ContextOptions, Format};
use kamap_core::models::{Action, MappingDef, MappingFilter, MappingMeta, SourceLocator};

use super::{load_config, workspace_root};

#[derive(Args)]
pub struct MappingArgs {
    #[command(subcommand)]
    pub command: MappingCommands,

    /// Path to config file
    #[arg(long, global = true)]
    pub config: Option<String>,

    /// Write to kamap.yaml (shared/team config) instead of .kamap.yaml (personal)
    #[arg(long, global = true)]
    pub shared: bool,
}

#[derive(Subcommand)]
pub enum MappingCommands {
    /// Add a single mapping
    Add(AddArgs),
    /// Batch add mappings from JSON stdin
    AddBatch(AddBatchArgs),
    /// Remove a mapping by ID
    Remove(RemoveArgs),
    /// List all mappings
    List(ListArgs),
    /// Validate all mappings
    Validate(ValidateArgs),
    // NOTE: discover 功能暂时关闭，实现代码保留在 run_discover 中
    // /// Auto-discover mapping candidates
    // Discover(DiscoverArgs),
    /// Export mappings
    Export(ExportArgs),
    /// Import mappings
    Import(ImportArgs),
    /// Export project context for AI
    ExportContext(ExportContextArgs),
}

// === Add ===

#[derive(Args)]
pub struct AddArgs {
    /// Source file path or glob
    #[arg(long)]
    pub source: String,
    /// Asset ID
    #[arg(long)]
    pub asset: String,
    /// Reason
    #[arg(long)]
    pub reason: Option<String>,
    /// Line range (e.g., "10-45") — static, not recommended; prefer --anchor
    #[arg(long)]
    pub lines: Option<String>,
    /// Semantic anchor: text pattern to locate the code block (e.g., "fn login", "class AuthService")
    #[arg(long)]
    pub anchor: Option<String>,
    /// Anchor context: outer scope for disambiguation (e.g., "impl Token")
    #[arg(long)]
    pub anchor_context: Option<String>,
    /// Action
    #[arg(long)]
    pub action: Option<String>,
    /// Dry run (default: true, use --apply to write)
    #[arg(long, default_value = "false")]
    pub apply: bool,
    /// Output format
    #[arg(long, short, default_value = "text")]
    pub output: String,
}

// === AddBatch ===

#[derive(Args)]
pub struct AddBatchArgs {
    /// Read JSON from stdin
    #[arg(long)]
    pub stdin: bool,
    /// JSON file path (alternative to stdin)
    #[arg(long)]
    pub file: Option<String>,
    /// Actually write changes
    #[arg(long, default_value = "false")]
    pub apply: bool,
    /// Output format
    #[arg(long, short, default_value = "json")]
    pub output: String,
}

// === Remove ===

#[derive(Args)]
pub struct RemoveArgs {
    /// Mapping ID to remove
    #[arg(long)]
    pub id: String,
    /// Output format
    #[arg(long, short, default_value = "text")]
    pub output: String,
}

// === List ===

#[derive(Args)]
pub struct ListArgs {
    /// Filter by asset ID
    #[arg(long)]
    pub asset: Option<String>,
    /// Output format
    #[arg(long, short, default_value = "text")]
    pub output: String,
}

// === Validate ===

#[derive(Args)]
pub struct ValidateArgs {
    /// Output format
    #[arg(long, short, default_value = "text")]
    pub output: String,
}

// === Discover (暂时关闭) ===

// #[derive(Args)]
// pub struct DiscoverArgs {
//     /// Include low confidence candidates
//     #[arg(long)]
//     pub include_low_confidence: bool,
//     /// Output format
//     #[arg(long, short, default_value = "text")]
//     pub output: String,
// }

// === Export ===

#[derive(Args)]
pub struct ExportArgs {
    /// Export format: json, yaml, csv
    #[arg(long, default_value = "json")]
    pub format: String,
}

// === Import ===

#[derive(Args)]
pub struct ImportArgs {
    /// Import format: json, yaml
    #[arg(long, default_value = "json")]
    pub format: String,
    /// Merge strategy: append, merge, replace
    #[arg(long, default_value = "append")]
    pub strategy: String,
    /// Read from stdin
    #[arg(long)]
    pub stdin: bool,
    /// File path
    #[arg(long)]
    pub file: Option<String>,
    /// Apply changes
    #[arg(long)]
    pub apply: bool,
}

// === ExportContext ===

#[derive(Args)]
pub struct ExportContextArgs {
    /// Output format
    #[arg(long, short, default_value = "json")]
    pub output: String,
}

pub fn run(args: MappingArgs) -> Result<()> {
    let shared = args.shared;
    match args.command {
        MappingCommands::Add(a) => run_add(a, args.config.as_deref(), shared),
        MappingCommands::AddBatch(a) => run_add_batch(a, args.config.as_deref(), shared),
        MappingCommands::Remove(a) => run_remove(a, args.config.as_deref(), shared),
        MappingCommands::List(a) => run_list(a, args.config.as_deref()),
        MappingCommands::Validate(a) => run_validate(a, args.config.as_deref()),
        // MappingCommands::Discover(a) => run_discover(a, args.config.as_deref()),
        MappingCommands::Export(a) => run_export(a, args.config.as_deref()),
        MappingCommands::Import(a) => run_import(a, args.config.as_deref(), shared),
        MappingCommands::ExportContext(a) => run_export_context(a, args.config.as_deref()),
    }
}

fn run_add(args: AddArgs, config_path: Option<&str>, shared: bool) -> Result<()> {
    let mut cm = load_config(config_path)?;

    let lines = args.lines.as_ref().and_then(|l| {
        let parts: Vec<&str> = l.split('-').collect();
        if parts.len() == 2 {
            let start: u32 = parts[0].trim().parse().ok()?;
            let end: u32 = parts[1].trim().parse().ok()?;
            Some([start, end])
        } else {
            None
        }
    });

    let action = args.action.as_deref().map(|a| match a {
        "update" => Action::Update,
        "verify" => Action::Verify,
        "acknowledge" => Action::Acknowledge,
        _ => Action::Review,
    });

    let mapping = MappingDef {
        id: format!("map_{}", &uuid::Uuid::new_v4().to_string()[..8]),
        source: SourceLocator {
            path: args.source.clone(),
            lines,
            anchor: args.anchor.clone(),
            anchor_context: args.anchor_context.clone(),
        },
        asset: args.asset.clone(),
        segment: None,
        reason: args.reason.clone(),
        action,
        confidence: None,
        meta: Some(MappingMeta {
            origin: "manual".to_string(),
            added_at: Some(chrono::Utc::now().to_rfc3339()),
            confidence: None,
        }),
    };

    if !args.apply {
        let validation = cm.validate_mapping(&mapping);
        if args.output == "json" {
            println!(
                "{}",
                serde_json::json!({
                    "status": "dry_run",
                    "valid": validation.valid,
                    "issues": validation.issues,
                    "mapping": mapping,
                    "message": "Use --apply to write."
                })
            );
        } else {
            println!("Dry run — mapping would be added:");
            println!("  ID:     {}", mapping.id);
            println!("  Source: {}", mapping.source.path);
            println!("  Asset:  {}", mapping.asset);
            if !validation.valid {
                println!("  ⚠️ Issues: {:?}", validation.issues);
            }
            println!("\nUse --apply to write.");
        }
        return Ok(());
    }

    let id = cm.add_mapping(mapping)?;
    cm.save_to(shared)?;

    if args.output == "json" {
        println!("{}", serde_json::json!({"status": "added", "id": id}));
    } else {
        println!("✅ Mapping added: {}", id);
    }

    Ok(())
}

fn run_add_batch(args: AddBatchArgs, config_path: Option<&str>, shared: bool) -> Result<()> {
    let mut cm = load_config(config_path)?;

    let input = if args.stdin {
        let mut buf = String::new();
        std::io::stdin().read_to_string(&mut buf)?;
        buf
    } else if let Some(file) = &args.file {
        std::fs::read_to_string(file)?
    } else {
        anyhow::bail!("Please specify --stdin or --file");
    };

    #[derive(serde::Deserialize)]
    struct BatchInput {
        mappings: Vec<BatchMappingInput>,
    }

    #[derive(serde::Deserialize)]
    struct BatchMappingInput {
        source_path: String,
        asset_id: String,
        #[serde(default)]
        reason: Option<String>,
        #[serde(default)]
        source_lines: Option<[u32; 2]>,
        #[serde(default)]
        anchor: Option<String>,
        #[serde(default)]
        anchor_context: Option<String>,
        #[serde(default)]
        segment: Option<serde_json::Value>,
        #[serde(default)]
        action: Option<String>,
    }

    let batch: BatchInput = serde_json::from_str(&input)?;
    let mappings: Vec<MappingDef> = batch
        .mappings
        .into_iter()
        .map(|m| MappingDef {
            id: format!("map_{}", &uuid::Uuid::new_v4().to_string()[..8]),
            source: SourceLocator {
                path: m.source_path,
                lines: m.source_lines,
                anchor: m.anchor,
                anchor_context: m.anchor_context,
            },
            asset: m.asset_id,
            segment: m.segment,
            reason: m.reason,
            action: m.action.as_deref().map(|a| match a {
                "update" => Action::Update,
                "verify" => Action::Verify,
                "acknowledge" => Action::Acknowledge,
                _ => Action::Review,
            }),
            confidence: None,
            meta: Some(MappingMeta {
                origin: "ai-generated".to_string(),
                added_at: Some(chrono::Utc::now().to_rfc3339()),
                confidence: None,
            }),
        })
        .collect();

    let total = mappings.len();

    if !args.apply {
        // Dry run: 校验所有映射
        let mut results = Vec::new();
        for (i, m) in mappings.iter().enumerate() {
            let v = cm.validate_mapping(m);
            results.push(serde_json::json!({
                "index": i,
                "valid": v.valid,
                "id": m.id,
                "issues": v.issues,
            }));
        }
        let valid_count = results.iter().filter(|r| r["valid"].as_bool().unwrap_or(false)).count();
        println!(
            "{}",
            serde_json::json!({
                "status": "dry_run",
                "results": results,
                "summary": {"total": total, "valid": valid_count, "invalid": total - valid_count},
                "message": "All valid. Use --apply to write."
            })
        );
        return Ok(());
    }

    let result = cm.add_mappings_batch(mappings)?;
    cm.save_to(shared)?;

    println!(
        "{}",
        serde_json::json!({
            "status": "applied",
            "added": result.added,
            "errors": result.errors,
            "summary": {"added": result.added.len(), "errors": result.errors.len()}
        })
    );

    Ok(())
}

fn run_remove(args: RemoveArgs, config_path: Option<&str>, shared: bool) -> Result<()> {
    let mut cm = load_config(config_path)?;
    let removed = cm.remove_mapping(&args.id)?;
    cm.save_to(shared)?;

    if args.output == "json" {
        println!(
            "{}",
            serde_json::json!({"status": if removed { "removed" } else { "not_found" }, "id": args.id})
        );
    } else if removed {
        println!("✅ Mapping '{}' removed.", args.id);
    } else {
        println!("⚠️  Mapping '{}' not found.", args.id);
    }

    Ok(())
}

fn run_list(args: ListArgs, config_path: Option<&str>) -> Result<()> {
    let cm = load_config(config_path)?;
    let filter = args.asset.as_ref().map(|a| MappingFilter {
        asset_id: Some(a.clone()),
        source_path: None,
    });
    let mappings = cm.list_mappings(filter.as_ref());

    if args.output == "json" {
        println!("{}", serde_json::to_string_pretty(&mappings)?);
    } else {
        if mappings.is_empty() {
            println!("No mappings defined.");
            return Ok(());
        }
        println!("Mappings ({}):\n", mappings.len());
        for m in &mappings {
            let scope_str = if let Some(ref anchor) = m.source.anchor {
                if let Some(ref ctx) = m.source.anchor_context {
                    format!(" @[{} > {}]", ctx, anchor)
                } else {
                    format!(" @[{}]", anchor)
                }
            } else {
                m.source
                    .lines
                    .map(|l| format!(":{}-{}", l[0], l[1]))
                    .unwrap_or_default()
            };
            println!(
                "  {} {} → asset:{}",
                m.id,
                format!("{}{}", m.source.path, scope_str),
                m.asset
            );
            if let Some(ref reason) = m.reason {
                println!("    reason: {}", reason);
            }
        }
    }

    Ok(())
}

fn run_validate(args: ValidateArgs, config_path: Option<&str>) -> Result<()> {
    let cm = load_config(config_path)?;
    let workspace = workspace_root(cm.path());
    let report = cm.validate_with_workspace(&workspace);

    if args.output == "json" {
        println!(
            "{}",
            serde_json::json!({
                "valid": report.is_valid(),
                "errors": report.errors,
                "warnings": report.warnings,
            })
        );
    } else {
        if report.is_valid() && report.warnings.is_empty() {
            println!("✅ All mappings are valid.");
        } else if report.is_valid() {
            println!("✅ All mappings are valid (with warnings).\n");
        } else {
            println!("❌ Validation failed:\n");
            for err in &report.errors {
                println!("  ERROR: {}", err);
            }
        }
        for warn in &report.warnings {
            println!("  WARNING: {}", warn);
        }
    }

    Ok(())
}

// NOTE: discover 功能暂时关闭，保留实现代码供后续启用
// fn run_discover(args: DiscoverArgs, config_path: Option<&str>) -> Result<()> {
//     let cm = load_config(config_path)?;
//     let workspace = workspace_root(cm.path());
//     let opts = kamap_core::builder::DiscoveryOptions {
//         include_low_confidence: args.include_low_confidence,
//     };
//
//     let candidates = kamap_core::builder::run_discovery(&workspace, cm.config(), &opts)?;
//
//     if args.output == "json" {
//         println!("{}", serde_json::to_string_pretty(&candidates)?);
//     } else {
//         if candidates.is_empty() {
//             println!("No mapping candidates discovered.");
//             return Ok(());
//         }
//         println!("Discovered {} mapping candidates:\n", candidates.len());
//         for c in &candidates {
//             let lines_str = c
//                 .source
//                 .lines
//                 .map(|l| format!(":{}-{}", l[0], l[1]))
//                 .unwrap_or_default();
//             println!(
//                 "  [{:.0}%] {}{} → {} ({:?})",
//                 c.confidence * 100.0,
//                 c.source.path,
//                 lines_str,
//                 c.asset_id,
//                 c.origin
//             );
//             println!("    reason: {}", c.reason);
//         }
//     }
//
//     Ok(())
// }

fn run_export(args: ExportArgs, config_path: Option<&str>) -> Result<()> {
    let cm = load_config(config_path)?;
    let format = match args.format.as_str() {
        "yaml" => Format::Yaml,
        "csv" => Format::Csv,
        _ => Format::Json,
    };
    let output = cm.export_mappings(&format)?;
    println!("{}", output);
    Ok(())
}

fn run_import(args: ImportArgs, config_path: Option<&str>, shared: bool) -> Result<()> {
    let mut cm = load_config(config_path)?;

    let input = if args.stdin {
        let mut buf = String::new();
        std::io::stdin().read_to_string(&mut buf)?;
        buf
    } else if let Some(file) = &args.file {
        std::fs::read_to_string(file)?
    } else {
        anyhow::bail!("Please specify --stdin or --file");
    };

    let format = match args.format.as_str() {
        "yaml" => Format::Yaml,
        _ => Format::Json,
    };

    let strategy = match args.strategy.as_str() {
        "merge" => kamap_core::models::MergeStrategy::Merge,
        "replace" => kamap_core::models::MergeStrategy::Replace,
        _ => kamap_core::models::MergeStrategy::Append,
    };

    if !args.apply {
        println!(
            "{}",
            serde_json::json!({
                "status": "dry_run",
                "message": "Use --apply to write."
            })
        );
        return Ok(());
    }

    let result = cm.import_mappings(&input, &format, strategy)?;
    cm.save_to(shared)?;

    println!(
        "{}",
        serde_json::json!({
            "status": "imported",
            "added": result.added,
            "updated": result.updated,
            "skipped": result.skipped,
        })
    );

    Ok(())
}

fn run_export_context(args: ExportContextArgs, config_path: Option<&str>) -> Result<()> {
    let cm = load_config(config_path)?;
    let workspace = workspace_root(cm.path());
    let opts = ContextOptions {
        include_unmapped: true,
        include_naming_hints: true,
    };

    let context = cm.export_context(&workspace, &opts)?;

    if args.output == "json" {
        println!("{}", serde_json::to_string_pretty(&context)?);
    } else {
        println!("Project Context:");
        println!("  Code files: {}", context.code_files.len());
        println!("  Assets: {}", context.assets.len());
        println!("  Existing mappings: {}", context.existing_mappings.len());
        println!("  Unmapped code files: {}", context.unmapped_code_files.len());
        println!("  Unmapped assets: {}", context.unmapped_assets.len());
    }

    Ok(())
}
