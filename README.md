# Git Verify Tool

EXPERIMENTAL! Not production ready.

A tool for validating git commits against configurable rules defined in a JSON file.
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

### Rule Types

#### 1. Content Deletion

Forbids file deletions.

```json
{
  "type": "content_deletion",
  "name": "No file deletions allowed",
  "enabled": true
}
```

#### 2. Depth Limit

Limits how deep specific folder names can appear in the directory structure.

```json
{
  "type": "depth_limit",
  "name": "Limit depth of specific folders",
  "enabled": true,
  "patterns": ["my-dir", "my-dir-pending"],
  "max_depth": 3
}
```

#### 3. Filename Match

Allows or forbids files matching glob patterns.

```json
{
  "type": "filename_match",
  "name": "Forbid temp files",
  "enabled": true,
  "patterns": ["**/tmp/**", "**/*.tmp", "**/temp/**"],
  "action": "forbid"
}
```

Supported actions:
- `forbid` - Reject commits with matching paths
- `require` - Accept commits with matching paths

#### 4. Content Match

Validates content of files matching glob patterns.

```json
{
  "type": "content_match",
  "name": "Validate structured files",
  "enabled": true,
  "patterns": ["**/*.json", "**/*.csv", "**/*.yaml", "**/*.yml", "**/*.toml"]
}
```

Currently validates:
- `.json` files for valid JSON syntax
- `.csv` files for valid CSV format
- `.yaml` / `.yml` files for valid YAML syntax
- `.toml` files for valid TOML syntax

#### 5. Line Deletion

Prevents line deletions in protected files. Detects both full file deletions and individual line removals using line-level diffing.

```json
{
  "type": "line_deletion",
  "name": "No line deletions in protected files",
  "enabled": true,
  "patterns": ["**/*.protected", "**/protected/**"]
}
```

Binary files are skipped with a warning instead of failing.

### Example Configuration

A complete example configuration:

```json
{
  "rules": [
    {
      "type": "content_deletion",
      "name": "No file deletions allowed",
      "enabled": true
    },
    {
      "type": "depth_limit",
      "name": "Limit depth of specific folders",
      "enabled": true,
      "patterns": ["my-dir", "my-dir-pending"],
      "max_depth": 3
    },
    {
      "type": "filename_match",
      "name": "Forbid temp files",
      "enabled": true,
      "patterns": ["**/tmp/**", "**/*.tmp", "**/temp/**"],
      "action": "forbid"
    },
    {
      "type": "content_match",
      "name": "Validate structured files",
      "enabled": true,
      "patterns": ["**/*.json", "**/*.csv", "**/*.yaml", "**/*.toml"]
    },
    {
      "type": "line_deletion",
      "name": "No line deletions in protected files",
      "enabled": true,
      "patterns": ["**/*.protected", "**/protected/**"]
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

- `gix` - Git library
- `anyhow` - Error handling
- `serde` & `serde_json` - JSON serialization/deserialization
- `csv` - CSV validation
- `globset` - Glob pattern matching
- `similar` - Line-level diffing for line deletion detection
- `clap` - CLI argument parsing
- `serde_yaml` - YAML validation
- `toml` - TOML validation
