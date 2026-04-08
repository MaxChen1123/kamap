# Kamap Formatter 与注释/声明映射功能使用指南

## 二、注释与声明映射关系

Kamap 提供三种自动发现映射关系的策略（Discovery），让你无需手动逐条添加映射，即可从代码和文档中的声明自动提取映射候选。

### 2.1 总览

| 策略 | 标记 | 置信度 | 默认开启 | 适用场景 |
|------|------|--------|----------|----------|
| **Annotation（代码注释）** | `@kamap` | 0.9（高） | ✅ 是 | 在代码行内直接声明映射 |
| **Frontmatter（文档元数据）** | YAML frontmatter | 0.85（高） | ✅ 是 | 在 Markdown 文档头部声明反向映射 |
| **Naming Convention（命名约定）** | 目录命名规则 | 0.6（低） | ❌ 否 | 通过目录名自动推断映射 |

配置入口在 `kamap.yaml` 的 `discovery` 节：

```yaml
discovery:
  annotation:
    enabled: true          # 是否启用注释扫描
    marker: "@kamap"       # 注释标记（可自定义）
  frontmatter:
    enabled: true          # 是否启用 frontmatter 解析
    key: kamap             # YAML 键名
  naming:
    enabled: false         # 默认关闭
    rules: []              # 命名规则列表
```

### 2.2 Annotation（代码注释扫描）

#### 工作原理

扫描器会遍历工作区中所有代码文件，查找包含 `@kamap` 标记的注释行，解析出资产 ID、原因、片段等信息，生成映射候选。

**核心实现**：`kamap-core/src/builder/annotation.rs` → `AnnotationScanner`

#### 支持的注释格式

| 语言 | 注释格式 | 示例 |
|------|----------|------|
| Rust / JS / TS / Go / Java / C / C++ | `//` | `// @kamap asset:doc reason:"..."` |
| Python / Ruby / Shell | `#` | `# @kamap asset:doc` |
| SQL / Haskell | `--` | `-- @kamap asset:doc` |
| CSS / 多行 | `/* */` | `/* @kamap asset:doc */` |

#### 支持的文件扩展名

`.rs`, `.ts`, `.tsx`, `.js`, `.jsx`, `.py`, `.go`, `.java`, `.rb`, `.c`, `.cpp`, `.h`

#### 注释语法

```
// @kamap asset:<asset_id> [reason:"<原因>"] [segment:{<JSON>}]
```

**可用字段**：

| 字段 | 必填 | 说明 | 示例 |
|------|------|------|------|
| `asset` | **是** | 目标资产的 ID | `asset:auth-doc` |
| `reason` | 否 | 映射原因（可用引号包裹） | `reason:"认证逻辑"` |
| `segment` | 否 | 资产片段信息（JSON） | `segment:{"heading":"Login"}` |

#### 使用示例

**Rust 代码**：
```rust
// @kamap asset:auth-doc reason:"认证逻辑" segment:{"heading":"Login"}
pub fn login(user: &str, pass: &str) -> Result<Token, Error> {
    // ...
}

// @kamap asset:api-doc reason:"Token 刷新接口"
pub fn refresh_token(token: &str) -> Result<NewToken, Error> {
    // ...
}
```

**TypeScript 代码**：
```typescript
// @kamap asset:auth-doc reason:"登录页面组件" segment:{"heading":"UI"}
export function LoginForm() { ... }

# Python 代码（使用 # 注释）
# @kamap asset:data-doc reason:"用户模型定义"
class User(BaseModel):
    ...
```

#### 行范围估算

注释扫描器会自动将映射的行范围设置为注释所在行及其后 10 行（`[line, line + 10]`），作为初步估算。你可以在发现后用 `kamap mapping update` 手动调整精确范围。

### 2.3 Frontmatter（Markdown 文档声明）

#### 工作原理

解析 Markdown 文件头部的 YAML frontmatter，读取其中 `kamap.relates-to` 列表，生成从文档到源代码的反向映射候选。

**核心实现**：`kamap-core/src/builder/frontmatter.rs` → `FrontmatterParser`

#### Frontmatter 语法

```markdown
---
title: 认证模块文档
kamap:
  relates-to:
    - path: src/auth/**/*.ts
      reason: "认证模块实现"
      lines: "10-45"
      segment:
        heading: "Authentication"
    - path: src/auth/login.ts
      reason: "登录逻辑"
---
```

#### 字段说明

| 字段 | 必填 | 说明 | 示例 |
|------|------|------|------|
| `path` | **是** | 源代码路径或 glob 模式 | `src/auth/**/*.rs` |
| `reason` | 否 | 关联原因（默认 `"Declared in frontmatter"`） | `"认证模块实现"` |
| `lines` | 否 | 行范围，字符串格式 `"start-end"` | `"10-45"` |
| `segment` | 否 | 文档片段信息 | `{heading: "Authentication"}` |

#### 使用示例

**文档 `docs/api.md`**：
```markdown
---
title: API 接口文档
kamap:
  relates-to:
    - path: "src/api/**/*.ts"
      reason: "API 层实现代码"
      segment:
        heading: "REST API"
    - path: "src/api/handler.ts"
      reason: "请求处理逻辑"
      lines: "1-80"
---

# REST API

## Authentication
...
```

#### 注意事项

- Frontmatter 必须以 `---` 开头和结尾
- `kamap` 键名可通过 `discovery.frontmatter.key` 配置项自定义
- 解析出的映射是**反向映射**：文档本身作为 Asset，指向它关联的源代码路径

### 2.4 Naming Convention（命名约定）

#### 工作原理

根据目录命名规则，自动将同名的代码目录与文档关联起来。例如 `src/auth/` 目录自动匹配 `docs/auth.md` 文档。

**核心实现**：`kamap-core/src/builder/naming.rs` → `NamingMatcher`

#### 配置方式

在 `kamap.yaml` 中添加命名规则：

```yaml
discovery:
  naming:
    enabled: true
    rules:
      # 规则：src/{module}/ 下的代码 → docs/{module}.md
      - source: "src/{module}/**"
        asset_pattern: "docs/{module}.md"
      # 更多规则...
      - source: "packages/{name}/src/**"
        asset_pattern: "docs/packages/{name}.md"
```

#### 规则字段

| 字段 | 说明 | 示例 |
|------|------|------|
| `source` | 源代码 glob 模式，`{module}` 为通配变量 | `"src/{module}/**"` |
| `asset_pattern` | 资产目标路径模板，`{module}` 与 source 中对应 | `"docs/{module}.md"` |

#### 匹配逻辑

1. 扫描 `source` 模式中 `{module}` 对应的实际目录（如 `src/auth/`, `src/api/`）
2. 用实际目录名替换 `asset_pattern` 中的 `{module}`
3. 在已注册资产中查找匹配 `target` 的资产
4. 如果找到，生成一个置信度为 0.6 的映射候选

#### 示例

项目结构：
```
src/
  auth/
  api/
  db/
docs/
  auth.md      ← asset id: auth-doc, target: docs/auth.md
  api.md       ← asset id: api-doc, target: docs/api.md
  db.md        ← asset id: db-doc, target: docs/db.md
```

配置规则 `source: "src/{module}/**"`, `asset_pattern: "docs/{module}.md"` 后，运行 discover 会自动产生：
- `src/auth/**` → `auth-doc` （Naming convention: auth → docs/auth.md）
- `src/api/**` → `api-doc`
- `src/db/**` → `db-doc`

> 由于命名约定的置信度较低（0.6），默认情况下需要加 `--include-low-confidence` 参数才能看到这些候选。

### 2.5 使用 Discover 命令

#### 基本用法

```bash
# 发现所有映射候选（仅包含高置信度：annotation + frontmatter）
kamap mapping discover

# 包含低置信度候选（naming convention）
kamap mapping discover --include-low-confidence

# JSON 输出（方便 AI 处理）
kamap mapping discover -o json
```

#### 输出示例

```
Discovered 4 mapping candidates:

  [90%] src/auth/login.rs:15-25 → auth-doc (code_annotation)
    reason: 认证逻辑

  [90%] src/api/handler.rs:42-52 → api-doc (code_annotation)
    reason: Token 刷新接口

  [85%] src/auth/**/*.ts → docs/api.md (asset_frontmatter)
    reason: 认证模块实现

  [60%] src/db/** → db-doc (naming_convention)
    reason: Naming convention: db → docs/db.md
```

#### JSON 输出格式

```json
[
  {
    "source": { "path": "src/auth/login.rs", "lines": [15, 25] },
    "asset_id": "auth-doc",
    "reason": "认证逻辑",
    "confidence": 0.9,
    "origin": "code_annotation",
    "segment": { "heading": "Login" }
  }
]
```

#### origin（来源类型）

| 值 | 来源策略 | 置信度 |
|----|----------|--------|
| `code_annotation` | `@kamap` 代码注释 | 0.9 |
| `asset_frontmatter` | Markdown frontmatter | 0.85 |
| `naming_convention` | 目录命名约定 | 0.6 |
| `plugin_discovery` | 插件发现（预留） | — |
| `co_change_history` | 共变历史（预留） | — |

### 2.6 发现后的工作流

Discover 命令只是**预览**候选映射，不会自动写入配置。典型工作流如下：

```bash
# 1. 运行发现，查看候选
kamap mapping discover -o json > candidates.json

# 2. 审查候选内容，确认无误后批量导入
cat candidates.json | kamap mapping add-batch --stdin --apply

# 3. 验证映射有效性
kamap mapping validate

# 4. 查看所有映射
kamap mapping list
```

或者使用 AI 辅助流程：

```bash
# 1. 导出项目上下文（包含已有映射、未映射的代码/资产、命名提示）
kamap mapping export-context -o json > context.json

# 2. AI 分析 context.json 后生成新的映射建议

# 3. 通过 stdin 批量导入 AI 生成的映射
echo '{"mappings":[...]}' | kamap mapping add-batch --stdin --apply
```

---

## 三、最佳实践

### 3.1 团队协作推荐

1. **优先使用 Annotation**：在编写代码时顺手加上 `@kamap` 注释，成本低且置信度高
2. **文档维护者使用 Frontmatter**：在撰写文档时同步更新 relates-to 声明
3. **大型项目启用 Naming**：对于有良好目录组织的项目，开启 naming 可以覆盖大部分常规映射
4. **定期 Discover + Review**：在 CI 中集成 `kamap check`，定期运行 `kamap mapping discover` 补充新映射

### 3.2 配置示例

```yaml
# kamap.yaml — 推荐的完整配置
version: "1"

plugins:
  - name: localfs
    enabled: true

assets:
  - id: auth-doc
    provider: localfs
    type: markdown
    target: docs/auth.md
  - id: api-doc
    provider: localfs
    type: markdown
    target: docs/api.md

mappings: []  # 通过 discover 自动填充

discovery:
  annotation:
    enabled: true
    marker: "@kamap"
  frontmatter:
    enabled: true
    key: kamap
  naming:
    enabled: true
    rules:
      - source: "src/{module}/**"
        asset_pattern: "docs/{module}.md"

policies:
  - match:
      asset_priority: high
    severity: error
```
