use anyhow::Result;
use clap::Args;

#[derive(Args)]
pub struct DescribeArgs {
    /// Output format: text, json
    #[arg(long, short, default_value = "json")]
    pub output: String,
}

pub fn run(args: DescribeArgs) -> Result<()> {
    let description = serde_json::json!({
        "name": "kamap",
        "version": env!("CARGO_PKG_VERSION"),
        "description": "Knowledge Asset Mapping — Git-based impact analysis framework",
        "commands": [
            {
                "name": "scan",
                "description": "Scan for impacted assets based on Git changes. Writes results to .kamap/to-ack.json. Previously acknowledged impacts (same HEAD) are filtered.",
                "params": {
                    "base": {"type": "string", "default": "origin/main", "description": "Base Git ref"},
                    "head": {"type": "string", "default": "HEAD", "description": "Head Git ref"},
                    "output": {"type": "string", "enum": ["text", "json"], "default": "text"}
                }
            },
            {
                "name": "scan ack",
                "description": "Acknowledge impacts as handled (document synced). Marks entries in .kamap/to-ack.json. Next scan at same HEAD will skip acknowledged items.",
                "params": {
                    "all": {"type": "boolean", "description": "Acknowledge all pending impacts"},
                    "ids": {"type": "string", "description": "Comma-separated mapping IDs to acknowledge"},
                    "output": {"type": "string", "enum": ["text", "json"]}
                }
            },
            {
                "name": "check",
                "description": "Policy check for CI (exits non-zero on error-severity impacts)",
                "params": {
                    "base": {"type": "string", "default": "origin/main"},
                    "head": {"type": "string", "default": "HEAD"},
                    "output": {"type": "string", "enum": ["text", "json"]}
                }
            },
            {
                "name": "mapping add",
                "description": "Add a single mapping",
                "params": {
                    "source": {"type": "string", "required": true, "description": "Source file path or glob"},
                    "asset": {"type": "string", "required": true, "description": "Asset ID"},
                    "reason": {"type": "string", "description": "Reason for the mapping"},
                    "lines": {"type": "string", "description": "Line range (e.g., 10-45)"},
                    "action": {"type": "string", "enum": ["review", "update", "verify", "acknowledge"]},
                    "dry-run": {"type": "boolean", "default": true},
                    "shared": {"type": "boolean", "default": false, "description": "Write to kamap.yaml (shared) instead of .kamap.yaml (personal)"}
                }
            },
            {
                "name": "mapping add-batch",
                "description": "Batch add mappings from JSON stdin",
                "params": {
                    "stdin": {"type": "boolean", "description": "Read JSON from stdin"},
                    "dry-run": {"type": "boolean", "default": true},
                    "apply": {"type": "boolean", "description": "Actually write changes"},
                    "shared": {"type": "boolean", "default": false, "description": "Write to kamap.yaml (shared) instead of .kamap.yaml (personal)"}
                }
            },
            {
                "name": "mapping list",
                "description": "List all mappings",
                "params": {
                    "asset": {"type": "string", "description": "Filter by asset ID"},
                    "output": {"type": "string", "enum": ["text", "json"]}
                }
            },
            {
                "name": "mapping remove",
                "description": "Remove a mapping by ID",
                "params": {
                    "id": {"type": "string", "required": true}
                }
            },
            {
                "name": "mapping validate",
                "description": "Validate all mappings"
            },
            {
                "name": "mapping discover",
                "description": "Auto-discover mapping candidates from annotations, frontmatter, naming conventions"
            },
            {
                "name": "mapping export-context",
                "description": "Export project context for AI analysis"
            },
            {
                "name": "asset add",
                "description": "Register a new asset"
            },
            {
                "name": "asset list",
                "description": "List all registered assets"
            },
            {
                "name": "asset remove",
                "description": "Remove an asset"
            },
            {
                "name": "asset check",
                "description": "Health check all assets"
            },
            {
                "name": "explain",
                "description": "Explain a mapping, asset, or source relationship"
            },
            {
                "name": "index build",
                "description": "Build or rebuild the runtime index"
            },
            {
                "name": "index stats",
                "description": "Show index statistics"
            }
        ]
    });

    if args.output == "json" {
        println!("{}", serde_json::to_string_pretty(&description)?);
    } else {
        println!("kamap — Knowledge Asset Mapping\n");
        println!("Available commands:");
        if let Some(commands) = description.get("commands").and_then(|c| c.as_array()) {
            for cmd in commands {
                let name = cmd.get("name").and_then(|n| n.as_str()).unwrap_or("?");
                let desc = cmd
                    .get("description")
                    .and_then(|d| d.as_str())
                    .unwrap_or("");
                println!("  {:<30} {}", name, desc);
            }
        }
        println!("\nUse --output json for machine-readable output.");
    }

    Ok(())
}
