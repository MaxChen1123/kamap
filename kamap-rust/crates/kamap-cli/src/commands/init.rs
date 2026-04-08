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
    let shared_path = std::path::Path::new(SHARED_CONFIG_NAME);
    let local_path = std::path::Path::new(LOCAL_CONFIG_NAME);

    if shared_path.exists() || local_path.exists() {
        if args.output == "json" {
            println!(
                "{}",
                serde_json::json!({
                    "status": "exists",
                    "message": "Config file(s) already exist"
                })
            );
        } else {
            println!("⚠️  Config file(s) already exist. Skipping initialization.");
        }
        return Ok(());
    }

    // 创建共享配置 (kamap.yaml) — 仅包含 plugins 和 discovery 等团队共享设置
    let cm = ConfigManager::new_default(shared_path);
    cm.save()?;

    // 创建个人配置 (.kamap.yaml) — 默认写入目标
    let local_cm = ConfigManager::new_default(local_path);
    local_cm.save()?;

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
        println!("   Created: .kamap.yaml         (personal, NOT committed to Git)");
        println!("   Created: .kamap/             (working directory)");
        println!();
        println!("💡 Default: all commands write to .kamap.yaml (personal config).");
        println!("   Use --shared to write to kamap.yaml (team config) instead.");
        println!();
        println!("Next steps:");
        println!("   1. Define assets:     kamap asset add --id my-doc --provider localfs --type markdown --target docs/my-doc.md");
        println!("   2. Add mappings:      kamap mapping add --source 'src/**/*.ts' --asset my-doc --reason 'implementation'");
        println!("   3. Scan for impacts:  kamap scan");
    }

    Ok(())
}
