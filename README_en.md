# kamap

kamap is a Git-based **code-to-knowledge-asset mapping and impact analysis framework**. It establishes explicit mapping relationships between source code and documents (or other knowledge assets), and automatically identifies which documents need to be updated when code changes.

## Other Languages

- [中文](README.md)

## Quick Start

### Build

```bash
cd kamap-rust
cargo build --release -p kamap-cli
```

The binary will be located at `kamap-rust/target/release/kamap`. Add it to your `PATH`.

### Initialize

Run in the root of a Git repository:

```bash
kamap init
```

This generates `kamap.yaml` (shared team config) and `.kamap.yaml` (personal config).

### Register an Asset

```bash
kamap asset add \
  --id api-doc \
  --provider localfs \
  --type markdown \
  --target docs/api.md \
  --apply
```

### Create a Mapping

```bash
# Use a semantic anchor to map precisely to a function/class (recommended)
kamap mapping add \
  --source src/api/handler.rs \
  --asset api-doc \
  --anchor 'fn handle_request' \
  --reason 'Request handler function' \
  --apply

# Use anchor-context for disambiguation (when multiple matches exist)
kamap mapping add \
  --source src/api/handler.rs \
  --asset api-doc \
  --anchor 'fn new' \
  --anchor-context 'impl RequestHandler' \
  --reason 'Constructor' \
  --apply

# Whole-file mapping (for small files or config files)
kamap mapping add \
  --source 'src/api/**/*.rs' \
  --asset api-doc \
  --reason 'API implementation code' \
  --apply
```

> **Semantic anchors** (`--anchor`) are the recommended mapping method introduced in kamap v0.2. Unlike static line ranges (`--lines`), anchors dynamically resolve to the actual location of the target function/class/block, making them resilient to code refactoring. `--lines` is still supported but not recommended.

### Scan for Impacts

```bash
kamap scan
```

Scans current Git changes and outputs a list of affected knowledge assets. For CI integration, use `kamap check` — it returns a non-zero exit code when error-level impacts are found.

## Packaging as a Skill

kamap can be packaged as a Skill for use by AI agents.

```bash
./scripts/build-skill.sh
```

The script compiles a release build and packages the output as `kamap-skill.zip`, containing the binary and Skill description files.

Optional arguments:

```bash
./scripts/build-skill.sh --debug              # debug build
./scripts/build-skill.sh --target <triple>    # cross-compile
```

## Installing the Skill for an Agent

1. Extract `kamap-skill.zip` into the agent's skill directory (e.g., CodeBuddy Code's user-level or project-level skill directory).

2. Once installed, the agent will recognize trigger phrases from `SKILL.md` and automatically activate kamap in scenarios such as:
   - "document sync" / "impact analysis" / "knowledge asset" / "code-doc mapping"
   - "scan changes" / "which docs need updating"

3. Agent usage examples:
   - **"Scan which documents are affected by current changes"** — Agent calls `kamap scan` to analyze impacted assets
   - **"Create a mapping between src/auth and docs/auth.md"** — Agent calls `kamap asset add` + `kamap mapping add`
   - **"Check if any documents need updating"** — Agent calls `kamap check` for CI-level validation

All write operations in the Skill default to dry-run mode and require `--apply` to take effect, ensuring safe agent usage.

## Documentation

- [Command Reference & Architecture](kamap-rust/README.md)

## License

MIT
