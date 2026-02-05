#!/bin/bash
# Test Helpers Library for Git Verify Tool

# Project root (default to parent of tests directory if not set)
PROJECT_ROOT="${PROJECT_ROOT:-$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)}"

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
# Parameters: repo (path to git repository), file (relative path to file), content (file content), msg (commit message)
# Returns: 0 on success, 1 on failure
add_commit() {
    local repo="$1"
    local file="$2"
    local content="$3"
    local msg="$4"

    if [ ! -d "$repo/.git" ]; then
        echo "Error: Not a git repository: $repo" >&2
        return 1
    fi

    if [ -z "$file" ]; then
        echo "Error: File path required" >&2
        return 1
    fi

    if [ -z "$msg" ]; then
        echo "Error: Commit message required" >&2
        return 1
    fi

    mkdir -p "$(dirname "${repo}/${file}")" || return 1
    echo "$content" > "${repo}/${file}" || return 1

    (cd "$repo" && git add "$file" && git commit -m "$msg") || return 1
}

# Delete file and commit
# Parameters: repo (path to git repository), file (relative path to file), msg (commit message)
# Returns: 0 on success, 1 on failure
delete_commit() {
    local repo="$1"
    local file="$2"
    local msg="$3"

    if [ ! -d "$repo/.git" ]; then
        echo "Error: Not a git repository: $repo" >&2
        return 1
    fi

    if [ -z "$file" ]; then
        echo "Error: File path required" >&2
        return 1
    fi

    if [ -z "$msg" ]; then
        echo "Error: Commit message required" >&2
        return 1
    fi

    (cd "$repo" && git rm "$file" && git commit -m "$msg") || return 1
}

# Modify file and commit
# Parameters: repo (path to git repository), file (relative path to file), content (new file content), msg (commit message)
# Returns: 0 on success, 1 on failure
modify_commit() {
    local repo="$1"
    local file="$2"
    local content="$3"
    local msg="$4"

    if [ ! -d "$repo/.git" ]; then
        echo "Error: Not a git repository: $repo" >&2
        return 1
    fi

    if [ -z "$file" ]; then
        echo "Error: File path required" >&2
        return 1
    fi

    if [ -z "$msg" ]; then
        echo "Error: Commit message required" >&2
        return 1
    fi

    echo "$content" > "${repo}/${file}" || return 1
    (cd "$repo" && git add "$file" && git commit -m "$msg") || return 1
}

# Add file and commit on remote repository (wrapper for add_commit)
# Parameters: repo (path to git repository), file (relative path to file), content (file content), msg (commit message)
# Returns: 0 on success, 1 on failure
add_commit_on_remote() {
    add_commit "$@"
}

# Delete file and commit on remote repository (wrapper for delete_commit)
# Parameters: repo (path to git repository), file (relative path to file), msg (commit message)
# Returns: 0 on success, 1 on failure
delete_file_on_remote() {
    delete_commit "$@"
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
# Parameters: config (path to config file), repo (path to git repository)
# Returns: 0 on success, 1 on failure, outputs tool stdout and stderr
run_tool() {
    local config="$1"
    local repo="$2"

    if [ ! -f "$repo/.git/HEAD" ] 2>/dev/null && [ ! -d "$repo/.git" ]; then
        echo "Error: Not a git repository: $repo" >&2
        return 1
    fi

    if [ ! -f "$config" ]; then
        echo "Error: Config file not found: $config" >&2
        return 1
    fi

    if [ ! -x "$TOOL_BIN" ]; then
        echo "Error: Tool not found or not executable: $TOOL_BIN" >&2
        return 1
    fi

    "$TOOL_BIN" "$repo" "$config" 2>&1
}

# Get tool output (wrapper for run_tool)
# Parameters: config (path to config file), repo (path to git repository)
# Returns: 0 on success, 1 on failure, outputs tool stdout and stderr
get_tool_output() {
    run_tool "$@"
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
# Parameters: desc (test description), output (tool output string), pattern (grep pattern to match)
# Returns: none, updates test counters
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
