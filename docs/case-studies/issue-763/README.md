# Issue 763: OpenCode VS Code extension integration

## Root cause

Formal AI supported the OpenCode CLI but did not expose a target for the
official `sst-dev.opencode` VS Code extension. The extension creates a terminal
named `opencode`, sets `OPENCODE_CALLER=vscode`, and launches
`opencode --port <port>`. It therefore consumes the standard OpenCode provider
configuration and can share the CLI's OpenAI-compatible provider shape.

Upstream references:

- [VS Code extension implementation](https://github.com/anomalyco/opencode/blob/dev/sdks/vscode/src/extension.ts)
- [OpenCode custom providers](https://opencode.ai/docs/providers/#custom-provider)
- [OpenCode configuration](https://opencode.ai/docs/config/)

## Reproduction and implementation

`red-test.log` records the initial focused test failing because
`opencode-vscode` was not a seeded integration. The new seed target launches a
fresh VS Code window with a temporary `OPENCODE_CONFIG`, and persistent setup
merges the same provider into `~/.config/opencode/opencode.json`. The
`opencode-code` alias follows both setup and undo paths.

The two `red-test-agent-*` directories preserve the requested Agent CLI
self-authorship attempts. Both attempts failed inside the agent engine before
producing a valid test (one malformed the file and the retry repeatedly emitted
unknown tool events), so the reproducer was repaired manually and no
self-authorship trailer is claimed.

## Real-extension evidence

`opencode-vscode-e2e/` was produced by the automated Linux harness using the
real Marketplace extension. It verifies extension activation, terminal name,
`OPENCODE_CALLER=vscode`, the extension-inherited provider config, the
`/api/openai/v1/chat/completions` path, a `bash` tool call, and the subsequent
tool-result request. The recorded Marketplace version is in
`extension-version.txt`.

Run it with a VS Code executable available as `code`:

```bash
experiments/opencode_vscode_e2e/run.sh
```

Focused automated coverage is in `tests/issue_763.rs`; `focused-test.log`
records the passing test run. Final local verification also passed formatting,
all-target/all-feature Clippy, file-size and hardcoded-language guards, 1,797
all-feature tests (two ignored), and doctests.
