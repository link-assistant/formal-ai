# Issue 841: terminal-visible agent CLI tests

Issue [#841](https://github.com/link-assistant/formal-ai/issues/841) asked
Formal AI's end-to-end tests to verify what users actually see in OpenCode,
Claude Code, and Codex. The existing tests asserted native JSON exchanges, so a
terminal repaint failure such as [#819](https://github.com/link-assistant/formal-ai/issues/819)
could remain invisible until a person resized the window.

## Outcome

The agent CLI matrix now runs all three interactive clients in real PTYs and
stores five interoperable artifacts for every run:

- a lossless, unrolled text transcript;
- ordered settled frames, including scrollback;
- an asciicast v2 recording;
- a static SVG snapshot;
- an animated SVG replay.

CI uploads the complete artifact tree with `if: always()`. The harness copies
partial captures out of its temporary workspace before returning a failure, so
the recording survives the exact case where it is most useful.

Formal AI owns no PTY, VT parser, terminal emulator, or animation renderer. Its
test package depends only on published `agent-commander`; terminal capture is a
published transitive `command-stream` capability.

## Upstream result

The reusable implementation was completed and released in both upstream
projects before Formal AI switched to it.

| Layer | JavaScript release | Rust release | Merged work |
| --- | --- | --- | --- |
| Generic PTY, resize, VT rendering, scrollback, unrolling, asciicast, SVG, and timeout artifacts | [`command-stream` 0.15.0](https://github.com/link-foundation/command-stream/releases/tag/js-v0.15.0) | [`command-stream` 0.13.1](https://github.com/link-foundation/command-stream/releases/tag/rust-v0.13.1) | [#176](https://github.com/link-foundation/command-stream/pull/176), [#177](https://github.com/link-foundation/command-stream/pull/177), [#179](https://github.com/link-foundation/command-stream/pull/179) |
| Agent-specific executable/argument builders, startup interactions, semantic events, and replay bundles | [`agent-commander` 0.9.1](https://github.com/link-assistant/agent-commander/releases/tag/js_0.9.1) | [`agent-commander` 0.2.7](https://github.com/link-assistant/agent-commander/releases/tag/rust_0.2.7) | [#44](https://github.com/link-assistant/agent-commander/pull/44), [#45](https://github.com/link-assistant/agent-commander/pull/45) |

The JavaScript packages are also available from
[npm: command-stream 0.15.0](https://www.npmjs.com/package/command-stream/v/0.15.0)
and
[npm: agent-commander 0.9.1](https://www.npmjs.com/package/agent-commander/v/0.9.1);
the Rust packages are available from
[crates.io: command-stream 0.13.1](https://crates.io/crates/command-stream/0.13.1)
and
[crates.io: agent-commander 0.2.7](https://crates.io/crates/agent-commander/0.2.7).

## Why the original capture was insufficient

The local prototype ran `script -c`, fed raw bytes to xterm, and globally
deduplicated normalized lines. It could not provide a portable child PTY,
propagate `TIOCSWINSZ`, distinguish repeated states at different times, or
preserve the history that had scrolled out of the final viewport. Formal AI
also had to know OpenCode's command line directly.

The published stack instead separates responsibilities:

```text
Formal AI regression
  → agent-commander client builder and startup interaction
    → command-stream PTY and typed input/resize events
      → terminal emulator, settled frames, scrollback, and replay renderers
```

The old local capture module and its OpenCode-only wrapper were deleted after
the package releases were consumed.

## Acceptance proof

### #819: repaint, history, and explicit resize

The unit regression uses a two-row terminal so the first state must scroll out
of the viewport. It requires the unrolled transcript to contain the user,
tool, and result states once each and in order, while the final frame retains
the first state in `lines` but not in `screen`.

A second fixture accepts text (`123`), a control key (`Tab`), `Enter`, and an
explicit resize from 24×6 to 40×10. Its success state is emitted only after the
child PTY observes the new dimensions. The real #819 runs also resize each
client after the discovered path is rendered.

The complete live matrix passed:

```text
Agent direct
OpenCode direct
Claude Code direct
Codex direct
OpenCode TUI + replay
Claude Code TUI + replay
Codex TUI + replay
```

Each TUI dialog log independently proves:

```text
user prompt → assistant find call → client tool result → assistant final path
```

Evidence:

- [OpenCode transcript](tui-artifacts/path-discovery/opencode/transcript.txt)
  and [animation](tui-artifacts/path-discovery/opencode/recording.svg)
- [Claude Code transcript](tui-artifacts/path-discovery/claude/transcript.txt)
  and [animation](tui-artifacts/path-discovery/claude/recording.svg)
- [Codex transcript](tui-artifacts/path-discovery/codex/transcript.txt)
  and [animation](tui-artifacts/path-discovery/codex/recording.svg)
- [full matrix log](test-logs/full-client-matrix.log)

### #838: report multiselect and resulting issue body

The OpenCode test waits for each rendered selection before sending the next
input:

```text
[✓] Harness log
[✓] Server log
[✓] GitHub issue
```

It then asserts the review screen, submits the question, and requires all
three context export commands plus `gh issue create`. The captured
[resulting issue body](tui-artifacts/report-flow/issue-body.md) must contain
report provenance, the complete-context heading, and exported conversation
content.

![OpenCode report multiselect with all three destinations selected](tui-artifacts/report-flow/report-flow.png)

Evidence: [transcript](tui-artifacts/report-flow/terminal/transcript.txt),
[animation](tui-artifacts/report-flow/terminal/recording.svg), and
[executed actions](tui-artifacts/report-flow/report-actions.log).

### #840: task ladder through the user interface

`experiments/issue_840_task_ladder/run_ladder.sh` retains its HTTP measurement
mode and adds `MODE=tui`. Every selected node can now run through OpenCode,
score the rendered transcript after removing only the exact user-prompt echo,
and save a separate replay bundle. `REQUIRE_ALL_PASS=1` turns a selected subset
into a CI gate.

CI runs the representative `838.L1` node. It found
`Desktop/Archive/hive-control-center`, did not leak the private-key decoy, and
passed 1/1 through the real TUI.

Evidence: [results](tui-artifacts/task-ladder/results.json),
[transcript](tui-artifacts/task-ladder/838.L1/transcript.txt), and
[animation](tui-artifacts/task-ladder/838.L1/recording.svg).

## Reproduce

Build Formal AI, then run:

```bash
cargo build --release

PORT=19500 \
ARTIFACT_DIR=/tmp/formal-ai-tui-artifacts/path-discovery \
experiments/agent_cli_e2e/run_issue_819.sh

PORT=19600 \
ARTIFACT_DIR=/tmp/formal-ai-tui-artifacts/report-flow \
experiments/agent_cli_e2e/run_issue_819_report_tui.sh

PORT=19700 \
MODE=tui \
ONLY=838.L1 \
REQUIRE_ALL_PASS=1 \
OUT=/tmp/formal-ai-tui-artifacts/task-ladder-results.json \
TUI_ARTIFACT_DIR=/tmp/formal-ai-tui-artifacts/task-ladder \
experiments/issue_840_task_ladder/run_ladder.sh
```

The focused PTY regressions run with:

```bash
cd experiments/agent_cli_e2e/issue_819_tui
bun install --frozen-lockfile
bun test
```

## Self-hosting evidence

Before the dependency switch, Formal AI was run through its own Agent CLI
against this repository. The session identified the relevant source links and
returned an optional change plan; the harness then read its own authored Links
Notation before completing. The native stream, server log, session projection,
and authored source links are retained under
[`self-hosting-evidence/`](self-hosting-evidence/).
