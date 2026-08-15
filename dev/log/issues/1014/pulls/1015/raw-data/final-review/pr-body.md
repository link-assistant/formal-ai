## Summary

- Preserve strict self-development evidence for manual releases while cleanly deferring an ineligible automatic release.
- Build the all-feature macOS nextest archive once, extract it at the original workspace path, and fan it out to eight bounded consumers without increasing the existing timeout limits.
- Discover and fail closed on every tracked Bun/npm lock; resolve all five lock audits and remove duplicate install-time advisory noise.
- Keep the dependency-free VS Code lint suite separate from the installed package-graph test; ship Playwright in the VSIX and verify browser capture from the extracted artifact.
- Isolate Gemini mutable state, limit lifecycle trust to pinned OpenCode, use exact argv for Unix process-tree ownership, and prevent archived manifests/diagnostics from becoming live projects.
- Preserve the complete issue, PR, CI, research, tests-first, and upstream-report record.

## Root causes and reproduction

The baseline contained a real Auto Release policy result in the wrong automatic control flow, three macOS jobs redundantly compiling before nextest partitioning, four vulnerable JavaScript surfaces hidden behind non-blocking install summaries, Gemini harness noise, and archived manifests discovered as dependency projects.

Fresh exact-SHA runs exposed integration boundaries that local state had masked:

- Playwright server code followed optional removed `chromium-bidi` paths when bundled into the VSIX.
- OpenCode needs its pinned package-owned postinstall to choose the platform binary.
- nextest's default temporary extraction cannot satisfy legacy compile-time Cargo binary paths.
- the source terminology gate scanned immutable raw evidence, and the requirements aggregate was stale.
- the dependency-backed VS Code bundle test had been appended to a source-only suite that intentionally runs without `vscode/node_modules`.
- artifact transfer/extraction consumed part of the ten-minute macOS job envelope before the 480-second test budget began.
- command-stream 0.15 inserted a shell between Formal AI's exact argv and the Unix process group, exposing a descendant-termination race at the 20 ms timeout boundary.
- five archive consumers still left 434–502 seconds of test execution plus setup inside the ten-minute cap, canceling three slices.
- helper-owned integration servers inherited one real home memory file, serializing concurrent response recording on its advisory lock.
- desktop release tests retained a stale command-stream 0.15 assertion after the intentional production upgrade to 0.16.

Minimal red/green experiments cover each boundary. The canonical index contains the twelve named baseline logs, initial and pushed-candidate workflow logs, run/job/artifact metadata, check annotations, all three PR discussion surfaces, the reconstructed timeline, nine requirements, solution alternatives, and complete finding ledger:

[Issue 1014 evidence index](https://github.com/link-assistant/formal-ai/blob/issue-1014-fa915643117e/dev/log/issues/1014/pulls/1015/README.md)

## Verification

- issue #1014 contract suite: 14/14
- complete all-feature Rust run: 3,756 tests across all harnesses, including 2,806 in the largest unit harness; four intentional ignores; doctests pass
- exact registered Rust, wasm, and web stages: pass, including strict all-feature Clippy, examples, Rustdoc, ShellCheck, generated files, and policy gates
- exact 20 ms descendant-termination regression: 10/10 stress runs
- desktop: 140/140; VS Code: 51 source-only tests plus one real package-graph test; web: 75/75 plus production build
- all five JavaScript lock audits: zero moderate-or-higher advisories
- 720-file VSIX and extracted-artifact Playwright browser capture: pass
- pinned OpenCode trusted-install and nextest relocation experiments: pass
- actionlint, formatting, requirements assembly, diff whitespace, and committed-range checks: pass

Full local and prior exact-SHA output is retained under `dev/log/issues/1014/pulls/1015/`. Final acceptance uses only workflows whose recorded head SHA matches the latest PR head.

## Upstream reports

- [Gemini CLI #28826](https://github.com/google-gemini/gemini-cli/issues/28826)
- [web-capture #153](https://github.com/link-assistant/web-capture/issues/153) and [#154](https://github.com/link-assistant/web-capture/issues/154)
- [html-to-markdown #459](https://github.com/xberg-io/html-to-markdown/issues/459)
- [Rust template #132](https://github.com/link-foundation/rust-ai-driven-development-pipeline-template/issues/132)
- [JavaScript template #134](https://github.com/link-foundation/js-ai-driven-development-pipeline-template/issues/134)
- [Python template #58](https://github.com/link-foundation/python-ai-driven-development-pipeline-template/issues/58)
- Existing [Playwright #33031](https://github.com/microsoft/playwright/issues/33031) documents the bundling limitation.

Fixes #1014
