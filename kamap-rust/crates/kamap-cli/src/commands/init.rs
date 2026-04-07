use anyhow::Result;
use clap::Args;

use kamap_core::config::ConfigManager;

use super::{SHARED_CONFIG_NAME, LOCAL_CONFIG_NAME};

#[derive(Args)]
pub struct InitArgs {
    /// Output format: text, json
    #[arg(long, default_value = "text")]
    pub output: String,
}

pub fn run(args: InitArgs) -> Result<()> {
    let config_path = std::path::Path::new(SHARED_CONFIG_NAME);

    if config_path.exists() {
        if args.output == "json" {
            println!(
                "{}",
                serde_json::json!({
                    "status": "exists",
                    "message": "kamap.yaml already exists"
                })
            );
        } else {
            println!("⚠️  kamap.yaml already exists. Skipping initialization.");
        }
        return Ok(());
    }

    let cm = ConfigManager::new_default(config_path);
    cm.save()?;

    // 创建 .kamap/ 工作目录
    std::fs::create_dir_all(".kamap")?;

    if args.output == "json" {
        println!(
            "{}",
            serde_json::json!({
                "status": "created",
                "config_file": SHARED_CONFIG_NAME,
                "local_config_file": LOCAL_CONFIG_NAME,
                "work_dir": ".kamap/"
            })
        );
    } else {
        println!("✅ kamap initialized!");
        println!("   Created: kamap.yaml          (shared, commit to Git)");
        println!("   Created: .kamap/             (working directory)");
        println!();
        println!("💡 Tip: Create .kamap.yaml for personal/local config (not committed to Git).");
        println!();
        println!("Next steps:");
        println!("   1. Define assets:     kamap asset add --id my-doc --provider localfs --type markdown --target docs/my-doc.md");
        println!("   2. Add mappings:      kamap mapping add --source 'src/**/*.ts' --asset my-doc --reason 'implementation'");
        println!("   3. Scan for impacts:  kamap scan");
    }

    Ok(())
}
