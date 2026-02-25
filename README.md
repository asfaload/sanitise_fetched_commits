# Git Verify Tool

EXPERIMENTAL! Not production ready.

A tool for validating git commits against configurable rules written as Rhai scripts and defined in a JSON file.
Vibe-coded exploration that might be the base for a production tool.

## Usage

### Basic Usage

```bash
# Validate repository in current directory using default config file (validation_rules.json)
./target/release/git-verify-tool

# Validate a specific repository
./target/release/git-verify-tool /path/to/repo

# Validate with a custom config file
./target/release/git-verify-tool /path/to/repo /path/to/custom_rules.json
```

### CLI Flags

```bash
# Show help
./target/release/git-verify-tool --help

# Show version
./target/release/git-verify-tool --version

# Verbose mode: show passes and warnings, not just violations
./target/release/git-verify-tool --verbose

# Dry-run mode: report violations but always exit 0
./target/release/git-verify-tool --dry-run

# JSON output: machine-readable structured output
./target/release/git-verify-tool --format json

# Custom remote name (default: origin)
./target/release/git-verify-tool --remote upstream

# Custom commit range: validate commits from REF_A to REF_B
./target/release/git-verify-tool --from <older-ref> --to <newer-ref>

# Validate from a specific commit up to the remote tracking branch
./target/release/git-verify-tool --from abc1234

# Validate from HEAD up to a specific commit
./target/release/git-verify-tool --to def5678
```

### Exit Codes

- `0` — All commits passed validation (or `--dry-run` mode)
- `1` — Validation failed (violations found)

### Building

```bash
cargo build --release
```

## Configuration

The tool requires a `validation_rules.json` configuration file in the current directory (or specify a custom path).

### Rule Format

All rules use the `"script"` type and point to a Rhai (`.rhai`) script file:

```json
{
  "type": "script",
  "name": "Human-readable rule name",
  "enabled": true,
  "script": "path/to/rule.rhai"
}
```

- **type** — Always `"script"`.
- **name** — A descriptive name shown in output.
- **enabled** — Set to `false` to skip the rule without removing it.
- **script** — Path to the `.rhai` script file. Resolved relative to the config file's directory.

### Writing a Rule Script

A rule script must define a `check_change(ctx)` function. It is called once for every file change in a commit. Optionally define a `finalize()` function that runs after all changes in a commit have been processed (useful for rules that need to see the full commit before deciding).

#### The `ctx` Object

`ctx` is a map passed to `check_change` with these fields:

| Field | Type | Description |
|-------|------|-------------|
| `path` | string | File path relative to repository root |
| `kind` | string | One of `"addition"`, `"deletion"`, `"modification"`, `"rewrite"` |
| `added_lines` | array of strings | Lines added in this change (empty for deletions) |
| `deleted_lines` | array of strings | Lines removed in this change (empty for additions) |
| `content` | string | Full file content after the change (empty for deletions) |

#### Return Values

Use the helper functions to return a result from `check_change` or `finalize`:

- `violation(msg)` — The change violates the rule. Causes a non-zero exit code.
- `pass(msg)` — The change explicitly passes (shown in `--verbose` mode).
- `warning(msg)` — Advisory message, does not cause failure.
- Return nothing (no explicit return, or early `return;`) to silently pass.

#### Stateful Rules with `this` and `finalize()`

The `this` keyword persists state between `check_change` calls within a single commit. State is automatically reset between commits. Use this together with `finalize()` for rules that need to inspect the full set of changes before deciding, such as requiring a specific file to be present.

```rhai
fn check_change(ctx) {
    if glob_match("**/CHANGELOG.md", ctx.path) {
        this.found = true;
        return pass("CHANGELOG.md found: " + ctx.path);
    }
}

fn finalize() {
    if this.found != true {
        return violation("CHANGELOG.md is required but was not found in commit");
    }
}
```

#### Helper Functions

These functions are available inside Rhai scripts:

| Function | Description |
|----------|-------------|
| `violation(msg)` | Return a violation result |
| `pass(msg)` | Return a pass result |
| `warning(msg)` | Return a warning result |
| `glob_match(pattern, path)` | Test if `path` matches a glob `pattern` |
| `validate_json(text)` | Returns `true` if `text` is valid JSON |
| `validate_csv(text)` | Returns `true` if `text` is valid CSV |
| `validate_yaml(text)` | Returns `true` if `text` is valid YAML |
| `validate_toml(text)` | Returns `true` if `text` is valid TOML |

### Example Scripts

#### Forbid file deletions

```rhai
fn check_change(ctx) {
    if ctx.kind == "deletion" {
        return violation("Deletion forbidden: " + ctx.path);
    }
}
```

#### Forbid files matching a pattern

```rhai
fn check_change(ctx) {
    if ctx.kind == "deletion" {
        return;
    }
    if glob_match("**/*.tmp", ctx.path) || glob_match("**/tmp/**", ctx.path) {
        return violation("Path matches forbidden pattern: " + ctx.path);
    }
}
```

#### Validate structured file content

```rhai
fn check_change(ctx) {
    if ctx.kind == "deletion" {
        return;
    }
    if glob_match("**/*.json", ctx.path) {
        if !validate_json(ctx.content) {
            return violation("Invalid JSON: " + ctx.path);
        }
    }
    if glob_match("**/*.yaml", ctx.path) || glob_match("**/*.yml", ctx.path) {
        if !validate_yaml(ctx.content) {
            return violation("Invalid YAML: " + ctx.path);
        }
    }
}
```

#### Prevent line deletions in protected files

```rhai
fn check_change(ctx) {
    if !glob_match("**/*.protected", ctx.path) && !glob_match("**/protected/**", ctx.path) {
        return;
    }
    if ctx.kind == "deletion" {
        return violation("File deleted (all lines removed): " + ctx.path);
    }
    if ctx.deleted_lines.len() > 0 {
        return violation("Lines deleted in protected file: " + ctx.path);
    }
}
```

### Example Configuration

A complete example configuration with multiple script rules:

```json
{
  "rules": [
    {
      "type": "script",
      "name": "No deletions allowed",
      "enabled": true,
      "script": "scripts/content_deletion.rhai"
    },
    {
      "type": "script",
      "name": "Forbid temporary files",
      "enabled": true,
      "script": "scripts/filename_forbid.rhai"
    },
    {
      "type": "script",
      "name": "Validate JSON and CSV",
      "enabled": true,
      "script": "scripts/content_match.rhai"
    },
    {
      "type": "script",
      "name": "Require CHANGELOG",
      "enabled": true,
      "script": "scripts/filename_require.rhai"
    }
  ]
}
```

## How It Works

1. Takes a path to a git repository as the first argument (default: current directory)
2. Detects the currently checked-out branch
3. Constructs the remote reference path (e.g., `refs/remotes/origin/main`)
4. Walks commits from the remote back to HEAD
5. Validates each commit against all enabled rules
6. Exits with code 0 if all commits pass, 1 otherwise

## Commit Walking Behavior

The tool walks commits using **first-parent-only traversal**. For merge commits, only the mainline (first parent) is followed — side-branch commits introduced by the merge are not individually validated.

### Default Range

By default, the tool resolves the remote tracking branch (e.g., `refs/remotes/origin/main`) as the start and `HEAD` as the stop. It walks backward from the remote ref and validates each commit until it reaches `HEAD`.

### Custom Range with `--from` / `--to`

- `--to <ref>` overrides the start of the walk (default: remote tracking branch). The tool walks backward from this ref.
- `--from <ref>` overrides the stop point (default: HEAD). The walk stops when this ref is reached.
- Both flags accept any git ref: commit hashes, branch names, tags, etc.

| Flags | Behavior |
|-------|----------|
| *(none)* | Detect branch, walk from remote tracking ref to HEAD |
| `--to REF` | Walk from REF to HEAD (skip branch/remote detection) |
| `--from REF` | Walk from remote tracking ref to REF |
| `--from A --to B` | Walk from B to A (skip all auto-detection) |

### Empty Range

If no commits are found in the specified range (e.g., `--from` and `--to` point to the same commit), the tool prints a warning and exits with code 0. In JSON mode, the report includes a `"warning"` field.

## Dependencies

- `rhai` - Embedded scripting engine for rule scripts
- `gix` - Git library
- `anyhow` - Error handling
- `serde` & `serde_json` - JSON serialization/deserialization
- `csv` - CSV validation (exposed to scripts via `validate_csv`)
- `globset` - Glob pattern matching (exposed to scripts via `glob_match`)
- `similar` - Line-level diffing for computing `added_lines`/`deleted_lines`
- `clap` - CLI argument parsing
- `serde_yaml` - YAML validation (exposed to scripts via `validate_yaml`)
- `toml` - TOML validation (exposed to scripts via `validate_toml`)
