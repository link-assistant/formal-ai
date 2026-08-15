# Pull request #1015 — complete CI diagnostic repair for #1014

Issue: <https://github.com/link-assistant/formal-ai/issues/1014>

Pull request: <https://github.com/link-assistant/formal-ai/pull/1015>

## Initial state and discussion audit

The prepared draft contained only its bootstrap commit and placeholder
description. Issue #1014, PR #1015, all three PR discussion surfaces (general
comments, inline review comments, and reviews), events, and timelines were
downloaded before implementation. There were no issue comments or reviewer
decisions to reconcile. Five initial PR workflows all passed, while the issue's
twelve main-branch runs contained one Auto Release failure plus actionable
warnings and hidden dependency-audit failures.

The complete immutable input and test evidence is indexed in
`dev/log/issues/1014/pulls/1015/README.md`.

## Decisions made in this pull request

- Keep release attribution strict, but represent an ineligible automatic
  release as a clean deferral; retain the hard manual release gate.
- Reuse one cargo-nextest archive for all three macOS core slices instead of
  raising timeouts or multiplying cold compilation.
- Dynamically audit both Bun locks and all three npm locks through one required
  gate, then silence duplicate install-time audit output.
- Upgrade the vulnerable JavaScript chains with focused overrides and run the
  affected desktop, extension, and web tests.
- Isolate Gemini's mutable home from the scanned project, explicitly prohibit
  unnecessary Bun lifecycle scripts, and prevent archived manifests from being
  treated as live dependency projects.
- File reproducible upstream reports for all six defects that are shared with
  dependencies or current templates.

## Verification and CI

The implementation-facing regression was written first and failed all seven
checks. After implementation, explicit evidence tests cover every requirement
and a whole-task test verifies their composition. The evidence archive also
retains actionlint, release-preflight, zero-vulnerability audit, desktop (140
tests), VS Code (51 tests), and web-build output.

Before #1015 is marked ready, the branch is formatted, linted, tested through
the repository's prescribed checks, reviewed against its full diff, updated
with main, pushed, and verified against fresh GitHub runs at the exact final
SHA. Those final run IDs and logs are appended to the canonical evidence rather
than inferred from the prepared branch's stale green checks.
