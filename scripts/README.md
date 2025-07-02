# Test Scripts

This directory contains convenience scripts for testing hooks with the `hooktest` utility.

## Available Scripts

### test-precheck.sh
Tests the basic `precheck` example that approves all commands.

```bash
./scripts/test-precheck.sh
```

### test-dangerous.sh
Tests the `dangerous_check` example with both safe and dangerous commands to demonstrate approval and blocking behavior.

```bash
./scripts/test-dangerous.sh
```

### test-with-cargo.sh
Shows how to use `hooktest` with `cargo run` during development, useful when you're actively working on hooks.

```bash
./scripts/test-with-cargo.sh
```

## Usage

All scripts:
1. Build the necessary binaries
2. Run `hooktest` with appropriate parameters
3. Display colorized output showing what Claude and users would see

These scripts are useful for:
- Quick testing during development
- Demonstrating hook behavior
- CI/CD integration
- Documentation examples