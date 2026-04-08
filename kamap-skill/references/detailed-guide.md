# kamap Detailed Reference Guide

## Complete Command Parameter Reference

### kamap init

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `--output` | string | No | `"text"` | Output format: text, json |

### kamap scan

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `--base` | string | No | `"origin/main"` | Base Git ref |
| `--head` | string | No | `"HEAD"` | Head Git ref |
| `--output` / `-o` | string | No | `"text"` | Output format: text, json |
| `--config` | string | No | — | Path to config file (overrides auto-discovery) |

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
      "action": "review",
      "source_lines": [10, 45],
      "segment": {"heading": "API Reference"}
    }
  ]
}
```

Fields: `source_path` (required), `asset_id` (required), `reason` (optional), `action` (optional), `source_lines` (optional, `[start, end]`), `segment` (optional, JSON object). Origin is auto-set to `"ai-generated"`.

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

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `--output` / `-o` | string | No | `"text"` | Output format |

#### mapping discover (temporarily disabled)

> **Note**: This subcommand is currently disabled. The parameters below are for reference only.

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `--include-low-confidence` | bool | No | `false` | Include low confidence candidates |
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

### kamap plugin (subcommands)

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

## Dual Configuration File Design

kamap uses two configuration files:

| File | Purpose | Commit to Git |
|------|---------|---------------|
| `kamap.yaml` | Team/repo shared configuration | ✅ Yes |
| `.kamap.yaml` | Developer personal configuration | ❌ No |

- By default, all write operations write to `.kamap.yaml` (personal config)
- Use the `--shared` flag to write to `kamap.yaml` (team config)
- Both files are **automatically merged** when loaded
- Commands with write operations that support `--shared`: `mapping add`, `mapping add-batch`, `mapping remove`, `mapping import`, `asset add`, `asset remove`

---

## Mapping Auto-Discovery Strategies (CLI temporarily disabled)

> **Note**: The `mapping discover` CLI command is temporarily disabled. The discovery strategies described below are still implemented in `kamap-core` and can be re-enabled in the future.

### @kamap Code Annotations (Confidence: 0.9)

Declare mapping relationships in code comments:

```rust
// @kamap asset:auth-doc reason:"认证逻辑" segment:{"heading":"Login"}
pub fn login() { ... }
```

```python
# @kamap asset:data-doc reason:"数据模型定义"
class User(BaseModel): ...
```

Supported comment formats: `//`, `#`, `/* */`, `--`

### Markdown Frontmatter (Confidence: 0.85)

Declare relationships in Markdown document headers:

```markdown
---
kamap:
  relates-to:
    - path: src/auth/**/*.ts
      reason: "认证模块实现"
    - path: src/auth/login.ts
      lines: "10-45"
      segment:
        heading: "Login Flow"
---
```

### Naming Conventions (Confidence: 0.6, disabled by default)

Auto-infer through directory naming rules:

```yaml
discovery:
  naming:
    enabled: true
    rules:
      - source: "src/{module}/**"
        asset_pattern: "docs/{module}.md"
```

---

## Built-in Plugins

| Plugin | Provider | Asset Types | Capabilities |
|--------|----------|-------------|-------------|
| Local File | `localfs` | `markdown`, `text`, `config` | Segment parsing, content reading, health check, metadata, mapping discovery |
| SQLite | `sqlite` | `sqlite-db` | Segment parsing (table names), health check, metadata |
