# Verified requirements R311–R465 (chunk A)

Source: `req-chunk-311-465.ndjson` (149 requirements) verified against repo HEAD
(main, includes PR #926). Companion output: `verified-311-465.ndjson`.

## Verdict counts

| Verdict | Count |
|---|---|
| DELIVERED | 95 |
| PARTIAL | 47 |
| NOT-DELIVERED | 7 |
| OBSOLETE | 0 |
| UNVERIFIABLE-LOCALLY | 0 |
| **Total** | **149** |

`needs_issue=true`: 3 (R331-3, R331-5, R425-1— no open tracker covers them).

## NOT-DELIVERED (and tracking status)

| id | tracked_by | needs_issue | requirement (short) |
|---|---|---|---|
| R331-3 | — | **true** | Turing-complete substitution rules convertible to Rust/JS/etc — no cross-language rule-compilation artifact found |
| R331-5 | — | **true** | Server/Telegram execution via docker + link-foundation/box/box-cli — no docker-sandboxed execution path found for server/Telegram surfaces (see also chunk B's #716 Docker sandboxing, same gap) |
| R386-8 | #710 | false | Best-experience adoption from link-assistant/meta-expression's latest version — not integrated; #710's audit already records this class of gap |
| R395-10 | #710 | false | Data changes should default to link-foundation/link-cli-style transactions — no transactional-by-default mutation path; overlaps chunk B's #395 transactional time-travel memory focus point (still not delivered) |
| R399-15 | #710 | false | Provable 1:1 meaning-to-type correspondence — no provable-uniqueness mechanism found |
| R425-1 | — | **true** | Broader question/task type support (e.g. "make me a PDF with a list of X") — no generic file-generation-as-answer handler found |
| R440-2 | #924 | false | CI/CD checks authored in natural language and self-enforced by Formal AI — all gates are hand-written Rust/Python; nearest coverage is open epic #924 (Formal AI authors real repo changes each release), which doesn't specifically cover NL-authored CI checks |

Proposed titles for the 3 needing new issues:
- R331-3: "Compile Turing-complete substitution rules to Rust/JavaScript/other target languages"
- R331-5: "Sandbox server/Telegram code execution via link-foundation/box(-cli) docker"
- R425-1: "Generalize answer surface to file-artifact outputs (PDF, etc.), not just chat text"

## UNVERIFIABLE-LOCALLY

None in this chunk — every requirement resolved to DELIVERED/PARTIAL/NOT-DELIVERED with static evidence. Several DELIVERED/PARTIAL verdicts still carry a `handcheck` value where a **live/runtime** confirmation would materially strengthen the verdict (static evidence establishes the artifact exists; it doesn't prove end-to-end behavior):

- R312-1 — run unseen coding prompts and compare quality to Gemini/DeepSeek claims
- R331-6 — drive an execution-requiring prompt, confirm no fabricated expected-output appears pre-verification
- R353-1 — install the VS Code extension in vscode.dev to confirm the web-extension entry point actually loads
- R439-2 — run `agent --model formalai/formal-ai` against `formal-ai serve` and diff output shape against `claude -p` JSON
- R444-3 — live-run a how-to prompt with network access to confirm true multi-source synthesis vs. raw search dumps

## Focus-point findings

**PR #399 definition-of-done (RML proving / ~35k src domain literals / M-id format).** Most of the 18 R399-* sub-requirements are DELIVERED with concrete artifacts (grounding-floor tests, closure-gate tests in `tests/unit/mod.rs`, overrides-layer architecture, lossless-conversion round-trip tests). Two remain genuinely short: R399-10 (src/ domain-literal elimination) is PARTIAL, and R399-15 (provable 1:1 meaning-to-type correspondence) is NOT-DELIVERED — no mechanism proves uniqueness, only asserts it by convention.

**#416 external benchmark pass-rate evidence.** No requirement ID in this 311–465 range traces to source `#416` (the chunk's source ids run through the low-400s and don't include a 416-tagged item) — nothing to verify against here; the benchmarks-evidence question is better addressed where R416-tagged items actually occur (checked: none in this file). `docs/benchmarks.md` does exist repo-wide and is cited elsewhere in this chunk (R312-1's honest 0/20 HumanEval/MBPP numbers) as the canonical pass-rate ledger.

**#444 CONTRIBUTING guide.** DELIVERED — `CONTRIBUTING.md` (640 lines) exists and codifies fix-everywhere, debug/verbose, case-study-folder, upstream-reporting, and single-PR no-deferral clauses (R444-5).

**Silently-dropped items:**
- **#331 execution stack** — mixed: local isolated-workspace execution for program synthesis is DELIVERED (R315-1, R331-7), but cross-language rule compilation (R331-3) and server/Telegram docker sandboxing (R331-5) are NOT-DELIVERED and untracked.
- **#386 cache-eviction policy + historical requirements inventory** — R386-11 (API-cache retention policy) is PARTIAL; R386-2/R386-4 (full-codebase rewrite, universal-reasoning refactor of all handlers) are PARTIAL — directionally true but not complete; R386-8 (meta-expression adoption) is NOT-DELIVERED, tracked loosely by #710.
- **#395 transactional time-travel memory** — R395-10 (default transactional data changes) is NOT-DELIVERED; R395-1/R395-6/R395-8/R395-9 (data-oriented reasoning, auto-generalizing coding algorithm, end-to-end vision flow, outside-world data collection on unknowns) are all PARTIAL, not fully realized.
- **#440 self-authored CI checks** — R440-2 confirmed NOT-DELIVERED: no CI gate is authored in natural language and enforced by Formal AI itself; everything is hand-written Rust/Python.

**#413 unanswered meta-builder unification question (recurs in #423/#424/#439/#448).** Verified in depth (this chunk's R412-2/R413-1, both previously missing from the earlier pass and filled in this run):
- `src/dreaming.rs`/`dreaming_application.rs` implement a generalization-retention "meta-algorithm amendment" mechanism.
- `src/solver_handlers/installation_conversion.rs:845-936` has a concrete, test-pinned `meta_algorithm`/`algorithm_construction` trace (`render_meta_algorithm`, pinned by `tests/unit/installation_conversion.rs:179 conversion_answer_exposes_algorithm_construction_trace`) — this is real and shipped in PR #424.
- However this trace is local to the installation-conversion handler family only. No other coding handler (`program_synthesis.rs`, `coding_catalog.rs`, `rule_synthesis.rs`) imports or shares it.
- `ROADMAP.md:122` still explicitly states, as of current HEAD: *"A task-agnostic meta-builder ('algorithm that builds algorithms', R7) is the tracked next step"* — confirming unification across all coding paths never landed.
- konard's direct in-thread question on PR #413 (proceed with the increment vs. expand scope in-PR) was never answered before merge — matches the notes field's characterization exactly.
- **Verdict: PARTIAL for both R412-2 and R413-1, both flagged `needs_issue=true`** — no open GitHub issue tracks the meta-builder unification specifically; `docs/case-studies/issue-412` and the ROADMAP line are the only trace.

## Surprises

- The previous audit pass's merge script (`verdicts-311-465.py`) was silently short 6 requirement IDs (R409-1, R410-1, R411-1, R412-1, R412-2, R413-1) — an entire cluster around issues #409–#413, including the specifically-requested #413 meta-builder question. All 6 have now been researched and added; R412-2/R413-1 required the deepest investigation of the chunk and turned up a genuinely interesting nuance: the meta-algorithm trace *does* exist and *is* tested, just not unified across handlers, so a lazy read would misjudge this as either fully DELIVERED (trace exists) or fully NOT-DELIVERED (unification claim is false) — the accurate verdict is PARTIAL with the scope gap stated precisely.
- R409 (icon packs) undersells itself in the original issue ask ("top 5 most popular") — the shipped set is FontAwesome + 5 others (6 total), matching exactly.
