#!/bin/bash
source "$(dirname "$0")/lib/test_helpers.sh"

TEST_DIR=$(setup_test "edge_cases")

# Test 1: Invalid config file
BAD_CONFIG="${TEST_DIR}/bad_config.json"
echo '{invalid json}' > "$BAD_CONFIG"

assert_fail "Invalid config fails" run_tool "$BAD_CONFIG" "$TEST_DIR/local"

# Test 2: Missing config file
assert_fail "Missing config fails" run_tool "${TEST_DIR}/nonexistent.json" "/nonexistent"

# Test 3: Empty local repo with no remote
EMPTY_LOCAL="${TEST_DIR}/empty_local"
mkdir -p "$EMPTY_LOCAL"
(cd "$EMPTY_LOCAL"
 git init
 git config user.email "test@example.com"
 git config user.name "Test User"
 git commit --allow-empty -m "Initial commit")

echo '{"rules":[]}' > "${TEST_DIR}/empty_config.json"

assert_fail "No remote branch fails" run_tool "${TEST_DIR}/empty_config.json" "$EMPTY_LOCAL"

# Test 4: Detached HEAD state
REMOTE="${TEST_DIR}/edge_remote"
LOCAL="${TEST_DIR}/edge_local"
CONFIG="${TEST_DIR}/edge_config.json"

cp "$(dirname "$0")/fixtures/configs/rules_no_deletion.json" "$CONFIG"

create_remote "$REMOTE"
create_local "$REMOTE" "$LOCAL"

add_commit "$LOCAL" "file.txt" "content" "Add file"

(cd "$LOCAL"
 git push -u origin $(git branch --show-current) 2>/dev/null || git push
 git checkout HEAD --detach)

assert_fail "Detached HEAD fails" run_tool "$CONFIG" "$LOCAL"

print_summary
