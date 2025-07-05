# hooktest

Testing utility for Claude Code hooks. Simulates hook execution with custom inputs.


## Usage

```bash
hooktest <HOOK_TYPE> [OPTIONS] -- <HOOK_COMMAND> [ARGS...]
```

## Example

Test a pre-tool hook that blocks dangerous commands:

```bash
# Test with a safe command
hooktest pretool --tool Bash --tool-input command="ls -la" -- ./my-hook

# Test with a dangerous command
hooktest pretool --tool Bash --tool-input command="rm -rf /" -- ./my-hook

# Complex inputs with JSON
hooktest posttool --tool Write \
  --tool-input file_path="/tmp/test.txt" \
  --tool-response output="File written" \
  --tool-response-json bytes_written=42 \
  -- ./my-hook
```

## Hook Types

- `pretool`: Test pre-tool execution hooks
- `posttool`: Test post-tool execution hooks  
- `notification`: Test notification hooks
- `stop`: Test stop event hooks
- `subagentstop`: Test subagent stop hooks

