#!/bin/bash
set -e  # Exit on error
cargo build -p hooktest --quiet
cargo build --example precheck --quiet
./target/debug/hooktest \
    pretool \
    --sessionid test-session-$(date +%s) \
    --tool Bash \
    --input '{"command": "echo Hello from hooktest!"}' \
    -- \
    ./target/debug/examples/precheck
