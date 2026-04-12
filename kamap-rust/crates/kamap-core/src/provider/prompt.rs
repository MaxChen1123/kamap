use crate::config::ProviderDef;
use crate::models::{Action, AssetDef, SourceMatch};

/// prompt 渲染上下文
pub struct PromptContext<'a> {
    pub asset: &'a AssetDef,
    pub source: &'a SourceMatch,
    pub reason: &'a str,
    pub action: &'a Action,
    pub mapping_id: &'a str,
}

/// 根据 provider 定义和上下文渲染 action prompt
pub fn render_action_prompt(provider: &ProviderDef, ctx: &PromptContext) -> String {
    if let Some(ref template) = provider.prompt_template {
        render_template(template, ctx)
    } else {
        builtin_prompt(&provider.name, ctx)
    }
}

/// 查找 provider 定义，如果配置中没有则 fallback 到内置默认
pub fn resolve_provider<'a>(providers: &'a [ProviderDef], name: &str) -> Option<&'a ProviderDef> {
    providers.iter().find(|p| p.name == name)
}

/// 为未在 providers 中显式定义的内置 provider 生成默认定义
pub fn default_provider(name: &str) -> ProviderDef {
    ProviderDef {
        name: name.to_string(),
        prompt_template: None,
    }
}

// ─── 内置 prompt ───

fn builtin_prompt(name: &str, ctx: &PromptContext) -> String {
    let source_str = format_source(ctx.source);
    let action_str = format_action(ctx.action);

    match name {
        "localfs" => format!(
            "代码变更影响了本地文件 {target}。\n\n\
             变更来源: {source}\n\
             影响原因: {reason}\n\
             建议操作: {action}\n\n\
             请直接读取 {target} 并根据代码变更进行更新。",
            target = ctx.asset.target,
            source = source_str,
            reason = ctx.reason,
            action = action_str,
        ),
        "sqlite" => format!(
            "代码变更影响了 SQLite 数据库 {target}。\n\n\
             变更来源: {source}\n\
             影响原因: {reason}\n\
             建议操作: {action}\n\n\
             请检查是否需要更新 schema 或数据。",
            target = ctx.asset.target,
            source = source_str,
            reason = ctx.reason,
            action = action_str,
        ),
        _ => format!(
            "代码变更影响了资产 {id} ({target})。\n\n\
             Provider: {provider}\n\
             变更来源: {source}\n\
             影响原因: {reason}\n\
             建议操作: {action}",
            id = ctx.asset.id,
            target = ctx.asset.target,
            provider = name,
            source = source_str,
            reason = ctx.reason,
            action = action_str,
        ),
    }
}

// ─── 模板渲染（简单 {{var}} 替换） ───

fn render_template(template: &str, ctx: &PromptContext) -> String {
    let source_str = format_source(ctx.source);
    let action_str = format_action(ctx.action);

    let mut result = template.to_string();

    // 基础变量
    result = result.replace("{{asset.id}}", &ctx.asset.id);
    result = result.replace("{{asset.target}}", &ctx.asset.target);
    result = result.replace("{{asset.type}}", &ctx.asset.asset_type);
    result = result.replace("{{asset.provider}}", &ctx.asset.provider);
    result = result.replace("{{source.path}}", &source_str);
    result = result.replace("{{reason}}", ctx.reason);
    result = result.replace("{{action}}", &action_str);
    result = result.replace("{{mapping_id}}", ctx.mapping_id);

    // meta 字段：支持 {{asset.meta.xxx}} 格式
    for (key, value) in &ctx.asset.meta {
        let placeholder = format!("{{{{asset.meta.{}}}}}", key);
        let value_str = match value {
            serde_json::Value::String(s) => s.clone(),
            other => other.to_string(),
        };
        result = result.replace(&placeholder, &value_str);
    }

    // source 子字段
    match ctx.source {
        SourceMatch::WholeFile { path } => {
            result = result.replace("{{source.file}}", path);
            result = result.replace("{{source.hunks}}", "(whole file)");
        }
        SourceMatch::LineRange { path, matched_hunks } => {
            result = result.replace("{{source.file}}", path);
            let hunks: Vec<String> = matched_hunks
                .iter()
                .map(|h| format!("L{}-{}", h.start_line, h.end_line))
                .collect();
            result = result.replace("{{source.hunks}}", &hunks.join(", "));
        }
    }

    result
}

fn format_source(source: &SourceMatch) -> String {
    match source {
        SourceMatch::WholeFile { path } => path.clone(),
        SourceMatch::LineRange { path, matched_hunks } => {
            let hunks: Vec<String> = matched_hunks
                .iter()
                .map(|h| format!("{}-{}", h.start_line, h.end_line))
                .collect();
            format!("{}:{}", path, hunks.join(","))
        }
    }
}

fn format_action(action: &Action) -> String {
    match action {
        Action::Update => "update".to_string(),
        Action::Review => "review".to_string(),
        Action::Verify => "verify".to_string(),
        Action::Acknowledge => "acknowledge".to_string(),
        Action::Custom(s) => s.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn make_asset(provider: &str, target: &str) -> AssetDef {
        AssetDef {
            id: "test-asset".to_string(),
            provider: provider.to_string(),
            asset_type: "markdown".to_string(),
            target: target.to_string(),
            meta: HashMap::new(),
        }
    }

    fn make_source() -> SourceMatch {
        SourceMatch::WholeFile {
            path: "src/auth/login.rs".to_string(),
        }
    }

    #[test]
    fn test_builtin_localfs_prompt() {
        let provider = ProviderDef {
            name: "localfs".to_string(),
            prompt_template: None,
        };
        let asset = make_asset("localfs", "README.md");
        let source = make_source();
        let ctx = PromptContext {
            asset: &asset,
            source: &source,
            reason: "README 相关代码变更",
            action: &Action::Update,
            mapping_id: "map_123",
        };
        let prompt = render_action_prompt(&provider, &ctx);
        assert!(prompt.contains("README.md"));
        assert!(prompt.contains("src/auth/login.rs"));
        assert!(prompt.contains("update"));
    }

    #[test]
    fn test_custom_template() {
        let provider = ProviderDef {
            name: "iwiki".to_string(),
            prompt_template: Some(
                "Update iwiki doc {{asset.meta.title}} (ID: {{asset.target}}). Source: {{source.path}}, Reason: {{reason}}".to_string()
            ),
        };
        let mut meta = HashMap::new();
        meta.insert(
            "title".to_string(),
            serde_json::Value::String("Auth Design".to_string()),
        );
        let asset = AssetDef {
            id: "auth-doc".to_string(),
            provider: "iwiki".to_string(),
            asset_type: "document".to_string(),
            target: "12345678".to_string(),
            meta,
        };
        let source = make_source();
        let ctx = PromptContext {
            asset: &asset,
            source: &source,
            reason: "login function changed",
            action: &Action::Update,
            mapping_id: "map_456",
        };
        let prompt = render_action_prompt(&provider, &ctx);
        assert!(prompt.contains("Auth Design"));
        assert!(prompt.contains("12345678"));
        assert!(prompt.contains("src/auth/login.rs"));
        assert!(prompt.contains("login function changed"));
    }

    #[test]
    fn test_fallback_prompt_for_unknown_builtin() {
        let provider = ProviderDef {
            name: "unknown-provider".to_string(),
            prompt_template: None,
        };
        let asset = make_asset("unknown-provider", "some/target");
        let source = make_source();
        let ctx = PromptContext {
            asset: &asset,
            source: &source,
            reason: "test",
            action: &Action::Review,
            mapping_id: "map_789",
        };
        let prompt = render_action_prompt(&provider, &ctx);
        assert!(prompt.contains("test-asset"));
        assert!(prompt.contains("unknown-provider"));
    }
}
