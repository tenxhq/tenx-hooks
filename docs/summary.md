# Claude Code Hooks - Summary

This document summarizes what I can understand about the Claude Code hooks from
the docs, with some guesswork, sleuthing through Github issues, and testing to
guide me. The documentation alone is quite ambiguous.

## Output Visibility: Normal vs Transcript Mode

* **Normal mode (chat pane)**
  • `stderr` from *blocking* exit‑code 2 appears here.
  • `stderr` from non‑blocking exit codes 1 or 3‑255 also appears here. (new)
  • JSON `reason` (Pre / Post / Stop) shows here for user‑directed messages.
  • JSON `stopReason` appears when `continue:false`.

* **Transcript mode (Ctrl‑R)**
  • Always shows the raw `stdout` buffer **unless** `suppressOutput:true`.
  • Shows hook lifecycle logs added by Claude Code itself.

---

## How `stdout` Is Classified 

1. Claude runs **one** `JSON.parse` on the *entire* `stdout`.
2. If that parse succeeds and consumes all bytes ⇒ “structured JSON path”.
3. Otherwise the buffer is treated as **plain text** and the run falls back to
   the simple exit‑code path.
4. Mixed output (JSON plus banner text, blank lines, etc.) therefore **always
   becomes plain text**. (new)

`stderr` is never parsed as JSON; it is merely surfaced according to the exit
code.

---

## PreToolUse

Runs **before** a tool executes. Can approve, rewrite, or block the call.

### Input (stdin)

`session_id`, `transcript_path`, `tool_name`, `tool_input` (all unchanged).

### Exit codes

* **0** → Structured JSON path *or* plain‑text success.
* **2** → Blocks the tool; `stderr` text passed to Claude as the block reason.
* **1 or 3‑255** → Non‑blocking warning; `stderr` text shown to the user. (new)

### JSON output (exit 0)

Supported keys:

```
decision        "approve" | "block"   -- omit for passthrough
reason          string                -- user sees when approve, Claude when block
continue        true | false          -- stop Claude entirely (default true)
stopReason      string                -- visible when continue=false
suppressOutput  true | false          -- hide raw JSON from transcript (default false)
```

**Decision types:**
- `"approve"`: Bypasses the permission system, tool executes immediately
- `"block"`: Prevents tool execution, reason is passed to Claude as feedback
- **Passthrough** (decision omitted/null): Defers to Claude's regular approval
  flow - the agent may show an approval dialogue or proceed based on its
  configuration

---

## PostToolUse

Runs **after** a tool finishes successfully.

### Input (stdin)

`session_id`, `transcript_path`, `tool_name`, `tool_input`, `tool_response`.

### Exit codes

* **0** → Structured JSON path or plain‑text success.
* **2** → Sends automated *feedback* message to Claude (content =`stderr`).
* **1 or 3‑255** → Non‑blocking warning; `stderr` shown to user.

### JSON output (exit 0)

Same keys as PreToolUse, except `decision` semantics:

**`decision` field:**
- `"block"`: Suppresses normal tool_result message. Injects `reason` as error message to Claude.
- Omitted/undefined: Normal tool_result passed to Claude. Any `reason` is discarded.
- No `"approve"` option (tool already executed).

**Key behaviors:**
- `"block"` makes Claude see an error instead of the actual tool output
- `continue: false` always overrides `decision` - halts session after tool
  completes
- Exit 0 required for JSON path; exit 2 similar to block but uses stderr

---

## Notification

Runs when Claude emits a notification (e.g., “Shell command finished”).

### Input (stdin)

`session_id`, `transcript_path`, `tool_name`, `notification_type`, `title`, `body`.

### Exit codes

* **0** → JSON or plain‑text success.
* **2** → *No special handling* (acts like a warning).
* **1 or 3‑255** → Warning; `stderr` shown to user.

### JSON output (exit 0)

Only these keys are valid:

```
continue        true | false
stopReason      string
suppressOutput  true | false
```

---

## Stop

Runs after Claude thinks it is finished responding.

### Input (stdin)

`session_id`, `transcript_path`.

### Exit codes

* **0** → JSON or plain‑text success (lets Claude end).
* **2** → Cancels the stop; Claude keeps running.
* **1 or 3‑255** → Warning; Claude still stops, `stderr` shown to user.

### JSON output (exit 0)

Supported keys:

```
decision        "block"               -- omit for passthrough
reason          string                -- tells Claude how to proceed when blocked
continue        true | false          -- stop Claude entirely (default true)
stopReason      string                -- visible when continue=false
suppressOutput  true | false          -- hide raw JSON from transcript (default false)
```

**Decision types for Stop:**
- `"block"`: Prevents Claude from stopping, forces continuation with the provided reason
- **Passthrough** (decision omitted/null): Allows Claude to stop normally

An empty matcher in `settings.json` makes this hook fire once for *every*
sub‑agent. Guard with a matcher or `stop_hook_active` to avoid loops. (new)

---

## Key Points

1. The two output interfaces are **mutually exclusive**:
   – Structured JSON path = exit 0 *and* valid JSON in `stdout`.
   – Simple exit‑code path = any other combination.

2. **Only exit 2 blocks or alters flow.**
   Exit 1 and 3‑255 merely warn; they never stop tool execution by themselves.

3. `suppressOutput` hides the raw JSON blob **after** it has already been
   parsed, keeping the transcript clean without sacrificing control.

4. Mixed JSON + text is impossible; any non‑JSON byte drops you into the simple path.

5. Always guard Stop hooks when sub‑agents are enabled, or they will trigger once per agent.

6. Template interpolation and hot‑reload remain future features; design your
   implementation around the current behaviour.
