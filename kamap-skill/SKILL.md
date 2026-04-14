---
name: kamap
description: This skill should be used when users need to manage knowledge asset mappings, perform code-to-documentation impact analysis, track document synchronization, or maintain code-knowledge relationships in a Git repository. Trigger phrases include "document sync", "impact analysis", "knowledge asset", "code-doc mapping", "which docs need updating", "scan changes", "register asset", "add mapping", "kamap", "文档同步", "影响分析", "知识资产", "映射关系", "文档更新".
---

# kamap — Knowledge Asset Mapping

## Overview

kamap is a Git-based knowledge asset mapping and impact analysis framework. It establishes mapping relationships between code and knowledge assets (documents, databases, configurations), automatically locates affected assets when code changes, and reminds developers to synchronize updates.

The kamap binary is located at:
- **macOS / Linux**: `{SKILL_DIR}/bin/kamap`
- **Windows**: `{SKILL_DIR}/bin/kamap.exe`

**IMPORTANT**: When constructing commands, you MUST detect the current operating system and use the correct binary path. Use `{SKILL_DIR}/bin/kamap.exe` on Windows and `{SKILL_DIR}/bin/kamap` on macOS/Linux. Throughout this document, `{SKILL_DIR}/bin/kamap` is used as shorthand — always append `.exe` on Windows.

## When to Use

Activate this skill when the user's request involves any of the following:

- **Document synchronization**: Updating docs after code changes, checking if documentation is stale, aligning code and docs
- **Impact analysis**: Analyzing which assets are affected by code changes, PR/code review documentation checks, CI documentation validation
- **Mapping management**: Creating, listing, validating, or discovering code-to-asset mappings
- **Asset management**: Registering, listing, checking, or removing knowledge assets
- **Project initialization**: Setting up kamap tracking for a repository

## Key Conventions

1. **All write operations default to dry-run**: Add `--apply` to actually write changes
2. **Always use `--output json`**: For structured, parseable output (most commands default to `text`)
3. **Assets must be registered before mappings**: Run `asset add` before `mapping add`
4. **Scan requires a Git repository**: Ensure running inside a Git repo
5. **Dual config files**: `kamap.yaml` (shared/team, commit to Git) and `.kamap.yaml` (personal, gitignored). Default writes to `.kamap.yaml`; use `--shared` flag to write to `kamap.yaml`
6. **Config file lookup**: kamap searches upward from current directory for `kamap.yaml` or `.kamap.yaml`
7. **CRITICAL — Personal config by default**: All `asset add`, `mapping add`, `mapping add-batch` commands write to the **personal** config (`.kamap.yaml`) by default. You MUST **NOT** use `--shared` unless the user **explicitly** states the asset/mapping should be shared or team-level. When in doubt, always default to personal config.
8. **CRITICAL — Check existing assets before adding**: Before running `asset add`, you MUST first run `asset list --output json` to inspect all currently registered assets. This prevents duplicate registrations and helps you reference existing asset IDs when adding mappings.
9. **CRITICAL — No `asset-type` or `type` subcommand**: There is NO subcommand called `asset-type` or `asset type` or `type`. The asset type (e.g. `markdown`, `text`, `config`, `sqlite-db`) is specified via the `--type` **flag** on the `asset add` subcommand. The correct usage is: `kamap asset add --id <id> --provider <provider> --type <type> --target <path>`. Do NOT confuse `--type` (a flag) with a subcommand.
10. **CRITICAL — Use semantic anchors or whole-file mapping**: When configuring mappings, choose the appropriate granularity:

    - **Whole-file mapping (no anchor, no lines)** — **prefer this when appropriate**: Use this when the entire file is relevant to the asset. This is the simpler and correct choice in many common situations:
      - The file is small or single-purpose (e.g. a single module, a config file, a data model definition)
      - Most or all of the file's content relates to the target asset
      - The file contains tightly coupled code where changes to any part could affect the asset
      - The file is a non-code file (YAML, TOML, JSON, Markdown, etc.)
      **Do NOT add an anchor just for the sake of having one** — if the whole file matters, a whole-file mapping is both simpler and more correct.
    - **Semantic anchor (`anchor` field)**: Use this **only when** a specific function, class, struct, or code block within a **large file** is relevant, while other parts of the same file are not. The `anchor` is a text pattern (e.g. `"fn login"`, `"class AuthService"`, `"def handle_request"`) that kamap uses to **dynamically locate** the code block at scan time. Unlike static line numbers, anchors automatically track code that moves due to insertions/deletions elsewhere in the file, eliminating false positives from line drift.
    - **Do NOT use static line numbers** (`--lines` / `source_lines`): Static line numbers become stale as soon as code is inserted or deleted above the mapped range, causing both false positives and missed detections. Always prefer `anchor` over `source_lines`.

    **When to use anchor vs whole-file — decision rule**:
    - File has **one main concern** (e.g. `login.rs` only does login) → **whole-file mapping**
    - File has **multiple unrelated concerns** (e.g. `commands.rs` with `fn scan`, `fn check`, `fn init` each mapping to different docs) → **anchor per function**
    - Not sure → **default to whole-file mapping**; it's better to have a slightly broad mapping than a broken one

    **How to choose an anchor**: Read the source file, identify the function/class/struct name that is relevant, and use it as the anchor text. The anchor should be **unique within the file** — if there are multiple items with the same name (e.g. `fn new` in different impl blocks), use `anchor_context` to disambiguate:
    ```json
    {"source_path":"src/auth.rs","asset_id":"token-doc","anchor":"fn new","anchor_context":"impl Token","reason":"Token constructor"}
    ```
11. **CRITICAL — Use batch commands for multiple items**: When adding **2 or more** assets, you **MUST** use `asset add-batch` instead of calling `asset add` multiple times. Similarly, use `mapping add-batch` for multiple mappings. Running multiple single-add commands in parallel causes write race conditions. Batch commands are atomic and safe.

## Core Capabilities

### 1. Impact Analysis (scan / check)

Automatically identify affected knowledge assets when code changes:

```bash
# Scan Git changes for impact on knowledge assets (default: origin/main..HEAD)
{SKILL_DIR}/bin/kamap scan --output json

# Scan with custom Git refs
{SKILL_DIR}/bin/kamap scan --base origin/develop --head feature-branch --output json

# CI mode check (non-zero exit code when error-level impacts exist)
{SKILL_DIR}/bin/kamap check --base origin/main --head HEAD --output json
```

**Typical scenario**: After modifying code, run scan to see which documents need synchronization. The output includes `change_type` field per impact (`added`/`modified`/`deleted`/`renamed`) to identify the type of Git change, and `action` field: `update`, `review`, `verify`, or `acknowledge`.

### 2. Mapping Management (mapping)

Establish and manage associations between code and knowledge assets.

> **IMPORTANT**: By default, all mapping write commands (`add`, `add-batch`) write to the **personal** config (`.kamap.yaml`). Only add `--shared` when the user **explicitly** requests shared/team-level mappings. If the user does not mention "shared", "团队", "共享", do NOT use `--shared`.

```bash
# Add a mapping with semantic anchor (RECOMMENDED)
{SKILL_DIR}/bin/kamap mapping add \
  --source src/auth/login.ts \
  --asset auth-doc \
  --anchor 'function login' \
  --reason '登录函数实现' \
  --action update \
  --apply --output json

# Add a whole-file mapping (for small files or config files)
{SKILL_DIR}/bin/kamap mapping add \
  --source 'src/config/**/*.yaml' \
  --asset config-doc \
  --reason '配置文件' \
  --action review \
  --apply --output json

# Add with anchor + context for disambiguation
{SKILL_DIR}/bin/kamap mapping add \
  --source src/auth.rs \
  --asset token-doc \
  --anchor 'fn new' \
  --anchor-context 'impl Token' \
  --reason 'Token constructor' \
  --action update \
  --apply --output json

# Batch add mappings from JSON (via stdin or --file)
# NOTE: Use "anchor" field for precise block-level mappings
echo '{"mappings":[
  {"source_path":"src/auth.rs","asset_id":"login-doc","anchor":"fn login","reason":"登录实现","action":"update"},
  {"source_path":"src/auth.rs","asset_id":"logout-doc","anchor":"fn logout","reason":"登出实现","action":"update"},
  {"source_path":"src/config.rs","asset_id":"config-doc","reason":"配置模块（整文件）","action":"review"}
]}' | {SKILL_DIR}/bin/kamap mapping add-batch --stdin --apply --output json

# Batch add from file
{SKILL_DIR}/bin/kamap mapping add-batch --file mappings.json --apply --output json

# List all mappings
{SKILL_DIR}/bin/kamap mapping list --output json

# Filter mappings by asset
{SKILL_DIR}/bin/kamap mapping list --asset my-doc --output json

# Remove a mapping by ID
{SKILL_DIR}/bin/kamap mapping remove --id map_abc123 --output json

# Validate all mappings (includes anchor validity checks)
{SKILL_DIR}/bin/kamap mapping validate --output json

# Export mappings (uses --format, not --output; supports json, yaml, csv)
{SKILL_DIR}/bin/kamap mapping export --format json

# Import mappings (strategy: append (default), merge, replace)
{SKILL_DIR}/bin/kamap mapping import --file mappings.json --format json --strategy merge --apply
{SKILL_DIR}/bin/kamap mapping import --stdin --format yaml --apply

# Auto-discover mapping candidates (from @kamap annotations, frontmatter, naming conventions)
{SKILL_DIR}/bin/kamap mapping discover --output json
{SKILL_DIR}/bin/kamap mapping discover --include-low-confidence --output json

# Export project context for AI analysis (default output: json)
{SKILL_DIR}/bin/kamap mapping export-context --output json
```

**Batch JSON format** — the `mappings` array accepts objects with:
- `source_path` (required): Source file path or glob
- `asset_id` (required): Target asset ID
- `anchor` (recommended): Semantic anchor text to locate the relevant code block (e.g. `"fn login"`, `"class AuthService"`). Used for dynamic block-level matching that is immune to line drift. Omit only for whole-file mappings.
- `anchor_context` (optional): Outer scope text for disambiguation when multiple blocks share the same anchor name (e.g. `"impl Token"`)
- `reason` (optional but recommended): Why this mapping exists
- `action` (optional): `"review"`, `"update"`, `"verify"`, `"acknowledge"`
- `source_lines` (deprecated, avoid): `[start, end]` static line range — prefer `anchor` instead
- `segment` (optional): JSON object for targeting specific asset sections

### 3. Asset Management (asset)

Register and manage knowledge assets.

> **IMPORTANT**: Before adding any asset, you MUST first run `asset list` to check all existing registered assets. This avoids duplicate registrations and ensures you are aware of available asset IDs for mapping.
>
> **IMPORTANT**: By default, `asset add` writes to the **personal** config (`.kamap.yaml`). Only add `--shared` when the user **explicitly** requests the asset be shared/team-level. If the user does not mention "shared", "团队", "共享", do NOT use `--shared`.
>
> **CRITICAL — Prefer batch add over multiple single adds**: When registering **2 or more** assets, you **MUST** use `asset add-batch` instead of calling `asset add` multiple times. Running multiple `asset add` commands in parallel causes a race condition where later writes overwrite earlier ones, resulting in lost assets. `add-batch` handles all assets in a single atomic operation. **Never call `asset add` multiple times in parallel.**

```bash
# Step 1: ALWAYS list existing assets first
{SKILL_DIR}/bin/kamap asset list --output json

# Step 2a: Register a SINGLE asset (writes to PERSONAL config by default)
{SKILL_DIR}/bin/kamap asset add \
  --id my-doc \
  --provider localfs \
  --type markdown \
  --target docs/my-doc.md \
  --apply --output json

# Step 2b: PREFERRED — Batch register MULTIPLE assets in one atomic operation
echo '{"assets":[
  {"id":"readme-zh","provider":"localfs","type":"markdown","target":"README.md"},
  {"id":"readme-en","provider":"localfs","type":"markdown","target":"README_en.md"},
  {"id":"api-doc","provider":"localfs","type":"markdown","target":"docs/api.md"}
]}' | {SKILL_DIR}/bin/kamap asset add-batch --stdin --apply --output json

# Batch add from file
{SKILL_DIR}/bin/kamap asset add-batch --file assets.json --apply --output json

# Register to SHARED config (ONLY when user explicitly asks for shared/team config)
# For single asset:
{SKILL_DIR}/bin/kamap asset add \
  --id my-doc \
  --provider localfs \
  --type markdown \
  --target docs/my-doc.md \
  --shared --apply --output json

# For batch (shared):
echo '{"assets":[...]}' | {SKILL_DIR}/bin/kamap asset add-batch --stdin --shared --apply --output json

# Remove an asset
{SKILL_DIR}/bin/kamap asset remove --id my-doc --output json

# Health check all assets
{SKILL_DIR}/bin/kamap asset check --output json
```

**Batch JSON format** — the `assets` array accepts objects with:
- `id` (required): Unique asset identifier
- `provider` (required): Plugin provider name (e.g. `localfs`, `sqlite`)
- `type` (required): Asset type (e.g. `markdown`, `text`, `config`, `sqlite-db`)
- `target` (required): Asset target path or URL

**Built-in providers**: `localfs` (types: `markdown`, `text`, `config`) and `sqlite` (type: `sqlite-db`).

### 4. Index Management (index)

Build and manage runtime indexes (stored at `.kamap/index.db`):

```bash
# Build/rebuild index
{SKILL_DIR}/bin/kamap index build --output json

# View index statistics
{SKILL_DIR}/bin/kamap index stats --output json
```

### 5. Project Initialization (init)

```bash
{SKILL_DIR}/bin/kamap init --output json
```

Creates `kamap.yaml` (shared) + `.kamap.yaml` (personal) + `.kamap/` working directory.

### 6. Relationship Explanation (explain)

Explain relationships for a mapping, asset, or source file (must specify exactly one):

```bash
{SKILL_DIR}/bin/kamap explain --mapping map_abc123 --output json
{SKILL_DIR}/bin/kamap explain --asset my-doc --output json
{SKILL_DIR}/bin/kamap explain --source src/auth/mod.rs --output json
```

### 7. Tool Self-Description (describe)

Output machine-readable tool description (default output: json):

```bash
{SKILL_DIR}/bin/kamap describe --output json
```

### 8. Provider Management (provider)

Providers define how kamap generates action prompts when impacts are detected. Built-in providers (`localfs`, `sqlite`) have default prompts; custom providers use user-defined prompt templates.

```bash
{SKILL_DIR}/bin/kamap provider list --output json
{SKILL_DIR}/bin/kamap provider info --name localfs --output json
```

### 9. Plugin Management (plugin) — deprecated

> **Note**: The `plugin` command is deprecated. Use `provider` instead.

```bash
{SKILL_DIR}/bin/kamap plugin list --output json
{SKILL_DIR}/bin/kamap plugin info --name localfs --output json
```

## Provider System (v2)

kamap v2 introduces a **prompt-driven provider architecture**. Instead of kamap directly reading/writing remote assets, it generates **action prompts** that tell the Agent how to handle each impacted asset.

### How it works

1. `kamap scan` detects code changes and identifies impacted assets
2. For each impact, kamap looks up the asset's provider and renders an **action_prompt**
3. The `action_prompt` is included in the scan JSON output
4. The Agent reads the prompt and uses whatever tools are appropriate (MCP, Skill, direct file access, etc.)

### Scan output with action_prompt

Each impact in `kamap scan --output json` now includes an `action_prompt` field and a `changed_lines` field:

```json
{
  "mapping_id": "map_xxx",
  "asset_id": "auth-design-doc",
  "provider": "iwiki",
  "change_type": "modified",
  "changed_lines": { "additions": 5, "deletions": 2, "total": 7 },
  "action_prompt": "代码变更影响了 iwiki 文档「认证模块设计文档」(文档 ID: 12345678)...",
  "action": "update",
  "reason": "登录函数实现变更"
}
```

### Configuring custom providers

Define providers in `kamap.yaml` with a `prompt_template`:

```yaml
providers:
  - name: iwiki
    prompt_template: |
      代码变更影响了 iwiki 文档「{{asset.meta.title}}」(文档 ID: {{asset.target}})。

      变更来源: {{source.path}}
      影响原因: {{reason}}
      建议操作: {{action}}

      请通过 iwiki MCP 完成以下操作：
      1. 调用 getDocument(docId: "{{asset.target}}") 读取文档当前内容
      2. 阅读代码变更，判断文档哪些部分需要更新
      3. 调用 saveDocument 保存修改后的文档
```

Template variables: `{{asset.id}}`, `{{asset.target}}`, `{{asset.type}}`, `{{asset.provider}}`, `{{asset.meta.*}}`, `{{source.path}}`, `{{source.file}}`, `{{source.hunks}}`, `{{reason}}`, `{{action}}`, `{{mapping_id}}`, `{{change_type}}`, `{{changed_lines}}`, `{{changed_lines.additions}}`, `{{changed_lines.deletions}}`, `{{changed_lines.total}}`.

Built-in providers (`localfs`, `sqlite`) have default prompts and don't need `prompt_template`.

## Recommended Workflows

### Workflow A: Post-Coding Document Sync

**Phase 1 — Handle existing mapping impacts:**

1. After code changes, run `{SKILL_DIR}/bin/kamap scan --output json`
2. For each impact, **first assess whether the change warrants a document update** based on the `changed_lines` field and the nature of the change:

   **CRITICAL — Change significance assessment (do NOT skip):**

   Before updating any document, you MUST evaluate whether the code change is significant enough to be reflected in the document. Each impact includes a `changed_lines` field with `additions`, `deletions`, and `total` counts. Use this along with the diff content to judge:

   - **Skip document update** (just acknowledge) when:
     - The change is a small internal implementation detail (e.g. adding a private helper function, adjusting internal logic, fixing a minor bug) that does not affect the behavior, interface, or concepts described in the document
     - The change is cosmetic (renaming local variables, reformatting, adding comments in code)
     - The document describes things at a higher abstraction level (architecture, workflow, API surface) and the change is below that level of detail
     - Adding this detail would make the document inconsistent in granularity with its existing content — **match the document's existing level of detail, do not make it more granular**

   - **Update the document** when:
     - The change affects public APIs, interfaces, configuration options, command-line arguments, or user-visible behavior
     - The change adds/removes/modifies a feature, workflow step, or concept that the document describes
     - The change contradicts or invalidates something currently stated in the document
     - The change is large enough (in scope, not just line count) to represent a meaningful behavioral difference

   **Rule of thumb**: Read the existing document first. If the document doesn't describe things at the level of detail of this change, don't add that detail — just acknowledge the impact. A document that stays consistent in granularity is more useful than one that has random implementation details mixed in with high-level descriptions.

   After assessment, for impacts that DO need document updates:
   - **For localfs assets**: directly read and update the local file as indicated
   - **For custom provider assets** (iwiki, notion, etc.): follow the `action_prompt` instructions, which may involve calling MCP tools, Skills, or other methods

   For impacts that do NOT need document updates, skip directly to acknowledgement.
3. After handling each impact, acknowledge it:
   ```bash
   # Acknowledge specific impacts by mapping ID
   {SKILL_DIR}/bin/kamap scan ack --ids map_abc123,map_def456 --output json
   # Or acknowledge all at once
   {SKILL_DIR}/bin/kamap scan ack --all --output json
   ```
4. After updating documents, scan again to confirm nothing was missed

**Phase 2 — Review unmapped changes (CRITICAL — do NOT skip):**

After handling all existing mapping impacts, you MUST proactively review the current changes for unmapped code and documents. This ensures new code and new documents are covered by kamap, not just pre-existing mappings.

5. Review the Git diff (e.g. `git diff origin/main..HEAD --name-status`) and identify:
   - **New or significantly modified code files** that are NOT covered by any existing mapping
   - **New document files** (`.md`, `.adoc`, `.rst`, or files under `docs/` etc.) that are NOT registered as assets
6. For each category, decide whether action is needed:

   **New/modified code files without mappings:**
   - Ask yourself: does this code implement functionality that is described (or should be described) in any existing knowledge asset?
   - If YES → create mappings to the relevant asset(s) using Workflow C procedures (read the file, decide granularity, batch add mappings)
   - If NO (e.g. test files, generated code, trivial scripts, internal refactoring with no user-facing impact) → skip
   - When unsure → err on the side of creating a mapping; a slightly broad mapping is better than a missing one

   **New document files not registered as assets:**
   - Ask yourself: is this document a knowledge asset that should be tracked by kamap? (e.g. design docs, API docs, user guides, architecture docs)
   - If YES → register it as an asset (`asset add`), then create mappings from relevant code files to this new asset
   - If NO (e.g. changelog entries, release notes, temporary notes) → skip
   - If the new document covers functionality that was previously mapped to a broader document, consider whether to add additional mappings to the new, more specific document

   **Decision guidelines — what to SKIP:**
   - Test files (`*_test.*`, `test_*.*`, `tests/`, `__tests__/`)
   - Generated/compiled files
   - Lock files, dependency manifests (unless they are explicitly tracked assets)
   - Trivial config changes (`.gitignore`, editor configs)
   - Internal refactoring that does not change external behavior or APIs

7. If new assets were registered or new mappings were added, run `{SKILL_DIR}/bin/kamap mapping validate --output json` to verify integrity

### Workflow B: Project Initialization

1. `{SKILL_DIR}/bin/kamap init --output json` to initialize the project
2. Check existing registered assets first:
   ```bash
   {SKILL_DIR}/bin/kamap asset list --output json
   ```
3. Register knowledge assets (writes to **personal** config by default; add `--shared` only if user explicitly requests). **Use batch add when registering multiple assets**:
   ```bash
   # Single asset
   {SKILL_DIR}/bin/kamap asset add --id <id> --provider localfs --type markdown --target <path> --apply --output json

   # Multiple assets (PREFERRED — atomic, no race conditions)
   echo '{"assets":[
     {"id":"<id1>","provider":"localfs","type":"markdown","target":"<path1>"},
     {"id":"<id2>","provider":"localfs","type":"markdown","target":"<path2>"}
   ]}' | {SKILL_DIR}/bin/kamap asset add-batch --stdin --apply --output json
   ```
4. `{SKILL_DIR}/bin/kamap mapping export-context --output json` to export project context
5. Analyze code-document relationships, then batch write mappings (personal config by default):
   ```bash
   echo '{"mappings":[...]}' | {SKILL_DIR}/bin/kamap mapping add-batch --stdin --apply --output json
   ```
6. `{SKILL_DIR}/bin/kamap mapping validate --output json` to verify mapping integrity
7. `{SKILL_DIR}/bin/kamap mapping discover --output json` to find more mapping candidates

### Workflow C: AI-Assisted Mapping Generation

1. `{SKILL_DIR}/bin/kamap mapping export-context --output json` to get project context
2. Analyze the context: code files, existing assets, existing mappings, unmapped code files, unmapped assets
3. **For each unmapped code file that relates to an asset**, decide the mapping granularity:
   a. **Read the file** to understand its structure (functions, structs, classes, modules)
   b. **For large files with distinct sections**: identify which functions/classes relate to which asset, and use `anchor` for each
   c. **For small or single-purpose files**: use whole-file mapping (no anchor)
   d. **For disambiguation**: if a file has multiple items with the same name (e.g. `fn new` in different impl blocks), use `anchor_context`
4. Generate the batch JSON:
   ```json
   {"mappings":[
     {"source_path":"src/commands/scan.rs","asset_id":"readme-zh","anchor":"fn run_scan","reason":"scan 命令主实现","action":"update"},
     {"source_path":"src/commands/scan.rs","asset_id":"readme-zh","anchor":"fn run_ack","reason":"scan ack 子命令实现","action":"update"},
     {"source_path":"src/config/manager.rs","asset_id":"readme-en","anchor":"fn load_merged","reason":"Config loading logic","action":"review"},
     {"source_path":"src/models/source.rs","asset_id":"readme-en","reason":"Source model (whole file)","action":"review"}
   ]}
   ```
   Notice how `scan.rs` has **two separate anchor-based mappings** for different functions, while `source.rs` uses a whole-file mapping since it's a small data model file.
5. `echo '{"mappings":[...]}' | {SKILL_DIR}/bin/kamap mapping add-batch --stdin --apply --output json` to batch write
6. `{SKILL_DIR}/bin/kamap mapping validate --output json` to validate

For detailed information on auto-discovery strategies (@kamap annotations, frontmatter, naming conventions) and complete command parameter reference, see `{SKILL_DIR}/references/detailed-guide.md`. Note: the `mapping discover` CLI command is **temporarily disabled**.
