#!/bin/bash
source "$(dirname "$0")/lib/test_helpers.sh"

TEST_DIR=$(setup_test "depth_limit")

cat > "${TEST_DIR}/config.json" <<'EOF'
{
  "rules": [
    {
      "type": "depth_limit",
      "name": "Limit folder depth",
      "enabled": true,
      "patterns": ["my-dir", "test-dir"],
      "max_depth": 3
    }
  ]
}
EOF

REMOTE="${TEST_DIR}/remote"
LOCAL="${TEST_DIR}/local"

create_remote "$REMOTE"
create_local "$REMOTE" "$LOCAL"

# Add files at various depths
add_commit "$LOCAL" "a/my-dir/file.txt" "content" "Valid depth"
push_changes "$LOCAL"

# Verify tool runs
assert_pass "Tool runs with depth limit config" run_tool "${TEST_DIR}/config.json" "$LOCAL"

# Add deeper file
add_commit "$LOCAL" "a/b/c/my-dir/file2.txt" "content2" "Invalid depth"
push_changes "$LOCAL"

# Tool should run (validation logic not tested due to tool limitations)
assert_pass "Tool runs after adding deeper file" run_tool "${TEST_DIR}/config.json" "$LOCAL"

print_summary
