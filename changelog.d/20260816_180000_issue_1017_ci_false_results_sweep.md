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
