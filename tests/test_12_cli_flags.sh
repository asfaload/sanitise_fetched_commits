#!/bin/bash
source "$(dirname "$0")/lib/test_helpers.sh"

TEST_DIR=$(setup_test "cli_flags")

# --- Setup: Create a repo with a violation (deletion) ---

CONFIG="${PROJECT_ROOT}/tests/fixtures/configs/rules_no_deletion.json"

REMOTE="${TEST_DIR}/remote"
LOCAL="${TEST_DIR}/local"

create_remote "$REMOTE"
add_commit_on_remote "$REMOTE" "important.txt" "important content" "Add important file"
create_local "$REMOTE" "$LOCAL"
delete_file_on_remote "$REMOTE" "important.txt" "Remove important file"
fetch_changes "$LOCAL"

# --- Sub-test 1: --dry-run exits 0 even with violations ---

OUTPUT1=$(run_tool_with_flags "$CONFIG" "$LOCAL" --dry-run); EXIT_CODE1=$?

if [ "$EXIT_CODE1" -eq 0 ]; then
    printf "${GREEN}[PASS]${NC} --dry-run exits 0 on violations\n"
    ((TESTS_PASSED++))
else
    printf "${RED}[FAIL]${NC} --dry-run exits 0 on violations (got exit code $EXIT_CODE1)\n"
    ((TESTS_FAILED++))
fi
((TESTS_RUN++))

assert_output_contains "Dry-run output contains violation" "$OUTPUT1" "Deletion forbidden"
assert_output_contains "Dry-run output contains '(dry-run)'" "$OUTPUT1" "dry-run"

# --- Sub-test 2: --verbose shows Pass lines ---

# Use a config with a require rule and a commit that satisfies it
CONFIG_REQUIRE="${PROJECT_ROOT}/tests/fixtures/configs/rules_filename_require.json"

REMOTE2="${TEST_DIR}/remote2"
LOCAL2="${TEST_DIR}/local2"

create_remote "$REMOTE2"
add_commit_on_remote "$REMOTE2" "initial.txt" "initial content" "Initial commit"
create_local "$REMOTE2" "$LOCAL2"
add_commit_on_remote "$REMOTE2" "CHANGELOG.md" "## v1.0" "Add changelog"
fetch_changes "$LOCAL2"

OUTPUT_VERBOSE=$(run_tool_with_flags "$CONFIG_REQUIRE" "$LOCAL2" --verbose); EXIT_VERBOSE=$?
OUTPUT_QUIET=$(run_tool "$CONFIG_REQUIRE" "$LOCAL2"); EXIT_QUIET=$?

# Verbose should show Pass lines
assert_output_contains "--verbose shows Pass lines" "$OUTPUT_VERBOSE" "matches required pattern"

# Non-verbose should not show Pass lines
if echo "$OUTPUT_QUIET" | grep -q "matches required pattern"; then
    printf "${RED}[FAIL]${NC} Without --verbose, Pass lines are hidden\n"
    ((TESTS_FAILED++))
else
    printf "${GREEN}[PASS]${NC} Without --verbose, Pass lines are hidden\n"
    ((TESTS_PASSED++))
fi
((TESTS_RUN++))

# --- Sub-test 3: --format json produces valid JSON ---

OUTPUT_JSON=$(run_tool_with_flags "$CONFIG" "$LOCAL" --format json); EXIT_JSON=$?

# Try to parse the JSON output (use python since it's commonly available)
if echo "$OUTPUT_JSON" | python3 -m json.tool > /dev/null 2>&1; then
    printf "${GREEN}[PASS]${NC} --format json produces valid JSON\n"
    ((TESTS_PASSED++))
else
    printf "${RED}[FAIL]${NC} --format json produces valid JSON\n"
    echo "JSON output was: $OUTPUT_JSON"
    ((TESTS_FAILED++))
fi
((TESTS_RUN++))

# Check that JSON contains expected fields
assert_output_contains "JSON contains 'commits_checked'" "$OUTPUT_JSON" "commits_checked"
assert_output_contains "JSON contains 'violations'" "$OUTPUT_JSON" "total_violations"
assert_output_contains "JSON contains 'passed'" "$OUTPUT_JSON" '"passed"'

# --- Sub-test 4: --remote with custom remote name ---

# Using a non-existent remote should fail gracefully
OUTPUT_REMOTE=$(run_tool_with_flags "$CONFIG" "$LOCAL" --remote nonexistent 2>&1); EXIT_REMOTE=$?

if [ "$EXIT_REMOTE" -ne 0 ]; then
    printf "${GREEN}[PASS]${NC} --remote with invalid remote fails gracefully\n"
    ((TESTS_PASSED++))
else
    printf "${RED}[FAIL]${NC} --remote with invalid remote fails gracefully\n"
    ((TESTS_FAILED++))
fi
((TESTS_RUN++))

assert_output_contains "Error mentions remote name" "$OUTPUT_REMOTE" "nonexistent"

print_summary
