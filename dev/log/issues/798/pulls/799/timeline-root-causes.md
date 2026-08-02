# Timeline, root causes, and solutions

## Timeline (UTC)

- 2026-07-19 14:24: CI/CD run 29690721137 began for default-branch commit
  `eeecd44`; normal tests and packaging passed.
- 14:28: the direct WASM worker `rustc` invocation emitted eight dead-code
  warnings even though the workflow exported `RUSTFLAGS=-Dwarnings`.
- 14:43: Auto Release rejected v0.300.0 because the recorded Formal AI share
  would decrease from 5.74% to 5.58%. This was the pipeline's terminal error.
- 14:43: Desktop Release run 29691392494 started through `workflow_run`, because
  the release had already been created by the earlier release job.
- 14:44: VSIX packaging succeeded but warned that 9,015 files (4,608 JavaScript)
  were included.
- 14:50 and 14:53: macOS arm64 and x64 failed while ad-hoc signing Chrome for
  Testing Framework with `unsealed contents present in the root directory`.
- 14:57–15:04: Linux and Windows artifacts uploaded; Windows also exposed the
  platform-specific `existed` unused-variable warning. Finalize correctly marked
  the release incomplete because both macOS artifacts were absent.

## Root causes and chosen fixes

### macOS framework seal

`@electron/osx-sign` walks the app's `Contents` tree and signs nested code. The
bundled Playwright browser is an opaque runtime resource under
`Contents/Resources/browser-runtime`, not Electron application code. Descending
into and independently re-signing its framework aliases changed the nested
framework layout that Apple's verifier seals. The signer now composes (rather
than replaces) upstream ignore rules and excludes that resource subtree. The
outer `.app` is still signed, so the browser remains covered by the app resource
seal. Unit tests cover roots, descendants, lookalikes, and composition.

### warning false negatives

Cargo consumes `RUSTFLAGS`; a shell script invoking `rustc` directly does not.
The worker build now supplies `-D warnings` itself and documents three modules
that are intentionally only partially reused in the no-std worker with narrow
`#[allow(dead_code)]` module attributes. Desktop Release now exports
`RUSTFLAGS=-Dwarnings` to its entire platform matrix. The Windows-only state in
`shared_memory.rs` is itself `#[cfg(unix)]`, matching its only consumer.

### VSIX warning

The extension copied a module whose dynamic imports require two npm dependency
graphs, then packaged the entire `node_modules` directory. `esbuild` now bundles
that adapter and its runtime dependencies into the vendored module;
`.vscodeignore` excludes `node_modules`. Browser binaries remain deliberately
included because offline web capture is a product requirement.

Regenerating the extension lockfile while adding the bundler also repaired two
missing Kreuzberg musl optional-package records. npm 11 rejected the old file as
out of sync; a clean Node 22/npm 10 install now succeeds and is preserved in
`vscode-npm-ci-node22.log`.

### self-hosting ratchet

The ratchet implementation and its tests are internally consistent. The
decrease reflected commits since v0.299.0 without qualifying evidence, not a
calculation error. Relaxing the threshold would convert a correct failure into
a false negative. Every implementation commit in this PR instead records the
same real session and committed evidence path.

## Alternatives rejected

- Disabling deep signature verification hides a corrupt release artifact.
- Globally allowing dead code hides future regressions outside the partial
  worker reuse boundary.
- Suppressing vsce's warning leaves startup and install costs unchanged.
- Lowering or resetting the self-hosting metric defeats its monotonic contract.
- Ignoring finalize's incomplete result would publish checksums as authoritative
  for a release known to be missing platforms.
