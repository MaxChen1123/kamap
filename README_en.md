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

### Typical Skill Scenarios

#### Scenario 1: Batch-configure mappings for existing docs

Just tell the Agent:

> **"Set up mappings for all documents under `docs/`"**

The Agent will automatically:
1. Scan all document files under `docs/` and register them as knowledge assets
2. Analyze which code files and functions each document relates to
3. Create code-to-document mappings accordingly

No need to specify any paths or parameters manually.

#### Scenario 2: Pre-commit impact check

Before committing, tell the Agent:

> **"Show me which document mappings are affected"**

The Agent will:
1. Scan current Git changes and list all impacted documents
2. Tell you which documents may need updating — you can ask the Agent to update them directly
3. **Proactively review unmapped changes**: The Agent also identifies new/modified code and documents that are not yet covered by any mapping, and sets up mappings on its own

#### Scenario 3: Personal config vs shared team config

By default, the Skill writes all configuration (asset registrations, mappings) to the **personal config** `.kamap.yaml` (gitignored).

If you want the configuration written to the **shared team config** `kamap.yaml` (committed to Git), explicitly tell the Agent:

> **"Write to the shared config"** or **"Use team config"**

Otherwise the Agent always defaults to personal config and won't affect the team repository.

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

Scans current Git changes and outputs a list of affected knowledge assets. Each impact includes an `action_prompt` field with specific instructions generated by the asset's Provider. For CI integration, use `kamap check` — it returns a non-zero exit code when error-level impacts are found.

### Provider System

Providers define how kamap generates action prompts (`action_prompt`) when impacts are detected. Built-in providers (`localfs`, `sqlite`) have default prompts; you can define custom providers (e.g. iwiki, notion) in `kamap.yaml` with a `prompt_template`:

```yaml
providers:
  - name: iwiki
    prompt_template: |
      Code changes affected iwiki doc "{{asset.meta.title}}" (doc ID: {{asset.target}}).
      Please read and update the document via iwiki MCP.
```

View registered providers:

```bash
kamap provider list
kamap provider info --name localfs
```

> `kamap plugin` is deprecated. Use `kamap provider` instead.

## Documentation

- [Command Reference & Architecture](kamap-rust/README.md)

## License

MIT
