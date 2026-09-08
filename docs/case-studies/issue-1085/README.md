# Issue #1085 — the links network is not the system that reasons

Issue: <https://github.com/link-assistant/formal-ai/issues/1085>

E108. The issue body is preserved in `raw-data/github/issue-1085.json`; every
number in it was reproduced from `main` at `dda02efb4` and re-checked
adversarially before the issue was rewritten (its section 8 lists the
corrections). This case study records what the pull request does about it and
what it found on the way.

## 1. Collected data

| Path | Content |
| --- | --- |
| `raw-data/github/issue-1085.json` | The issue as rewritten on 2026-09-08 |
| `raw-data/github/issue-1081.json` | The CI/CD issue whose remaining failures land in the same pull request |
| `ci-evidence/crates-io-me-endpoint.md` | The crates.io `/me` measurements and the run 34149311523 log lines |
| `ci-evidence/macos-durations.md` | macOS job durations over ten `main` runs |
| `requirements.md` | R1085-1 … R1085-17 with acceptance criteria |
| `solution-plan.md` | Push-by-push plan and the components surveyed |

## 2. What the pull request found first

The maintainer asked whether #1081 was fully delivered, since CI on `main` was
still red, and whether the cargo token had really expired. Four different jobs
had failed on the last four `main` runs:

| Run | Job | Cause |
| --- | --- | --- |
| 33994570842 | Build macOS test archive | killed at the 1400 s budget |
| 34061511110 | Auto Release | the self-development gate: no Formal AI-authored pull request in `v0.347.0..HEAD` |
| 34095902681 | Test (macos-15-intel / specification) | killed at the 1400 s budget with a 93.68% compiler-cache hit rate |
| 34149311523 | Release Preflight (added by #1082) | crates.io `/api/v1/me` answered 403; read as a revoked token |

The token was not expired. `GET /api/v1/me` is `AuthCheck::only_cookie()` in
crates.io and answers 403 to every API token; the same secret had published
v0.347.0 two days earlier. There is no crates.io read endpoint that accepts a
token, so the probe now records `unknown` with that reason and never sends the
token anywhere (`ci-evidence/crates-io-me-endpoint.md`).

## 3. Root cause, restated for the pull request

Behaviour is Rust; the doublets store is a write-behind projection; the
self-coding proofs measured trailers, echo tasks and a curated slice; gates and
evidence absorbed the effort. Each remedy in the issue maps to a requirement in
`requirements.md` and a slice in `solution-plan.md`.

## 4. Verification

- `cargo test --test unit self_hosting_metric` — attribution by model, behaviour
  paths, replay, per-epoch idempotency.
- `cargo test --test unit issue_1081` — the probe never calls `/me`, never sends
  the token, and reports `unknown`; budget shares hold.
- `cargo test --test unit issue_1014` — the floor is red-until-true from its own
  workflow and absent from both release jobs.
- `rust-script --test scripts/check-kernel-ratchet.rs && rust-script
  scripts/check-kernel-ratchet.rs` — the five ceilings hold at the values the
  branch measured.
- The `Self-development status` workflow on `main` — floor and kernel shrink,
  red until true.

## 5. The re-run guard the bot pull requests exposed

Two bot pull requests (#1093, #1094) each received the same authored commit
more than once. The guard that should have stopped a re-run piped the whole
branch history into `grep -q` under `pipefail`, so a present trailer made the
pipeline fail and the run authored again. Timeline and fix:
`ci-evidence/self-authored-guard-pipefail.md`.

## 6. What the ladder measured

The first run under the compile-and-test criteria passed 15 of 32 leaves. The
seventeen failures are three mechanisms, not seventeen problems: a continuation
cue routed to web search (8 leaves, #1095), an edit verified against a file the
planner generated (7 leaves, #1096), and a change reported without being made
(2 leaves, which is what these criteria exist to catch).
`ci-evidence/ladder-leaf-failures.md` carries the per-leaf table and the log
lines. `data/meta/ladder-ratchet.lino` now records 15 and may only rise.

## Upstream and sub-issues

- link-assistant/hive-mind#2229: `solve --model formal-ai` commits carry none of
  the `Formal-AI-*` trailers or the evidence bundle the version-3 metric
  attributes, so the bot-opened pull request runs through
  `.github/workflows/self-authored-pull-request.yml` and the pinned Agent CLI
  instead; the first task is #1091.
- #1087 (D6), #1088 (D7), #1089 (D8), #1090 (D9): sub-issues of #1085, blocked
  by it, each carrying the section 7 clauses.
- #1095 (E113), #1096 (E114): the two behaviour defects the compile-and-test
  ladder found, also sub-issues of #1085 and blocked by it.

