# Issue 819: local path discovery before web search

Issue [#819](https://github.com/link-assistant/formal-ai/issues/819) reported
that the request “Find hive-mind-control center folder on my desktop” opened a
web search and returned unrelated GitHub pages. The requested directory was
local:

```text
/Users/konard/Desktop/Archive/hive-control-center
```

## Outcome

Formal AI now treats an explicit local scope as an intent boundary. A request
with a seeded discovery action, local scope, and optional file/folder kind is
lowered to one client-side `find` command before shared Glob or web-search
routing is considered. The reported request produces:

```sh
find "${FORMAL_AI_DESKTOP_DIR:-$HOME/Desktop}" -type d \( \
  -iname '*hive*mind*control*center*' -o \
  -iname '*mind*control*center*' -o \
  -iname '*hive*control*center*' -o \
  -iname '*hive*mind*center*' -o \
  -iname '*hive*mind*control*' \
\) -print -quit
```

The exact query is tried first. One-token-omission variants then cover the
reported `hive-mind-control center` → `hive-control-center` mismatch without a
product- or repository-specific alias. The search stops at its first result.

The vocabulary covers desktop, home/computer, and current-directory scopes;
file and directory predicates; and English, Russian, Hindi, and Chinese action,
scope, and kind phrases. An exhaustive unit regression walks every declared
phrase. Open-web prompts without a local scope continue to use `websearch`.

A 56-case benchmark crosses those four languages with all three scopes and both
object kinds. Its first red run found a second, more general ambiguity:
“current directory” supplied the scope but `directory` was also being reused as
the requested object kind. Scope and action phrases are now removed before kind
classification, so “find the product brief document in the current directory”
correctly emits `-type f`.

## Reproduction and proof

The pre-fix regression routed the exact prompt to `websearch` and failed all
four local-path assertions. Its output is preserved in
[`raw-data/tests/reproduction-before-fix.log`](raw-data/tests/reproduction-before-fix.log).

The E2E harness creates an isolated directory at
`$FORMAL_AI_DESKTOP_DIR/Archive/hive-control-center`, runs the exact issue prompt
through real Agent, OpenCode, Claude, and Codex clients, and requires this full
sequence from each native protocol:

```text
user prompt → assistant find call → client tool result → assistant final path
```

It rejects web calls, repeated `find` calls, missing tool results, and final
answers without the discovered path. The environment override isolates tests;
normal production requests retain the `$HOME/Desktop` fallback.

The same harness launches the real OpenCode TUI in a PTY. The published
`link-foundation/command-stream` package streams every output chunk into
`@xterm/headless`; distinct rendered frames are retained and unrolled into a
deduplicated message/tool/result sequence. The complete passing frame capture is
[`raw-data/e2e/opencode-tui/tui-transcript.json`](raw-data/e2e/opencode-tui/tui-transcript.json).

Formal AI also drove Agent CLI in self-hosted mode to create
[`agent-authored-requirement.lino`](agent-authored-requirement.lino), then read
the file back before completing. The raw plan, native client stream, server
dialog, and server trace are retained under
[`raw-data/self-authoring/`](raw-data/self-authoring/).

Five further isolated self-hosted Agent CLI sessions authored and verified the
[benchmark manifest](../../../data/benchmarks/local-path-discovery-suite.lino)
and its English, Russian, Hindi, and Chinese partitions. Their native streams
are preserved beside the original self-authoring evidence.

See [requirements.md](requirements.md), [investigation.md](investigation.md),
and [raw-data/README.md](raw-data/README.md) for the full trace.
