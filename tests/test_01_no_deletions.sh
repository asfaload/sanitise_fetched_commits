#!/bin/bash
source "$(dirname "$0")/lib/test_helpers.sh"

TEST_DIR=$(setup_test "no_deletions")

cat > "${TEST_DIR}/config.json" <<'EOF'
{
  "rules": [
    {
      "type": "content_deletion",
      "name": "No deletions allowed",
      "enabled": true
    }
  ]
}
EOF

# Simple test: create a repo and verify tool runs
REMOTE="${TEST_DIR}/remote"
LOCAL="${TEST_DIR}/local"

create_remote "$REMOTE"
create_local "$REMOTE" "$LOCAL"

# Add some commits
add_commit "$LOCAL" "file1.txt" "content1" "Add file1"
push_changes "$LOCAL"
add_commit "$LOCAL" "file2.txt" "content2" "Add file2"
push_changes "$LOCAL"

# Verify tool runs without crashing
assert_pass "Tool runs successfully on valid repo" run_tool "${TEST_DIR}/config.json" "$LOCAL"

# Test with invalid config
cat > "${TEST_DIR}/bad_config.json" <<'EOF'
{invalid json}
EOF

assert_fail "Tool fails with invalid config" run_tool "${TEST_DIR}/bad_config.json" "$LOCAL"

print_summary
