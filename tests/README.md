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
- `create_remote(path)` - Initialize bare git repository
- `create_local(remote, local)` - Clone from remote

### File Operations
- `add_commit(repo, file, content, msg)` - Add file and commit
- `delete_commit(repo, file, msg)` - Delete file and commit
- `modify_commit(repo, file, content, msg)` - Modify file and commit

### Git Operations
- `push_changes(repo)` - Push commits to remote
- `fetch_changes(repo)` - Fetch from remote

### Assertions
- `assert_pass(description, command)` - Command should succeed (exit 0)
- `assert_fail(description, command)` - Command should fail (exit != 0)

### Reporting
- `print_summary()` - Show test results

## Current Test Limitations

The current tests primarily verify that the tool runs without crashing in various scenarios. Due to limitations in the tool's validation logic:

1. The tool validates commits FROM `origin/master` (remote head) ancestors DOWN TO HEAD (local head)
2. This means HEAD must be an ancestor of `origin/master` for validation to include any commits
3. In normal push-ahead scenarios (local HEAD ahead of origin/master), the validation range is empty

Tests currently verify:
- Tool runs successfully with each rule type
- Config loading works
- Basic repo setup/teardown
- Edge cases (invalid config, missing branches, detached HEAD)

The tests do NOT fully validate that violations are correctly detected because the test framework cannot reliably create scenarios where the tool's validation logic will traverse the commits containing those violations.

## Test Cases

| Test File | Tests | What It Verifies |
|-----------|-------|------------------|
| test_01_no_deletions.sh | 2 | Tool runs with content_deletion rule |
| test_02_depth_limit.sh | 2 | Tool runs with depth_limit rule |
| test_03_filename_match.sh | 2 | Tool runs with filename_match rule |
| test_04_content_match.sh | 2 | Tool runs with content_match rule |
| test_05_combined_rules.sh | 2 | Tool runs with multiple rules |
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
