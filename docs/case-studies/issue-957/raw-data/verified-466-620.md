# Delivery verification — requirements chunk #466–#620 (137 items)

Verified against the working tree at `/Users/konard/Code/Archive/link-assistant/formal-ai`
(main, v0.326.0). Every item probed with at least one concrete artifact check
(file:line, test name, seed entry, or CI step); no PR-claim was accepted as evidence
by itself.

## Verdict counts

| Verdict | Count |
| --- | --- |
| DELIVERED | 107 |
| PARTIAL | 26 |
| NOT-DELIVERED | 4 |
| OBSOLETE | 0 |
| UNVERIFIABLE-LOCALLY (as primary verdict) | 0 — 5 items carry hand-check riders instead |

needs_issue items: **1** (R608-1).

## NOT-DELIVERED (all tracked — no new issues needed here)

| Req | What | Tracked by |
| --- | --- | --- |
| R471-2 | Upstream ask: publish si-units as Rust + JS libraries — never filed (si-units repo has zero issues per the #710 audit) | #700 (title explicitly includes the filing) |
| R483-1 | Small-model formalization fallback via model-in-browser — no integration exists | #483 (open) |
| R483-2 | Fallback constraints (off-by-default, hardware-fit, on-demand) — documented in NON-GOALS.md:7 but feature absent | #483 (open) |
| R557-1 | Buttons embedded in the text field + skin system (glass/material) | #557 (open) |

## The one untracked gap → new issue

- **R608-1 (PARTIAL, needs_issue)** — #608 demanded the thinking trace on each protocol's
  STANDARD reasoning channel. Delivered: OpenAI `reasoning_content`
  (tests/unit/specification/openai_compatibility.rs:120-137), Responses reasoning events
  (src/responses_stream.rs:102,237), Anthropic thinking blocks (src/anthropic.rs:66-106).
  **Missing: Gemini thought parts** — src/gemini.rs:285-309 emits only text/functionCall
  parts; no `thought` part exists anywhere in src/. Proposed title:
  *"Gemini protocol surface: expose the thinking trace as thought parts (missing leg of #608)"*.

## PARTIAL items (26) — what exactly is missing

Tracked by an open issue:
- **R471-1** all-units support → narrow handler only (src/solver_handler_units.rs, 150 lines); ROADMAP marks it Not done — #700
- **R473-1** mixed-grammar upstream filing declined by the agent, never made — #710
- **R477-1 / R481-1 / R559-1** Google-parity class reasoning, atomic-substitution reconstruction, residual hardcoded handlers (e.g. src/solver_handler_docs.rs:9 hardcoded PANDAS const) — #923
- **R479-2 / R479-3** upstream-template half of the site-structure and template-comparison clauses (one filing ever: rust-template#85) — #894
- **R488-2** thinking-step localization on CLI/API/Telegram — #889
- **R491-1** least-action ranking exists (src/draft_portfolio.rs:189,268) but not the balanced-2-subtask reasoning optimization — #491
- **R501-1** parse-official-docs-through-meta-language pipeline absent — #710
- **R506-1** search-result event extraction/dedup/multi-source + add-found-events-to-calendar absent (generic .ics export exists: src/solver_handlers/calendar_ics.rs) — #710
- **R534-2** sccache adopted in CI (.github/actions/setup-sccache) but the promised hive-mind filing unverified — #710 + hand-check
- **R538-4** self-AST census complete (192 modules in data/meta/self-ast) but CST/AST→Rust regeneration absent; debug view — #667
- **R546-2** start-command/start-agent adopted (Dockerfile:30, desktop/lib/agent-provider.cjs:18) but command-stream never adopted, no upstream filings — #710
- **R558-2** links→source regeneration absent (rebuild_plan recompiles from crate, not from links) — #924
- **R559-2 / R560-5** "one general algorithm" — registry is sole dispatch but the executable catalogue is still per-method Rust — #922
- **R563-1** 80% random-file summarization quality ratchet not enforced — #893

Untracked but minor (needs_issue=false, noted for the record):
- **R488-4** Telegram debounced progressive thinking delivered (src/telegram.rs:27, 1.2 s debounce) but the final blockquote is appended AFTER the answer (telegram.rs:344), not on top.
- **R512-4** whole-repo final-QA sweep is process-only (per-requirement tests are enforced via CONTRIBUTING rule 6).
- **R525-2** e2e timeouts delivered (playwright.local.config.js:110-117); the "never force-push" rule is codified NOWHERE — cheap CONTRIBUTING fix.
- **R538-3** WASM worker exists but 15 JS worker modules still hold solver logic under the shrink-only ratchet (scripts/check-worker-line-budget.rs) — the ratchet itself prevents regression.
- **R538-6** playground/small-commit operational constraints uncodified (disk policy IS enforced).
- **R552-2** GitHub-regexp-crawl "100→10 shortest" tooling never built.
- **R552-3** web-capture#141/meta-language#168 filed, but adoption of the meta-language document model unverified.

## Hand-check list (runtime / external verification)

| Req | Check |
| --- | --- |
| R468-5 | Inspect the weekly external-benchmarks workflow run: honest passed/total rows + green ratchet |
| R520-1 | Confirm agent#271/#272 and agent-commander#39/#40 closed with shipped features |
| R534-2 | Search link-assistant/hive-mind issues for the shared-sccache-container request |
| R552-3 | Check web-capture#141 status; whether formal-ai consumes the meta-language document model |
| R620-1 | Re-run `with-formal-ai gemini` (incl. `--global`) on a machine with cached Google OAuth — open #909 says the --global path regressed |

## Surprising discoveries

1. **Several "confirmed silent drops" were later delivered after all.**
   - #468 public-benchmark testing → real upstream-suite harness with a monotonic ratchet
     (`.github/workflows/external-benchmarks.yml`, src/external_benchmarks/, SWE-bench slice) via #698.
   - #505 meta-language search fusion → src/search_fusion.rs (statement fusion through
     language-independent meaning links + relative-meta-logic tiers), pinned by
     tests/unit/issue_709_search_fusion.rs — landed under #709.
   - #534 sccache → adopted in this repo's CI; only the hive-mind filing half is still unverified.
   - #546 link-foundation/start → partially adopted: `start-command` baked into the container
     image and `start-agent` is the desktop commander default; command-stream remains unused.
2. **R488-4 was mislabeled "likely lost" in the extraction** — the debounced, progressively
   edited Telegram thinking message DOES exist (src/telegram.rs:27, 1_200 ms, max 4 edits);
   only the final placement (after the answer) deviates.
3. **R491 (still-open least action) is further along than its open state suggests** —
   least-action ranking is implemented in draft/portfolio selection.
4. **The gemini reasoning channel is the only genuinely new untracked gap** in a range of
   137 requirements — everything else missing already has an open tracker (#700, #710,
   #889, #893, #894, #922–#924, #483, #491, #557, #667, #909).
5. **Language generality is now data-driven**: since #706, adding a language is a seed edit
   (data/seed/language-detection.lino); the old "all languages became 4" criticism is now an
   architecture-level non-issue, though seed coverage is still small.
6. **Enforcement artifacts for every standing rule born in this range exist today**:
   no-hardcoded-NL (scripts/check-hardcoded-language.rs + check-web-hardcoded-ui-strings.mjs
   in CI), file-size caps (check-file-size.rs: Rust 1000 / lino+worker 1500), bun+Chakra/JSX
   (package.json:6,17), refusal-anti-pattern.md required reading (CONTRIBUTING.md:33-50),
   Agent-CLI e2e in CI (test-agent-cli-e2e, release.yml:758), host-CLI spawn guard
   (agent-provider.cjs:697 + test in CI), graph-terminology lint, self-AST freshness check,
   self-hosting evidence check. The only uncodified rule from the range: **no-force-push**.
