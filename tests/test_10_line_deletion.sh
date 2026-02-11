#!/bin/bash
source "$(dirname "$0")/lib/test_helpers.sh"

TEST_DIR=$(setup_test "line_deletion")
CONFIG="${PROJECT_ROOT}/tests/fixtures/configs/rules_line_deletion.json"

# --- Test 1: Modification that removes lines should FAIL ---
REMOTE="${TEST_DIR}/remote1"
LOCAL="${TEST_DIR}/local1"
create_remote "$REMOTE"
add_commit_on_remote "$REMOTE" "data.protected" "line1
line2
line3" "Add protected file"
create_local "$REMOTE" "$LOCAL"
modify_commit "$REMOTE" "data.protected" "line1
line3" "Remove a line"
fetch_changes "$LOCAL"
OUTPUT=$(get_tool_output "$CONFIG" "$LOCAL"); EXIT_CODE=$?
if [ $EXIT_CODE -ne 0 ]; then
    printf "${GREEN}[PASS]${NC} Fails when lines deleted in protected file\n"
    ((TESTS_PASSED++))
else
    printf "${RED}[FAIL]${NC} Fails when lines deleted in protected file\n"
    ((TESTS_FAILED++))
fi
((TESTS_RUN++))
assert_output_contains "Output mentions lines deleted" "$OUTPUT" "Lines deleted in protected file"

# --- Test 2: Modification that only ADDS lines should PASS ---
REMOTE2="${TEST_DIR}/remote2"
LOCAL2="${TEST_DIR}/local2"
create_remote "$REMOTE2"
add_commit_on_remote "$REMOTE2" "data.protected" "line1
line2" "Add protected file"
create_local "$REMOTE2" "$LOCAL2"
modify_commit "$REMOTE2" "data.protected" "line1
line2
line3" "Add a line"
fetch_changes "$LOCAL2"
OUTPUT2=$(get_tool_output "$CONFIG" "$LOCAL2"); EXIT_CODE2=$?
if [ $EXIT_CODE2 -eq 0 ]; then
    printf "${GREEN}[PASS]${NC} Passes when only additions in protected file\n"
    ((TESTS_PASSED++))
else
    printf "${RED}[FAIL]${NC} Passes when only additions in protected file\n"
    ((TESTS_FAILED++))
fi
((TESTS_RUN++))

# --- Test 3: Deletion of matching file should FAIL ---
REMOTE3="${TEST_DIR}/remote3"
LOCAL3="${TEST_DIR}/local3"
create_remote "$REMOTE3"
add_commit_on_remote "$REMOTE3" "protected/config.txt" "content" "Add file in protected dir"
create_local "$REMOTE3" "$LOCAL3"
delete_file_on_remote "$REMOTE3" "protected/config.txt" "Delete protected file"
fetch_changes "$LOCAL3"
OUTPUT3=$(get_tool_output "$CONFIG" "$LOCAL3"); EXIT_CODE3=$?
if [ $EXIT_CODE3 -ne 0 ]; then
    printf "${GREEN}[PASS]${NC} Fails when protected file deleted entirely\n"
    ((TESTS_PASSED++))
else
    printf "${RED}[FAIL]${NC} Fails when protected file deleted entirely\n"
    ((TESTS_FAILED++))
fi
((TESTS_RUN++))
assert_output_contains "Output mentions file deleted" "$OUTPUT3" "File deleted"

# --- Test 4: Lines deleted in NON-matching file should PASS ---
REMOTE4="${TEST_DIR}/remote4"
LOCAL4="${TEST_DIR}/local4"
create_remote "$REMOTE4"
add_commit_on_remote "$REMOTE4" "readme.txt" "line1
line2
line3" "Add unprotected file"
create_local "$REMOTE4" "$LOCAL4"
modify_commit "$REMOTE4" "readme.txt" "line1" "Remove lines from unprotected file"
fetch_changes "$LOCAL4"
OUTPUT4=$(get_tool_output "$CONFIG" "$LOCAL4"); EXIT_CODE4=$?
if [ $EXIT_CODE4 -eq 0 ]; then
    printf "${GREEN}[PASS]${NC} Passes when lines deleted in non-protected file\n"
    ((TESTS_PASSED++))
else
    printf "${RED}[FAIL]${NC} Passes when lines deleted in non-protected file\n"
    ((TESTS_FAILED++))
fi
((TESTS_RUN++))

print_summary
