use std::path::Path;

use anyhow::Result;

use crate::config::schema::NamingRule;
use crate::config::ProjectConfig;
use crate::models::SourceLocator;

use super::{CandidateOrigin, DiscoveryOptions, DiscoveryStrategy, MappingCandidate};

/// 基于命名约定的映射发现
pub struct NamingMatcher {
    rules: Vec<NamingRule>,
}

impl NamingMatcher {
    pub fn new(rules: &[NamingRule]) -> Self {
        Self {
            rules: rules.to_vec(),
        }
    }
}

impl DiscoveryStrategy for NamingMatcher {
    fn name(&self) -> &str {
        "naming"
    }

    fn discover(
        &self,
        workspace: &Path,
        config: &ProjectConfig,
        _opts: &DiscoveryOptions,
    ) -> Result<Vec<MappingCandidate>> {
        let mut candidates = Vec::new();

        for rule in &self.rules {
            // 简单实现：将 {module} 提取出来做匹配
            // 例如 source: "src/{module}/**" → asset_pattern: "docs/{module}.md"
            if let Some(module_start) = rule.source.find("{module}") {
                let prefix = &rule.source[..module_start];
                let src_prefix = workspace.join(prefix);
                if src_prefix.exists() && src_prefix.is_dir() {
                    for entry in std::fs::read_dir(&src_prefix)? {
                        let entry = entry?;
                        if entry.path().is_dir() {
                            let module_name = entry.file_name().to_string_lossy().to_string();
                            let asset_target =
                                rule.asset_pattern.replace("{module}", &module_name);

                            // 检查资产是否存在
                            if let Some(asset) = config.assets.iter().find(|a| a.target == asset_target) {
                                candidates.push(MappingCandidate {
                                    source: SourceLocator {
                                        path: rule.source.replace("{module}", &module_name),
                                        lines: None,
                                    },
                                    asset_id: asset.id.clone(),
                                    reason: format!(
                                        "Naming convention: {} → {}",
                                        module_name, asset_target
                                    ),
                                    confidence: 0.6,
                                    origin: CandidateOrigin::NamingConvention,
                                    segment: None,
                                });
                            }
                        }
                    }
                }
            }
        }

        Ok(candidates)
    }
}
