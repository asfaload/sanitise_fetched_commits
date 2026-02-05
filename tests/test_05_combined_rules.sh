#!/bin/bash
source "$(dirname "$0")/lib/test_helpers.sh"

TEST_DIR=$(setup_test "combined_rules")

REMOTE="${TEST_DIR}/remote"
LOCAL="${TEST_DIR}/local"
CONFIG="${PROJECT_ROOT}/tests/fixtures/configs/rules_all.json"

# Setup repositories
create_remote "$REMOTE"
add_commit_on_remote "$REMOTE" "important.txt" "important content" "Add important file"
add_commit_on_remote "$REMOTE" "valid.json" '{"name":"test","value":42}' "Add valid JSON"
add_commit_on_remote "$REMOTE" "my-dir/file.txt" "content at depth 2" "Add file at valid depth"
create_local "$REMOTE" "$LOCAL"

# Add violation 1: deletion (triggers content_deletion rule)
delete_file_on_remote "$REMOTE" "important.txt" "Remove important file"

# Add violation 2: depth limit exceeded (triggers depth_limit rule)
add_commit_on_remote "$REMOTE" "a/b/c/d/my-dir/too-deep.txt" "content at depth 5" "Add file exceeding max depth"

# Add violation 3: forbidden filename (triggers filename_match rule)
add_commit_on_remote "$REMOTE" "temp.tmp" "temporary content" "Add forbidden temp file"

# Add violation 4: invalid JSON (triggers content_match rule)
add_commit_on_remote "$REMOTE" "invalid.json" '{"invalid":}' "Add invalid JSON"

# Fetch all violations in local
fetch_changes "$LOCAL"

# Run tool once on local repository to check all violations
OUTPUT=$(get_tool_output "$CONFIG" "$LOCAL"); EXIT_CODE=$?

# Verify tool fails (some violation detected)
if [ $EXIT_CODE -ne 0 ]; then
    printf "${GREEN}[PASS]${NC} Tool fails when multiple violations detected\n"
    ((TESTS_PASSED++))
else
    printf "${RED}[FAIL]${NC} Tool fails when multiple violations detected\n"
    ((TESTS_FAILED++))
fi
((TESTS_RUN++))

# Verify output contains all error types
assert_output_contains "Output contains 'Deletion forbidden'" "$OUTPUT" "Deletion forbidden"
assert_output_contains "Output contains 'my-dir.*too deep'" "$OUTPUT" "my-dir.*too deep"
assert_output_contains "Output contains 'forbidden pattern'" "$OUTPUT" "forbidden pattern"
assert_output_contains "Output contains 'Invalid JSON'" "$OUTPUT" "Invalid JSON"

print_summary
