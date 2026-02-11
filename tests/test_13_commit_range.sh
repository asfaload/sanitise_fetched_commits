#!/bin/bash
source "$(dirname "$0")/lib/test_helpers.sh"

TEST_DIR=$(setup_test "commit_range")

CONFIG="${PROJECT_ROOT}/tests/fixtures/configs/rules_no_deletion.json"

REMOTE="${TEST_DIR}/remote"
LOCAL="${TEST_DIR}/local"

# --- Setup: repo with 4 commits (A=add, B=add, C=delete/violation, D=add) ---

create_remote "$REMOTE"
add_commit_on_remote "$REMOTE" "file_a.txt" "content A" "Commit A"
COMMIT_A=$(cd "$REMOTE" && git rev-parse HEAD)

create_local "$REMOTE" "$LOCAL"

add_commit_on_remote "$REMOTE" "file_b.txt" "content B" "Commit B"
COMMIT_B=$(cd "$REMOTE" && git rev-parse HEAD)

delete_file_on_remote "$REMOTE" "file_a.txt" "Commit C - delete file"
COMMIT_C=$(cd "$REMOTE" && git rev-parse HEAD)

add_commit_on_remote "$REMOTE" "file_d.txt" "content D" "Commit D"
COMMIT_D=$(cd "$REMOTE" && git rev-parse HEAD)

fetch_changes "$LOCAL"

# --- Test 1: --from A --to D (range includes violation) ---

OUTPUT1=$(run_tool_with_flags "$CONFIG" "$LOCAL" --from "$COMMIT_A" --to "$COMMIT_D" 2>&1); EXIT1=$?

if [ "$EXIT1" -ne 0 ]; then
    printf "${GREEN}[PASS]${NC} --from A --to D exits non-zero (violation in range)\n"
    ((TESTS_PASSED++))
else
    printf "${RED}[FAIL]${NC} --from A --to D exits non-zero (violation in range) (got exit code $EXIT1)\n"
    ((TESTS_FAILED++))
fi
((TESTS_RUN++))

assert_output_contains "Range A..D contains deletion violation" "$OUTPUT1" "Deletion forbidden"

# --- Test 2: --from A --to B (range skips violation) ---

OUTPUT2=$(run_tool_with_flags "$CONFIG" "$LOCAL" --from "$COMMIT_A" --to "$COMMIT_B" 2>&1); EXIT2=$?

if [ "$EXIT2" -eq 0 ]; then
    printf "${GREEN}[PASS]${NC} --from A --to B exits 0 (no violation in range)\n"
    ((TESTS_PASSED++))
else
    printf "${RED}[FAIL]${NC} --from A --to B exits 0 (no violation in range) (got exit code $EXIT2)\n"
    ((TESTS_FAILED++))
fi
((TESTS_RUN++))

# --- Test 3: --from B --to B (empty range) ---

OUTPUT3=$(run_tool_with_flags "$CONFIG" "$LOCAL" --from "$COMMIT_B" --to "$COMMIT_B" 2>&1); EXIT3=$?

if [ "$EXIT3" -eq 0 ]; then
    printf "${GREEN}[PASS]${NC} --from B --to B exits 0 (empty range)\n"
    ((TESTS_PASSED++))
else
    printf "${RED}[FAIL]${NC} --from B --to B exits 0 (empty range) (got exit code $EXIT3)\n"
    ((TESTS_FAILED++))
fi
((TESTS_RUN++))

assert_output_contains "Empty range shows warning" "$OUTPUT3" "No commits found"

# --- Test 4: --from B --to B --format json (JSON warning field) ---

OUTPUT4=$(run_tool_with_flags "$CONFIG" "$LOCAL" --from "$COMMIT_B" --to "$COMMIT_B" --format json 2>&1); EXIT4=$?

if [ "$EXIT4" -eq 0 ]; then
    printf "${GREEN}[PASS]${NC} --from B --to B --format json exits 0\n"
    ((TESTS_PASSED++))
else
    printf "${RED}[FAIL]${NC} --from B --to B --format json exits 0 (got exit code $EXIT4)\n"
    ((TESTS_FAILED++))
fi
((TESTS_RUN++))

assert_output_contains "JSON contains warning field" "$OUTPUT4" '"warning"'

# --- Test 5: --from nonexistent_ref (bad ref) ---

OUTPUT5=$(run_tool_with_flags "$CONFIG" "$LOCAL" --from nonexistent_ref 2>&1); EXIT5=$?

if [ "$EXIT5" -ne 0 ]; then
    printf "${GREEN}[PASS]${NC} --from nonexistent_ref exits non-zero\n"
    ((TESTS_PASSED++))
else
    printf "${RED}[FAIL]${NC} --from nonexistent_ref exits non-zero (got exit code $EXIT5)\n"
    ((TESTS_FAILED++))
fi
((TESTS_RUN++))

assert_output_contains "Error mentions bad ref" "$OUTPUT5" "nonexistent_ref"

# --- Test 6: --to nonexistent_ref (bad ref) ---

OUTPUT6=$(run_tool_with_flags "$CONFIG" "$LOCAL" --to nonexistent_ref 2>&1); EXIT6=$?

if [ "$EXIT6" -ne 0 ]; then
    printf "${GREEN}[PASS]${NC} --to nonexistent_ref exits non-zero\n"
    ((TESTS_PASSED++))
else
    printf "${RED}[FAIL]${NC} --to nonexistent_ref exits non-zero (got exit code $EXIT6)\n"
    ((TESTS_FAILED++))
fi
((TESTS_RUN++))

assert_output_contains "Error mentions bad ref" "$OUTPUT6" "nonexistent_ref"

# --- Test 7: --to B alone (HEAD as stop) ---

OUTPUT7=$(run_tool_with_flags "$CONFIG" "$LOCAL" --to "$COMMIT_B" 2>&1); EXIT7=$?

if [ "$EXIT7" -eq 0 ]; then
    printf "${GREEN}[PASS]${NC} --to B alone exits 0 (HEAD as stop)\n"
    ((TESTS_PASSED++))
else
    printf "${RED}[FAIL]${NC} --to B alone exits 0 (HEAD as stop) (got exit code $EXIT7)\n"
    ((TESTS_FAILED++))
fi
((TESTS_RUN++))

print_summary
