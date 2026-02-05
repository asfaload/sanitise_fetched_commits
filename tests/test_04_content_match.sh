#!/bin/bash
source "$(dirname "$0")/lib/test_helpers.sh"

TEST_DIR=$(setup_test "content_match")

REMOTE="${TEST_DIR}/remote"
LOCAL="${TEST_DIR}/local"
CONFIG="${PROJECT_ROOT}/tests/fixtures/configs/rules_content_match.json"

create_remote "$REMOTE"
add_commit_on_remote "$REMOTE" "valid.json" '{"name":"test","value":42}' "Add valid JSON"
create_local "$REMOTE" "$LOCAL"
add_commit_on_remote "$REMOTE" "invalid.json" '{"invalid":}' "Add invalid JSON"
fetch_changes "$LOCAL"

OUTPUT=$(get_tool_output "$CONFIG" "$LOCAL"); EXIT_CODE=$?

if [ $EXIT_CODE -ne 0 ]; then
    printf "${GREEN}[PASS]${NC} Tool fails when invalid JSON detected\n"
    ((TESTS_PASSED++))
else
    printf "${RED}[FAIL]${NC} Tool fails when invalid JSON detected\n"
    ((TESTS_FAILED++))
fi
((TESTS_RUN++))
assert_output_contains "Output contains 'Invalid JSON'" "$OUTPUT" "Invalid JSON"

print_summary
