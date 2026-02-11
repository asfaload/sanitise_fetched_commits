#!/bin/bash
source "$(dirname "$0")/lib/test_helpers.sh"

TEST_DIR=$(setup_test "happy_path")

CONFIG="${PROJECT_ROOT}/tests/fixtures/configs/rules_all.json"

# --- Sub-test 1: A single valid commit passes all rules ---

REMOTE="${TEST_DIR}/remote1"
LOCAL="${TEST_DIR}/local1"

create_remote "$REMOTE"
add_commit_on_remote "$REMOTE" "initial.txt" "initial content" "Initial commit"
create_local "$REMOTE" "$LOCAL"
add_commit_on_remote "$REMOTE" "readme.txt" "hello world" "Add a normal text file"
fetch_changes "$LOCAL"

OUTPUT=$(get_tool_output "$CONFIG" "$LOCAL"); EXIT_CODE=$?

if [ $EXIT_CODE -eq 0 ]; then
    printf "${GREEN}[PASS]${NC} Tool passes when no violations exist\n"
    ((TESTS_PASSED++))
else
    printf "${RED}[FAIL]${NC} Tool passes when no violations exist\n"
    ((TESTS_FAILED++))
fi
((TESTS_RUN++))
assert_output_contains "Output contains 'All commits passed validation'" "$OUTPUT" "All commits passed validation"

# --- Sub-test 2: Another valid commit (valid JSON) still passes ---

REMOTE2="${TEST_DIR}/remote2"
LOCAL2="${TEST_DIR}/local2"

create_remote "$REMOTE2"
add_commit_on_remote "$REMOTE2" "initial.txt" "initial content" "Initial commit"
create_local "$REMOTE2" "$LOCAL2"
add_commit_on_remote "$REMOTE2" "data.json" '{"name":"test","value":42}' "Add valid JSON file"
fetch_changes "$LOCAL2"

OUTPUT2=$(get_tool_output "$CONFIG" "$LOCAL2"); EXIT_CODE2=$?

if [ $EXIT_CODE2 -eq 0 ]; then
    printf "${GREEN}[PASS]${NC} Tool passes with valid JSON file\n"
    ((TESTS_PASSED++))
else
    printf "${RED}[FAIL]${NC} Tool passes with valid JSON file\n"
    ((TESTS_FAILED++))
fi
((TESTS_RUN++))
assert_output_contains "Output contains 'All commits passed validation' for JSON test" "$OUTPUT2" "All commits passed validation"

print_summary
