# Kamap 插件系统指南

## 一、总览

Kamap 通过插件系统（Plugin System）扩展对不同类型资产（Asset）的支持。每个插件是一个独立的 Rust crate，实现 `AssetPlugin` trait，提供资产校验、健康检查、内容读取、元信息获取等能力。

### 架构一览

```
kamap-core/src/plugin/
├── protocol.rs      # AssetPlugin trait 定义（插件协议）
├── registry.rs      # PluginRegistry 注册表
└── mod.rs

kamap-plugin-localfs/  # 内置插件：本地文件系统
├── src/lib.rs         # LocalFsPlugin
└── src/markdown.rs    # Markdown heading 提取工具

kamap-plugin-sqlite/   # 内置插件：SQLite 数据库
└── src/lib.rs         # SqlitePlugin

kamap-cli/src/commands/
├── mod.rs             # build_plugin_registry() 工厂函数
└── plugin.rs          # kamap plugin list/info 命令
```

---

## 二、插件协议 — `AssetPlugin` trait

**核心实现**：`kamap-core/src/plugin/protocol.rs`

所有插件必须实现 `AssetPlugin` trait（要求 `Send + Sync`）：

```rust
pub trait AssetPlugin: Send + Sync {
    // === 身份标识 ===
    fn provider(&self) -> &str;              // 唯一标识，如 "localfs"
    fn asset_types(&self) -> Vec<String>;    // 支持的资产类型列表
    fn capabilities(&self) -> Capabilities;  // 能力声明

    // === 生命周期 ===
    fn init(&mut self, config: &serde_json::Value) -> Result<()>;

    // === 必选方法 ===
    fn validate_asset(&self, asset: &AssetDef) -> Result<Validation>;

    // === 可选方法（有默认空实现）===
    fn resolve_segment(&self, asset: &AssetDef, segment: &Value) -> Result<Option<SegmentInfo>>;
    fn get_meta(&self, asset: &AssetDef) -> Result<Option<AssetMeta>>;
    fn read_content(&self, asset: &AssetDef, segment: Option<&Value>) -> Result<Option<String>>;
    fn discover_mappings(&self, asset: &AssetDef) -> Result<Vec<MappingCandidate>>;
    fn health_check(&self, asset: &AssetDef) -> Result<HealthStatus>;
}
```

### 2.1 能力声明 — `Capabilities`

每个插件通过 `capabilities()` 方法声明自己支持哪些操作，调用方可在运行时按需调用：

```rust
pub struct Capabilities {
    pub can_resolve_segment: bool,     // 解析资产片段（如 Markdown heading）
    pub can_read_content: bool,        // 读取资产内容
    pub can_discover_mappings: bool,   // 从资产中发现映射关系
    pub can_health_check: bool,        // 健康检查
    pub can_get_meta: bool,            // 获取资产元信息
}
```

### 2.2 方法说明

| 方法 | 必选 | 说明 |
|------|------|------|
| `provider()` | 是 | 返回唯一标识符，与配置中 `assets[].provider` 对应 |
| `asset_types()` | 是 | 返回此插件支持的资产类型列表 |
| `capabilities()` | 是 | 声明插件支持的操作 |
| `init(config)` | 是 | 初始化，接收 `plugins[].config` 中的自定义配置 |
| `validate_asset(asset)` | 是 | 校验资产定义是否合法 |
| `resolve_segment(asset, segment)` | 否 | 解析资产片段信息（如 Markdown heading → `SegmentInfo`） |
| `get_meta(asset)` | 否 | 获取资产元信息（标题、修改时间等） |
| `read_content(asset, segment)` | 否 | 读取资产内容（全文或指定片段） |
| `discover_mappings(asset)` | 否 | 从资产中自动发现映射候选 |
| `health_check(asset)` | 否 | 检查资产健康状态（Healthy / Unhealthy / Unknown） |

### 2.3 关键数据结构

**`Validation`** — 校验结果：

```rust
pub struct Validation {
    pub valid: bool,
    pub message: Option<String>,
}
```

**`AssetMeta`** — 资产元信息：

```rust
pub struct AssetMeta {
    pub title: Option<String>,
    pub last_modified: Option<String>,
    pub owner: Option<String>,
    pub extra: HashMap<String, serde_json::Value>,
}
```

**`HealthStatus`** — 健康状态：

```rust
pub enum HealthStatus {
    Healthy,    // 资产可正常访问
    Unhealthy,  // 资产无法访问（如文件不存在）
    Unknown,    // 无法判断（默认值）
}
```

---

## 三、插件注册表 — `PluginRegistry`

**核心实现**：`kamap-core/src/plugin/registry.rs`

```rust
pub struct PluginRegistry {
    plugins: HashMap<String, Box<dyn AssetPlugin>>,
}
```

注册表以 `provider()` 返回值为 key，提供以下 API：

| 方法 | 说明 |
|------|------|
| `register(plugin)` | 注册插件，自动以 `provider()` 为 key |
| `get(provider)` | 按 provider 名查找插件（不可变引用） |
| `get_mut(provider)` | 按 provider 名查找插件（可变引用） |
| `list()` | 列出所有已注册插件名 |

### 插件装配

当前所有内置插件通过硬编码注册（`kamap-cli/src/commands/mod.rs`）：

```rust
pub fn build_plugin_registry() -> PluginRegistry {
    let mut registry = PluginRegistry::new();
    registry.register(Box::new(LocalFsPlugin::new()));
    registry.register(Box::new(SqlitePlugin::new()));
    registry
}
```

> **注意**：当前不支持动态插件加载。添加新插件需要编写 crate → 在 CLI 的 `Cargo.toml` 加依赖 → 在 `build_plugin_registry()` 中注册 → 重新编译。

---

## 四、内置插件

### 4.1 LocalFS 插件 — `localfs`

**Crate**：`kamap-plugin-localfs`

处理本地文件系统上的资产（Markdown 文档、文本文件、配置文件等）。

| 属性 | 值 |
|------|-----|
| provider | `"localfs"` |
| 资产类型 | `markdown`, `text`, `config` |
| 全部 5 项能力 | 全部支持 |

**各方法行为**：

- **`validate_asset`**：检查 `target` 字段非空
- **`health_check`**：检查 `target` 路径对应的文件是否存在
- **`get_meta`**：读取文件系统元数据，返回文件名作为 `title`、文件修改时间作为 `last_modified`
- **`read_content`**：使用 `fs::read_to_string()` 读取文件全文
- **`resolve_segment`**：对 `markdown` 类型资产，解析 `segment.heading` 字段，返回格式化的 `SegmentInfo`（如 `"## Login Flow"`）
- **`discover_mappings`**：当前返回空（frontmatter 解析由 builder 模块独立处理）

**附带工具**：`markdown.rs` 提供 `extract_heading_section()` 函数，可按 heading slug 提取 Markdown 文档中指定章节的内容和行范围。

### 4.2 SQLite 插件 — `sqlite`

**Crate**：`kamap-plugin-sqlite`

处理 SQLite 数据库文件资产。

| 属性 | 值 |
|------|-----|
| provider | `"sqlite"` |
| 资产类型 | `sqlite-db` |
| resolve_segment | 支持 |
| read_content | **不支持** |
| discover_mappings | **不支持** |
| health_check | 支持 |
| get_meta | 支持 |

**各方法行为**：

- **`validate_asset`**：检查 `target` 字段非空
- **`health_check`**：检查文件存在 + 尝试 `rusqlite::Connection::open()` 确认可正常连接
- **`get_meta`**：打开数据库，查询 `sqlite_master` 表获取所有表名，放入 `extra.tables`
- **`resolve_segment`**：解析 `segment.table` 字段，返回 `"Table: <name>"` 格式的 `SegmentInfo`

---

## 五、配置

### 5.1 `kamap.yaml` 中的 plugins 字段

```yaml
plugins:
  - name: localfs        # 对应 provider() 返回值
    enabled: true        # 是否启用（默认 true）
    config:              # 可选，传给 init() 的自定义配置
      key: value
  - name: sqlite
    enabled: true
```

对应 Schema 定义（`kamap-core/src/config/schema.rs`）：

```rust
pub struct PluginDef {
    pub name: String,
    pub enabled: bool,
    pub config: Option<serde_json::Value>,
}
```

### 5.2 资产定义中的 provider 引用

在 `assets` 中通过 `provider` 字段关联插件：

```yaml
assets:
  - id: auth-doc
    provider: localfs       # ← 对应 LocalFsPlugin
    type: markdown
    target: docs/auth.md
  - id: app-db
    provider: sqlite        # ← 对应 SqlitePlugin
    type: sqlite-db
    target: data/app.db
```

---

## 六、CLI 命令

### 6.1 `kamap plugin list`

列出所有已注册插件及其类型：

```bash
kamap plugin list
# Registered plugins (2):
#   localfs — types: ["markdown", "text", "config"]
#   sqlite  — types: ["sqlite-db"]

kamap plugin list -o json
```

### 6.2 `kamap plugin info`

查看指定插件的详细信息和能力：

```bash
kamap plugin info --name localfs
# Plugin: localfs
#   Types: ["markdown", "text", "config"]
#   Capabilities:
#     Resolve segment:   true
#     Read content:      true
#     Discover mappings: true
#     Health check:      true
#     Get meta:          true
```

### 6.3 `kamap asset check`

通过插件进行资产健康检查：

```bash
kamap asset check
#   ✅ auth-doc (Healthy)
#   ❌ missing-doc (Unhealthy)
#   ❓ unknown-provider (Unknown (no plugin))
```

此命令遍历所有资产，根据 `provider` 字段查找对应插件并调用 `health_check()`。

---

## 七、开发新插件

### 7.1 步骤

1. **创建 crate**：

   ```bash
   cargo new --lib crates/kamap-plugin-myplugin
   ```

2. **添加依赖**（`Cargo.toml`）：

   ```toml
   [dependencies]
   kamap-core = { path = "../kamap-core" }
   serde = { workspace = true }
   serde_json = { workspace = true }
   anyhow = { workspace = true }
   ```

3. **实现 `AssetPlugin` trait**：

   ```rust
   use kamap_core::plugin::protocol::{AssetPlugin, Capabilities, Validation};
   use kamap_core::models::{AssetDef, AssetMeta, HealthStatus, SegmentInfo};

   pub struct MyPlugin;

   impl AssetPlugin for MyPlugin {
       fn provider(&self) -> &str { "myplugin" }

       fn asset_types(&self) -> Vec<String> {
           vec!["my-type".to_string()]
       }

       fn capabilities(&self) -> Capabilities {
           Capabilities {
               can_health_check: true,
               can_get_meta: true,
               ..Default::default()
           }
       }

       fn init(&mut self, _config: &serde_json::Value) -> anyhow::Result<()> {
           Ok(())
       }

       fn validate_asset(&self, asset: &AssetDef) -> anyhow::Result<Validation> {
           Ok(Validation {
               valid: !asset.target.is_empty(),
               message: if asset.target.is_empty() {
                   Some("Target is empty".into())
               } else {
                   None
               },
           })
       }

       // 按需实现其他可选方法...
   }
   ```

4. **在 workspace 中注册**（`Cargo.toml`）：

   ```toml
   [workspace]
   members = [
       # ...
       "crates/kamap-plugin-myplugin",
   ]
   ```

5. **在 CLI 中装配**（`kamap-cli/src/commands/mod.rs`）：

   ```rust
   pub fn build_plugin_registry() -> PluginRegistry {
       let mut registry = PluginRegistry::new();
       registry.register(Box::new(LocalFsPlugin::new()));
       registry.register(Box::new(SqlitePlugin::new()));
       registry.register(Box::new(kamap_plugin_myplugin::MyPlugin::new())); // 新增
       registry
   }
   ```

6. **重新编译**：`cargo build`

### 7.2 设计建议

- `provider()` 名称要全局唯一，建议用小写字母 + 连字符
- 通过 `capabilities()` 准确声明能力，不要谎报
- `validate_asset()` 应尽量快速执行，不做网络/IO 操作
- `health_check()` 可以做轻量 IO（如检查文件存在、尝试连接），但应避免长时间阻塞
- 利用 `init(config)` 接收自定义配置，支持灵活的插件参数化

---

## 八、当前限制

| 限制 | 说明 |
|------|------|
| **静态编译** | 不支持动态加载（.so/.dylib），新插件需修改代码重新编译 |
| **`init()` 未调用** | `build_plugin_registry()` 中未调用 `init()`，`plugins[].config` 不会被传入 |
| **`enabled` 未生效** | 配置中的 `plugins[].enabled` 字段当前未被读取，插件总是被注册 |
| **使用面窄** | 仅 `asset check` 和 `plugin list/info` 命令使用了 `PluginRegistry`；核心的 scan/check/mapping 流程不经过插件 |
| **路径解析** | 插件直接使用 `Path::new(&asset.target)`，不会自动拼接 workspace root |
