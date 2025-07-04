#!/bin/bash

# Test script for Stop hooks

echo "=== Testing Stop Hook ==="
echo

# Build the example hook
echo "Building stop_handler example..."
cargo build --example stop_handler
echo

# Test 1: Normal stop (allow)
echo "Test 1: Normal stop (should allow)"
echo "----------------------------------"
cargo run --bin hooktest -- stop \
    --sessionid "test-123" \
    -- target/debug/examples/stop_handler

echo
echo

# Test 2: Continue session (block)
echo "Test 2: Continue session (should block)"
echo "---------------------------------------"
cargo run --bin hooktest -- stop \
    --sessionid "continue-456" \
    -- target/debug/examples/stop_handler

echo
echo

# Test 3: Stop hook already active (prevent loops)
echo "Test 3: Stop hook active (should allow to prevent loops)"
echo "---------------------------------------------------------"
cargo run --bin hooktest -- stop \
    --sessionid "test-789" \
    --active true \
    -- target/debug/examples/stop_handler

echo
echo

# Test 4: Custom transcript path
echo "Test 4: Custom transcript path"
echo "------------------------------"
cargo run --bin hooktest -- stop \
    --sessionid "test-999" \
    --transcript "/custom/path/transcript.json" \
    -- target/debug/examples/stop_handler