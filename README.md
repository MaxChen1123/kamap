# kamap

[English](README_en.md)

kamap 是一个基于 Git 的**代码-知识资产映射与影响分析框架**。它在代码和文档（或其他知识资产）之间建立显式的映射关系，当代码发生变更时，自动识别哪些文档需要同步更新。

## 快速开始

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
kamap mapping add \
  --source 'src/api/**/*.rs' \
  --asset api-doc \
  --reason '接口实现代码' \
  --apply
```

### 扫描影响

```bash
kamap scan
```

扫描当前 Git 变更，输出受影响的知识资产列表。在 CI 中可使用 `kamap check`，存在 error 级别影响时会返回非零退出码。

## 打包 Skill

kamap 可以打包为 Skill，供 Agent 使用。

```bash
./scripts/build-skill.sh
```

脚本会编译 release 版本并将产物打包为 `kamap-skill.zip`，包含可执行文件和 Skill 描述文件。

可选参数：

```bash
./scripts/build-skill.sh --debug              # debug 构建
./scripts/build-skill.sh --target <triple>    # 交叉编译
```

## 为 Agent 安装 Skill

1. 将打包好的 `kamap-skill.zip` 解压到 Agent 的 skill 目录（如 CodeBuddy Code 的用户级或项目级 skill 目录）。

2. 安装后，Agent 会识别 `SKILL.md` 中的触发短语，在以下场景自动激活 kamap：
   - "文档同步" / "影响分析" / "知识资产" / "映射关系"
   - "scan changes" / "which docs need updating" / "impact analysis"

3. Agent 使用示例：
   - **"扫描当前变更影响了哪些文档"** — Agent 调用 `kamap scan` 分析受影响的知识资产
   - **"帮我把 src/auth 和 docs/auth.md 建立映射"** — Agent 调用 `kamap asset add` + `kamap mapping add`
   - **"检查文档是否需要更新"** — Agent 调用 `kamap check` 进行 CI 级别检查

Skill 中的所有写操作默认为 dry-run，需要 `--apply` 才会实际写入，确保 Agent 使用安全。

## 详细文档

- [命令参考与架构说明](kamap-rust/README.md)

## License

MIT
