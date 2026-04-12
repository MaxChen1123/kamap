# kamap

[English](README_en.md)

> *每一次 commit，都在让你的文档悄悄过期。*

**kamap** 追踪代码变更对文档的影响，让文档始终同步。

kamap 是一个基于 Git 的**代码-知识资产映射与影响分析框架**。它在代码和文档（或其他知识资产）之间建立显式的映射关系，当代码发生变更时，自动识别哪些文档需要同步更新。

## 推荐方式：通过 Skill 让 Agent 使用 kamap

**推荐将 kamap 作为 Skill 安装到 AI Agent 中使用**，而不是手动执行 CLI 命令。通过 Skill，Agent 可以在你编码过程中自动完成影响分析、映射管理和文档同步检查，无需你记忆任何命令。

### 1. 构建 Skill

```bash
./scripts/build-skill.sh
```

脚本会编译 release 版本并将产物打包为 `kamap-skill.zip`，包含可执行文件和 Skill 描述文件。

可选参数：

```bash
./scripts/build-skill.sh --debug              # debug 构建
./scripts/build-skill.sh --target <triple>    # 交叉编译
```

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
- **"检查文档是否需要更新"** — Agent 调用 `kamap check` 进行 CI 级别检查

Skill 中的所有写操作默认为 dry-run，需要 `--apply` 才会实际写入，确保 Agent 使用安全。

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

扫描当前 Git 变更，输出受影响的知识资产列表。每个影响包含 `action_prompt` 字段，提供具体的操作指引（由 Provider 生成）。在 CI 中可使用 `kamap check`，存在 error 级别影响时会返回非零退出码。

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
