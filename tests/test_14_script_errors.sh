#!/bin/bash
source "$(dirname "$0")/lib/test_helpers.sh"

TEST_DIR=$(setup_test "script_errors")

# Create a minimal git repo so run_tool's pre-check passes
REMOTE="${TEST_DIR}/remote"
LOCAL="${TEST_DIR}/local"
create_remote "$REMOTE"
add_commit_on_remote "$REMOTE" "dummy.txt" "dummy" "Initial commit"
create_local "$REMOTE" "$LOCAL"

# Test 1: Script file not found
assert_fail "Missing script file fails" run_tool \
    "${PROJECT_ROOT}/tests/fixtures/configs/rules_script_missing.json" "$LOCAL"

# Test 2: Script with syntax error
assert_fail "Bad syntax script fails" run_tool \
    "${PROJECT_ROOT}/tests/fixtures/configs/rules_script_bad_syntax.json" "$LOCAL"

# Test 3: Script missing check_change function
assert_fail "Script without check_change fails" run_tool \
    "${PROJECT_ROOT}/tests/fixtures/configs/rules_script_no_check.json" "$LOCAL"

print_summary
