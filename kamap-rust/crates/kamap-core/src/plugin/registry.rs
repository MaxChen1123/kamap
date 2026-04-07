use std::collections::HashMap;

use super::protocol::AssetPlugin;

/// 插件注册表
pub struct PluginRegistry {
    plugins: HashMap<String, Box<dyn AssetPlugin>>,
}

impl PluginRegistry {
    pub fn new() -> Self {
        Self {
            plugins: HashMap::new(),
        }
    }

    /// 注册插件
    pub fn register(&mut self, plugin: Box<dyn AssetPlugin>) {
        let name = plugin.provider().to_string();
        self.plugins.insert(name, plugin);
    }

    /// 获取插件
    pub fn get(&self, provider: &str) -> Option<&dyn AssetPlugin> {
        self.plugins.get(provider).map(|p| p.as_ref())
    }

    /// 获取可变插件
    pub fn get_mut(&mut self, provider: &str) -> Option<&mut Box<dyn AssetPlugin>> {
        self.plugins.get_mut(provider)
    }

    /// 列出所有已注册插件
    pub fn list(&self) -> Vec<&str> {
        self.plugins.keys().map(|k| k.as_str()).collect()
    }
}

impl Default for PluginRegistry {
    fn default() -> Self {
        Self::new()
    }
}
