#!/bin/bash

PROJECT_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
export PROJECT_ROOT

echo "=== Git Verify Tool - Test Suite ==="
echo ""

# Build
echo "Building git-verify-tool..."
cd "$PROJECT_ROOT"
cargo build --release 2>&1 | tail -1

echo ""
echo "Running tests..."
echo ""

# Run tests sequentially
FAILED=0

for test in tests/test_*.sh; do
    test_name=$(basename "$test")
    echo "=== $test_name ==="
    if ! bash "$test"; then
        FAILED=1
        echo "✗ FAILED"
    else
        echo "✓ PASSED"
    fi
    echo ""
done

# Exit with code
if [ $FAILED -eq 0 ]; then
    echo "All test suites passed!"
    echo "To clean up test artifacts: rm -rf /tmp/git-verify-tests"
    exit 0
else
    echo "Some test suites failed!"
    echo "Test artifacts saved to: /tmp/git-verify-tests"
    exit 1
fi
