# Git Verify Tool

A tool for validating git commits against configurable rules defined in a JSON file.

## Usage

### Basic Usage

```bash
# Validate repository in current directory using default config file (validation_rules.json)
./target/debug/git-verify-tool

# Validate a specific repository
./target/debug/git-verify-tool /path/to/repo

# Validate with a custom config file
./target/debug/git-verify-tool /path/to/repo /path/to/custom_rules.json
```

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
  "name": "Validate JSON and CSV files",
  "enabled": true,
  "patterns": ["**/*.json", "**/*.csv"]
}
```

Currently validates:
- `.json` files for valid JSON syntax
- `.csv` files for valid CSV format

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
      "name": "Validate JSON and CSV files",
      "enabled": true,
      "patterns": ["**/*.json", "**/*.csv"]
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

## Dependencies

- `gix` - Git library
- `anyhow` - Error handling
- `serde` & `serde_json` - JSON serialization/deserialization
- `csv` - CSV validation
- `globset` - Glob pattern matching
