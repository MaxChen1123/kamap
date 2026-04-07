pub mod annotation;
pub mod frontmatter;
pub mod naming;

use std::path::Path;

use anyhow::Result;

use crate::config::ProjectConfig;
use crate::models::SourceLocator;

/// 映射候选来源
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CandidateOrigin {
    NamingConvention,
    CodeAnnotation,
    AssetFrontmatter,
    CoChangeHistory,
    PluginDiscovery,
}

/// 映射候选
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MappingCandidate {
    pub source: SourceLocator,
    pub asset_id: String,
    pub reason: String,
    pub confidence: f32,
    pub origin: CandidateOrigin,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub segment: Option<serde_json::Value>,
}

/// 发现选项
#[derive(Debug, Clone, Default)]
pub struct DiscoveryOptions {
    pub include_low_confidence: bool,
}

/// 映射发现策略 trait
pub trait DiscoveryStrategy: Send + Sync {
    fn name(&self) -> &str;

    fn discover(
        &self,
        workspace: &Path,
        config: &ProjectConfig,
        opts: &DiscoveryOptions,
    ) -> Result<Vec<MappingCandidate>>;
}

/// 运行所有启用的发现策略
pub fn run_discovery(
    workspace: &Path,
    config: &ProjectConfig,
    opts: &DiscoveryOptions,
) -> Result<Vec<MappingCandidate>> {
    let mut all_candidates = Vec::new();

    if config.discovery.annotation.enabled {
        let scanner = annotation::AnnotationScanner::new(&config.discovery.annotation.marker);
        let candidates = scanner.discover(workspace, config, opts)?;
        all_candidates.extend(candidates);
    }

    if config.discovery.frontmatter.enabled {
        let parser = frontmatter::FrontmatterParser::new(&config.discovery.frontmatter.key);
        let candidates = parser.discover(workspace, config, opts)?;
        all_candidates.extend(candidates);
    }

    if config.discovery.naming.enabled {
        let matcher = naming::NamingMatcher::new(&config.discovery.naming.rules);
        let candidates = matcher.discover(workspace, config, opts)?;
        all_candidates.extend(candidates);
    }

    Ok(all_candidates)
}
