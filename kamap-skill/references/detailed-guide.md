# kamap Detailed Reference Guide

## Complete Command Parameter Reference

### kamap init

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `--output` | string | No | `"text"` | Output format: text, json |

### kamap scan

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `--base` | string | No | `"HEAD"` | Base Git ref (i.e. latest commit) |
| `--head` | string | No | `"workdir"` | Head Git ref or `"workdir"` for uncommitted changes |
| `--output` / `-o` | string | No | `"text"` | Output format: text, json |
| `--config` | string | No | — | Path to config file (overrides auto-discovery) |

### kamap scan ack

Mark impacts as acknowledged (document already synced). Entries in `.kamap/to-ack.json` are marked as acked, so they won't show up in the next scan on the same HEAD.

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `--all` | bool | One of two* | — | Acknowledge all pending impacts |
| `--ids` | string | One of two* | — | Comma-separated mapping IDs to acknowledge |
| `--output` / `-o` | string | No | `"text"` | Output format: text, json |
| `--config` | string | No | — | Path to config file |

*Must specify either `--all` or `--ids`.

### kamap check

Same parameters as `scan`. Exits with code 1 when error-level impacts are found (CI-friendly).

### kamap explain

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `--mapping` | string | One of three* | — | Mapping ID to explain |
| `--asset` | string | One of three* | — | Asset ID to explain |
| `--source` | string | One of three* | — | Source file path to explain |
| `--output` / `-o` | string | No | `"text"` | Output format: text, json |
| `--config` | string | No | — | Path to config file |

*Must specify exactly one of `--mapping`, `--asset`, or `--source`.

### kamap describe

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `--output` / `-o` | string | No | **`"json"`** | Output format: text, json |

Note: This is the only top-level command that defaults to `json` output.

### kamap mapping (subcommands)

**Global flags** (available on all mapping subcommands):
- `--config <path>` — Override config file path
- `--shared` — Write to `kamap.yaml` (team) instead of `.kamap.yaml` (personal)

#### mapping add

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `--source` | string | **Yes** | — | Source file path or glob pattern |
| `--asset` | string | **Yes** | — | Asset ID |
| `--reason` | string | No | — | Why this mapping exists |
| `--lines` | string | No | — | Line range, format: `"10-45"` |
| `--action` | string | No | — | One of: `review`, `update`, `verify`, `acknowledge` |
| `--apply` | bool | No | `false` | Actually write (default: dry-run) |
| `--output` / `-o` | string | No | `"text"` | Output format |

Auto-generates `map_` prefixed UUID. Origin marked as `"manual"`.

#### mapping add-batch

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `--stdin` | bool | One of two* | — | Read JSON from stdin |
| `--file` | string | One of two* | — | JSON file path |
| `--apply` | bool | No | `false` | Actually write (default: dry-run) |
| `--output` / `-o` | string | No | **`"json"`** | Output format (defaults to json) |

*Must specify either `--stdin` or `--file`.

**Batch JSON input format:**
```json
{
  "mappings": [
    {
      "source_path": "src/foo.rs",
      "asset_id": "my-doc",
      "reason": "Implementation code",
      "anchor": "fn handle_request",
      "anchor_context": "impl Server",
      "action": "review"
    },
    {
      "source_path": "src/config.rs",
      "asset_id": "config-doc",
      "reason": "Config module (whole file)",
      "action": "review"
    }
  ]
}
```

Fields: `source_path` (required), `asset_id` (required), `reason` (optional), `action` (optional), `anchor` (optional, recommended — semantic anchor text for dynamic block-level matching), `anchor_context` (optional — outer scope for disambiguation), `source_lines` (optional, `[start, end]` — deprecated, prefer `anchor`), `segment` (optional, JSON object). Origin is auto-set to `"ai-generated"`.

#### mapping remove

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `--id` | string | **Yes** | — | Mapping ID to remove |
| `--output` / `-o` | string | No | `"text"` | Output format |

#### mapping list

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `--asset` | string | No | — | Filter by asset ID |
| `--output` / `-o` | string | No | `"text"` | Output format |

#### mapping validate

Validates all mapping definitions. Checks include:
- Asset references exist
- Source paths are non-empty
- Line ranges are valid (start ≤ end)
- **Anchor validity**: For exact-path (non-glob) mappings with anchors, verifies that the anchor text can be found in the current source file. Glob mappings with anchors produce a warning that static validation is not possible.

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `--output` / `-o` | string | No | `"text"` | Output format |

#### mapping export

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `--format` | string | No | `"json"` | Export format: json, yaml, csv |

**Note:** Uses `--format` (not `--output`). Prints exported content directly to stdout.

#### mapping import

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `--stdin` | bool | One of two* | — | Read from stdin |
| `--file` | string | One of two* | — | File path |
| `--format` | string | No | `"json"` | Import format: json, yaml |
| `--strategy` | string | No | `"append"` | Merge strategy: append, merge, replace |
| `--apply` | bool | No | `false` | Actually write (default: dry-run) |

*Must specify either `--stdin` or `--file`.

#### mapping export-context

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `--output` / `-o` | string | No | **`"json"`** | Output format (defaults to json) |

### kamap asset (subcommands)

**Global flags** (available on all asset subcommands):
- `--config <path>` — Override config file path
- `--shared` — Write to `kamap.yaml` (team) instead of `.kamap.yaml` (personal)

#### asset add

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `--id` | string | **Yes** | — | Asset ID |
| `--provider` | string | **Yes** | — | Provider: `localfs`, `sqlite` |
| `--type` | string | **Yes** | — | Asset type: `markdown`, `text`, `config`, `sqlite-db` |
| `--target` | string | **Yes** | — | Target path or location |
| `--apply` | bool | No | `false` | Actually write (default: dry-run) |
| `--output` / `-o` | string | No | `"text"` | Output format |

#### asset add-batch

Batch register multiple assets from JSON (atomic operation, avoids race conditions from parallel single adds).

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `--stdin` | bool | One of two* | — | Read JSON from stdin |
| `--file` | string | One of two* | — | JSON file path |
| `--apply` | bool | No | `false` | Actually write (default: dry-run) |
| `--output` / `-o` | string | No | `"text"` | Output format |

*Must specify either `--stdin` or `--file`.

**Batch JSON input format:**
```json
{
  "assets": [
    {"id": "my-doc", "provider": "localfs", "type": "markdown", "target": "docs/my-doc.md"},
    {"id": "readme", "provider": "localfs", "type": "markdown", "target": "README.md"}
  ]
}
```

Fields: `id` (required), `provider` (required), `type` (required), `target` (required).

#### asset remove

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `--id` | string | **Yes** | — | Asset ID to remove |
| `--output` / `-o` | string | No | `"text"` | Output format |

#### asset list

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `--output` / `-o` | string | No | `"text"` | Output format |

#### asset check

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `--output` / `-o` | string | No | `"text"` | Output format |

### kamap index (subcommands)

**Global flag**: `--config <path>`

#### index build

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `--output` / `-o` | string | No | `"text"` | Output format |

Creates/rebuilds SQLite index at `.kamap/index.db`.

#### index stats

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `--output` / `-o` | string | No | `"text"` | Output format |

### kamap provider (subcommands)

**Global flag**: `--config <path>`

#### provider list

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `--output` / `-o` | string | No | `"text"` | Output format |

Lists all providers: built-in (`localfs`, `sqlite`) and custom (defined in config).

#### provider info

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `--name` | string | **Yes** | — | Provider name |
| `--output` / `-o` | string | No | `"text"` | Output format |

### kamap plugin (subcommands) — deprecated

> **Note**: Use `kamap provider` instead. The `plugin` command is retained for backward compatibility.

#### plugin list

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `--output` / `-o` | string | No | `"text"` | Output format |

#### plugin info

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `--name` | string | **Yes** | — | Plugin name |
| `--output` / `-o` | string | No | `"text"` | Output format |

---

## Provider System (v2)

kamap v2 replaces the old plugin system with a **prompt-driven provider architecture**. Providers define how kamap generates action prompts when code changes impact assets.

### Provider types

| Type | Description | `prompt_template` |
|------|-------------|-------------------|
| **builtin** | `localfs`, `sqlite` — shipped with kamap, have default prompts | Optional (override default) |
| **custom** | User-defined providers (notion, confluence, etc.) | Required |

### Configuration

Providers are defined in the `providers` section of `kamap.yaml`:

```yaml
providers:
  - name: notion
    prompt_template: |
      代码变更影响了 Notion 页面「{{asset.meta.title}}」(页面 ID: {{asset.target}})。

      变更来源: {{source.path}}
      影响原因: {{reason}}
      建议操作: {{action}}

      请通过 Notion MCP 完成以下操作：
      1. 调用 getPage(pageId: "{{asset.target}}") 读取页面当前内容
      2. 阅读代码变更，判断页面哪些部分需要更新
      3. 调用 updateBlock 保存修改后的页面

  - name: feishu-doc
    prompt_template: |
      代码变更影响了飞书文档「{{asset.meta.title}}」({{asset.target}})。
      请打开飞书文档进行审查。变更来源: {{source.path}}，原因: {{reason}}
```

### Template variables

| Variable | Description |
|----------|-------------|
| `{{asset.id}}` | Asset ID |
| `{{asset.target}}` | Asset target (path, URL, doc ID, etc.) |
| `{{asset.type}}` | Asset type |
| `{{asset.provider}}` | Provider name |
| `{{asset.meta.*}}` | Any field in asset's `meta` map |
| `{{source.path}}` | Source file path with hunk info |
| `{{source.file}}` | Source file path only |
| `{{source.hunks}}` | Changed line ranges |
| `{{reason}}` | Mapping reason |
| `{{action}}` | Suggested action (review/update/verify/acknowledge) |
| `{{mapping_id}}` | Mapping ID |

### How scan uses providers

When `kamap scan` detects an impact, it:
1. Looks up the asset's `provider` name
2. Finds the matching provider definition in `providers` config
3. Renders the `prompt_template` (or built-in default prompt) with the impact context
4. Includes the rendered `action_prompt` string in the scan output

The Agent then reads `action_prompt` and uses whatever tools are appropriate (MCP, Skill, direct file access, etc.) to handle the impact.

---

## Dual Configuration File Design

kamap uses two configuration files:

| File | Purpose | Commit to Git |
|------|---------|---------------|
| `kamap.yaml` | Team/repo shared configuration | ✅ Yes |
| `.kamap.yaml` | Developer personal configuration | ❌ No |

- By default, all write operations write to `.kamap.yaml` (personal config)
- Use the `--shared` flag to write to `kamap.yaml` (team config)
- Both files are **automatically merged** when loaded
- Commands with write operations that support `--shared`: `mapping add`, `mapping add-batch`, `mapping remove`, `mapping import`, `asset add`, `asset add-batch`, `asset remove`

---

## Built-in Providers

| Provider | Asset Types | Default Prompt |
|----------|-------------|----------------|
| `localfs` | `markdown`, `text`, `config` | "请直接读取 {target} 并根据代码变更进行更新" |
| `sqlite` | `sqlite-db` | "请检查是否需要更新 schema 或数据" |
