---
bump: patch
---

### Fixed

- Fix issue #1017 in pull request #1018: make the step execution budget own the
  deadline instead of `timeout-minutes`, so an overrun reports `failure` with an
  `::error` naming the budget rather than degrading into a `cancelled` run and a
  skipped release. Every budget is now checked against the job cap it sits
  under, which surfaced two further at-risk jobs; the macOS core lane runs
  sixteen duration-skew-tolerant slices; `cargo audit` runs on the default
  branch and on a schedule with its one false positive ignored behind a proof
  line CI re-derives; the CodeQL Rust extractor is pinned to a `std` it can
  parse so live code stops being extracted with errors; the link check tests its
  report parser and no longer reports links it never checked; every read-only
  job belongs to a concurrency group that never cancels the default branch; and
  nested CI evidence is no longer silently excluded by `.gitignore`.
- Answer the first request in a process without round-tripping a whole module's
  CST/AST. Rule recall built the canonical learning ledger — and therefore parsed
  the pinned planner module — before checking whether the ledger could answer the
  prompt at all, which cost over ten seconds inside the *first* HTTP response and
  timed out two macOS integration tests at the harness's thirty-second limit.
  The lookup now proves a miss from the canonical failure trace before building
  anything, and the pinned round-trip is computed once per process. Recall
  behaviour is unchanged and no promotion gate is relaxed. Set
  `FORMAL_AI_TRACE_SLOW_INIT=1` (off by default) to report each whole-source
  parse with its size and duration.
- Stop a `python3` agent command's *start-up* from deciding whether it succeeded.
  Commands run with a cleared environment, which on macOS also removed `TMPDIR` —
  where `/usr/bin/python3`'s `xcrun` stub keeps the resolution cache — so every
  invocation paid a full re-resolution and a loaded runner exceeded the
  fifteen-second floor while the command itself was fine. The child now receives
  one constructed `TMPDIR` and nothing else, the floor is a sixty-second backstop
  documented against measurements rather than a frozen literal, and
  `FORMAL_AI_TRACE_COMMANDS=1` (off by default) reports the executed path, the
  budget and the elapsed time.
- Stop a *successful* macOS desktop package from being reported as a failure.
  electron-builder downloads its toolsets with a single request whose only
  deadline is ten minutes, and a stalled one is recorded in an append-only error
  list that `awaitTasks()` rethrows even after the DMG, the ZIP and both
  blockmaps have been written. Packaging now seeds the checksum-validated
  toolset cache before every build on every platform — every prefetch failure
  degrades to a warning, so it can never be the reason a build fails — and the
  retry wrapper treats the stall as transient while refusing any attempt the
  job clock cannot finish, so the backstop cannot manufacture a `cancelled` run
  of its own. `FORMAL_AI_PREFETCH_VERBOSE=1` (off by default) reports each
  toolset's cache decision and every download attempt.
- Record the reviewed npm install scripts of the `desktop` and `vscode` projects
  by package name in `allowScripts`. npm 11 warns about install scripts that are
  not recorded and documents that a future release will block them, which would
  have failed every desktop and `.vsix` build on the next runner-image bump and,
  later, silently stopped `node-pty`, `keytar` and `esbuild` building their
  native halves. An unreviewed install script still fails the install, but the
  report now names each one and the exact `npm approve-scripts
  --no-allow-scripts-pin` command that clears it.
- Stop a push to the base branch from failing macOS core slices that have
  nothing wrong with them. The archive job and each of the sixteen slices ran
  the fresh-merge simulation separately, and each resolved the base branch tip
  *at its own start time*; because the runner pool serializes the slices across
  roughly forty minutes, one commit landing on `main` mid-run gave the archive
  one merged tree and the later slices another, so every slice that started
  after the push failed its archive tree check. The archive now records the base
  commit it merged and every slice merges that same commit, which is the
  property the tree check was always asserting.
- Stop the desktop release from shipping installers built from different source
  trees. `release.yml` and `desktop-release.yml` also merge the base branch in
  more than one job, but neither compares trees across jobs, so the same
  divergence the macOS lane reports was silent there: in one run the `linux-x64`
  and `macos-arm64` installers were built against one base commit and
  `windows-arm64`, starting an hour later, against another, and all six were
  published as one release set. A reusable `pin-base-commit.yml` now resolves the
  base branch tip once per workflow and every merge — the six packaging legs, the
  `.vsix` job, `lint`, `test`, and the macOS archive and its sixteen slices
  through a new `base-commit` input — merges that one commit.
- Stop the language test coverage gate from demanding evidence in five
  languages for a change that cannot regress any of them. Any edit under
  `src/solver_handlers/` counted as language-facing, so rewording one
  English-only diagnostic string — in a handler whose meanings live in seed data
  and which has no localized counterpart — blocked the pull request. Changes
  under the language-independent code prefixes are now judged per changed line,
  while seed and translation data stay file-level and a line naming a locale or
  carrying non-Latin script still counts.
