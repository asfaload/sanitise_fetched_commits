#!/bin/bash
# Test Helpers Library for Git Verify Tool

# Tool location
TOOL_BIN="${PROJECT_ROOT}/target/release/git-verify-tool"

# Test environment
TEST_ROOT="${TEST_ROOT:-/tmp/git-verify-tests}"
TEST_ID="${TEST_ID:-$(date +%Y%m%d_%H%M%S)_$RANDOM}"
TEST_DIR="${TEST_ROOT}/${TEST_ID}"

# Test counters
TESTS_RUN=0
TESTS_PASSED=0
TESTS_FAILED=0

# Colors
GREEN='\033[0;32m'
RED='\033[0;31m'
NC='\033[0m'

# Setup test directory
setup_test() {
    local test_name="$1"
    mkdir -p "${TEST_DIR}"
    echo "${TEST_DIR}"
}

# Create remote repository
create_remote() {
    git init "$1"
}

# Clone remote to local repository
create_local() {
    git clone "$1" "$2"
    (cd "$2" && git config user.email "test@example.com")
    (cd "$2" && git config user.name "Test User")
}

# Add file and commit
add_commit() {
    local repo="$1"
    local file="$2"
    local content="$3"
    local msg="$4"

    mkdir -p "$(dirname "${repo}/${file}")"
    echo "$content" > "${repo}/${file}"
    (cd "$repo" && git add "$file" && git commit -m "$msg")
}

# Delete file and commit
delete_commit() {
    local repo="$1"
    local file="$2"
    local msg="$3"

    (cd "$repo" && git rm "$file" && git commit -m "$msg")
}

# Modify file and commit
modify_commit() {
    local repo="$1"
    local file="$2"
    local content="$3"
    local msg="$4"

    echo "$content" > "${repo}/${file}"
    (cd "$repo" && git add "$file" && git commit -m "$msg")
}

# Add file and commit on remote repository
add_commit_on_remote() {
    local repo="$1"
    local file="$2"
    local content="$3"
    local msg="$4"

    mkdir -p "$(dirname "${repo}/${file}")"
    echo "$content" > "${repo}/${file}"
    (cd "$repo" && git add "$file" && git commit -m "$msg")
}

# Delete file and commit on remote repository
delete_file_on_remote() {
    local repo="$1"
    local file="$2"
    local msg="$3"

    (cd "$repo" && git rm "$file" && git commit -m "$msg")
}

# Push changes to remote
push_changes() {
    local repo="$1"
    (cd "$repo" && git push -u origin master 2>/dev/null || git push -u origin main 2>/dev/null || git push)
}

# Fetch changes from remote
fetch_changes() {
    local repo="$1"
    (cd "$repo" && git fetch)
}

# Create validation config file
create_config() {
    local path="$1"
    cat > "$path" <<'EOF'
{
  "rules": []
}
EOF
}

# Apply specific rule to config
add_rule_to_config() {
    local config="$1"
    local rule_json="$2"

    # Read existing config
    local content
    content=$(cat "$config")

    # Insert rule before closing brace
    echo "${content%}}}" | sed 's/$/\n  '"${rule_json}"',/' > "${config}.tmp"
    echo "${config}.tmp"
}

# Run the git-verify tool
run_tool() {
    local config="$1"
    local repo="$2"
    "$TOOL_BIN" "$repo" "$config" 2>&1
}

# Get tool output
get_tool_output() {
    local config="$1"
    local repo="$2"
    "$TOOL_BIN" "$repo" "$config" 2>&1
}

# Assertion: command should pass (exit 0)
assert_pass() {
    local desc="$1"
    shift

    if "$@" > /dev/null 2>&1; then
        printf "${GREEN}[PASS]${NC} %s\n" "$desc"
        ((TESTS_PASSED++))
    else
        printf "${RED}[FAIL]${NC} %s\n" "$desc"
        ((TESTS_FAILED++))
    fi
    ((TESTS_RUN++))
}

# Assertion: command should fail (exit != 0)
assert_fail() {
    local desc="$1"
    shift

    if ! "$@" > /dev/null 2>&1; then
        printf "${GREEN}[PASS]${NC} %s\n" "$desc"
        ((TESTS_PASSED++))
    else
        printf "${RED}[FAIL]${NC} %s\n" "$desc"
        ((TESTS_FAILED++))
    fi
    ((TESTS_RUN++))
}

# Assertion: output should contain pattern
assert_output_contains() {
    local desc="$1"
    local output="$2"
    local pattern="$3"

    if echo "$output" | grep -q "$pattern"; then
        printf "${GREEN}[PASS]${NC} %s\n" "$desc"
        ((TESTS_PASSED++))
    else
        printf "${RED}[FAIL]${NC} %s\n" "$desc"
        ((TESTS_FAILED++))
    fi
    ((TESTS_RUN++))
}

# Print test summary
print_summary() {
    echo ""
    echo "Tests: $TESTS_RUN  Passed: $TESTS_PASSED  Failed: $TESTS_FAILED"
    if [ $TESTS_FAILED -eq 0 ]; then
        echo "All tests passed!"
        return 0
    else
        echo "Test artifacts saved to: $TEST_DIR"
        return 1
    fi
}
