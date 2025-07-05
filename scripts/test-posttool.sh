#!/bin/bash

# Test script for PostToolUse hooks

echo "=== Testing PostToolUse Hook ==="
echo

# Build the example hook
echo "Building posttool_logger example..."
cargo build --example posttool_logger
echo

# Test 1: Normal command (passthrough)
echo "Test 1: Normal command (should passthrough)"
echo "-------------------------------------------"
cargo run --bin hooktest -- posttool \
    --sessionid "test-123" \
    --tool "Bash" \
    --tool-input command="ls -la" \
    --tool-response output="total 24\ndrwxr-xr-x  2 user user 4096 Jan 1 12:00 .\n" \
    -- target/debug/examples/posttool_logger

echo
echo

# Test 2: Command with sensitive data (blocked)
echo "Test 2: Command with sensitive data (should block)"
echo "-------------------------------------------------"
cargo run --bin hooktest -- posttool \
    --sessionid "test-456" \
    --tool "Bash" \
    --tool-input command="export DB_PASSWORD=secret123" \
    --tool-response output="" \
    -- target/debug/examples/posttool_logger

echo
echo

# Test 3: Tool other than Bash
echo "Test 3: Different tool (Read)"
echo "-----------------------------"
cargo run --bin hooktest -- posttool \
    --sessionid "test-789" \
    --tool "Read" \
    --tool-input file_path="/etc/hosts" \
    --tool-response content="127.0.0.1 localhost\n" \
    -- target/debug/examples/posttool_logger
