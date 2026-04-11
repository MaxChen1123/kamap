use std::path::{Path, PathBuf};
use std::fs::OpenOptions;

use anyhow::{Context, Result};
use fs2::FileExt;

use crate::models::{AssetDef, AssetFilter, BatchResult, MappingDef, MappingFilter, MappingUpdate, MergeStrategy};

use super::schema::ProjectConfig;

/// 配置校验结果
#[derive(Debug, Clone)]
pub struct ValidationReport {
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}

impl ValidationReport {
    pub fn is_valid(&self) -> bool {
        self.errors.is_empty()
    }
}

/// 单个映射校验结果
#[derive(Debug, Clone)]
pub struct ValidationResult {
    pub valid: bool,
    pub issues: Vec<String>,
}

/// 导入结果
#[derive(Debug, Clone)]
pub struct ImportResult {
    pub added: usize,
    pub updated: usize,
    pub skipped: usize,
}

/// 输出格式
#[derive(Debug, Clone)]
pub enum Format {
    Json,
    Yaml,
    Csv,
}

/// AI 上下文导出选项
#[derive(Debug, Clone, Default)]
pub struct ContextOptions {
    pub include_unmapped: bool,
    pub include_naming_hints: bool,
}

/// 项目上下文（导出给 AI）
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ProjectContext {
    pub code_files: Vec<CodeFileInfo>,
    pub assets: Vec<AssetSummary>,
    pub existing_mappings: Vec<MappingSummary>,
    pub unmapped_code_files: Vec<String>,
    pub unmapped_assets: Vec<String>,
    pub naming_hints: Vec<NamingHint>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CodeFileInfo {
    pub path: String,
    #[serde(rename = "type")]
    pub file_type: String,
    pub size: u64,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AssetSummary {
    pub id: String,
    pub provider: String,
    pub target: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MappingSummary {
    pub source: String,
    pub asset: String,
    pub reason: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct NamingHint {
    pub pattern: String,
    pub likely_asset: String,
}

/// 读写双向的配置管理器
///
/// 支持双配置文件：
/// - `kamap.yaml` — 团队/仓库共享配置，提交到 Git
/// - `.kamap.yaml` — 开发者个人配置，不提交到 Git
///
/// 加载时两个文件会合并，`.kamap.yaml` 中的内容追加到 `kamap.yaml` 之后（个人覆盖共享）。
/// 写入操作默认写入 `.kamap.yaml`（个人），可通过 `--shared` 标志写入 `kamap.yaml`（共享）。
pub struct ConfigManager {
    /// 主配置文件路径（用于写入的默认目标，优先 .kamap.yaml）
    path: PathBuf,
    /// 共享配置文件路径 (kamap.yaml)，如果存在
    shared_path: Option<PathBuf>,
    /// 个人配置文件路径 (.kamap.yaml)，如果存在
    local_path: Option<PathBuf>,
    /// 合并后的运行时配置
    config: ProjectConfig,
    /// 共享配置（原始，用于写入时拆分）
    shared_config: Option<ProjectConfig>,
    /// 个人配置（原始，用于写入时拆分）
    local_config: Option<ProjectConfig>,
}

impl ConfigManager {
    /// 从单个文件加载配置（向后兼容）
    pub fn load(path: &Path) -> Result<Self> {
        let content = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read config file: {}", path.display()))?;
        let config: ProjectConfig = serde_yaml::from_str(&content)
            .with_context(|| "Failed to parse YAML config")?;
        Ok(Self {
            path: path.to_path_buf(),
            shared_path: Some(path.to_path_buf()),
            local_path: None,
            shared_config: Some(config.clone()),
            local_config: None,
            config,
        })
    }

    /// 加载并合并双配置文件
    ///
    /// - `shared_path`: `kamap.yaml`（团队共享）
    /// - `local_path`: `.kamap.yaml`（个人本地）
    ///
    /// 两个文件都可选，但至少要有一个成功加载。
    /// 如果其中一个文件损坏（格式错误、为空等），会打印警告并继续使用另一个正常的配置。
    /// 只有两个文件都无法加载时才返回错误。
    /// 合并规则：assets、mappings、plugins、policies 追加合并，个人配置的 discovery 覆盖共享的。
    pub fn load_merged(shared_path: Option<&Path>, local_path: Option<&Path>) -> Result<Self> {
        let mut shared_error: Option<String> = None;
        let mut local_error: Option<String> = None;

        let shared_config = if let Some(p) = shared_path {
            if p.exists() {
                match std::fs::read_to_string(p) {
                    Ok(content) => {
                        match serde_yaml::from_str::<ProjectConfig>(&content) {
                            Ok(config) => Some(config),
                            Err(e) => {
                                shared_error = Some(format!(
                                    "Failed to parse shared config ({}): {}",
                                    p.display(), e
                                ));
                                None
                            }
                        }
                    }
                    Err(e) => {
                        shared_error = Some(format!(
                            "Failed to read shared config ({}): {}",
                            p.display(), e
                        ));
                        None
                    }
                }
            } else {
                None
            }
        } else {
            None
        };

        let local_config = if let Some(p) = local_path {
            if p.exists() {
                match std::fs::read_to_string(p) {
                    Ok(content) => {
                        match serde_yaml::from_str::<ProjectConfig>(&content) {
                            Ok(config) => Some(config),
                            Err(e) => {
                                local_error = Some(format!(
                                    "Failed to parse local config ({}): {}",
                                    p.display(), e
                                ));
                                None
                            }
                        }
                    }
                    Err(e) => {
                        local_error = Some(format!(
                            "Failed to read local config ({}): {}",
                            p.display(), e
                        ));
                        None
                    }
                }
            } else {
                None
            }
        } else {
            None
        };

        // 如果两个配置都无法加载，返回错误（包含具体的失败原因）
        if shared_config.is_none() && local_config.is_none() {
            let mut reasons = Vec::new();
            if let Some(e) = &shared_error {
                reasons.push(e.clone());
            }
            if let Some(e) = &local_error {
                reasons.push(e.clone());
            }
            if reasons.is_empty() {
                anyhow::bail!("No config file found. Run `kamap init` to create kamap.yaml");
            } else {
                anyhow::bail!(
                    "All config files failed to load:\n  {}",
                    reasons.join("\n  ")
                );
            }
        }

        // 如果只有一个文件出错，打印警告但继续使用另一个正常的配置
        if let Some(e) = &shared_error {
            eprintln!("⚠️  {}", e);
            eprintln!("   Continuing with local config (.kamap.yaml) only.");
        }
        if let Some(e) = &local_error {
            eprintln!("⚠️  {}", e);
            eprintln!("   Continuing with shared config (kamap.yaml) only.");
        }

        // 合并配置
        let merged = Self::merge_configs(shared_config.as_ref(), local_config.as_ref());

        // 默认写入目标：优先个人配置路径 (.kamap.yaml)
        let write_path = local_path
            .map(|p| p.to_path_buf())
            .or_else(|| shared_path.map(|p| p.to_path_buf()))
            .unwrap();

        Ok(Self {
            path: write_path,
            shared_path: shared_path.map(|p| p.to_path_buf()),
            local_path: local_path.map(|p| p.to_path_buf()),
            shared_config,
            local_config,
            config: merged,
        })
    }

    /// 合并两个配置：shared（基础） + local（追加/覆盖）
    fn merge_configs(shared: Option<&ProjectConfig>, local: Option<&ProjectConfig>) -> ProjectConfig {
        match (shared, local) {
            (Some(s), None) => s.clone(),
            (None, Some(l)) => l.clone(),
            (Some(s), Some(l)) => {
                let mut merged = s.clone();

                // plugins: 追加 local 中 shared 没有的插件
                for lp in &l.plugins {
                    if !merged.plugins.iter().any(|p| p.name == lp.name) {
                        merged.plugins.push(lp.clone());
                    }
                }

                // assets: 追加 local 中 shared 没有的资产，同 ID 的 local 覆盖
                for la in &l.assets {
                    if let Some(existing) = merged.assets.iter_mut().find(|a| a.id == la.id) {
                        *existing = la.clone();
                    } else {
                        merged.assets.push(la.clone());
                    }
                }

                // mappings: 追加 local 中 shared 没有的映射，同 ID 的 local 覆盖
                for lm in &l.mappings {
                    if let Some(existing) = merged.mappings.iter_mut().find(|m| m.id == lm.id) {
                        *existing = lm.clone();
                    } else {
                        merged.mappings.push(lm.clone());
                    }
                }

                // policies: 直接追加
                merged.policies.extend(l.policies.iter().cloned());

                // discovery: local 覆盖 shared
                merged.discovery = l.discovery.clone();

                merged
            }
            (None, None) => ProjectConfig::default(),
        }
    }

    /// 创建默认配置
    pub fn new_default(path: &Path) -> Self {
        Self {
            path: path.to_path_buf(),
            shared_path: Some(path.to_path_buf()),
            local_path: None,
            shared_config: None,
            local_config: None,
            config: ProjectConfig::default(),
        }
    }

    /// 保存配置到文件（默认写入个人配置 .kamap.yaml）
    pub fn save(&self) -> Result<()> {
        let content = serde_yaml::to_string(&self.config)
            .with_context(|| "Failed to serialize config")?;
        std::fs::write(&self.path, content)
            .with_context(|| format!("Failed to write config file: {}", self.path.display()))?;
        Ok(())
    }

    /// 保存到指定的配置文件
    ///
    /// - `shared = false`（默认）：写入 `.kamap.yaml`（个人配置）
    /// - `shared = true`：写入 `kamap.yaml`（团队共享配置）
    ///
    /// 当双配置文件同时存在时，只写入**目标层应有的内容**：
    /// - 写入 local 时：从合并后的 config 中减去 shared 层的内容
    /// - 写入 shared 时：从合并后的 config 中减去 local 层的内容
    /// 如果只有单个配置文件，则直接写入完整的 config。
    pub fn save_to(&self, shared: bool) -> Result<()> {
        let target = if shared {
            self.shared_path.as_ref().ok_or_else(|| anyhow::anyhow!("No shared config path (kamap.yaml) set"))?
        } else {
            self.local_path.as_ref().unwrap_or(&self.path)
        };

        // 确定要保存的配置内容
        let config_to_save = if shared {
            // 写 shared：如果有 local 层，从 config 中剔除 local 独有的内容
            match &self.local_config {
                Some(local_orig) => Self::subtract_layer(&self.config, local_orig),
                None => self.config.clone(),
            }
        } else {
            // 写 local：如果有 shared 层，从 config 中剔除 shared 已有的内容
            match &self.shared_config {
                Some(shared_orig) => Self::subtract_layer(&self.config, shared_orig),
                None => self.config.clone(),
            }
        };

        let content = serde_yaml::to_string(&config_to_save)
            .with_context(|| "Failed to serialize config")?;
        std::fs::write(target, content)
            .with_context(|| format!("Failed to write config file: {}", target.display()))?;
        Ok(())
    }

    /// 带文件锁的原子「加载→修改→保存」操作。
    ///
    /// 使用排他锁（exclusive lock）确保同一时刻只有一个进程可以修改配置文件，
    /// 避免并发写入导致数据丢失。
    ///
    /// `shared_path` / `local_path`：配置文件路径（与 `load_merged` 相同）
    /// `shared`：写入目标（true=kamap.yaml, false=.kamap.yaml）
    /// `modify_fn`：在持有锁期间对 ConfigManager 进行修改的闭包
    pub fn locked_modify<F>(
        shared_path: Option<&Path>,
        local_path: Option<&Path>,
        shared: bool,
        modify_fn: F,
    ) -> Result<Self>
    where
        F: FnOnce(&mut Self) -> Result<()>,
    {
        // 确定锁文件路径：在目标配置文件旁边创建 .lock 文件
        let lock_dir = shared_path
            .or(local_path)
            .and_then(|p| p.parent())
            .unwrap_or(Path::new("."));
        let lock_file_path = lock_dir.join(".kamap.lock");

        // 打开（或创建）锁文件并获取排他锁
        let lock_file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(false)
            .open(&lock_file_path)
            .with_context(|| format!("Failed to open lock file: {}", lock_file_path.display()))?;

        lock_file.lock_exclusive()
            .with_context(|| "Failed to acquire exclusive lock on config")?;

        // 在锁保护下：加载最新配置
        let mut cm = Self::load_merged(shared_path, local_path)?;

        // 执行修改
        let result = modify_fn(&mut cm);

        if result.is_ok() {
            // 保存修改后的配置
            cm.save_to(shared)?;
        }

        // 释放锁
        lock_file.unlock()
            .with_context(|| "Failed to release config lock")?;

        result?;
        Ok(cm)
    }

    /// 从合并后的 config 中减去另一层的原始内容，得到当前层的增量。
    ///
    /// - plugins: 移除 other 中已有的插件
    /// - assets: 移除与 other 中完全相同的资产（同 ID 同内容），保留新增或修改的
    /// - mappings: 同上
    /// - policies: 保留全部（无法精确区分来源）
    /// - discovery: 如果与 other 相同则使用默认值，否则保留
    fn subtract_layer(merged: &ProjectConfig, other: &ProjectConfig) -> ProjectConfig {
        let mut result = merged.clone();

        // plugins: 移除 other 中已有的（按 name 匹配）
        result.plugins.retain(|p| !other.plugins.iter().any(|op| op.name == p.name));

        // assets: 移除与 other 中完全相同的（同 ID + 同 target），保留新增或修改的
        result.assets.retain(|a| {
            !other.assets.iter().any(|oa| oa.id == a.id && oa.target == a.target
                && oa.provider == a.provider && oa.asset_type == a.asset_type)
        });

        // mappings: 移除 other 中已有的（按 ID 匹配，且 source path 相同）
        result.mappings.retain(|m| {
            !other.mappings.iter().any(|om| om.id == m.id && om.source.path == m.source.path)
        });

        // policies: 保留全部（无法精确区分来源，追加模式）
        // discovery: 保留当前值（写入方拥有 discovery 的完整控制权）

        result
    }

    /// 获取配置的不可变引用
    pub fn config(&self) -> &ProjectConfig {
        &self.config
    }

    /// 获取配置路径（默认写入目标）
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// 获取共享配置路径
    pub fn shared_path(&self) -> Option<&Path> {
        self.shared_path.as_deref()
    }

    /// 获取个人配置路径
    pub fn local_path(&self) -> Option<&Path> {
        self.local_path.as_deref()
    }

    // === Asset CRUD ===

    /// 列出资产
    pub fn list_assets(&self, filter: Option<&AssetFilter>) -> Vec<&AssetDef> {
        self.config
            .assets
            .iter()
            .filter(|a| {
                if let Some(f) = filter {
                    if let Some(ref provider) = f.provider {
                        if &a.provider != provider {
                            return false;
                        }
                    }
                    if let Some(ref asset_type) = f.asset_type {
                        if &a.asset_type != asset_type {
                            return false;
                        }
                    }
                }
                true
            })
            .collect()
    }

    /// 获取单个资产
    pub fn get_asset(&self, id: &str) -> Option<&AssetDef> {
        self.config.assets.iter().find(|a| a.id == id)
    }

    /// 添加资产
    pub fn add_asset(&mut self, asset: AssetDef) -> Result<()> {
        if self.config.assets.iter().any(|a| a.id == asset.id) {
            anyhow::bail!("Asset with id '{}' already exists", asset.id);
        }
        self.config.assets.push(asset);
        Ok(())
    }

    /// 移除资产
    pub fn remove_asset(&mut self, id: &str) -> Result<bool> {
        let len_before = self.config.assets.len();
        self.config.assets.retain(|a| a.id != id);
        Ok(self.config.assets.len() < len_before)
    }

    // === Mapping CRUD ===

    /// 列出映射
    pub fn list_mappings(&self, filter: Option<&MappingFilter>) -> Vec<&MappingDef> {
        self.config
            .mappings
            .iter()
            .filter(|m| {
                if let Some(f) = filter {
                    if let Some(ref asset_id) = f.asset_id {
                        if &m.asset != asset_id {
                            return false;
                        }
                    }
                    if let Some(ref source_path) = f.source_path {
                        if &m.source.path != source_path {
                            return false;
                        }
                    }
                }
                true
            })
            .collect()
    }

    /// 获取单个映射
    pub fn get_mapping(&self, id: &str) -> Option<&MappingDef> {
        self.config.mappings.iter().find(|m| m.id == id)
    }

    /// 添加单个映射，返回映射 ID
    pub fn add_mapping(&mut self, mapping: MappingDef) -> Result<String> {
        // 校验资产是否存在
        if self.get_asset(&mapping.asset).is_none() {
            anyhow::bail!("Asset '{}' not found", mapping.asset);
        }
        let id = mapping.id.clone();
        self.config.mappings.push(mapping);
        Ok(id)
    }

    /// 批量添加映射
    pub fn add_mappings_batch(&mut self, mappings: Vec<MappingDef>) -> Result<BatchResult> {
        let mut result = BatchResult {
            added: vec![],
            skipped: vec![],
            errors: vec![],
        };

        for (idx, mapping) in mappings.into_iter().enumerate() {
            if self.get_asset(&mapping.asset).is_none() {
                result.errors.push((idx, format!("Asset '{}' not found", mapping.asset)));
                continue;
            }
            let id = mapping.id.clone();
            self.config.mappings.push(mapping);
            result.added.push(id);
        }

        Ok(result)
    }

    /// 移除映射
    pub fn remove_mapping(&mut self, id: &str) -> Result<bool> {
        let len_before = self.config.mappings.len();
        self.config.mappings.retain(|m| m.id != id);
        Ok(self.config.mappings.len() < len_before)
    }

    /// 更新映射
    pub fn update_mapping(&mut self, id: &str, update: MappingUpdate) -> Result<()> {
        let mapping = self
            .config
            .mappings
            .iter_mut()
            .find(|m| m.id == id)
            .ok_or_else(|| anyhow::anyhow!("Mapping '{}' not found", id))?;

        if let Some(reason) = update.reason {
            mapping.reason = Some(reason);
        }
        if let Some(action) = update.action {
            mapping.action = Some(action);
        }
        if let Some(confidence) = update.confidence {
            mapping.confidence = Some(confidence);
        }
        if let Some(segment) = update.segment {
            mapping.segment = Some(segment);
        }

        Ok(())
    }

    // === 校验 ===

    /// 校验整个配置
    pub fn validate(&self) -> ValidationReport {
        let mut report = ValidationReport {
            errors: vec![],
            warnings: vec![],
        };

        // 检查映射引用的资产是否存在
        for mapping in &self.config.mappings {
            if !self.config.assets.iter().any(|a| a.id == mapping.asset) {
                report.errors.push(format!(
                    "Mapping '{}' references non-existent asset '{}'",
                    mapping.id, mapping.asset
                ));
            }
        }

        // 检查资产 ID 唯一性
        let mut seen_ids = std::collections::HashSet::new();
        for asset in &self.config.assets {
            if !seen_ids.insert(&asset.id) {
                report.errors.push(format!("Duplicate asset id: '{}'", asset.id));
            }
        }

        // 检查映射 source path 不为空
        for mapping in &self.config.mappings {
            if mapping.source.path.is_empty() {
                report.warnings.push(format!(
                    "Mapping '{}' has empty source path",
                    mapping.id
                ));
            }
        }

        report
    }

    /// 校验单个映射
    pub fn validate_mapping(&self, mapping: &MappingDef) -> ValidationResult {
        let mut issues = vec![];

        if mapping.source.path.is_empty() {
            issues.push("Source path is empty".to_string());
        }
        if mapping.asset.is_empty() {
            issues.push("Asset id is empty".to_string());
        }
        if !self.config.assets.iter().any(|a| a.id == mapping.asset) {
            issues.push(format!("Asset '{}' not found", mapping.asset));
        }
        if let Some(ref lines) = mapping.source.lines {
            if lines[0] > lines[1] {
                issues.push("Line range start > end".to_string());
            }
        }

        ValidationResult {
            valid: issues.is_empty(),
            issues,
        }
    }

    // === 导入导出 ===

    /// 导出映射
    pub fn export_mappings(&self, format: &Format) -> Result<String> {
        match format {
            Format::Json => {
                serde_json::to_string_pretty(&self.config.mappings)
                    .with_context(|| "Failed to serialize mappings to JSON")
            }
            Format::Yaml => {
                serde_yaml::to_string(&self.config.mappings)
                    .with_context(|| "Failed to serialize mappings to YAML")
            }
            Format::Csv => {
                let mut lines = vec!["id,source_path,source_lines,asset,reason,action".to_string()];
                for m in &self.config.mappings {
                    let lines_str = m
                        .source
                        .lines
                        .map(|l| format!("{}-{}", l[0], l[1]))
                        .unwrap_or_default();
                    lines.push(format!(
                        "{},{},{},{},{},{}",
                        m.id,
                        m.source.path,
                        lines_str,
                        m.asset,
                        m.reason.as_deref().unwrap_or(""),
                        m.action
                            .as_ref()
                            .map(|a| format!("{:?}", a).to_lowercase())
                            .unwrap_or_default()
                    ));
                }
                Ok(lines.join("\n"))
            }
        }
    }

    /// 导入映射
    pub fn import_mappings(
        &mut self,
        data: &str,
        format: &Format,
        strategy: MergeStrategy,
    ) -> Result<ImportResult> {
        let incoming: Vec<MappingDef> = match format {
            Format::Json => serde_json::from_str(data)?,
            Format::Yaml => serde_yaml::from_str(data)?,
            Format::Csv => {
                anyhow::bail!("CSV import not yet supported");
            }
        };

        let mut result = ImportResult {
            added: 0,
            updated: 0,
            skipped: 0,
        };

        match strategy {
            MergeStrategy::Replace => {
                result.added = incoming.len();
                self.config.mappings = incoming;
            }
            MergeStrategy::Append => {
                result.added = incoming.len();
                self.config.mappings.extend(incoming);
            }
            MergeStrategy::Merge => {
                for m in incoming {
                    if let Some(existing) = self.config.mappings.iter_mut().find(|e| e.id == m.id) {
                        *existing = m;
                        result.updated += 1;
                    } else {
                        self.config.mappings.push(m);
                        result.added += 1;
                    }
                }
            }
        }

        Ok(result)
    }

    // === AI 上下文导出 ===

    /// 导出项目上下文供 AI 使用
    pub fn export_context(&self, workspace: &Path, _opts: &ContextOptions) -> Result<ProjectContext> {
        // 收集代码文件
        let mut code_files = vec![];
        let src_dir = workspace.join("src");
        if src_dir.exists() {
            collect_code_files(&src_dir, workspace, &mut code_files)?;
        }

        // 资产摘要
        let assets: Vec<AssetSummary> = self
            .config
            .assets
            .iter()
            .map(|a| AssetSummary {
                id: a.id.clone(),
                provider: a.provider.clone(),
                target: a.target.clone(),
            })
            .collect();

        // 已有映射摘要
        let existing_mappings: Vec<MappingSummary> = self
            .config
            .mappings
            .iter()
            .map(|m| MappingSummary {
                source: m.source.path.clone(),
                asset: m.asset.clone(),
                reason: m.reason.clone(),
            })
            .collect();

        // 找出未映射的代码文件
        let mapped_paths: std::collections::HashSet<&str> = self
            .config
            .mappings
            .iter()
            .map(|m| m.source.path.as_str())
            .collect();
        let unmapped_code_files: Vec<String> = code_files
            .iter()
            .filter(|f| !mapped_paths.contains(f.path.as_str()))
            .map(|f| f.path.clone())
            .collect();

        // 找出未映射的资产
        let mapped_assets: std::collections::HashSet<&str> = self
            .config
            .mappings
            .iter()
            .map(|m| m.asset.as_str())
            .collect();
        let unmapped_assets: Vec<String> = self
            .config
            .assets
            .iter()
            .filter(|a| !mapped_assets.contains(a.id.as_str()))
            .map(|a| a.id.clone())
            .collect();

        Ok(ProjectContext {
            code_files,
            assets,
            existing_mappings,
            unmapped_code_files,
            unmapped_assets,
            naming_hints: vec![],
        })
    }
}

fn collect_code_files(dir: &Path, base: &Path, out: &mut Vec<CodeFileInfo>) -> Result<()> {
    if !dir.is_dir() {
        return Ok(());
    }
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            collect_code_files(&path, base, out)?;
        } else if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
            let file_type = match ext {
                "rs" => "rust",
                "ts" | "tsx" => "typescript",
                "js" | "jsx" => "javascript",
                "py" => "python",
                "go" => "go",
                "java" => "java",
                "md" => "markdown",
                _ => continue,
            };
            let rel = path.strip_prefix(base).unwrap_or(&path);
            let size = std::fs::metadata(&path).map(|m| m.len()).unwrap_or(0);
            out.push(CodeFileInfo {
                path: crate::path_util::to_forward_slash(rel),
                file_type: file_type.to_string(),
                size,
            });
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::SourceLocator;

    fn sample_config() -> ConfigManager {
        let mut cm = ConfigManager::new_default(Path::new("kamap.yaml"));
        cm.add_asset(AssetDef {
            id: "test-doc".to_string(),
            provider: "localfs".to_string(),
            asset_type: "markdown".to_string(),
            target: "docs/test.md".to_string(),
            meta: Default::default(),
        })
        .unwrap();
        cm
    }

    #[test]
    fn test_add_remove_asset() {
        let mut cm = sample_config();
        assert_eq!(cm.list_assets(None).len(), 1);
        assert!(cm.get_asset("test-doc").is_some());
        assert!(cm.remove_asset("test-doc").unwrap());
        assert_eq!(cm.list_assets(None).len(), 0);
    }

    #[test]
    fn test_add_mapping() {
        let mut cm = sample_config();
        let id = cm
            .add_mapping(MappingDef {
                id: "m1".to_string(),
                source: SourceLocator {
                    path: "src/**/*.rs".to_string(),
                    lines: None,
                    anchor: None,
                    anchor_context: None,
                },
                asset: "test-doc".to_string(),
                segment: None,
                reason: Some("test".to_string()),
                action: None,
                confidence: None,
                meta: None,
            })
            .unwrap();
        assert_eq!(id, "m1");
        assert_eq!(cm.list_mappings(None).len(), 1);
    }

    #[test]
    fn test_add_mapping_missing_asset() {
        let mut cm = sample_config();
        let result = cm.add_mapping(MappingDef {
            id: "m1".to_string(),
            source: SourceLocator {
                path: "src/**/*.rs".to_string(),
                lines: None,
                anchor: None,
                anchor_context: None,
            },
            asset: "nonexistent".to_string(),
            segment: None,
            reason: None,
            action: None,
            confidence: None,
            meta: None,
        });
        assert!(result.is_err());
    }

    #[test]
    fn test_validate() {
        let mut cm = sample_config();
        cm.config.mappings.push(MappingDef {
            id: "bad".to_string(),
            source: SourceLocator {
                path: "src/x.rs".to_string(),
                lines: None,
                anchor: None,
                anchor_context: None,
            },
            asset: "nonexistent".to_string(),
            segment: None,
            reason: None,
            action: None,
            confidence: None,
            meta: None,
        });
        let report = cm.validate();
        assert!(!report.is_valid());
        assert!(report.errors[0].contains("nonexistent"));
    }
}
