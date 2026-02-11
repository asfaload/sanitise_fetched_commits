#!/bin/bash
source "$(dirname "$0")/lib/test_helpers.sh"

TEST_DIR=$(setup_test "yaml_toml_validation")

CONFIG="${PROJECT_ROOT}/tests/fixtures/configs/rules_content_match_extended.json"

# --- Sub-test 1: Invalid YAML should fail ---

REMOTE1="${TEST_DIR}/remote1"
LOCAL1="${TEST_DIR}/local1"

create_remote "$REMOTE1"
add_commit_on_remote "$REMOTE1" "initial.txt" "initial content" "Initial commit"
create_local "$REMOTE1" "$LOCAL1"

# Add invalid YAML
INVALID_YAML=$(cat "${PROJECT_ROOT}/tests/fixtures/files/invalid.yaml")
add_commit_on_remote "$REMOTE1" "config.yaml" "$INVALID_YAML" "Add invalid YAML"
fetch_changes "$LOCAL1"

OUTPUT1=$(get_tool_output "$CONFIG" "$LOCAL1"); EXIT_CODE1=$?

assert_fail "Tool fails when invalid YAML detected" test "$EXIT_CODE1" -eq 0
assert_output_contains "Output contains 'Invalid YAML'" "$OUTPUT1" "Invalid YAML"

# --- Sub-test 2: Valid YAML should pass ---

REMOTE2="${TEST_DIR}/remote2"
LOCAL2="${TEST_DIR}/local2"

create_remote "$REMOTE2"
add_commit_on_remote "$REMOTE2" "initial.txt" "initial content" "Initial commit"
create_local "$REMOTE2" "$LOCAL2"

VALID_YAML=$(cat "${PROJECT_ROOT}/tests/fixtures/files/valid.yaml")
add_commit_on_remote "$REMOTE2" "config.yaml" "$VALID_YAML" "Add valid YAML"
fetch_changes "$LOCAL2"

OUTPUT2=$(get_tool_output "$CONFIG" "$LOCAL2"); EXIT_CODE2=$?

assert_pass "Tool passes with valid YAML file" test "$EXIT_CODE2" -eq 0

# --- Sub-test 3: Invalid TOML should fail ---

REMOTE3="${TEST_DIR}/remote3"
LOCAL3="${TEST_DIR}/local3"

create_remote "$REMOTE3"
add_commit_on_remote "$REMOTE3" "initial.txt" "initial content" "Initial commit"
create_local "$REMOTE3" "$LOCAL3"

INVALID_TOML=$(cat "${PROJECT_ROOT}/tests/fixtures/files/invalid.toml")
add_commit_on_remote "$REMOTE3" "config.toml" "$INVALID_TOML" "Add invalid TOML"
fetch_changes "$LOCAL3"

OUTPUT3=$(get_tool_output "$CONFIG" "$LOCAL3"); EXIT_CODE3=$?

assert_fail "Tool fails when invalid TOML detected" test "$EXIT_CODE3" -eq 0
assert_output_contains "Output contains 'Invalid TOML'" "$OUTPUT3" "Invalid TOML"

# --- Sub-test 4: Valid TOML should pass ---

REMOTE4="${TEST_DIR}/remote4"
LOCAL4="${TEST_DIR}/local4"

create_remote "$REMOTE4"
add_commit_on_remote "$REMOTE4" "initial.txt" "initial content" "Initial commit"
create_local "$REMOTE4" "$LOCAL4"

VALID_TOML=$(cat "${PROJECT_ROOT}/tests/fixtures/files/valid.toml")
add_commit_on_remote "$REMOTE4" "config.toml" "$VALID_TOML" "Add valid TOML"
fetch_changes "$LOCAL4"

OUTPUT4=$(get_tool_output "$CONFIG" "$LOCAL4"); EXIT_CODE4=$?

assert_pass "Tool passes with valid TOML file" test "$EXIT_CODE4" -eq 0

print_summary
