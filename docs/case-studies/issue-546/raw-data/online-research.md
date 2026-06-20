# Issue 546 Online Research Notes

Captured on 2026-06-20 while preparing PR #547.

## link-foundation/start

- URL: <https://github.com/link-foundation/start>
- Captured metadata: [`link-foundation-start.json`](link-foundation-start.json)
- Description from GitHub metadata: command execution and gamification with
  GitHub auto-reporting support.
- Relevance: useful for the broader agent-command execution stack. The current
  issue is narrower: the already-approved `shell` tool should choose the host
  target by default.

## link-foundation/command-stream

- URL: <https://github.com/link-foundation/command-stream>
- Captured repository metadata:
  [`link-foundation-command-stream.json`](link-foundation-command-stream.json)
- Captured npm metadata: [`npm-command-stream.json`](npm-command-stream.json)
- NPM package/version captured: `command-stream@0.14.0`
- Package description captured from npm metadata: streaming shell utility with
  async iteration and EventEmitter support, optimized for Bun runtime.
- Relevance: possible future implementation for the injected `runOnHost`
  command runner. This PR keeps the direct host-shell runner in Node
  `child_process` because Electron and VS Code extension hosts here are
  CommonJS Node environments and the fix must be low risk.

## Repository code search

- `link-assistant/formal-ai` command-stream search:
  [`link-assistant-command-stream-code-search.json`](link-assistant-command-stream-code-search.json)
- `link-assistant/formal-ai` box-dind/shell search:
  [`link-assistant-box-dind-shell-code-search.json`](link-assistant-box-dind-shell-code-search.json)

No confirmed upstream defect was found. No upstream issue was filed.
