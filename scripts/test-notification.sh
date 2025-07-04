#!/bin/bash

# Test script for Notification hooks

echo "=== Testing Notification Hook ==="
echo

# Build the example hook
echo "Building notification_handler example..."
cargo build --example notification_handler
echo

# Test 1: Normal notification (passthrough)
echo "Test 1: Normal notification (should continue)"
echo "--------------------------------------------"
cargo run --bin hooktest -- notification \
    --sessionid "test-123" \
    --message "Claude needs permission to run a command" \
    --title "Claude Code" \
    -- target/debug/examples/notification_handler

echo
echo

# Test 2: Dangerous operation notification (should stop)
echo "Test 2: Dangerous operation (should stop)"
echo "----------------------------------------"
cargo run --bin hooktest -- notification \
    --sessionid "test-456" \
    --message "Claude wants to run a potentially dangerous command: rm -rf" \
    --title "Claude Code" \
    -- target/debug/examples/notification_handler

echo
echo

# Test 3: Production environment notification (should stop)
echo "Test 3: Production environment (should stop)"
echo "-------------------------------------------"
cargo run --bin hooktest -- notification \
    --sessionid "test-789" \
    --message "Claude wants to modify production database" \
    --title "Claude Code" \
    -- target/debug/examples/notification_handler

echo
echo

# Test 4: Custom notification
echo "Test 4: Custom notification"
echo "---------------------------"
cargo run --bin hooktest -- notification \
    --sessionid "test-999" \
    --message "Build completed successfully" \
    --title "Build Status" \
    -- target/debug/examples/notification_handler