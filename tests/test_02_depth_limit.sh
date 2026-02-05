#!/bin/bash
source "$(dirname "$0")/lib/test_helpers.sh"

TEST_DIR=$(setup_test "depth_limit")

REMOTE="${TEST_DIR}/remote"
LOCAL="${TEST_DIR}/local"
CONFIG="${PROJECT_ROOT}/tests/fixtures/configs/rules_depth_limit.json"

create_remote "$REMOTE"
add_commit_on_remote "$REMOTE" "my-dir/file.txt" "content at depth 2" "Add file at valid depth"
create_local "$REMOTE" "$LOCAL"
add_commit_on_remote "$REMOTE" "a/b/c/d/my-dir/too-deep.txt" "content at depth 5" "Add file exceeding max depth"
fetch_changes "$LOCAL"

OUTPUT=$(get_tool_output "$CONFIG" "$LOCAL"); EXIT_CODE=$?

if [ $EXIT_CODE -ne 0 ]; then
    printf "${GREEN}[PASS]${NC} Tool fails when depth limit exceeded\n"
    ((TESTS_PASSED++))
else
    printf "${RED}[FAIL]${NC} Tool fails when depth limit exceeded\n"
    ((TESTS_FAILED++))
fi
((TESTS_RUN++))
assert_output_contains "Output contains 'my-dir' too deep" "$OUTPUT" "my-dir.*too deep"

print_summary
