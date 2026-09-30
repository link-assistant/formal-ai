# Issue coverage ledger — the 2026-09-30 bulk effort

Branch `qa-reasoning-coding-bulk-fixes` (PR #1188). This document tracks, per open issue at
branch start, what is **done**, **partial**, or **not started**, at the requirement level.
It is hand-maintained while the drafting fleet lands and is committed with the branch; the
generated requirement surfaces (`REQUIREMENTS.md`, `docs/requirements-traceability.md`,
`data/meta/requirement-status-ledger/`) are regenerated from it at integration.

Status vocabulary:
- **done (pushed)** — committed and pushed in the first 13 commits of this PR.
- **done (drafted)** — code, seeds, and tests committed on this branch; integration wiring
  (dispatch entries, module lifts, test registration, seed-registry rows) is applied in the
  integration commit before the push.
- **partial** — committed slice plus an explicitly named remainder.
- **not started** — no files exist; re-dispatch planned.

## Wave 1 — delivered and pushed (8)

| Issue | Status | Notes |
|---|---|---|
| #1161 | done (pushed) | clients registry `config_env`, JSON/CLI projection, release CLI binaries |
| #1172 | done (pushed) | word-boundary subject match, Rust + JS worker parity |
| #1173 | done (pushed) | fallback executes the search; prose → evidence-log notation |
| #1174 | done (pushed) | summarization + text-transform family (4 handlers) |
| #1175 | done (pushed) | routing guards; 843-line extraction; phrase-vocabulary seed |
| #1176 | done (pushed) | calendar offsets, statistics, unit conversion, word problems |
| #1177 | done (pushed) | nine code-task handlers; cue-loading descent fix in ten loaders |
| #1181 | done (pushed) | shared `cli_env` bool parser; install scripts |

Recorded deferrals for wave 1 (already filed as open requirement rows): wordnet pass for 13
function words; 7 dictionaryapi.dev fetches (522 outage); `tsc --noEmit` over hand-ported
worker shards.

## Wave 2 — QA, reasoning, coding, self-coding (19 + 3 umbrellas)

| Issue | Status | Commit(s) | Remaining |
|---|---|---|---|
| #1178 creative composition | done (drafted) | `71dd597eb` | wiring: registry ×3, mod/lift/dispatch, probe lift-out, DoD docs; js/ts parity (build-gated) |
| #1179 fact-check sources | done (drafted) | `46d5dcfa8` | wiring: 4 one-liners (lib.rs, extract.rs:133, audit.rs:178, test reg); prose extractor, CLI threshold, CI job, ten-fixture test |
| #1180 history context | done (drafted) | `d81d549e0` | wiring: lib.rs, registry, boundary row, test reg; 1275-line file split |
| #1184 derivation records | done (drafted) | `b32fb4abd` | wiring: 5 one-liners; R9 js/ts parity |
| #1185 repair loop | done (drafted) | `99221181c` | wiring: modules.rs, command_reroute seam, RecipeProgress field; R8 parity |
| #1186 formalization task | done (drafted) | `dc3671dc7` | wiring: dispatch-before-search, lift, registry; R7 parity; upstream #185 |
| #1163 internet as knowledge | done (drafted) | `689c44c49` | wiring: R4/R5 (registry row + source_research fetch branch); parity |
| #1164 code examples as knowledge | done (drafted) | `5d76720ae` | wiring: adopt_decomposed_procedure call site; parity |
| #1165 discovery production path | done (drafted) | `16bbbcb84` | wiring: solver.rs/general_execution.rs call sites; R4 deletions post-rediscovery |
| #1166 obligations not phrases | done (drafted) | `feb868e8d` | wiring: ci_workflow.rs/program_contract.rs call sites |
| #1167 code in meta language | done (drafted) | `98f7de4ae` | wiring: 2 test regs, CI generator line, composition.rs:404 delegate; closure token `r` |
| #1168 latest versions | done (drafted) | `1394803df` | wiring: generators read discovery module (fork report pending) |
| #1169 dependency currency | in flight | — | fork J: gate + lockfile + npm + actions + base image |
| #1171 parity document | done (drafted) | `9d137e808` | wiring: llm-task-classes.lino, probe sets, runner, release hook, test reg |
| #1154 progress cmd key | done (drafted) | `5a164f690` | wiring: test reg; work-item-steps mirror already committed |
| #1155 read validation | done (drafted) | `239917faa` | wiring: test reg (issue-named file rename optional) |
| #1156 doubled output | not started | — | root fix in program_contract.rs (dedupe print-clause literals) + verify script |
| #1157 idempotent commit chain | not started | — | work-item-steps guard + reroute verify-before-execute |
| #1158 PR feedback/ready | not started | — | data keys + reroute post-commit steps |
| #1159 prerequisite recovery | not started | — | wire crate::prerequisite into reroute failure arm |
| #1160 regular prompt | not started | — | restart_feedback.rs + planner route |
| #1170, #1162, #1183 (umbrellas) | leaf-tracked | — | close status derives from their leaves above |

## Wave 3 — infra, web, knowledge, backlog (32)

| Issue | Status | Commit(s) | Remaining |
|---|---|---|---|
| #1187 workflow credentials | done (drafted) | `2f86f34b4` | fork 3A report pending (workflows guard vs J) |
| #1088 evidence out of repo | partial | `c5ab72b74` | fork 3A report pending |
| #1090 traceability column | partial | `4770967c1` | fork 3A report pending |
| #954 module map | done (drafted) | `329fec41b` | move manifest execution at integration |
| #955 hand-check suite | done (drafted) | `443d9a133` | fork 3A report pending |
| #1182 links-notation adoption | in flight | — | fork 3A |
| #1153 slim image | in flight | — | fork 3A |
| #1084 multi-arch images | in flight | — | fork 3A |
| #934 engine hard-fail | done (drafted) | `b2772db2a` | i18n keys ×2, worker-line-budget re-measure, ROADMAP line, case study |
| #953 tool-router confinement | done (drafted) | `f366d2c4c` + sweep | rust `/v1/tools/authorize` endpoint + embedded spec mirror; release workflow `build:web` |
| #825 autocomplete | done (drafted) | `9d1992fdb` | Playwright e2e typing case |
| #951 main.jsx split | not started | — | inventory → logic extraction → removal ledger |
| #557 embedded buttons/skins | not started | — | material skin option + composer buttons ≥768px |
| #667 debugging view | not started | — | four-pane view over derivation records |
| #665 PWA + npm package | not started | — | service worker + package scaffolding |
| #666 VS Code extension | not started | — | listing assets + publish workflows |
| #670 WebVM experiment | not started | — | experiments/webvm scaffold |
| #869 RU meeting prompt | done (drafted) | `b0a49de8b` | regression pin; dispatch precedence optional |
| #447 dialog complaint | partial | `ac95f9656` | CSS fix per case-study plan (js was fork-forbidden) |
| #836 legality advisory | done (drafted) | `d266b1319` | wiring: lib.rs + dispatch + registry ×2 |
| #800 amazon.in charger | done (drafted) | `7a6d9814e` | wiring: mod/lift/dispatch + registry ×2 |
| #872 App Store kids games | done (drafted) | `48dae57d5` | same wiring as #800 |
| #700 SI units | done (drafted) | `772075a42` | wiring: lib.rs + registry + unit_conversion one-liner; upstream filing (main files) |
| #901 TRIZ automation | done (drafted) | `680f9bf3b` | wiring: lib.rs + registry + dispatch (teaching layer, after concrete) |
| #491 least action | done (drafted) | `6913661dc` | wiring: lib.rs + test reg; integration sites documented |
| #483 small-model fallback | done (drafted) | `5f2985a47` | wiring: lib.rs + registry + settings surface; Candle deps optional |
| #453 moonshot tasks | not started | — | docs/case-studies/issue-453/moonshot-tasks.md |
| #651 backlog issue bodies | not started | — | docs/backlog/*.md; main files via gh |
| #861 anonymous sentry | not started | — | telemetry.rs + consent seed |
| #668 shareable packages | not started | — | associative_packages.rs round-trip |
| #669 cloud memory sync | not started | — | cloud_sync.rs event-log sync |
| #940 research documents | not started | — | research_documents/ PDF+DOCX |

## CI repair (dedicated fixer, in flight)

Fixing the 15 failures at the pushed tip: full + specification tests, lint/format, coverage,
release CLI ×5, dogfood regeneration gate, evidence checks ×2, agent ladder. Fixes land as
`fix(ci): ...` commits; forbidden-file fixes are manifested in `docs/ci-fixes-manifest.md`.

## Running total

- done (pushed): 8
- done (drafted, wiring pending): 25
- partial: 3 (#1088, #1090, #447)
- in flight: 4 issues (#1169, #1182, #1153, #1084) + CI fixer
- not started: 17 (#1156–#1160, #951, #557, #667, #665, #666, #670, #453, #651, #861, #668, #669, #940)

Not-started and partial issues are re-dispatched at ≤3 concurrent agents once the current
fleet drains, per the standing concurrency policy. This ledger is updated at every landing.
