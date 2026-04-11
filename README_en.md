# kamap

> *Every commit silently makes your docs a little more outdated.*

**kamap** tracks how code changes affect your documents, keeping them always in sync.

kamap is a Git-based **code-to-knowledge-asset mapping and impact analysis framework**. It establishes explicit mapping relationships between source code and documents (or other knowledge assets), and automatically identifies which documents need to be updated when code changes.

## Other Languages

- [中文](README.md)

## Recommended: Use kamap as an Agent Skill

**We recommend installing kamap as a Skill for your AI Agent** rather than using the CLI manually. With the Skill, your Agent can automatically perform impact analysis, mapping management, and document sync checks during your coding workflow — no need to memorize any commands.

### 1. Build the Skill

```bash
./scripts/build-skill.sh
```

The script compiles a release build and packages the output as `kamap-skill.zip`, containing the binary and Skill description files.

Optional arguments:

```bash
./scripts/build-skill.sh --debug              # debug build
./scripts/build-skill.sh --target <triple>    # cross-compile
```

> Requires a Rust toolchain. If not installed, see [rustup.rs](https://rustup.rs/).

### 2. Install the Skill

Extract `kamap-skill.zip` into the agent's skill directory.

### 3. Usage

Once installed, the agent will recognize trigger phrases from `SKILL.md` and automatically activate kamap in scenarios such as:

- "document sync" / "impact analysis" / "knowledge asset" / "code-doc mapping"
- "scan changes" / "which docs need updating"

Agent usage examples:

- **"Scan which documents are affected by current changes"** — Agent calls `kamap scan` to analyze impacted assets
- **"Create a mapping between src/auth and docs/auth.md"** — Agent calls `kamap asset add` + `kamap mapping add`
- **"Check if any documents need updating"** — Agent calls `kamap check` for CI-level validation

All write operations in the Skill default to dry-run mode and require `--apply` to take effect, ensuring safe agent usage.

### Why Use the Skill?

- **Zero memorization**: No need to remember CLI commands or flags — just describe your intent in natural language
- **Automated workflows**: The Agent handles the full "check existing assets → register assets → create mappings → validate" pipeline automatically
- **Safe by default**: All write operations are dry-run; the Agent confirms before executing

## Manual CLI Usage

If you don't use an AI Agent, or need to integrate with CI/CD, you can use the CLI directly.

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

## Documentation

- [Command Reference & Architecture](kamap-rust/README.md)

## License

MIT
