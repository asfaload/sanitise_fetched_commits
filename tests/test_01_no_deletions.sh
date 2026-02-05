#!/bin/bash
source "$(dirname "$0")/lib/test_helpers.sh"

TEST_DIR=$(setup_test "no_deletions")

REMOTE="${TEST_DIR}/remote"
LOCAL="${TEST_DIR}/local"
CONFIG="${PROJECT_ROOT}/tests/fixtures/configs/rules_no_deletion.json"

create_remote "$REMOTE"
add_commit_on_remote "$REMOTE" "important.txt" "important content" "Add important file"
create_local "$REMOTE" "$LOCAL"
delete_file_on_remote "$REMOTE" "important.txt" "Remove important file"
fetch_changes "$LOCAL"

OUTPUT=$(get_tool_output "$CONFIG" "$LOCAL"); EXIT_CODE=$?

assert_fail "Tool fails when deletion detected" get_tool_output "$CONFIG" "$LOCAL" > /dev/null 2>&1
assert_output_contains "Output contains 'Deletion forbidden'" "$OUTPUT" "Deletion forbidden"

print_summary