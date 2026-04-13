# kamap — Knowledge Asset Mapping

[English](README_en.md)

> *每一次 commit，都在让你的文档悄悄过期。*

**kamap** 追踪代码变更对文档的影响，让文档始终同步。


## kamap 是什么

代码和文档之间天然存在关联——某个函数的实现变了，对应的 API 文档就该更新。但这种关联通常只存在于开发者的脑子里，没有任何工具去追踪它。kamap 就是用来解决这个问题的：**在代码和文档之间建立显式、可追踪的映射关系，当代码变更时自动告诉你哪些文档受到了影响**。

### 工作原理

kamap 的核心流程分三步：

1. **建立映射**：你（或 AI Agent）声明代码和文档的关联关系。比如 `src/auth.rs` 中的 `fn login` 映射到 `docs/auth.md`。映射支持整文件或精确到某个函数/类的**语义锚点**。

2. **检测变更**：当你修改代码后，kamap 通过 Git diff 获取变更的文件和行范围，然后与映射规则做匹配。如果使用了语义锚点，kamap 会动态解析函数/类的实际行范围，再判断变更是否命中。

3. **输出影响**：匹配到的映射会生成影响报告，告诉你哪些文档需要更新，并附带具体的操作指引（由 Provider 生成）。

```
代码变更 → Git diff → 映射规则匹配 → 影响报告 → 文档同步
```

kamap 不会帮你改文档，它只负责**发现**哪些文档该改。实际的更新工作可以由你手动完成，也可以让 AI Agent 根据影响报告自动处理。

## 推荐使用方式：通过 Skill 让 Agent 使用 kamap

**推荐将 kamap 作为 Skill 安装到 AI Agent 中使用**，而不是手动执行 CLI 命令。通过 Skill，Agent 可以在你编码过程中自动完成影响分析、映射管理和文档同步检查，无需你记忆任何命令。

### 1. 构建 Skill

```bash
./scripts/build-skill.sh
```

脚本会编译 release 版本并将产物打包为 `kamap-skill.zip`，包含可执行文件和 Skill 描述文件。



> 需要 Rust 工具链。如果尚未安装，参考 [rustup.rs](https://rustup.rs/)。

### 2. 安装 Skill

将打包好的 `kamap-skill.zip` 解压到 Agent 的 skill 目录。

### 3. 使用

安装后，Agent 会识别 `SKILL.md` 中的触发短语，在以下场景自动激活 kamap：

- "文档同步" / "影响分析" / "知识资产" / "映射关系"
- "scan changes" / "which docs need updating" / "impact analysis"

Agent 使用示例：

- **"扫描当前变更影响了哪些文档"** — Agent 调用 `kamap scan` 分析受影响的知识资产
- **"帮我把 src/auth 和 docs/auth.md 建立映射"** — Agent 调用 `kamap asset add` + `kamap mapping add`

Skill 中的所有写操作默认为 dry-run，需要 `--apply` 才会实际写入，确保 Agent 使用安全。

### Skill 典型使用场景

#### 场景一：为已有文档批量建立映射关系

直接告诉 Agent：

> **"给我配置一下 `docs/` 文件夹下面文档的映射关系"**

Agent 会自动执行以下流程：
1. 扫描 `docs/` 下的所有文档文件，注册为知识资产
2. 分析每篇文档涉及的代码文件和函数
3. 自动建立代码到文档的映射关系

你不需要手动指定任何路径或参数。

#### 场景二：提交前检查文档影响

在准备提交 commit 前，告诉 Agent：

> **"给我看一下哪些文档映射关系受到影响"**

Agent 会：
1. 扫描当前 Git 变更，列出所有受影响的文档
2. 告知你哪些文档可能需要更新，你可以让 Agent 直接帮你修改
3. **自动检查未覆盖的变更**：Agent 还会分析哪些新增/修改的代码和文档尚未建立映射关系，并主动配置

#### 场景三：个人配置 vs 团队共享配置

Skill 默认将所有配置（资产注册、映射关系）写入 **个人配置** `.kamap.yaml`（应被 `.gitignore` 忽略）。

如果你希望将配置写入**团队共享的** `kamap.yaml`（会提交到 Git），需要显式告诉 Agent：

> **"写到共享配置中"** 或 **"写到团队配置"**

否则 Agent 始终默认使用个人配置，不会影响团队仓库。

### 为什么推荐 Skill 方式？

- **零记忆成本**：不需要记 CLI 命令和参数，用自然语言告诉 Agent 你的意图即可
- **自动化工作流**：Agent 会自动完成"检查已有资产 → 注册资产 → 创建映射 → 验证"的完整流程
- **安全**：所有写操作默认 dry-run，Agent 需要确认后才会实际执行

## 手动使用 CLI

如果你不使用 AI Agent，或需要在 CI/CD 中集成，也可以直接使用 CLI。

### 构建

```bash
cd kamap-rust
cargo build --release -p kamap-cli
```

构建产物位于 `kamap-rust/target/release/kamap`，可将其加入 `PATH`。

### 初始化

在 Git 仓库根目录执行：

```bash
kamap init
```

这会生成 `kamap.yaml`（团队共享配置）和 `.kamap.yaml`（个人配置）。

### 注册资产

```bash
kamap asset add \
  --id api-doc \
  --provider localfs \
  --type markdown \
  --target docs/api.md \
  --apply
```

### 创建映射

```bash
# 使用语义锚点精确映射到某个函数/类（推荐）
kamap mapping add \
  --source src/api/handler.rs \
  --asset api-doc \
  --anchor 'fn handle_request' \
  --reason '请求处理函数' \
  --apply

# 使用 anchor-context 消歧（当同名锚点存在多个时）
kamap mapping add \
  --source src/api/handler.rs \
  --asset api-doc \
  --anchor 'fn new' \
  --anchor-context 'impl RequestHandler' \
  --reason '构造函数' \
  --apply

# 整文件映射（适用于小文件或配置文件）
kamap mapping add \
  --source 'src/api/**/*.rs' \
  --asset api-doc \
  --reason '接口实现代码' \
  --apply
```

> **语义锚点**（`--anchor`）是 kamap v0.2 引入的推荐映射方式。相比静态行号（`--lines`），锚点在代码重构后不会漂移，始终定位到目标函数/类/块的实际位置。`--lines` 仍然可用但不推荐。

### 扫描影响

```bash
kamap scan
```

扫描当前 Git 变更，输出受影响的知识资产列表。每个影响包含 `action_prompt` 字段，提供具体的操作指引（由 Provider 生成）。CI 场景下也可使用 `kamap check`，功能相同但存在 error 级别影响时返回非零退出码。

### Provider 系统

Provider 定义了 kamap 在检测到影响时如何生成操作指引（`action_prompt`）。内置 provider（`localfs`、`sqlite`）有默认 prompt；可通过 `kamap.yaml` 中的 `providers` 配置自定义 provider（如 iwiki、notion），并通过 `prompt_template` 定义操作指引模板：

```yaml
providers:
  - name: iwiki
    prompt_template: |
      代码变更影响了 iwiki 文档「{{asset.meta.title}}」(文档 ID: {{asset.target}})。
      请通过 iwiki MCP 读取并更新文档。
```

查看已注册的 provider：

```bash
kamap provider list
kamap provider info --name localfs
```

> `kamap plugin` 命令已废弃，请使用 `kamap provider` 替代。

## 详细文档

- [命令参考与架构说明](kamap-rust/README.md)

## License

MIT
