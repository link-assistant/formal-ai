# Investigation and design

## Failure reproduced

The exact reported prompt was first added as a unit regression with Bash,
`websearch`, and `webfetch` available. Before implementation, Formal AI selected
`websearch`; representative Russian, Hindi, and Chinese local requests also did
not produce a local command. The test was committed separately before the fix,
and its red output is retained in `raw-data/tests/reproduction-before-fix.log`.

## Root cause

Formal AI had two adjacent but incomplete concepts:

1. semantic shell intents could lower requests such as “print the working
   directory” to portable commands; and
2. code-search scopes could lower repository/source searches to `rg` or a
   client-native grep tool.

It had no association representing *path discovery by name in a local
filesystem scope*. The word “find” is intentionally not accepted as a generic
bare shell command when the rest of a prompt looks like prose, because “find
information about…” is normally a web request. With no stronger local-path
intent, the reported sentence reached the web router. In clients advertising a
shared Glob capability, that client-specific workspace operation could also be
considered before the missing local operation.

The defect was therefore routing and representation, not a bad web result. Web
search should never have been entered for this request.

## Related work

- [PR #765](https://github.com/link-assistant/formal-ai/pull/765) established
  seed-backed semantic shell intents and language-parity testing.
- [PR #769](https://github.com/link-assistant/formal-ai/pull/769) established
  the self-coding evidence pattern used by this study.
- [PR #803](https://github.com/link-assistant/formal-ai/pull/803) established
  native four-client dialog verification and exact per-dialog JSONL logs.
- Existing issue-624 and issue-781 harnesses provided the release-server/client
  lifecycle and native envelope examples instead of requiring a new framework.

The current `link-foundation/command-stream` package was inspected at version
0.14.1. It exposes streamed process chunks but does not itself model a terminal
screen. The harness therefore runs OpenCode through `script` to provide a real
PTY and feeds every chunk to `@xterm/headless`. This preserves terminal cursor
and repaint semantics before frame deduplication; stripping ANSI bytes alone
would not reconstruct a TUI frame.

## General solution

The associative seed now separates three axes:

- discovery action: find, locate, and equivalents;
- local scope: Desktop, user home/computer, or current directory/workspace; and
- object kind: directory or file.

All natural-language cues live in `data/seed/shell-intents.lino`; Rust only
parses associations and composes the command. A request must match an action and
an explicit local scope. Kind is optional, but when present supplies `-type d`
or `-type f`. This mandatory-scope rule is the main boundary that keeps “search
the web” and “find information online” on the web path.

After removing action, scope, kind, and ordinary argument noise, the composer
keeps at most eight alphanumeric query words. It emits the complete sequence
first, then variants omitting one word when at least three words remain. That
general rule solves small differences between a remembered name and a real path
without a domain-specific synonym table. `-iname` makes filesystem casing
irrelevant and `-print -quit` bounds output and traversal after a match.

The generated command still runs inside each client's normal permission and
sandbox boundary. Test-only `FORMAL_AI_DESKTOP_DIR` and `FORMAL_AI_HOME_DIR`
overrides make destructive or privacy-sensitive searches unnecessary during
automation; ordinary use falls back to `$HOME/Desktop` and `$HOME`. Mapping
“this computer” to the user's home rather than `/` deliberately avoids scanning
system and mounted filesystems.

## Routing order and parity

The planner checks explicit local-path discovery after direct file-read intent
but before shared capabilities and web routing. It only selects the route when a
Run-capable client tool is advertised. Protocol adapters then translate the
same plan to Agent/OpenCode Bash, Claude `Bash`, or Codex `exec_command`.

The deployment seed remains shared through `src/web/seed_loader.js`. There is no
separate JavaScript copy of this server-side agentic planner to patch; native
protocol integration tests and real-client E2E runs guard the common route
instead.

## Verification matrix

| Layer | What it proves |
|---|---|
| Unit, exact prompt | Local Bash `find`, Desktop root, directory predicate, and query terms. |
| Unit, executed fixture | The fuzzy command actually finds `Archive/hive-control-center`. |
| Unit, vocabulary sweep | Every seeded action/scope/kind phrase reaches the intended root or predicate. |
| Unit, negative web prompts | Open-world searches still use `websearch`. |
| Native integration | Chat Completions, Anthropic Messages, and Responses emit their correct tool envelopes. |
| Real CLI E2E | Agent, OpenCode, Claude, and Codex execute one `find`, return its output, and finish with the path. |
| Synthetic TUI regression | PTY frames are rendered, globally deduplicated, and unrolled in order. |
| Real OpenCode TUI | The exact prompt, command, tool result, and final path appear in rendered frames and server dialog records. |
| Self-hosting | Formal AI drives Agent CLI through a concrete Write/read-back task and retains its plan and raw dialog. |

The test root is intentionally random, so preserved E2E paths begin with
`/tmp/tmp...`; the command's production fallback remains `$HOME/Desktop`, which
would return the reporter's `/Users/konard/Desktop/Archive/hive-control-center`.
