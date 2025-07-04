# Claude Code Hooks - Summary

This document summarizes the Claude Code hooks system, incorporating official documentation and clarifications from recent issue discussions. The official docs have ambiguities, so this summary includes corrections and detail from GitHub issues and testing.

## Output Visibility: Normal vs Transcript Mode

* **Normal mode (chat pane)**
  - `stderr` from *blocking* exit‑code 2 appears here.
  - `stderr` from non‑blocking exit codes 1 or 3‑255 also appears here. (New in
    recent versions)
  - JSON `reason` fields (from PreToolUse/PostToolUse/Stop hooks) appear here
    as user-visible messages.
  - JSON `stopReason` appears here when a hook sets `continue:false`
    (explaining why Claude stopped).

* **Transcript mode (Ctrl‑R)**
  - Always shows the raw `stdout` content **unless** `suppressOutput:true`.
  - Shows hook lifecycle logs and debug messages added by Claude Code itself
    (e.g. “Running PreToolUse hook…”).

---

## How `stdout` Is Classified

1. For exit code 0 Claude attempts a `JSON.parse` on the *entire* `stdout`
   buffer.
2. If parsing succeeds and consumes the entire output, the run follows the
   **structured JSON path**.
3. If parsing fails (or extra non-JSON bytes are present), the output is
   treated as **plain text** and the run falls back to the simple exit-code
   path.
4. **Mixed output** (JSON plus any extra text, banner, etc.) therefore **always
   falls back to plain text** – partial JSON is not allowed.

`stderr` is never parsed as JSON; it’s handled purely based on the exit code.

---

## PreToolUse

Runs **before** a tool executes. Can pre-approve execution, block the call, or
defer to Claude’s normal approval flow.

### Input (stdin)

`session_id`, `transcript_path`, `tool_name`, `tool_input` (all as provided by Claude).

### Exit codes

* **0** – Hook succeeded. Follows structured JSON path *if* stdout is valid
  JSON; otherwise treated as plain-text success.
* **2** – Blocks the tool call. The `stderr` text is fed back to Claude as an
  automatic error/feedback message.
* **1** or **3-255** – Non-blocking warning. The `stderr` text is shown to the
  user (Claude still proceeds with the tool). (New behavior in latest versions)

### JSON output (exit 0)

Supported keys in `stdout` JSON:

```
decision        "approve" | "block"    – omit for passthrough
reason          string                 – shown to user if approve, to Claude if block
continue        true | false           – whether Claude should continue overall (default true)
stopReason      string                 – shown to user when continue=false
suppressOutput  true | false           – hide raw JSON output from transcript (default false)
```

**Decision types:**

* `"approve"` – Bypasses the normal permission system; the tool executes
  immediately (the `reason` is shown to the user but not to Claude).
* `"block"` – Prevents the tool from executing; the `reason` is passed to
  Claude as feedback (Claude treats it like an error).
* **Passthrough** (decision omitted or `undefined`) – Defers to Claude’s
  regular approval flow. The tool call is neither auto-approved nor blocked by
  the hook (Claude may prompt for approval or decide based on settings). Any
  provided `reason` is ignored in this case.

*Currently, modifying `tool_input` in the JSON output has no effect; PreToolUse
hooks cannot rewrite tool parameters in this version.*

---

## PostToolUse

Runs **after** a tool finishes successfully.

### Input (stdin)

`session_id`, `transcript_path`, `tool_name`, `tool_input`, `tool_response`
(the parameters and result of the tool that just ran).

### Exit codes

* **0** – Hook succeeded. Follows structured JSON path if stdout is valid JSON,
  else plain-text success (hook output shown in transcript).
* **2** – Provides automated *feedback* to Claude. The `stderr` text is
  injected as an error for Claude to process (as if the tool failed).
* **1** or **3-255** – Non-blocking warning. The `stderr` text is shown to the
  user, but Claude still receives the normal tool result.

### JSON output (exit 0)

Uses the same keys as PreToolUse hooks. The `decision` field here controls
post-execution handling:

* `"block"` – Suppresses the normal tool result message. Claude will instead
  receive an error with the provided `reason` (as if the tool failed).
* Omitted/`undefined` – No special action on the result. The normal tool result
  is passed through to Claude, and any `reason` is discarded.
* (No `"approve"` option, since the tool has already run.)

**Key behaviors:**

* `"block"` causes Claude to treat the tool as if it errored – the hook’s
  `reason` is delivered to Claude as the error message.
* `continue: false` in the JSON will override any `decision` and halt Claude
  completely after this hook (ending the session).
* Only exit 0 with a well-formed JSON will enter the structured path. An exit 2
  achieves a similar “block” outcome but via `stderr` instead of JSON.

---

## Notification

Runs when Claude Code emits a notification (e.g. “Shell command finished.”).

### Input (stdin)

`session_id`, `transcript_path`, `hook_event_name`, `message`.

*In practice the `title` field shown in docs is omitted; a `hook_event_name`
(value `"Notification"`) is included instead.*

### Exit codes

* **0** – Hook succeeded. JSON output (if any) is parsed, otherwise any plain
  stdout is treated as a success message (shown in transcript).
* **2** – No special handling (does **not** block Claude; exit 2 is treated
  like a non-blocking error here).
* **1** or **3-255** – Non-blocking warning; `stderr` text is shown to the
  user.

### JSON output (exit 0)

For Notification hooks, only these keys are recognized:

```
continue        true | false
stopReason      string
suppressOutput  true | false
```

There is no `decision` field for Notification hooks (it cannot approve/block
notifications). Setting `continue:false` will stop Claude (and show
`stopReason` to the user), whereas leaving it `true` lets Claude proceed
normally. `suppressOutput:true` hides the hook’s stdout from the transcript
(useful if the hook prints its own info).

---

## Stop

Runs after Claude believes it has finished responding (at the end of a session
turn). A Stop hook can conditionally prevent Claude from stopping.

### Input (stdin)

`session_id`, `transcript_path` (optional `stop_hook_active`).

*If a Stop hook already forced continuation earlier, `stop_hook_active: true`
will be present in the input.*

### Exit codes

* **0** – Hook succeeded. If stdout is valid JSON, it’s parsed for control;
  otherwise it’s a plain-text success and Claude is allowed to stop normally.
* **2** – Cancels Claude’s stopping: Claude will continue running as if the
  user requested it to keep going. (`stderr` text is fed to Claude as an
  instruction for continuation.)
* **1** or **3-255** – Non-blocking warning. Claude still stops, and the
  `stderr` warning is shown to the user.

### JSON output (exit 0)

Supported keys:

```
decision        "block"               – omit for passthrough
reason          string                – tells Claude how to proceed when blocked
continue        true | false          – stop Claude entirely after this hook (default true)
stopReason      string                – shown to user when continue=false
suppressOutput  true | false          – hide raw JSON from transcript (default false)
```

**Decision types for Stop:**

* `"block"` – Prevents Claude from stopping. Claude will instead continue as if
  the conversation is ongoing, using the provided `reason` as guidance (this
  `reason` is required to tell Claude what to do next).
* **Passthrough** (decision omitted or `undefined`) – Allows Claude to stop
  normally (the hook imposes no change).

If `continue:false` is set in the JSON, Claude will terminate the session
entirely (this overrides any `"block"` decision). Use `stopReason` to provide a
final message to the user in that case.

An unfiltered Stop hook (matcher set to `""`) will fire for *every* agent in a
multi-agent scenario – use a specific matcher or the `stop_hook_active` flag to
avoid infinite loops if Claude keeps continuing (as noted in docs).

---

## Transcript JSONL Format

Each Claude Code session is logged to a JSON Lines (`.jsonl`) file at the path
given by `transcript_path`. Each line in this file is a JSON object
representing one message or event in the conversation:

* **System (init)** – The first line of a session, with `type: "system"` and
  `subtype: "init"`. Contains session metadata like the model name, loaded
  tools, working directory, etc.
* **User** – A user message, with `type: "user"` and a `message` payload
  containing the user’s prompt.
* **Assistant** – An assistant message, with `type: "assistant"` and a
  `message` object containing Claude’s response. This may include code outputs
  or other actions taken by Claude. Hidden “thinking” steps and tool usage
  details (inputs and outputs) are also recorded here (they are not shown to
  the user in the UI, but they appear in the transcript log).
* **Result** – The final entry, with `type: "result"` (usually with `subtype:
  "success"`). This summarizes the outcome and includes metrics like duration,
  token usage, and possibly the final answer or status message.

---

## Key Points

* The two output handling paths are **mutually exclusive**
    * *Structured JSON path*: Requires exit 0 **and** a valid JSON object on
      `stdout`.
    * *Simple exit-code path*: Occurs for any other exit code or if `stdout` is not
      pure JSON.

* **Only exit 2 can automatically block or alter the flow.**
    * Exit 1 and 3-255 serve only as warnings; they never stop tool execution
      or Claude’s response on their own.

* `suppressOutput: true` hides the raw JSON blob in transcript mode **after**
   it has been parsed. This keeps the transcript cleaner without sacrificing
   the hook’s control logic.

* Mixed JSON + text output is impossible – any non-JSON byte in `stdout`
   causes Claude to treat the entire output as plain text (fallback path), not
   as structured output.

