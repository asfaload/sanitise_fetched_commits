#!/bin/bash
source "$(dirname "$0")/lib/test_helpers.sh"

TEST_DIR=$(setup_test "combined")

# Use the all-rules fixture
cp "$(dirname "$0")/fixtures/configs/rules_all.json" "${TEST_DIR}/config.json"

REMOTE="${TEST_DIR}/remote"
LOCAL="${TEST_DIR}/local"

create_remote "$REMOTE"
create_local "$REMOTE" "$LOCAL"

# Add files that satisfy all rules
add_commit "$LOCAL" "good.json" '{"name":"test"}' "Good JSON"
add_commit "$LOCAL" "a/my-dir/file.txt" "content" "Good depth"
push_changes "$LOCAL"

# Verify tool runs with all rules
assert_pass "Tool runs with combined rules config" run_tool "${TEST_DIR}/config.json" "$LOCAL"

# Add another file
add_commit "$LOCAL" "normal.txt" "content" "Normal file"
push_changes "$LOCAL"

assert_pass "Tool runs after adding file" run_tool "${TEST_DIR}/config.json" "$LOCAL"

print_summary
