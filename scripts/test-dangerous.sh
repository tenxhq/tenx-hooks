#!/bin/bash
set -e  # Exit on error

cargo build -p hooktest --quiet
cargo build --example dangerous_check --quiet

echo ""
echo "Testing SAFE command (should be approved)..."
echo ""

# Test with a safe command
./target/debug/hooktest \
    pretool \
    --sessionid safe-test-$(date +%s) \
    --tool Bash \
    --tool-input command="ls -la" \
    -- \
    ./target/debug/examples/dangerous_check

echo ""
echo "========================================="
echo ""
echo "Testing DANGEROUS command (should be blocked)..."
echo ""

# Test with a dangerous command
./target/debug/hooktest \
    pretool \
    --sessionid danger-test-$(date +%s) \
    --tool Bash \
    --tool-input command="rm -rf /" \
    -- \
    ./target/debug/examples/dangerous_check
