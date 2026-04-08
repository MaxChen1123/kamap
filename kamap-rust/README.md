# kamap — Knowledge Asset Mapping

基于 Git 的知识资产映射与影响分析框架。kamap 将源代码变更与知识资产（文档、数据库、配置等）关联起来，在代码发生变更时自动识别受影响的资产，提醒开发者同步更新。

## 核心概念

| 概念 | 说明 |
|------|------|
| **Asset（资产）** | 需要与代码保持同步的知识资产，如文档、数据库、配置文件等 |
| **Mapping（映射）** | 源代码路径/行范围与资产之间的关联关系 |
| **Impact（影响）** | 当代码变更命中映射规则时产生的影响报告 |
| **Plugin（插件）** | 不同资产类型的处理器，负责健康检查、内容读取、元信息获取等 |
| **Policy（策略）** | 定义影响的严重程度规则 |

## 快速开始

```bash
# 1. 初始化项目
kamap init

# 2. 注册资产
kamap asset add --id my-doc --provider localfs --type markdown --target docs/my-doc.md --apply

# 3. 添加映射
kamap mapping add --source 'src/**/*.rs' --asset my-doc --reason '实现代码' --apply

# 4. 扫描影响
kamap scan
```

## 项目结构

```
kamap.yaml           # 共享配置文件（团队/仓库共用，提交到 Git）
.kamap.yaml          # 个人配置文件（开发者私有，不提交到 Git）
.kamap/              # 工作目录
  └── index.db       # SQLite 运行时索引
```

### 双配置文件设计

| 文件 | 用途 | 是否提交 Git |
|------|------|-------------|
| `kamap.yaml` | 团队/仓库共享的资产与映射索引 | ✅ 是 |
| `.kamap.yaml` | 开发者个人的资产与映射索引 | ❌ 否 |

加载时两个文件会**自动合并**，`.kamap.yaml` 中的内容优先级更高（个人覆盖共享）。合并规则：
- **assets / mappings**: 同 ID 则个人覆盖共享，不同 ID 则追加
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

扫描 Git 变更，匹配映射规则，生成影响报告。

```bash
kamap scan                              # 默认对比 origin/main..HEAD
kamap scan --base main --head HEAD      # 指定 Git ref
kamap scan -o json                      # JSON 输出
kamap scan --config path/to/kamap.yaml # 指定配置文件
```

| 参数 | 默认值 | 说明 |
|------|--------|------|
| `--base` | `origin/main` | 基准 Git ref |
| `--head` | `HEAD` | 目标 Git ref |
| `--output` / `-o` | `text` | 输出格式 |
| `--config` | 自动查找 | 配置文件路径 |

**工作流程**: Git diff → 映射引擎匹配 → 影响分析 → 输出报告

---

### `kamap check`

策略检查（CI 友好）。与 `scan` 功能相同，但当存在 **error** 级别影响时以非零退出码退出，适合集成到 CI 流水线。

```bash
kamap check
kamap check --base main --head feature-branch -o json
```

| 参数 | 默认值 | 说明 |
|------|--------|------|
| `--base` | `origin/main` | 基准 Git ref |
| `--head` | `HEAD` | 目标 Git ref |
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

添加单个映射。默认 dry-run 模式，需 `--apply` 才实际写入。

```bash
kamap mapping add \
  --source 'src/auth/**/*.ts' \
  --asset auth-doc \
  --reason '认证模块实现' \
  --lines '10-45' \
  --action review \
  --apply
```

| 参数 | 必填 | 说明 |
|------|------|------|
| `--source` | 是 | 源文件路径或 glob 模式 |
| `--asset` | 是 | 目标 Asset ID |
| `--reason` | 否 | 映射原因 |
| `--lines` | 否 | 行范围，如 `"10-45"` |
| `--action` | 否 | 推荐动作: `review` / `update` / `verify` / `acknowledge` |
| `--apply` | 否 | 实际写入（默认 dry-run） |

#### `kamap mapping add-batch`

从 JSON 批量添加映射（适合 AI 生成）。

```bash
# 从 stdin
echo '{"mappings":[...]}' | kamap mapping add-batch --stdin --apply

# 从文件
kamap mapping add-batch --file mappings.json --apply
```

JSON 输入格式：
```json
{
  "mappings": [
    {
      "source_path": "src/foo.rs",
      "asset_id": "my-doc",
      "reason": "实现代码",
      "source_lines": [10, 45],
      "segment": {"heading": "Authentication"},
      "action": "review"
    }
  ]
}
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

校验所有映射定义的有效性（资产引用是否存在、路径是否为空等）。

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

#### `kamap asset add`

注册新资产。默认 dry-run 模式。

```bash
kamap asset add \
  --id my-doc \
  --provider localfs \
  --type markdown \
  --target docs/my-doc.md \
  --apply
```

| 参数 | 必填 | 说明 |
|------|------|------|
| `--id` | 是 | 资产唯一 ID |
| `--provider` | 是 | 插件 provider（`localfs` / `sqlite`） |
| `--type` | 是 | 资产类型（`markdown` / `text` / `config` / `sqlite-db`） |
| `--target` | 是 | 目标路径 |
| `--apply` | 否 | 实际写入（默认 dry-run） |

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

### `kamap plugin` — 插件管理

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

## 内置插件

### `localfs` — 本地文件系统

| 项目 | 值 |
|------|------|
| 支持资产类型 | `markdown`, `text`, `config` |
| Segment 解析 | Markdown heading（`{"heading": "xxx"}`） |
| 健康检查 | 文件是否存在 |
| 内容读取 | 支持 |
| 映射发现 | 支持（通过 frontmatter） |

### `sqlite` — SQLite 数据库

| 项目 | 值 |
|------|------|
| 支持资产类型 | `sqlite-db` |
| Segment 解析 | 表名（`{"table": "xxx"}`） |
| 健康检查 | 文件存在且可用 rusqlite 打开 |
| 内容读取 | 不支持 |
| Meta 信息 | 返回数据库中所有表名 |

---

## 映射发现（Discovery）

`kamap mapping discover` 命令支持三种自动发现策略：

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
  - id: map_abc123
    source:
      path: "src/auth/**/*.rs"
    asset: auth-doc
    reason: "认证模块实现代码"
    action: review

policies:
  - match:
      asset_priority: high
    severity: error

discovery:
  annotation:
    enabled: true
    marker: "@kamap"
  frontmatter:
    enabled: true
    key: kamap
  naming:
    enabled: false
    rules: []
```

---

## 项目架构

```
kamap-rust/
├── crates/
│   ├── kamap-core/           # 核心库（配置、映射引擎、影响分析、Git diff、存储）
│   ├── kamap-cli/            # CLI 工具
│   ├── kamap-mcp/            # MCP 协议支持
│   ├── kamap-plugin-localfs/ # 本地文件系统插件
│   └── kamap-plugin-sqlite/  # SQLite 插件
```

## License

MIT
