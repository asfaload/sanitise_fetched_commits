#!/bin/bash
source "$(dirname "$0")/lib/test_helpers.sh"

TEST_DIR=$(setup_test "require_action")

CONFIG="${PROJECT_ROOT}/tests/fixtures/configs/rules_filename_require.json"

# --- Sub-test 1 (fail case): Commit without required file should fail ---

REMOTE="${TEST_DIR}/remote1"
LOCAL="${TEST_DIR}/local1"

create_remote "$REMOTE"
add_commit_on_remote "$REMOTE" "initial.txt" "initial content" "Initial commit"
create_local "$REMOTE" "$LOCAL"
add_commit_on_remote "$REMOTE" "data.txt" "some data" "Add data file without changelog"
fetch_changes "$LOCAL"

OUTPUT=$(get_tool_output "$CONFIG" "$LOCAL"); EXIT_CODE=$?

if [ $EXIT_CODE -ne 0 ]; then
    printf "${GREEN}[PASS]${NC} Tool fails when required file is missing\n"
    ((TESTS_PASSED++))
else
    printf "${RED}[FAIL]${NC} Tool fails when required file is missing\n"
    ((TESTS_FAILED++))
fi
((TESTS_RUN++))
assert_output_contains "Output contains 'Required file pattern not found'" "$OUTPUT" "Required file pattern not found"

# --- Sub-test 2 (pass case): Commit with required file should pass ---

REMOTE2="${TEST_DIR}/remote2"
LOCAL2="${TEST_DIR}/local2"

create_remote "$REMOTE2"
add_commit_on_remote "$REMOTE2" "initial.txt" "initial content" "Initial commit"
create_local "$REMOTE2" "$LOCAL2"
add_commit_on_remote "$REMOTE2" "CHANGELOG.md" "## v1.0.0 - Initial release" "Add changelog"
fetch_changes "$LOCAL2"

OUTPUT2=$(run_tool_with_flags "$CONFIG" "$LOCAL2" --verbose); EXIT_CODE2=$?

if [ $EXIT_CODE2 -eq 0 ]; then
    printf "${GREEN}[PASS]${NC} Tool passes when required file is present\n"
    ((TESTS_PASSED++))
else
    printf "${RED}[FAIL]${NC} Tool passes when required file is present\n"
    ((TESTS_FAILED++))
fi
((TESTS_RUN++))
assert_output_contains "Output contains 'matches required pattern'" "$OUTPUT2" "matches required pattern"

print_summary
