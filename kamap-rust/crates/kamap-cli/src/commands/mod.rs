pub mod init;
pub mod scan;
pub mod check;
pub mod explain;
pub mod describe;
pub mod mapping;
pub mod asset;
pub mod index;
pub mod plugin;
pub mod provider;

use std::path::{Path, PathBuf};

use kamap_core::config::ConfigManager;
use kamap_core::plugin::PluginRegistry;

/// 共享配置文件名（团队/仓库共用，提交到 Git）
pub const SHARED_CONFIG_NAME: &str = "kamap.yaml";
/// 个人配置文件名（开发者自己的索引，不提交到 Git）
pub const LOCAL_CONFIG_NAME: &str = ".kamap.yaml";

/// 查找配置文件所在的目录（从当前目录向上查找）
///
/// 返回找到的目录路径，如果没找到则返回当前目录。
/// 查找规则：向上遍历目录，找到第一个包含 `kamap.yaml` 或 `.kamap.yaml` 的目录。
pub fn find_config_dir() -> PathBuf {
    let mut dir = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    loop {
        if dir.join(SHARED_CONFIG_NAME).exists() || dir.join(LOCAL_CONFIG_NAME).exists() {
            return dir;
        }
        if !dir.pop() {
            break;
        }
    }
    std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
}

/// 加载配置（支持双配置合并）
///
/// 如果指定了 `config_path`，则仅加载该文件（向后兼容）。
/// 否则自动查找 `kamap.yaml`（共享）和 `.kamap.yaml`（个人），合并加载。
/// 默认写入目标为 `.kamap.yaml`（个人配置），使用 `--shared` 写入 `kamap.yaml`。
pub fn load_config(config_path: Option<&str>) -> anyhow::Result<ConfigManager> {
    if let Some(path) = config_path {
        // 指定了具体路径，直接加载单个文件
        ConfigManager::load(&PathBuf::from(path))
    } else {
        let dir = find_config_dir();
        let shared = dir.join(SHARED_CONFIG_NAME);
        let local = dir.join(LOCAL_CONFIG_NAME);

        let shared_path = if shared.exists() { Some(shared.as_path()) } else { None };
        let local_path = if local.exists() { Some(local.as_path()) } else { None };

        ConfigManager::load_merged(shared_path, local_path)
    }
}

/// 构建默认的插件注册表
pub fn build_plugin_registry() -> PluginRegistry {
    let mut registry = PluginRegistry::new();
    registry.register(Box::new(kamap_plugin_localfs::LocalFsPlugin::new()));
    registry.register(Box::new(kamap_plugin_sqlite::SqlitePlugin::new()));
    registry
}

/// 获取 workspace 根目录
pub fn workspace_root(config_path: &Path) -> PathBuf {
    config_path.parent().unwrap_or(Path::new(".")).to_path_buf()
}
