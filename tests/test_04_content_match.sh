#!/bin/bash
source "$(dirname "$0")/lib/test_helpers.sh"

TEST_DIR=$(setup_test "content_match")

cat > "${TEST_DIR}/config.json" <<'EOF'
{
  "rules": [
    {
      "type": "content_match",
      "name": "Validate JSON and CSV",
      "enabled": true,
      "patterns": ["**/*.json", "**/*.csv"]
    }
  ]
}
EOF

REMOTE="${TEST_DIR}/remote"
LOCAL="${TEST_DIR}/local"

create_remote "$REMOTE"
create_local "$REMOTE" "$LOCAL"

# Add valid JSON
add_commit "$LOCAL" "valid.json" '{"name":"test","value":42}' "Valid JSON"
push_changes "$LOCAL"

# Verify tool runs
assert_pass "Tool runs with content match config" run_tool "${TEST_DIR}/config.json" "$LOCAL"

# Add invalid JSON
add_commit "$LOCAL" "invalid.json" '{"invalid":}' "Bad JSON"
push_changes "$LOCAL"

assert_pass "Tool runs after adding invalid JSON" run_tool "${TEST_DIR}/config.json" "$LOCAL"

print_summary
