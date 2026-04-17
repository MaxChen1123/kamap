# kamap — Knowledge Asset Mapping

基于 Git 的知识资产映射与影响分析框架。kamap 将源代码变更与知识资产（文档、数据库、配置等）关联起来，在代码发生变更时自动识别受影响的资产，提醒开发者同步更新。

## 核心概念

| 概念 | 说明 |
|------|------|
| **Asset（资产）** | 需要与代码保持同步的知识资产，如文档、数据库、配置文件等 |
| **Mapping（映射）** | 源代码路径/行范围/语义锚点与资产之间的关联关系 |
| **Anchor（锚点）** | 语义锚点，通过文本特征（如 `fn login`、`class AuthService`）动态定位代码块，避免行号漂移问题 |
| **Impact（影响）** | 当代码变更命中映射规则时产生的影响报告，包含变更类型（added/modified/deleted/renamed）等信息 |
| **Ack（确认）** | 开发者确认已同步文档后的标记，避免重复提醒 |
| **Provider（提供者）** | 定义影响检测后如何生成操作指引（`action_prompt`），内置 `localfs`/`sqlite`，可自定义扩展 |
| **Plugin（插件）** | 不同资产类型的处理器，负责健康检查、内容读取、元信息获取等（v1 兼容，逐步由 Provider 替代） |
| **Policy（策略）** | 定义影响的严重程度规则 |

## 快速开始

```bash
# 1. 初始化项目
kamap init

# 2. 注册资产
kamap asset add --id my-doc --provider localfs --type markdown --target docs/my-doc.md --apply

# 3. 添加映射（整文件）
kamap mapping add --source 'src/**/*.rs' --asset my-doc --reason '实现代码' --apply

# 3b. 添加映射（使用语义锚点定位特定代码块）
kamap mapping add --source src/auth.rs --asset my-doc --anchor 'fn login' --reason '登录逻辑' --apply

# 4. 扫描影响（默认对比最近一次提交与工作区）
kamap scan

# 5. 确认已同步的影响
kamap scan ack --all
```

## 项目结构

```
kamap.yaml           # 共享配置文件（团队/仓库共用，提交到 Git）
.kamap.yaml          # 个人配置文件（开发者私有，不提交到 Git）
.kamap/              # 工作目录
  ├── index.db       # SQLite 运行时索引
  └── to-ack.json    # scan 结果与确认状态（绑定到 HEAD commit）
```

### 双配置文件设计

| 文件 | 用途 | 是否提交 Git |
|------|------|-------------|
| `kamap.yaml` | 团队/仓库共享的资产与映射索引 | ✅ 是 |
| `.kamap.yaml` | 开发者个人的资产与映射索引 | ❌ 否 |

加载时两个文件会**自动合并**，`.kamap.yaml` 中的内容优先级更高（个人覆盖共享）。合并规则：
- **assets / mappings**: 同 ID 则个人覆盖共享，不同 ID 则追加
- **providers**: 同 name 则个人覆盖共享，不同 name 则追加
- **plugins**: 追加不重复的
- **policies**: 直接追加
- **discovery**: 个人配置整体覆盖共享配置

## 命令参考

所有命令均支持 `--output text`（默认）和 `--output json` 两种输出格式（部分命令默认 json）。

---

### `kamap init`

初始化 kamap 项目，创建 `kamap.yaml` 配置文件和 `.kamap/` 工作目录。

```bash
kamap init
kamap init --output json
```

---

### `kamap scan`

扫描 Git 变更，匹配映射规则，生成影响报告。默认对比最近一次提交（HEAD）与工作区（含 staged + unstaged），即检测"当前未提交的改动"影响了哪些文档。

```bash
kamap scan                              # 默认对比 HEAD..workdir
kamap scan --base origin/main           # 对比 origin/main..workdir
kamap scan --base main --head HEAD      # 对比 main..HEAD（仅已提交的变更）
kamap scan -o json                      # JSON 输出
kamap scan --config path/to/kamap.yaml  # 指定配置文件
```

| 参数 | 默认值 | 说明 |
|------|--------|------|
| `--base` | `HEAD` | 基准 Git ref |
| `--head` | `workdir` | 目标 Git ref，`workdir` 表示工作区（含 staged + unstaged） |
| `--output` / `-o` | `text` | 输出格式 |
| `--config` | 自动查找 | 配置文件路径 |

**工作流程**: Git diff → 映射引擎匹配（含 anchor 动态解析）→ 影响分析（含变更类型传递）→ Provider 渲染 `action_prompt` → 过滤已确认项 → 写入 `.kamap/to-ack.json` → 输出报告

扫描结果会自动写入 `.kamap/to-ack.json`，记录每个影响的确认状态。同一 HEAD commit 下已确认的影响不会重复显示。

每个影响条目包含 `change_type` 字段（`added`/`modified`/`deleted`/`renamed`），标识触发影响的 Git 变更类型，便于区分文件新增、修改、删除等不同场景。

---

### `kamap scan ack`

确认影响已处理（文档已同步），标记 `.kamap/to-ack.json` 中的条目为已确认。下次在同一 HEAD 上扫描时，已确认的影响将被过滤。

```bash
kamap scan ack --all                    # 确认所有待处理影响
kamap scan ack --ids map_abc123,map_def456  # 确认指定映射 ID
kamap scan ack --all -o json            # JSON 输出
```

| 参数 | 说明 |
|------|------|
| `--all` | 确认所有待处理影响 |
| `--ids` | 逗号分隔的映射 ID 列表 |
| `--output` / `-o` | 输出格式（默认 `text`） |
| `--config` | 配置文件路径 |

---

### `kamap check`

策略检查（CI 友好）。与 `scan` 功能相同，但当存在 **error** 级别影响时以非零退出码退出，适合集成到 CI 流水线。

```bash
kamap check
kamap check --base origin/main --head HEAD -o json
```

| 参数 | 默认值 | 说明 |
|------|--------|------|
| `--base` | `HEAD` | 基准 Git ref |
| `--head` | `workdir` | 目标 Git ref，`workdir` 表示工作区 |
| `--output` / `-o` | `text` | 输出格式 |
| `--config` | 自动查找 | 配置文件路径 |

---

### `kamap explain`

解释映射、资产或源文件的关联关系。三种查询模式，必须指定其一：

```bash
kamap explain --mapping map_abc123     # 查看映射详情
kamap explain --asset my-doc           # 查看资产及其所有关联映射
kamap explain --source src/auth/mod.rs # 查看源文件的所有关联映射
```

| 参数 | 说明 |
|------|------|
| `--mapping` | 要解释的 Mapping ID |
| `--asset` | 要解释的 Asset ID |
| `--source` | 要解释的源文件路径 |
| `--output` / `-o` | 输出格式（默认 `text`） |
| `--config` | 配置文件路径 |

---

### `kamap describe`

输出工具自描述信息，供 AI Agent 消费。

```bash
kamap describe              # 默认 JSON 输出
kamap describe -o text      # 文本格式
```

---

### `kamap mapping` — 映射管理

#### `kamap mapping add`

添加单个映射。默认 dry-run 模式，需 `--apply` 才实际写入。默认写入个人配置（`.kamap.yaml`），使用 `--shared` 写入共享配置（`kamap.yaml`）。

```bash
# 整文件映射
kamap mapping add \
  --source 'src/auth/**/*.ts' \
  --asset auth-doc \
  --reason '认证模块实现' \
  --action review \
  --apply

# 语义锚点映射（推荐，避免行号漂移）
kamap mapping add \
  --source src/auth.rs \
  --asset auth-doc \
  --anchor 'fn login' \
  --reason '登录函数实现' \
  --action update \
  --apply

# 带锚点上下文消歧（文件中有多个同名 fn new 时）
kamap mapping add \
  --source src/auth.rs \
  --asset token-doc \
  --anchor 'fn new' \
  --anchor-context 'impl Token' \
  --reason 'Token 构造函数' \
  --action update \
  --apply

# 静态行范围（不推荐，行号会漂移）
kamap mapping add \
  --source src/auth.ts \
  --asset auth-doc \
  --lines '10-45' \
  --apply

# 写入共享配置
kamap mapping add --source src/main.rs --asset readme --shared --apply
```

| 参数 | 必填 | 说明 |
|------|------|------|
| `--source` | 是 | 源文件路径或 glob 模式 |
| `--asset` | 是 | 目标 Asset ID |
| `--reason` | 否 | 映射原因 |
| `--anchor` | 否 | 语义锚点文本（如 `"fn login"`），用于动态定位代码块 |
| `--anchor-context` | 否 | 锚点上下文文本（如 `"impl Token"`），用于消歧 |
| `--lines` | 否 | 静态行范围，如 `"10-45"`（不推荐，优先用 anchor） |
| `--action` | 否 | 推荐动作: `review` / `update` / `verify` / `acknowledge` |
| `--apply` | 否 | 实际写入（默认 dry-run） |
| `--shared` | 否 | 写入共享配置 `kamap.yaml`（全局标志） |

#### `kamap mapping add-batch`

从 JSON 批量添加映射（适合 AI 生成）。默认写入个人配置，使用 `--shared` 写入共享配置。

```bash
# 从 stdin
echo '{"mappings":[...]}' | kamap mapping add-batch --stdin --apply

# 从文件
kamap mapping add-batch --file mappings.json --apply

# 写入共享配置
echo '{"mappings":[...]}' | kamap mapping --shared add-batch --stdin --apply
```

JSON 输入格式：
```json
{
  "mappings": [
    {
      "source_path": "src/foo.rs",
      "asset_id": "my-doc",
      "reason": "实现代码",
      "anchor": "fn handle_request",
      "anchor_context": "impl Server",
      "action": "update"
    },
    {
      "source_path": "src/config.rs",
      "asset_id": "config-doc",
      "reason": "配置模块（整文件）",
      "action": "review"
    },
    {
      "source_path": "src/legacy.rs",
      "asset_id": "legacy-doc",
      "source_lines": [10, 45],
      "segment": {"heading": "Authentication"},
      "action": "review"
    }
  ]
}
```

| 字段 | 必填 | 说明 |
|------|------|------|
| `source_path` | 是 | 源文件路径或 glob |
| `asset_id` | 是 | 目标 Asset ID |
| `reason` | 否 | 映射原因 |
| `anchor` | 否 | 语义锚点文本（推荐），动态定位代码块 |
| `anchor_context` | 否 | 锚点上下文，用于消歧 |
| `source_lines` | 否 | `[start, end]` 静态行范围（不推荐，优先用 anchor） |
| `segment` | 否 | 资产片段信息（由插件解释） |
| `action` | 否 | 推荐动作: `review` / `update` / `verify` / `acknowledge` |
```

#### `kamap mapping remove`

```bash
kamap mapping remove --id map_abc123
```

#### `kamap mapping list`

```bash
kamap mapping list                  # 列出所有映射
kamap mapping list --asset my-doc   # 按资产过滤
kamap mapping list -o json          # JSON 输出
```

#### `kamap mapping validate`

校验所有映射定义的有效性：
- 资产引用是否存在
- 路径是否为空
- 行范围是否合法
- **anchor 有效性**：对精确路径（非 glob）映射，检查 anchor 是否能在当前文件中找到。glob 映射的 anchor 无法静态校验，会输出 warning 提示。

```bash
kamap mapping validate
```

#### ~~`kamap mapping discover`~~ （暂时关闭）

> **注意**：`mapping discover` 子命令目前已暂时关闭，CLI 入口不可用。底层发现策略（Annotation、Frontmatter、Naming Convention）的实现代码仍保留，后续需要时可重新启用。

~~自动发现映射候选。支持三种策略（详见下文"映射发现"章节）。~~

```bash
# 以下命令当前不可用
# kamap mapping discover
# kamap mapping discover --include-low-confidence
# kamap mapping discover -o json
```

#### `kamap mapping export`

```bash
kamap mapping export --format json   # JSON（默认）
kamap mapping export --format yaml   # YAML
kamap mapping export --format csv    # CSV
```

#### `kamap mapping import`

```bash
kamap mapping import --file mappings.json --strategy append --apply
kamap mapping import --stdin --strategy merge --apply
```

| 合并策略 | 说明 |
|----------|------|
| `append` | 追加（默认） |
| `merge` | 按 ID 合并，已存在则更新 |
| `replace` | 完全替换现有映射 |

#### `kamap mapping export-context`

导出项目上下文供 AI 分析使用，包含代码文件列表、资产、已有映射、未映射的代码/资产。

```bash
kamap mapping export-context -o json
```

---

### `kamap asset` — 资产管理

默认写入个人配置（`.kamap.yaml`），使用 `--shared` 全局标志写入共享配置（`kamap.yaml`）。

#### `kamap asset add`

注册新资产。默认 dry-run 模式。

```bash
kamap asset add \
  --id my-doc \
  --provider localfs \
  --type markdown \
  --target docs/my-doc.md \
  --apply

# 写入共享配置
kamap asset --shared add --id my-doc --provider localfs --type markdown --target docs/my-doc.md --apply
```

| 参数 | 必填 | 说明 |
|------|------|------|
| `--id` | 是 | 资产唯一 ID |
| `--provider` | 是 | 插件 provider（`localfs` / `sqlite`） |
| `--type` | 是 | 资产类型（`markdown` / `text` / `config` / `sqlite-db`） |
| `--target` | 是 | 目标路径 |
| `--apply` | 否 | 实际写入（默认 dry-run） |
| `--shared` | 否 | 写入共享配置 `kamap.yaml`（全局标志） |

#### `kamap asset add-batch`

从 JSON 批量注册资产（原子操作，避免并发写入冲突）。

```bash
# 从 stdin
echo '{"assets":[
  {"id":"readme-zh","provider":"localfs","type":"markdown","target":"README.md"},
  {"id":"readme-en","provider":"localfs","type":"markdown","target":"README_en.md"}
]}' | kamap asset add-batch --stdin --apply

# 从文件
kamap asset add-batch --file assets.json --apply

# 写入共享配置
echo '{"assets":[...]}' | kamap asset --shared add-batch --stdin --apply
```

JSON 输入格式：
```json
{
  "assets": [
    {"id": "my-doc", "provider": "localfs", "type": "markdown", "target": "docs/my-doc.md"}
  ]
}
```

#### `kamap asset remove`

```bash
kamap asset remove --id my-doc
```

#### `kamap asset list`

```bash
kamap asset list
kamap asset list -o json
```

#### `kamap asset check`

对所有注册资产执行健康检查。

```bash
kamap asset check
```

输出示例：
```
  ✅ my-doc (Healthy)
  ❌ old-doc (Unhealthy)
```

---

### `kamap index` — 索引管理

#### `kamap index build`

将 YAML 配置构建为 SQLite 运行时索引（`.kamap/index.db`）。

```bash
kamap index build
```

#### `kamap index stats`

查看索引统计信息。

```bash
kamap index stats
```

---

### `kamap provider` — Provider 管理

#### `kamap provider list`

列出所有 provider（内置 + 配置中自定义的）。

```bash
kamap provider list
kamap provider list -o json
```

#### `kamap provider info`

查看 provider 详情（类型、prompt 模板、关联资产数）。

```bash
kamap provider info --name localfs
kamap provider info --name notion -o json
```

---

### `kamap plugin` — 插件管理（已废弃）

> **注意**：`plugin` 命令已废弃，请使用 `kamap provider` 替代。保留以兼容旧版本。

#### `kamap plugin list`

列出所有已注册插件。

```bash
kamap plugin list
kamap plugin list -o json
```

#### `kamap plugin info`

查看插件详细能力。

```bash
kamap plugin info --name localfs
```

---

## Provider 系统

Provider 定义了 kamap 在检测到影响时如何生成操作指引（`action_prompt`）。

### 工作原理

当 `kamap scan` 检测到影响时：
1. 查找资产的 `provider` 名称
2. 在 `providers` 配置中找到匹配的定义
3. 使用 `prompt_template`（或内置默认 prompt）渲染 `action_prompt`
4. 将 `action_prompt` 包含在 scan 输出中

### 内置 Provider

| Provider | 资产类型 | 默认 Prompt |
|----------|----------|-------------|
| `localfs` | `markdown`, `text`, `config` | 请直接读取 {target} 并根据代码变更进行更新 |
| `sqlite` | `sqlite-db` | 请检查是否需要更新 schema 或数据 |

内置 Provider 无需配置 `prompt_template`，也可通过配置覆盖默认 prompt。

### 自定义 Provider

在 `kamap.yaml` 中定义自定义 provider，通过 `prompt_template` 提供操作指引模板：

```yaml
providers:
  - name: notion
    prompt_template: |
      代码变更影响了 Notion 页面「{{asset.meta.title}}」(页面 ID: {{asset.target}})。
      请通过 Notion MCP 完成操作。
```

模板变量：`{{asset.id}}`、`{{asset.target}}`、`{{asset.type}}`、`{{asset.provider}}`、`{{asset.meta.*}}`、`{{source.path}}`、`{{source.file}}`、`{{source.hunks}}`、`{{reason}}`、`{{action}}`、`{{mapping_id}}`、`{{change_type}}`。

---

## 内置插件（v1 兼容）

> **注意**：插件系统为 v1 遗留功能，逐步由 Provider 系统替代。插件仍负责健康检查、内容读取等底层能力。

### `localfs` — 本地文件系统

| 项目 | 值 |
|------|------|
| 支持资产类型 | `markdown`, `text`, `config` |
| Segment 解析 | Markdown heading（`{"heading": "xxx"}`） |
| 健康检查 | 文件是否存在 |
| 内容读取 | 支持 |

### `sqlite` — SQLite 数据库

| 项目 | 值 |
|------|------|
| 支持资产类型 | `sqlite-db` |
| Segment 解析 | 表名（`{"table": "xxx"}`） |
| 健康检查 | 文件存在且可用 rusqlite 打开 |
| 内容读取 | 不支持 |
| Meta 信息 | 返回数据库中所有表名 |

---

## 映射发现（Discovery）（暂时关闭）

> **注意**：`mapping discover` 子命令目前已暂时关闭，CLI 入口不可用。底层发现策略的实现代码仍保留，以下文档供参考。

~~`kamap mapping discover` 命令支持三种自动发现策略：~~

### 1. 代码注释扫描（Annotation）

在代码注释中使用 `@kamap` 标记声明映射关系。

```rust
// @kamap asset:auth-doc reason:"认证逻辑" segment:{"heading":"Login"}
fn login() { ... }
```

支持的注释格式：`//`、`#`、`/* */`、`--`

扫描文件类型：`.rs`, `.ts`, `.tsx`, `.js`, `.jsx`, `.py`, `.go`, `.java`, `.rb`, `.c`, `.cpp`, `.h`

**置信度**: 0.9 | **默认开启**

### 2. Markdown Frontmatter

在 Markdown 文档的 frontmatter 中声明关联关系。

```markdown
---
kamap:
  relates-to:
    - path: src/auth/**/*.ts
      reason: "认证模块实现"
      lines: "10-45"
      segment:
        heading: "Authentication"
---
```

**置信度**: 0.85 | **默认开启**

### 3. 命名约定（Naming Convention）

通过目录命名规则自动匹配。

```yaml
# kamap.yaml 中配置
discovery:
  naming:
    enabled: true
    rules:
      - source: "src/{module}/**"
        asset_pattern: "docs/{module}.md"
```

**置信度**: 0.6 | **默认关闭**

---

## 配置文件格式

`kamap.yaml` 示例：

```yaml
version: "1"

providers:
  - name: notion
    prompt_template: |
      代码变更影响了 Notion 页面「{{asset.meta.title}}」(页面 ID: {{asset.target}})。
      变更来源: {{source.path}}
      影响原因: {{reason}}
      建议操作: {{action}}
      请通过 Notion MCP 读取并更新页面。

plugins:
  - name: localfs
    enabled: true
  - name: sqlite
    enabled: true

assets:
  - id: auth-doc
    provider: localfs
    type: markdown
    target: docs/auth.md

mappings:
  # 整文件映射
  - id: map_abc123
    source:
      path: "src/auth/**/*.rs"
    asset: auth-doc
    reason: "认证模块实现代码"
    action: review

  # 使用语义锚点的映射（推荐）
  - id: map_def456
    source:
      path: src/auth/login.rs
      anchor: "fn login"
    asset: auth-doc
    reason: "登录函数"
    action: update

  # 使用锚点上下文消歧
  - id: map_ghi789
    source:
      path: src/auth/token.rs
      anchor: "fn new"
      anchor_context: "impl Token"
    asset: auth-doc
    reason: "Token 构造函数"
    action: update

policies:
  - match:
      asset_priority: high
    severity: error
```

---

## 语义锚点（Anchor）

语义锚点是 kamap 的核心特性之一，解决了静态行号随代码插入/删除而漂移的问题。

### 工作原理

在 `scan` 时，映射引擎对配置了 `anchor` 的映射执行动态解析：

1. 读取源文件当前内容
2. 在文件中搜索包含 anchor 文本的行
3. 如果配置了 `anchor_context`，先定位 context 块，再在其中搜索 anchor
4. 从 anchor 行向上扩展（包含注释、attribute、decorator）
5. 向下检测代码块边界（花括号匹配 / 缩进检测，自动适配 Rust/Go/JS/Python 等）
6. 返回动态解析的行范围用于 hunk overlap 检测

### 优先级

映射匹配时行范围的解析优先级：

1. **anchor** → 动态解析行范围（推荐）
2. **static lines** → 使用配置中的固定行号（不推荐）
3. **无** → 整文件匹配

### 支持的语言

- **花括号语言**（Rust, Go, JavaScript, TypeScript, Java, C, C++）：通过 `{` `}` 计数确定块边界
- **缩进语言**（Python, YAML）：通过缩进层级确定块边界

---

## 项目架构

```
kamap-rust/
├── crates/
│   ├── kamap-core/               # 核心库
│   │   ├── anchor/               # 语义锚点解析器
│   │   ├── analyzer/             # 影响分析 + 策略评估
│   │   ├── ack/                  # scan ack 确认状态管理（to-ack.json）
│   │   ├── builder/              # 映射发现策略（annotation/frontmatter/naming）
│   │   ├── config/               # 配置管理（双配置合并、文件锁）
│   │   ├── git/                  # Git diff 分析
│   │   ├── mapping/              # 映射引擎（glob 匹配 + anchor 解析 + hunk overlap）
│   │   ├── models/               # 数据模型（Asset, Mapping, Impact, Source 等）
│   │   ├── output/               # 输出格式化（text/json）
│   │   ├── plugin/               # 插件协议与注册表（v1 兼容）
│   │   ├── provider/             # Provider 系统（v2：action prompt 渲染）
│   │   └── storage/              # SQLite 索引存储
│   ├── kamap-cli/                # CLI 工具
│   │   └── commands/             # 子命令实现（init/scan/check/mapping/asset/provider/...）
│   ├── kamap-mcp/                # MCP 协议支持（开发中）
│   ├── kamap-plugin-localfs/     # 本地文件系统插件
│   └── kamap-plugin-sqlite/      # SQLite 插件
```

## License

M