# Issue #1012 — complete CI/CD warning and failure audit

Issue: <https://github.com/link-assistant/formal-ai/issues/1012>

Pull request: <https://github.com/link-assistant/formal-ai/pull/1013>

## 1. Collected data

The canonical [evidence archive](../../../dev/log/issues/1012/pulls/1013/README.md)
contains every one of the ten issue-listed default-branch logs, all initial PR
logs, job/check metadata, discussion surfaces, three complete template source
trees, Hive Mind guidance, primary-source research, and red/green test logs.

## 2. Timeline

The main pipeline began at 07:16:55 UTC. Its Intel macOS core suite started at
07:20:17, continued reporting passing tests through 07:54:05, and was killed at
the 35-minute job limit. Pipeline Status correctly failed the cancelled result
at 07:54:38. Issue #1012 opened at 08:10:11 and PR #1013 at 08:11:19. The full
timestamped sequence is in the evidence archive.

## 3. Requirements

Requirements R1012-1 through R1012-9 cover the ten-run audit; narrow fixes for
every actionable diagnostic; bounded macOS sharding; full Rust, JS and Python
template comparison; Hive Mind practices; upstream reports; tests-first proof;
durable evidence and opt-in diagnostics; and delivery in this single PR.

## 4. Root causes

The sole non-passing workflow was a real capacity failure: cold compilation
plus a monolithic macOS core suite exceeded its limit while tests were healthy.
Its warning was unreachable because it ran only after the test command. Green
jobs still hid a cache HTTP 429, 1,131 Box cache-write failures, false
error-shaped absence probing, third-party Node deprecations, premature pipeline
consumers, misconfigured Gemini fallback, unsafe-looking Codex temp homes, and
a near-limit Rust source file. Counter-only sccache output cannot prove each
backend response, so a disabled-by-default verbose mode was added.

## 5. Research and prior art

cargo-nextest provides filter-aware sliced CI partitioning. Existing upstream
issues actions/download-artifact#484 and microsoft/vscode#319867 identify the
two third-party Node warnings. Official sccache documentation explains both
best-effort storage under service rate limits and the logging controls used by
the new opt-in trace. Gemini CLI source confirms the explicit-model,
`useRipgrep`, and numerical-router behavior. All source revisions and files are
preserved in the archive.

## 6. Tests-first reproduction

The initial issue suite failed eight minimal source-contract tests before any
fix. A ninth diagnostic invariant was added during root-cause analysis, and a
tenth pins the required self-hosting evidence. The final issue suite passed
10/10 and the complete CI/CD module passed 207/207. The corrected repository
suite passed 2,790 tests with zero failures after regenerating the required
self-AST census for the extracted module; issue #988, issue #961, and the Codex
integration regression also pass in the retained verification record.

## 7. Implemented fix

Three nextest core slices and a separate specification lane keep each macOS job
inside 25 minutes, while a watchdog warns during long commands. The Box matrix
now consumes one shared binary. Every v8 artifact step and each third-party CLI
warning gets a diagnostic-code-specific policy. Search producers are fully
consumed, expected TUI EPIPE is localized, Gemini uses sandbox-compatible
settings, Codex ephemeral homes live below the real user cache, stock Rust uses
a quiet absence probe and executable PATH, and solver configuration moved into
a cohesive module.

## 8. Formal AI / Agent CLI authorship

Two of the nine smallest requirement leaves (22%) were independently authored
through the repository's Formal AI workflow and a real external Agent CLI:

| Requirement leaf | Native Agent CLI session | Evidence |
| --- | --- | --- |
| Classify diagnostics, fix actionable causes, and trace uncertain cache backends | `ses_ffb20640bffeX0VahcuKR9bfXj` | `self-hosting-authorship/diagnostic-audit/` |
| Compare immutable template trees and report shared defects upstream | `ses_ffb1ee9b4ffedIt26xRyh8FEDy` | `self-hosting-authorship/template-comparison/` |

Each bundle retains the native client transcript, verbose Formal AI server
trace, exact generated leaf, deterministic in-repository Agent replay, and
session JSON. The regression replays both differently phrased tasks and proves
the retained `write_file` content byte for byte. The 22% claim applies only to
these two leaves; the integrated workflow, source, tests, and analysis remain
human-directed work.

## 9. Verification

The final clean head is checked with formatting, Clippy with warnings denied,
examples, docs, file-size and language gates, actionlint, the repository test
suite, web checks, and fresh GitHub workflows. Exact results and head SHAs are
recorded in the canonical archive before PR #1013 is marked ready.
