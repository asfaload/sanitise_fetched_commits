#!/bin/bash
source "$(dirname "$0")/lib/test_helpers.sh"

TEST_DIR=$(setup_test "filename_forbid")

cat > "${TEST_DIR}/config.json" <<'EOF'
{
  "rules": [
    {
      "type": "filename_match",
      "name": "Forbid temporary files",
      "enabled": true,
      "patterns": ["**/*.tmp", "**/tmp/**", "**/temp/**"],
      "action": "forbid"
    }
  ]
}
EOF

REMOTE="${TEST_DIR}/remote"
LOCAL="${TEST_DIR}/local"

create_remote "$REMOTE"
create_local "$REMOTE" "$LOCAL"

# Add normal file
add_commit "$LOCAL" "normal.txt" "content" "Normal file"
push_changes "$LOCAL"

# Verify tool runs
assert_pass "Tool runs with filename match config" run_tool "${TEST_DIR}/config.json" "$LOCAL"

# Add temp file
add_commit "$LOCAL" "temp.tmp" "temp content" "Temp file"
push_changes "$LOCAL"

assert_pass "Tool runs after adding temp file" run_tool "${TEST_DIR}/config.json" "$LOCAL"

print_summary
