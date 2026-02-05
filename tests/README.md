# Git Verify Tool - Test Suite

This directory contains bash integration tests for the git-verify-tool.

## Running Tests

Run all tests:
```bash
./tests/run_all_tests.sh
```

Run a single test:
```bash
bash tests/test_01_no_deletions.sh
```

Clean up test artifacts:
```bash
rm -rf /tmp/git-verify-tests
```

## Test Structure

- `lib/test_helpers.sh` - Main test library with reusable functions
- `test_*.sh` - Individual test files
- `fixtures/configs/` - Sample validation rule configurations
- `fixtures/files/` - Sample files for content validation tests
- `run_all_tests.sh` - Test suite runner

## Test Library Functions

The `lib/test_helpers.sh` provides these helper functions:

### Setup & Cleanup
- `setup_test(name)` - Create isolated test directory
- `create_remote(path)` - Initialize git repository (remote)
- `create_local(remote, local)` - Clone from remote to local repository

### File Operations
- `add_commit(repo, file, content, msg)` - Add file and commit
- `delete_commit(repo, file, msg)` - Delete file and commit
- `modify_commit(repo, file, content, msg)` - Modify file and commit
- `add_commit_on_remote(repo, file, content, msg)` - Add file and commit on remote (wraps add_commit)
- `delete_file_on_remote(repo, file, msg)` - Delete file and commit on remote (wraps delete_commit)

### Git Operations
- `push_changes(repo)` - Push commits to remote
- `fetch_changes(repo)` - Fetch from remote

### Configuration
- `create_config(path)` - Create default empty validation config file
- `add_rule_to_config(config, rule_json)` - Add a specific rule to existing config file

### Tool Execution
- `run_tool(config, repo)` - Run git-verify-tool with config on repository, returns stdout+stderr
- `get_tool_output(config, repo)` - Get tool output (wrapper for run_tool)

### Assertions
- `assert_pass(description, command)` - Command should succeed (exit 0)
- `assert_fail(description, command)` - Command should fail (exit != 0)
- `assert_output_contains(desc, output, pattern)` - Verify output contains expected pattern

### Reporting
- `print_summary()` - Show test results

## Test Approach

The tests validate git-verify-tool functionality by simulating a realistic workflow:

1. **Create remote repository** - Initialize a git repository as the "remote"
2. **Add initial commits** - Set up baseline state on the remote
3. **Clone to local** - Create a local clone of the remote
4. **Add violations on remote** - Push changes that violate validation rules to the remote
5. **Fetch in local** - Local clone fetches the new commits containing violations
6. **Run tool on local** - Execute git-verify-tool on the local clone
7. **Verify detection** - Check that tool detects violations (exit code and output)

This approach works because the tool validates commits between HEAD and origin/master. By:
- Adding violations to the remote repository
- Fetching those changes in a local clone (which updates origin/master references)
- Running the tool which validates the commits between local HEAD and the fetched remote state

The tests verify both:
- **Exit codes**: Non-zero when violations detected
- **Output patterns**: Expected violation messages in tool output

### Example Test Pattern

```bash
# 1. Setup test environment
TEST_DIR=$(setup_test "my_test")
REMOTE="${TEST_DIR}/remote"
LOCAL="${TEST_DIR}/local"
CONFIG="${PROJECT_ROOT}/tests/fixtures/configs/my_rule.json"

# 2. Create remote and add valid baseline
create_remote "$REMOTE"
add_commit_on_remote "$REMOTE" "good.txt" "good content" "Good file"

# 3. Create local clone
create_local "$REMOTE" "$LOCAL"

# 4. Add violation on remote
add_commit_on_remote "$REMOTE" "bad.txt" "bad content" "Bad file"

# 5. Fetch violation in local
fetch_changes "$LOCAL"

# 6. Run tool and capture output
OUTPUT=$(get_tool_output "$CONFIG" "$LOCAL"); EXIT_CODE=$?

# 7. Verify tool detected violation
if [ $EXIT_CODE -ne 0 ]; then
    printf "${GREEN}[PASS]${NC} Tool failed on violation\n"
    ((TESTS_PASSED++))
fi

# 8. Verify output contains expected message
assert_output_contains "Output contains expected message" "$OUTPUT" "expected pattern"
```

## Test Cases

| Test File | Tests | What It Verifies |
|-----------|-------|------------------|
| test_01_no_deletions.sh | 2 | content_deletion rule detects file deletions |
| test_02_depth_limit.sh | 2 | depth_limit rule validates commit history depth |
| test_03_filename_match.sh | 2 | filename_match rule detects matching filenames |
| test_04_content_match.sh | 2 | content_match rule detects matching file patterns |
| test_05_combined_rules.sh | 2 | Multiple rules work together correctly |
| test_06_edge_cases.sh | 4 | Error handling (bad config, missing remote, detached HEAD) |

## Writing New Tests

1. Source the test library:
```bash
source "$(dirname "$0")/lib/test_helpers.sh"
```

2. Create test directory and setup:
```bash
TEST_DIR=$(setup_test "my_test")
REMOTE="${TEST_DIR}/remote"
LOCAL="${TEST_DIR}/local"
CONFIG="${TEST_DIR}/config.json"
```

3. Use helper functions to create repos and commits
4. Use assertions to verify expected behavior
5. Call `print_summary` at the end

## Adding Fixture Configs

To add a new validation rules fixture:

```bash
cat > tests/fixtures/configs/my_rules.json <<'EOF'
{
  "rules": [
    {
      "type": "rule_type",
      "name": "Rule Name",
      "enabled": true,
      ...
    }
  ]
}
EOF
```
