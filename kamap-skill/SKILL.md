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
10. **CRITICAL — Prefer precise mappings**: When configuring mappings, you MUST strive for the highest possible precision. Avoid mapping an entire file when only a specific section is relevant. Use the `--lines` flag (for `mapping add`) or `source_lines` field (for `mapping add-batch`) to narrow the scope to the exact line range that is related to the target asset. For example, if only lines 20–80 of a file contain the relevant implementation, map `--lines '20-80'` instead of the whole file. Precise mappings significantly reduce false-positive impacts during `scan`, making the tool more useful and less noisy. When analyzing code to generate mappings, take the time to identify the specific functions, structs, or blocks that are truly related to each asset, and specify their line ranges accordingly.

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

**Typical scenario**: After modifying code, run scan to see which documents need synchronization. The output includes `action` field per impact: `update`, `review`, `verify`, or `acknowledge`.

### 2. Mapping Management (mapping)

Establish and manage associations between code and knowledge assets.

> **IMPORTANT**: By default, all mapping write commands (`add`, `add-batch`) write to the **personal** config (`.kamap.yaml`). Only add `--shared` when the user **explicitly** requests shared/team-level mappings. If the user does not mention "shared", "团队", "共享", do NOT use `--shared`.

```bash
# Add a single mapping (dry-run by default, use --apply to write)
# Writes to PERSONAL config by default
{SKILL_DIR}/bin/kamap mapping add \
  --source 'src/auth/**/*.ts' \
  --asset auth-doc \
  --reason '认证模块实现' \
  --action review \
  --apply --output json

# Add to SHARED config (ONLY when user explicitly asks for shared/team config)
{SKILL_DIR}/bin/kamap mapping add \
  --source src/auth/login.ts \
  --asset auth-doc \
  --lines '10-45' \
  --reason 'Login flow' \
  --action update \
  --shared --apply --output json

# Batch add mappings from JSON (via stdin or --file)
echo '{"mappings":[
  {"source_path":"src/foo.rs","asset_id":"my-doc","reason":"实现代码"},
  {"source_path":"src/bar.rs","asset_id":"my-doc","reason":"辅助模块","action":"review","source_lines":[10,45]}
]}' | {SKILL_DIR}/bin/kamap mapping add-batch --stdin --apply --output json

# Batch add from file
{SKILL_DIR}/bin/kamap mapping add-batch --file mappings.json --apply --output json

# List all mappings
{SKILL_DIR}/bin/kamap mapping list --output json

# Filter mappings by asset
{SKILL_DIR}/bin/kamap mapping list --asset my-doc --output json

# Remove a mapping by ID
{SKILL_DIR}/bin/kamap mapping remove --id map_abc123 --output json

# Validate all mappings
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
- `reason` (optional): Why this mapping exists
- `action` (optional): `"review"`, `"update"`, `"verify"`, `"acknowledge"`
- `source_lines` (optional): `[start, end]` line range array
- `segment` (optional): JSON object for targeting specific asset sections

### 3. Asset Management (asset)

Register and manage knowledge assets.

> **IMPORTANT**: Before adding any asset, you MUST first run `asset list` to check all existing registered assets. This avoids duplicate registrations and ensures you are aware of available asset IDs for mapping.
>
> **IMPORTANT**: By default, `asset add` writes to the **personal** config (`.kamap.yaml`). Only add `--shared` when the user **explicitly** requests the asset be shared/team-level. If the user does not mention "shared", "团队", "共享", do NOT use `--shared`.

```bash
# Step 1: ALWAYS list existing assets first
{SKILL_DIR}/bin/kamap asset list --output json

# Step 2: Register a new asset (writes to PERSONAL config by default)
{SKILL_DIR}/bin/kamap asset add \
  --id my-doc \
  --provider localfs \
  --type markdown \
  --target docs/my-doc.md \
  --apply --output json

# Register to SHARED config (ONLY when user explicitly asks for shared/team config)
{SKILL_DIR}/bin/kamap asset add \
  --id my-doc \
  --provider localfs \
  --type markdown \
  --target docs/my-doc.md \
  --shared --apply --output json

# Remove an asset
{SKILL_DIR}/bin/kamap asset remove --id my-doc --output json

# Health check all assets
{SKILL_DIR}/bin/kamap asset check --output json
```

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

### 8. Plugin Management (plugin)

```bash
{SKILL_DIR}/bin/kamap plugin list --output json
{SKILL_DIR}/bin/kamap plugin info --name localfs --output json
```

## Recommended Workflows

### Workflow A: Post-Coding Document Sync

1. After code changes, run `{SKILL_DIR}/bin/kamap scan --output json`
2. Based on the `action` field in scan results:
   - `update`: Directly update the corresponding document
   - `review`: Review whether the document needs updating
   - `verify`: Verify document-code consistency
   - `acknowledge`: Note the change but no document update needed
3. After handling each impact, acknowledge it:
   ```bash
   # Acknowledge specific impacts by mapping ID
   {SKILL_DIR}/bin/kamap scan ack --ids map_abc123,map_def456 --output json
   # Or acknowledge all at once
   {SKILL_DIR}/bin/kamap scan ack --all --output json
   ```
4. After updating documents, scan again to confirm nothing was missed

### Workflow B: Project Initialization

1. `{SKILL_DIR}/bin/kamap init --output json` to initialize the project
2. Check existing registered assets first:
   ```bash
   {SKILL_DIR}/bin/kamap asset list --output json
   ```
3. Register knowledge assets (writes to **personal** config by default; add `--shared` only if user explicitly requests):
   ```bash
   {SKILL_DIR}/bin/kamap asset add --id <id> --provider localfs --type markdown --target <path> --apply --output json
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
3. Generate mapping suggestions as batch JSON — for each mapping, identify the **precise line range** of the relevant code (specific functions, structs, or blocks) and include `source_lines` to avoid over-broad whole-file mappings
4. `echo '{"mappings":[...]}' | {SKILL_DIR}/bin/kamap mapping add-batch --stdin --apply --output json` to batch write
5. `{SKILL_DIR}/bin/kamap mapping validate --output json` to validate

For detailed information on auto-discovery strategies (@kamap annotations, frontmatter, naming conventions) and complete command parameter reference, see `{SKILL_DIR}/references/detailed-guide.md`. Note: the `mapping discover` CLI command is **temporarily disabled**.
