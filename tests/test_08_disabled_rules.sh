#!/bin/bash
source "$(dirname "$0")/lib/test_helpers.sh"

TEST_DIR=$(setup_test "disabled_rules")

REMOTE="${TEST_DIR}/remote"
LOCAL="${TEST_DIR}/local"
CONFIG="${PROJECT_ROOT}/tests/fixtures/configs/rules_disabled.json"

create_remote "$REMOTE"
add_commit_on_remote "$REMOTE" "initial.txt" "initial content" "Initial commit"
create_local "$REMOTE" "$LOCAL"
add_commit_on_remote "$REMOTE" "temp.tmp" "temporary content" "Add forbidden temp file"
fetch_changes "$LOCAL"

OUTPUT=$(get_tool_output "$CONFIG" "$LOCAL"); EXIT_CODE=$?

if [ $EXIT_CODE -eq 0 ]; then
    printf "${GREEN}[PASS]${NC} Tool passes when rule is disabled\n"
    ((TESTS_PASSED++))
else
    printf "${RED}[FAIL]${NC} Tool passes when rule is disabled\n"
    ((TESTS_FAILED++))
fi
((TESTS_RUN++))
assert_output_contains "Output contains 'All commits passed'" "$OUTPUT" "All commits passed"

print_summary
