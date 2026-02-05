#!/bin/bash
source "$(dirname "$0")/lib/test_helpers.sh"

TEST_DIR=$(setup_test "filename_forbid")

REMOTE="${TEST_DIR}/remote"
LOCAL="${TEST_DIR}/local"
CONFIG="${PROJECT_ROOT}/tests/fixtures/configs/rules_filename_forbid.json"

create_remote "$REMOTE"
add_commit_on_remote "$REMOTE" "normal.txt" "valid content" "Add normal file"
create_local "$REMOTE" "$LOCAL"
add_commit_on_remote "$REMOTE" "temp.tmp" "temporary content" "Add forbidden temp file"
fetch_changes "$LOCAL"

OUTPUT=$(get_tool_output "$CONFIG" "$LOCAL"); EXIT_CODE=$?

if [ $EXIT_CODE -ne 0 ]; then
    printf "${GREEN}[PASS]${NC} Tool fails when forbidden filename detected\n"
    ((TESTS_PASSED++))
else
    printf "${RED}[FAIL]${NC} Tool fails when forbidden filename detected\n"
    ((TESTS_FAILED++))
fi
((TESTS_RUN++))
assert_output_contains "Output contains 'forbidden pattern'" "$OUTPUT" "forbidden pattern"

print_summary
