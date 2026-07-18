# Issue 756: zero-configuration shared memory

Issue: <https://github.com/link-assistant/formal-ai/issues/756>

Pull request: <https://github.com/link-assistant/formal-ai/pull/779>

## Root cause

The server's `SyncStore` became file-backed only when
`FORMAL_AI_MEMORY_PATH` was set. Dreaming stopped without that variable, CLI
subcommands defaulted to a working-directory file, Electron used its private
`userData` file, and VS Code did not pass a memory path to its child server.
Desktop-managed Docker services also mounted only separate DinD volumes. The
project therefore had persistence mechanisms but no single default identity for
the memory file.

## Agent-CLI authorship evidence

The required live self-coding entry point was attempted first; the installed
`solve` rejected the `formal-ai` model before an Agent session could start
([`self-coding-live.log`](self-coding-live.log)). The local release server was
then driven directly with the real Agent CLI. Its captured sessions authored
the Rust regression test (`ses_088eec65affephYtm5xWDNxCnH`), the cross-surface
Node regression test (`ses_088e66b91ffed5bC3ZN1zREC6H`), the Rust resolver
(`ses_088e55fd1ffeY5b90hh2N1wbBu`), and the JavaScript resolver
(`ses_088e3ded4ffe02GiXQngIs8jJB`). Each evidence directory contains the exact
prompt, raw stream, normalized JSONL, server log, and persistent memory log.

The first runs also preserve the tool failures that motivated narrower,
literal file recipes. The red logs prove the tests failed for missing shared
path APIs before implementation.

## Solution and verification

- A single platform resolver selects the environment override, then
  `%APPDATA%\formal-ai\memory.lino` on Windows or
  `~/.formal-ai/memory.lino` on Unix/macOS.
- First native open creates the parent directory with mode `0700` on Unix and
  creates the memory file. CLI, HTTP sync, context capacity, and default-on
  dreaming all use it.
- Electron and VS Code pass the resolved host path to their local servers.
- Telegram, API server, and Agent environment containers share one memory bind
  mount while retaining separate `/var/lib/docker` volumes.
- `tests/issue_756.rs` proves resolution, secure first-run creation, durable
  writes, and reopening from a second surface. `desktop/scripts/issue-756.test.mjs`
  proves desktop/VS Code environment propagation and all Docker argument sets.

The server prints `formal-ai shared memory: <path>` at startup, giving operators
a direct assertion that every process selected the intended file.
