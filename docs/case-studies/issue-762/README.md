# Issue 762: OpenCode Desktop integration evidence

This case study records the real packaged-desktop verification for
`formal-ai with opencode-desktop`.

## Reproduction and fix

Before the registry entry existed, both cases in `tests/issue_762.rs` failed
with `unsupported tool opencode-desktop`. The fixed cases prove that a one-shot
launch receives an isolated OpenCode provider config without CLI-only arguments,
and that global setup participates in `--all`, preserves unrelated JSON, creates
a backup, and restores it exactly.

## Packaged desktop run

- Client: OpenCode Desktop v1.18.3, official
  `opencode-desktop-linux-x86_64.AppImage` release asset.
- Host: Linux under Xvfb; the AppImage was extracted and its real Electron
  `AppRun` executable was selected through `FORMAL_AI_OPENCODE_DESKTOP_BIN`.
- Provider: `formalai/formal-ai` through
  `http://127.0.0.1:18080/api/openai/v1`.
- Session result: the renderer showed the `formal-ai` badge, two Write calls,
  one Shell call (`cat desktop-e2e.txt`), and a two-file changed-files view.
- Lifecycle: the wrapper started `formal-ai serve --agent-mode`, then terminated
  both the temporary server and desktop process when the run ended.

## Preserved artifacts

- [Tool calls in the real renderer](../../../experiments/issue-762-desktop-e2e/tool-calls.png)
- [Expanded desktop diff](../../../experiments/issue-762-desktop-e2e/final-diff.png)
- [Rendered transcript](../../../experiments/issue-762-desktop-e2e/final-transcript.txt)
- [Injected environment](../../../experiments/issue-762-desktop-e2e/injected-environment.txt)
- [Injected OpenCode JSON](../../../experiments/issue-762-desktop-e2e/injected-opencode.json)
- [Desktop and wrapper log](../../../experiments/issue-762-desktop-e2e/desktop-wrapper.log)
- [CDP capture script](../../../experiments/issue_762_cdp.cjs)

The screenshots are evidence from the same session: the first exposes the
Write/Shell tool timeline and model badge, while the second expands the file
diff produced by those calls.
