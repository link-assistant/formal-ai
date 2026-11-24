# Requirements Traceability

Companion table for [REQUIREMENTS.md](../REQUIREMENTS.md). Per konard's
2026-08-04 standing requirement, every tracked requirement must record (1)
when it was delivered in code, (2) the automated test that pins it, and (3)
manual test confirmation. Populated from the 2026-08-04 requirement audit
(944 audited requirement records, `docs/case-studies/issue-914/` lineage).

**Status of the manual-confirmation column (issue #1090, E112 — decided
2026-09-30): aspirational.** The column records what a replayed session
capture or a hand run *would* confirm; `not yet confirmed` is its honest
resting state, not debt and not a failure — 1,132 of 1,214 rows read it
today and none of them gates anything. The finish branch (filling the
column from the agentic-CLI matrix's replayable captures) is the way the
column eventually earns its place; until then the retire branch governs:
**no new manual-confirmation ledger may be introduced until an existing
one is complete**, and the rule lives in CONTRIBUTING.md beside this
notice. `scripts/generate-requirement-status.rs` carries the same decision
in the ledger manifest (`manual_column "aspirational since 2026-09-30
(issue #1090)…"`), and `rust/tests/unit/issue_1090_manual_column_retired.rs`
pins the branch so a silent reversal fails CI.

Honesty rules applied throughout: `not yet confirmed` and `none recorded`
mean exactly that -- absence of a record, not failure. A `not delivered`
row names the issue that owns the work when one exists, and says plainly
when none does. Manual-confirmation entries are cited **only** for the
specific checks the 2026-08-04 audit test battery actually ran by hand (see
`docs/case-studies/issue-914/` audit report); every other row is honestly
"not yet confirmed" even where the automated suite passes. The table is
maintained by hand: `scripts/check-requirement-status.rs` holds the generated
ledger in `data/meta/requirement-status-ledger/` in parity with the assembled
requirements, while CI enforcement of this table's own freshness is tracked
by E105 (#957). As of the 2026-09-18 rebuild every one of the 1,146
requirement IDs in the assembled REQUIREMENTS.md has a row; the 298 rows
appended then are projections of the ledger records for IDs the 2026-08-04
audit never reached, and read `none recorded` wherever no delivery is
recorded. Of those 1,146 rows, 1,074 read `not yet confirmed` -- the count
issue #1090 (E112) asked the header to carry while the finish-or-retire
decision on the manual-confirmation column stays with the maintainer.

The ID collisions this table originally preserved were resolved by issue
#964 (executed in PR #997): the later duplicate blocks were renumbered to
fresh IDs — the issue-540 R396-R407 became R537-R548, the issue-657 R480
became R549, and the issue-674 R501-R509 became R550-R558. The `Shard`
column names the requirement's owning shard under `docs/requirements/` -- the
editable source of that row, as recorded in the generated ledger. It replaces
the column the 2026-08-04 audit filled with the then-current REQUIREMENTS.md
line number, which had gone stale for every row.

| ID | Shard | Delivered | Automated test | Manual confirmation |
| --- | --- | --- | --- | --- |
| R1 | docs/requirements/preamble-requirements-for-issue-1.md | pre-2026-07 (undated) | rust/tests/web/preamble-issue-1.test.mjs | not yet confirmed |
| R2 | docs/requirements/preamble-requirements-for-issue-1.md | pre-2026-07 (undated) | rust/tests/unit/formal_ai.rs; rust/tests/unit/specification/openai_compatibility.rs | not yet confirmed |
| R3 | docs/requirements/preamble-requirements-for-issue-1.md | pre-2026-07 (undated) | rust/tests/unit/formal_ai.rs; rust/tests/unit/specification/openai_compatibility.rs | not yet confirmed |
| R4 | docs/requirements/preamble-requirements-for-issue-1.md | pre-2026-07 (undated) | rust/tests/unit/formal_ai.rs; rust/tests/web/server.test.mjs | not yet confirmed |
| R5 | docs/requirements/preamble-requirements-for-issue-1.md | pre-2026-07 (undated) | rust/tests/web/preamble-issue-1.test.mjs | not yet confirmed |
| R6 | docs/requirements/preamble-requirements-for-issue-1.md | pre-2026-07 (undated) | rust/tests/unit/specification/probabilistic_reasoning/counted_utility.rs | not yet confirmed |
| R7 | docs/requirements/preamble-requirements-for-issue-1.md | pre-2026-07 (undated) | rust/tests/unit/formal_ai.rs | not yet confirmed |
| R8 | docs/requirements/preamble-requirements-for-issue-1.md | pre-2026-07 (undated) | rust/tests/unit/data_files.rs | not yet confirmed |
| R9 | docs/requirements/preamble-requirements-for-issue-1.md | pre-2026-07 (undated) | rust/tests/unit/formal_ai.rs; rust/tests/e2e/tests/demo.spec.js | manually confirmed 2026-08-04 (audit): README `chat --prompt "Hi"` en greeting run via built binary |
| R10 | docs/requirements/preamble-requirements-for-issue-1.md | pre-2026-07 (undated) | rust/tests/unit/formal_ai.rs; rust/tests/e2e/tests/demo.spec.js | manually confirmed 2026-08-04 (audit): README `chat --prompt "Write me hello world program in Rust"` run, JSON chat format checked |
| R11 | docs/requirements/preamble-requirements-for-issue-1.md | pre-2026-07 (undated) | rust/tests/unit/formal_ai.rs | not yet confirmed |
| R12 | docs/requirements/preamble-requirements-for-issue-1.md | pre-2026-07 (undated) | rust/tests/issue_660_cli.rs | not yet confirmed |
| R13 | docs/requirements/preamble-requirements-for-issue-1.md | pre-2026-07 (undated) | rust/tests/unit/formal_ai.rs; rust/tests/unit/server_route_manifest.rs | manually confirmed 2026-08-04 (audit): `formal-ai serve` started, `/v1/chat/completions` and `/v1/responses` called, server stopped cleanly |
| R14 | docs/requirements/preamble-requirements-for-issue-1.md | pre-2026-07 (undated) | rust/tests/unit/docker_runtime.rs | not yet confirmed |
| R15 | docs/requirements/preamble-requirements-for-issue-1.md | pre-2026-07 (undated) | rust/tests/e2e/tests/demo.spec.js | not yet confirmed |
| R16 | docs/requirements/preamble-requirements-for-issue-1.md | pre-2026-07 (undated) | rust/tests/e2e/tests/source-first-translation.spec.js; rust/tests/e2e/tests/demo.spec.js | not yet confirmed |
| R17 | docs/requirements/preamble-requirements-for-issue-1.md | pre-2026-07 (undated) | rust/tests/unit/specification/desktop_surface.rs | manually confirmed 2026-08-04 (audit): `npm --prefix desktop run smoke` passed |
| R18 | docs/requirements/preamble-requirements-for-issue-1.md | pre-2026-07 (undated) | rust/tests/unit/proof_request_configuration.rs | not yet confirmed |
| R19 | docs/requirements/preamble-requirements-for-issue-1.md | pre-2026-07 (undated) | rust/tests/web/preamble-issue-1.test.mjs | not yet confirmed |
| R20 | docs/requirements/preamble-requirements-for-issue-1.md | pre-2026-07 (undated) | rust/tests/unit/formal_ai.rs; rust/tests/e2e/tests/demo.spec.js | not yet confirmed |
| R21 | docs/requirements/preamble-requirements-for-issue-1.md | pre-2026-07 (undated) | rust/tests/e2e/tests/deformalize-step.spec.js; rust/tests/e2e/tests/demo.spec.js | not yet confirmed |
| R22 | docs/requirements/preamble-requirements-for-issue-1.md | pre-2026-07 (undated) | rust/tests/e2e/tests/demo.spec.js | manually confirmed 2026-08-04 (audit): README greetings run in en/ru/hi/zh (Hello/Привет/नमस्ते/你好) |
| R23 | docs/requirements/preamble-requirements-for-issue-1.md | pre-2026-07 (undated) | rust/tests/e2e/tests/demo.spec.js | not yet confirmed |
| R24 | docs/requirements/preamble-requirements-for-issue-1.md | pre-2026-07 (undated) | rust/tests/web/preamble-issue-1.test.mjs | not yet confirmed |
| R25 | docs/requirements/preamble-requirements-for-issue-1.md | pre-2026-07 (undated) | rust/tests/unit/data_files.rs | not yet confirmed |
| R26 | docs/requirements/issue-0006-ui-follow-up-requirements.md | pre-2026-07 (undated); issue #6 | issue-level coverage (not row-pinned): rust/tests/e2e/tests/demo.spec.js:80 | not yet confirmed |
| R27 | docs/requirements/issue-0006-ui-follow-up-requirements.md | pre-2026-07 (undated); issue #6 | issue-level coverage (not row-pinned): rust/tests/e2e/tests/demo.spec.js:80 | not yet confirmed |
| R28 | docs/requirements/issue-0006-ui-follow-up-requirements.md | pre-2026-07 (undated); issue #6 | issue-level coverage (not row-pinned): rust/tests/e2e/tests/demo.spec.js:80 | not yet confirmed |
| R29 | docs/requirements/issue-0006-ui-follow-up-requirements.md | pre-2026-07 (undated); issue #6 | issue-level coverage (not row-pinned): rust/tests/e2e/tests/demo.spec.js:80 | not yet confirmed |
| R30 | docs/requirements/issue-0006-ui-follow-up-requirements.md | pre-2026-07 (undated); issue #6 | issue-level coverage (not row-pinned): rust/tests/e2e/tests/demo.spec.js:80 | not yet confirmed |
| R31 | docs/requirements/issue-0008-telegram-bot-requirements.md | pre-2026-07 (undated); issue #8 | issue-level coverage (not row-pinned): rust/tests/unit/specification/agent_isolation.rs | not yet confirmed |
| R32 | docs/requirements/issue-0008-telegram-bot-requirements.md | pre-2026-07 (undated); issue #8 | issue-level coverage (not row-pinned): rust/tests/unit/specification/agent_isolation.rs | not yet confirmed |
| R33 | docs/requirements/issue-0008-telegram-bot-requirements.md | pre-2026-07 (undated); issue #8 | issue-level coverage (not row-pinned): rust/tests/unit/specification/agent_isolation.rs | not yet confirmed |
| R34 | docs/requirements/issue-0008-telegram-bot-requirements.md | pre-2026-07 (undated); issue #8 | issue-level coverage (not row-pinned): rust/tests/unit/specification/agent_isolation.rs | not yet confirmed |
| R35 | docs/requirements/issue-0008-telegram-bot-requirements.md | pre-2026-07 (undated); issue #8 | issue-level coverage (not row-pinned): rust/tests/unit/specification/agent_isolation.rs | not yet confirmed |
| R36 | docs/requirements/issue-0008-telegram-bot-requirements.md | pre-2026-07 (undated); issue #8 | issue-level coverage (not row-pinned): rust/tests/unit/specification/agent_isolation.rs | not yet confirmed |
| R37 | docs/requirements/issue-0008-telegram-bot-requirements.md | pre-2026-07 (undated); issue #8 | issue-level coverage (not row-pinned): rust/tests/unit/specification/agent_isolation.rs | not yet confirmed |
| R38 | docs/requirements/issue-0008-telegram-bot-requirements.md | pre-2026-07 (undated); issue #8 | issue-level coverage (not row-pinned): rust/tests/unit/specification/agent_isolation.rs | not yet confirmed |
| R39 | docs/requirements/issue-0008-telegram-bot-requirements.md | pre-2026-07 (undated); issue #8 | issue-level coverage (not row-pinned): rust/tests/unit/specification/agent_isolation.rs | not yet confirmed |
| R40 | docs/requirements/issue-0008-telegram-bot-requirements.md | pre-2026-07 (undated); issue #8 | issue-level coverage (not row-pinned): rust/tests/unit/specification/agent_isolation.rs | not yet confirmed |
| R41 | docs/requirements/issue-0008-telegram-bot-requirements.md | pre-2026-07 (undated); issue #8 | issue-level coverage (not row-pinned): rust/tests/unit/specification/agent_isolation.rs | not yet confirmed |
| R42 | docs/requirements/issue-0008-telegram-bot-requirements.md | pre-2026-07 (undated); issue #8 | issue-level coverage (not row-pinned): rust/tests/unit/specification/agent_isolation.rs | not yet confirmed |
| R43 | docs/requirements/issue-0008-telegram-bot-requirements.md | pre-2026-07 (undated); issue #8 | issue-level coverage (not row-pinned): rust/tests/unit/specification/agent_isolation.rs | not yet confirmed |
| R44 | docs/requirements/issue-0008-telegram-bot-requirements.md | pre-2026-07 (undated); issue #8 | issue-level coverage (not row-pinned): rust/tests/unit/specification/agent_isolation.rs | not yet confirmed |
| R45 | docs/requirements/issue-0008-telegram-bot-requirements.md | pre-2026-07 (undated); issue #8 | issue-level coverage (not row-pinned): rust/tests/unit/specification/agent_isolation.rs | not yet confirmed |
| R46 | docs/requirements/issue-0008-telegram-bot-requirements.md | pre-2026-07 (undated); issue #8 | issue-level coverage (not row-pinned): rust/tests/unit/specification/agent_isolation.rs | not yet confirmed |
| R47 | docs/requirements/issue-0008-telegram-bot-requirements.md | pre-2026-07 (undated); issue #8 | issue-level coverage (not row-pinned): rust/tests/unit/specification/agent_isolation.rs | not yet confirmed |
| R48 | docs/requirements/issue-0010-demo-feedback-and-identity-requirements.md | pre-2026-07 (undated); issue #10 | issue-level coverage (not row-pinned): rust/tests/e2e/tests/demo.spec.js:443 | not yet confirmed |
| R49 | docs/requirements/issue-0010-demo-feedback-and-identity-requirements.md | pre-2026-07 (undated); issue #10 | issue-level coverage (not row-pinned): rust/tests/e2e/tests/demo.spec.js:443 | not yet confirmed |
| R50 | docs/requirements/issue-0010-demo-feedback-and-identity-requirements.md | pre-2026-07 (undated); issue #10 | issue-level coverage (not row-pinned): rust/tests/e2e/tests/demo.spec.js:443 | not yet confirmed |
| R51 | docs/requirements/issue-0010-demo-feedback-and-identity-requirements.md | pre-2026-07 (undated); issue #10 | issue-level coverage (not row-pinned): rust/tests/e2e/tests/demo.spec.js:443 | not yet confirmed |
| R52 | docs/requirements/issue-0010-demo-feedback-and-identity-requirements.md | pre-2026-07 (undated); issue #10 | issue-level coverage (not row-pinned): rust/tests/e2e/tests/demo.spec.js:443 | not yet confirmed |
| R53 | docs/requirements/issue-0010-demo-feedback-and-identity-requirements.md | pre-2026-07 (undated); issue #10 | issue-level coverage (not row-pinned): rust/tests/e2e/tests/demo.spec.js:443 | not yet confirmed |
| R54 | docs/requirements/issue-0010-demo-feedback-and-identity-requirements.md | pre-2026-07 (undated); issue #10 | issue-level coverage (not row-pinned): rust/tests/e2e/tests/demo.spec.js:443 | not yet confirmed |
| R55 | docs/requirements/issue-0012-holistic-vision-requirements.md | pre-2026-07 (undated); issue #12 | issue-level coverage (not row-pinned): rust/tests/unit/docs_requirements.rs | not yet confirmed |
| R56 | docs/requirements/issue-0012-holistic-vision-requirements.md | pre-2026-07 (undated); issue #12 | issue-level coverage (not row-pinned): rust/tests/unit/docs_requirements.rs | not yet confirmed |
| R57 | docs/requirements/issue-0012-holistic-vision-requirements.md | pre-2026-07 (undated); issue #12 | issue-level coverage (not row-pinned): rust/tests/unit/docs_requirements.rs | not yet confirmed |
| R58 | docs/requirements/issue-0012-holistic-vision-requirements.md | pre-2026-07 (undated); issue #12 | issue-level coverage (not row-pinned): rust/tests/unit/docs_requirements.rs | not yet confirmed |
| R59 | docs/requirements/issue-0012-holistic-vision-requirements.md | pre-2026-07 (undated); issue #12 | issue-level coverage (not row-pinned): rust/tests/unit/docs_requirements.rs | not yet confirmed |
| R60 | docs/requirements/issue-0012-holistic-vision-requirements.md | pre-2026-07 (undated); issue #12 | issue-level coverage (not row-pinned): rust/tests/unit/docs_requirements.rs | not yet confirmed |
| R61 | docs/requirements/issue-0012-holistic-vision-requirements.md | pre-2026-07 (undated); issue #12 | issue-level coverage (not row-pinned): rust/tests/unit/docs_requirements.rs | manually confirmed 2026-08-04 (audit): `chat --prompt "What is 8% of $50?"` run twice, outputs byte-identical (cmp) |
| R62 | docs/requirements/issue-0012-holistic-vision-requirements.md | pre-2026-07 (undated); issue #12 | issue-level coverage (not row-pinned): rust/tests/unit/docs_requirements.rs | not yet confirmed |
| R63 | docs/requirements/issue-0012-holistic-vision-requirements.md | pre-2026-07 (undated); issue #12 | issue-level coverage (not row-pinned): rust/tests/unit/docs_requirements.rs | not yet confirmed |
| R64 | docs/requirements/issue-0012-holistic-vision-requirements.md | pre-2026-07 (undated); issue #12 | issue-level coverage (not row-pinned): rust/tests/unit/docs_requirements.rs | not yet confirmed |
| R65 | docs/requirements/issue-0012-holistic-vision-requirements.md | pre-2026-07 (undated); issue #12 | issue-level coverage (not row-pinned): rust/tests/unit/docs_requirements.rs | not yet confirmed |
| R66 | docs/requirements/issue-0012-holistic-vision-requirements.md | pre-2026-07 (undated); issue #12 | rust/tests/unit/specification/reasoning_loop.rs | not yet confirmed |
| R67 | docs/requirements/issue-0133-duckduckgo-default-combined-ranking-and-expanded-provider-diagnostics.md | delivered 2026-09-18 by issue #1138 B1: the universal loop grounds unresolved surfaces through the trusted sources with recorded provenance; the fabricated-provenance removal is #843 (PR #853) | rust/tests/unit/specification/source_cache.rs::external_lookups_record_source_url; ::implementation_does_not_advertise_external_fetches_for_local_prompts | not yet confirmed |
| R68 | docs/requirements/issue-0012-holistic-vision-requirements.md | pre-2026-07 (undated); issue #12 | issue-level coverage (not row-pinned): rust/tests/unit/docs_requirements.rs | not yet confirmed |
| R69 | docs/requirements/issue-0012-holistic-vision-requirements.md | pre-2026-07 (undated); issue #12 | issue-level coverage (not row-pinned): rust/tests/unit/docs_requirements.rs | not yet confirmed |
| R70 | docs/requirements/issue-0012-holistic-vision-requirements.md | pre-2026-07 (undated); issue #12 | rust/tests/unit/docs_requirements.rs | not yet confirmed |
| R71 | docs/requirements/issue-0012-holistic-vision-requirements.md | pre-2026-07 (undated); issue #12 | rust/tests/web/issue-0012-specification-pins.test.mjs | not yet confirmed |
| R72 | docs/requirements/issue-0012-holistic-vision-requirements.md | pre-2026-07 (undated); issue #12 | rust/tests/unit/specification/reasoning_loop.rs | not yet confirmed |
| R73 | docs/requirements/issue-0012-holistic-vision-requirements.md | pre-2026-07 (undated); issue #12 | issue-level coverage (not row-pinned): rust/tests/unit/docs_requirements.rs | not yet confirmed |
| R74 | docs/requirements/issue-0012-holistic-vision-requirements.md | pre-2026-07 (undated); issue #12 | issue-level coverage (not row-pinned): rust/tests/unit/docs_requirements.rs | not yet confirmed |
| R75 | docs/requirements/issue-0012-holistic-vision-requirements.md | pre-2026-07 (undated); issue #12 | issue-level coverage (not row-pinned): rust/tests/unit/docs_requirements.rs | not yet confirmed |
| R76 | docs/requirements/issue-0012-holistic-vision-requirements.md | pre-2026-07 (undated); issue #12 | issue-level coverage (not row-pinned): rust/tests/unit/docs_requirements.rs | not yet confirmed |
| R77 | docs/requirements/issue-0012-holistic-vision-requirements.md | pre-2026-07 (undated); issue #12 | rust/tests/unit/specification/transparent_state.rs | not yet confirmed |
| R78 | docs/requirements/issue-0012-holistic-vision-requirements.md | pre-2026-07 (undated); issue #12 | issue-level coverage (not row-pinned): rust/tests/unit/docs_requirements.rs | not yet confirmed |
| R79 | docs/requirements/issue-0012-holistic-vision-requirements.md | pre-2026-07 (undated); issue #12 | rust/tests/unit/specification/source_cache.rs | not yet confirmed |
| R80 | docs/requirements/issue-0012-holistic-vision-requirements.md | pre-2026-07 (undated); issue #12 | rust/tests/unit/specification/agent_isolation.rs | not yet confirmed |
| R81 | docs/requirements/issue-0012-holistic-vision-requirements.md | pre-2026-07 (undated); issue #12 | rust/tests/unit/specification/network_visualization.rs | not yet confirmed |
| R82 | docs/requirements/issue-0012-holistic-vision-requirements.md | pre-2026-07 (undated); issue #12 | rust/tests/unit/specification/reasoning_loop.rs::answers_are_repeatable_for_the_same_prompt | not yet confirmed |
| R83 | docs/requirements/issue-0014-unified-surface-requirements.md | pre-2026-07 (undated); issue #14 | issue-level coverage (not row-pinned): rust/tests/unit/specification/conversation_history.rs | not yet confirmed |
| R84 | docs/requirements/issue-0014-unified-surface-requirements.md | pre-2026-07 (undated); issue #14 | issue-level coverage (not row-pinned): rust/tests/unit/specification/conversation_history.rs | not yet confirmed |
| R85 | docs/requirements/issue-0014-unified-surface-requirements.md | pre-2026-07 (undated); issue #14 | rust/tests/unit/specification/reasoning_paths.rs::arithmetic_* | not yet confirmed |
| R86 | docs/requirements/issue-0014-unified-surface-requirements.md | pre-2026-07 (undated); issue #14 | rust/tests/unit/specification/reasoning_paths.rs::concept_lookup_* | not yet confirmed |
| R87 | docs/requirements/issue-0014-unified-surface-requirements.md | pre-2026-07 (undated); issue #14 | rust/tests/unit/specification/reasoning_paths.rs::solve_with_history_* | not yet confirmed |
| R88 | docs/requirements/issue-0014-unified-surface-requirements.md | pre-2026-07 (undated); issue #14 | rust/tests/unit/specification/reasoning_paths.rs::javascript_* | not yet confirmed |
| R89 | docs/requirements/issue-0014-unified-surface-requirements.md | pre-2026-07 (undated); issue #14 | issue-level coverage (not row-pinned): rust/tests/unit/specification/conversation_history.rs | not yet confirmed |
| R90 | docs/requirements/issue-0016-multilingual-wikipedia-and-append-only-memory-requirements.md | pre-2026-07 (undated); issue #16 | issue-level coverage (not row-pinned): rust/tests/unit/specification/multilingual.rs | not yet confirmed |
| R91 | docs/requirements/issue-0016-multilingual-wikipedia-and-append-only-memory-requirements.md | pre-2026-07 (undated); issue #16 | issue-level coverage (not row-pinned): rust/tests/unit/specification/multilingual.rs | not yet confirmed |
| R92 | docs/requirements/issue-0016-multilingual-wikipedia-and-append-only-memory-requirements.md | pre-2026-07 (undated); issue #16 | issue-level coverage (not row-pinned): rust/tests/unit/specification/multilingual.rs | not yet confirmed |
| R93 | docs/requirements/issue-0016-multilingual-wikipedia-and-append-only-memory-requirements.md | pre-2026-07 (undated); issue #16 | rust/tests/e2e/tests/multilingual-chat.spec.js; rust/tests/e2e/playwright.local.config.js | not yet confirmed |
| R94 | docs/requirements/issue-0016-multilingual-wikipedia-and-append-only-memory-requirements.md | pre-2026-07 (undated); issue #16 | issue-level coverage (not row-pinned): rust/tests/unit/specification/multilingual.rs | not yet confirmed |
| R95 | docs/requirements/issue-0016-multilingual-wikipedia-and-append-only-memory-requirements.md | pre-2026-07 (undated); issue #16 | rust/tests/e2e/tests/multilingual-memory-settings.spec.js | not yet confirmed |
| R96 | docs/requirements/issue-0016-multilingual-wikipedia-and-append-only-memory-requirements.md | pre-2026-07 (undated); issue #16 | issue-level coverage (not row-pinned): rust/tests/unit/specification/multilingual.rs | not yet confirmed |
| R97 | docs/requirements/issue-0016-multilingual-wikipedia-and-append-only-memory-requirements.md | pre-2026-07 (undated); issue #16 | issue-level coverage (not row-pinned): rust/tests/unit/specification/multilingual.rs | not yet confirmed |
| R98 | docs/requirements/issue-0016-multilingual-wikipedia-and-append-only-memory-requirements.md | pre-2026-07 (undated); issue #16 | issue-level coverage (not row-pinned): rust/tests/unit/specification/multilingual.rs | not yet confirmed |
| R99 | docs/requirements/issue-0016-multilingual-wikipedia-and-append-only-memory-requirements.md | pre-2026-07 (undated); issue #16 | issue-level coverage (not row-pinned): rust/tests/unit/specification/multilingual.rs | not yet confirmed |
| R100 | docs/requirements/issue-0016-multilingual-wikipedia-and-append-only-memory-requirements.md | pre-2026-07 (undated); issue #16 | issue-level coverage (not row-pinned): rust/tests/unit/specification/multilingual.rs | not yet confirmed |
| R101 | docs/requirements/issue-0016-follow-up-universal-data-seed-across-every-interface-pr-17-reopen.md | PR #17 (issue #16) | issue-level coverage (not row-pinned): rust/tests/unit/specification/multilingual.rs | not yet confirmed |
| R102 | docs/requirements/issue-0016-follow-up-universal-data-seed-across-every-interface-pr-17-reopen.md | PR #17 (issue #16) | issue-level coverage (not row-pinned): rust/tests/unit/specification/multilingual.rs | not yet confirmed |
| R103 | docs/requirements/issue-0016-follow-up-universal-data-seed-across-every-interface-pr-17-reopen.md | PR #17 (issue #16) | rust/tests/source/source_tests/seed/tests.rs | not yet confirmed |
| R104 | docs/requirements/issue-0016-follow-up-universal-data-seed-across-every-interface-pr-17-reopen.md | PR #17 (issue #16) | rust/tests/source/source_tests/seed/tests.rs | not yet confirmed |
| R105 | docs/requirements/issue-0016-follow-up-universal-data-seed-across-every-interface-pr-17-reopen.md | PR #17 (issue #16) | rust/tests/web/issue-0016-seed-mirror.test.mjs | not yet confirmed |
| R106 | docs/requirements/issue-0016-follow-up-universal-data-seed-across-every-interface-pr-17-reopen.md | PR #17 (issue #16) | issue-level coverage (not row-pinned): rust/tests/unit/specification/multilingual.rs | not yet confirmed |
| R107 | docs/requirements/issue-0016-follow-up-universal-data-seed-across-every-interface-pr-17-reopen.md | PR #17 (issue #16) | issue-level coverage (not row-pinned): rust/tests/unit/specification/multilingual.rs | not yet confirmed |
| R108 | docs/requirements/issue-0016-follow-up-universal-data-seed-across-every-interface-pr-17-reopen.md | PR #17 (issue #16) | issue-level coverage (not row-pinned): rust/tests/unit/specification/multilingual.rs | not yet confirmed |
| R109 | docs/requirements/issue-0018-full-memory-export-import-requirements.md | pre-2026-07 (undated); issue #18 | issue-level coverage (not row-pinned): rust/tests/e2e/tests/issue-672-migration-replay.spec.js | not yet confirmed |
| R110 | docs/requirements/issue-0018-full-memory-export-import-requirements.md | pre-2026-07 (undated); issue #18 | issue-level coverage (not row-pinned): rust/tests/e2e/tests/issue-672-migration-replay.spec.js | not yet confirmed |
| R111 | docs/requirements/issue-0018-full-memory-export-import-requirements.md | pre-2026-07 (undated); issue #18 | issue-level coverage (not row-pinned): rust/tests/e2e/tests/issue-672-migration-replay.spec.js | not yet confirmed |
| R112 | docs/requirements/issue-0078-shorter-issue-reporting-requirements.md | pre-2026-07 (undated); issue #18 | issue-level coverage (not row-pinned): rust/tests/e2e/tests/issue-672-migration-replay.spec.js | not yet confirmed |
| R113 | docs/requirements/issue-0018-full-memory-export-import-requirements.md | pre-2026-07 (undated); issue #18 | issue-level coverage (not row-pinned): rust/tests/e2e/tests/issue-672-migration-replay.spec.js | not yet confirmed |
| R114 | docs/requirements/issue-0018-full-memory-export-import-requirements.md | pre-2026-07 (undated); issue #18 | rust/tests/e2e/tests/multilingual-memory-settings.spec.js; memory::tests::full_memory_round_trip_* | not yet confirmed |
| R115 | docs/requirements/issue-0078-shorter-issue-reporting-requirements.md | pre-2026-07 (undated); issue #78 | rust/tests/web/issue-0078-issue-reporting.test.mjs | not yet confirmed |
| R116 | docs/requirements/issue-0078-shorter-issue-reporting-requirements.md | pre-2026-07 (undated); issue #78 | rust/tests/e2e/tests/demo.spec.js | not yet confirmed |
| R117 | docs/requirements/issue-0078-shorter-issue-reporting-requirements.md | pre-2026-07 (undated); issue #78 | rust/tests/web/issue-0078-issue-reporting.test.mjs | not yet confirmed |
| R118 | docs/requirements/issue-0078-shorter-issue-reporting-requirements.md | pre-2026-07 (undated); issue #78 | rust/tests/web/issue-0078-issue-reporting.test.mjs | not yet confirmed |
| R119 | docs/requirements/issue-0078-shorter-issue-reporting-requirements.md | pre-2026-07 (undated); issue #78 | rust/tests/e2e/tests/multilingual-chat.spec.js; rust/tests/e2e/tests/demo.spec.js | not yet confirmed |
| R120 | docs/requirements/issue-0096-calculator-delegation-requirements.md | pre-2026-07 (undated); issue #96 | rust/tests/unit/specification/calculator_delegation.rs | manually confirmed 2026-08-04 (audit): README arithmetic examples run in en (`8% of $50`) and ru (currency conversion) |
| R121 | docs/requirements/issue-0096-calculator-delegation-requirements.md | pre-2026-07 (undated); issue #96 | rust/tests/unit/specification/calculator_delegation.rs | not yet confirmed |
| R122 | docs/requirements/issue-0096-calculator-delegation-requirements.md | pre-2026-07 (undated); issue #96 | rust/tests/unit/specification/calculator_delegation.rs | not yet confirmed |
| R123 | docs/requirements/issue-0096-calculator-delegation-requirements.md | pre-2026-07 (undated); issue #96 | rust/tests/unit/specification/calculator_delegation.rs | not yet confirmed |
| R124 | docs/requirements/issue-0096-calculator-delegation-requirements.md | pre-2026-07 (undated); issue #96 | rust/tests/unit/specification/calculator_delegation.rs | not yet confirmed |
| R125 | docs/requirements/issue-0096-calculator-delegation-requirements.md | pre-2026-07 (undated); issue #96 | rust/tests/web/issue-0096-calculator-arithmetic.test.mjs | not yet confirmed |
| R126 | docs/requirements/issue-0096-calculator-delegation-requirements.md | pre-2026-07 (undated); issue #96 | rust/tests/e2e/tests/multilingual-memory-settings.spec.js | not yet confirmed |
| R127 | docs/requirements/issue-0096-calculator-delegation-requirements.md | pre-2026-07 (undated); issue #96 | rust/tests/unit/specification/calculator_delegation.rs | not yet confirmed |
| R128 | docs/requirements/issue-0096-calculator-delegation-requirements.md | pre-2026-07 (undated); issue #96 | rust/tests/web/case-study-preservation.test.mjs | not yet confirmed |
| R129 | docs/requirements/issue-0103-test-matrix-and-architecture-requirements.md | pre-2026-07 (undated); issue #103 | rust/tests/unit/specification/prompt_variations.rs; rust/tests/unit/specification/chat_surface.rs | not yet confirmed |
| R130 | docs/requirements/issue-0103-test-matrix-and-architecture-requirements.md | pre-2026-07 (undated); issue #103 | rust/tests/unit/specification/prompt_variations.rs; rust/tests/unit/specification/multilingual.rs | not yet confirmed |
| R131 | docs/requirements/issue-0103-test-matrix-and-architecture-requirements.md | pre-2026-07 (undated); issue #103 | issue-level coverage (not row-pinned): rust/tests/unit/specification/prompt_variations.rs | not yet confirmed |
| R132 | docs/requirements/issue-0103-test-matrix-and-architecture-requirements.md | pre-2026-07 (undated); issue #103 | rust/tests/unit/specification/prompt_variations.rs | not yet confirmed |
| R133 | docs/requirements/issue-0103-test-matrix-and-architecture-requirements.md | pre-2026-07 (undated); issue #103 | issue-level coverage (not row-pinned): rust/tests/unit/specification/prompt_variations.rs | not yet confirmed |
| R134 | docs/requirements/issue-0103-test-matrix-and-architecture-requirements.md | pre-2026-07 (undated); issue #103 | issue-level coverage (not row-pinned): rust/tests/unit/specification/prompt_variations.rs | not yet confirmed |
| R135 | docs/requirements/issue-0103-test-matrix-and-architecture-requirements.md | pre-2026-07 (undated); issue #103 | issue-level coverage (not row-pinned): rust/tests/unit/specification/prompt_variations.rs | not yet confirmed |
| R136 | docs/requirements/issue-0103-test-matrix-and-architecture-requirements.md | pre-2026-07 (undated); issue #103 | issue-level coverage (not row-pinned): rust/tests/unit/specification/prompt_variations.rs | not yet confirmed |
| R137 | docs/requirements/issue-0117-lino-i18n-catalog-requirements.md | pre-2026-07 (undated); issue #117 | rust/tests/web/issue-0117-i18n-catalog.test.mjs | not yet confirmed |
| R138 | docs/requirements/issue-0117-lino-i18n-catalog-requirements.md | pre-2026-07 (undated); issue #117 | rust/tests/web/issue-0117-i18n-catalog.test.mjs | not yet confirmed |
| R139 | docs/requirements/issue-0117-lino-i18n-catalog-requirements.md | pre-2026-07 (undated); issue #117 | rust/tests/web/issue-0117-i18n-catalog.test.mjs | not yet confirmed |
| R140 | docs/requirements/issue-0117-lino-i18n-catalog-requirements.md | pre-2026-07 (undated); issue #117 | rust/tests/web/issue-0117-i18n-catalog.test.mjs; rust/tests/e2e/scripts/check-i18n-catalog.mjs | not yet confirmed |
| R141 | docs/requirements/issue-0117-lino-i18n-catalog-requirements.md | pre-2026-07 (undated); issue #117 | rust/tests/e2e/tests/demo.spec.js | not yet confirmed |
| R142 | docs/requirements/issue-0117-lino-i18n-catalog-requirements.md | pre-2026-07 (undated); issue #117 | rust/tests/web/case-study-preservation.test.mjs | not yet confirmed |
| R143 | docs/requirements/issue-0115-github-evidence-collection-and-hive-mind-trace-requirements.md | pre-2026-07 (undated); issue #115 | rust/tests/web/case-study-preservation.test.mjs | not yet confirmed |
| R144 | docs/requirements/issue-0115-github-evidence-collection-and-hive-mind-trace-requirements.md | pre-2026-07 (undated); issue #115 | rust/tests/unit/github_logs.rs | not yet confirmed |
| R145 | docs/requirements/issue-0115-github-evidence-collection-and-hive-mind-trace-requirements.md | pre-2026-07 (undated); issue #115 | rust/tests/unit/github_logs.rs | not yet confirmed |
| R146 | docs/requirements/issue-0115-github-evidence-collection-and-hive-mind-trace-requirements.md | pre-2026-07 (undated); issue #115 | rust/tests/unit/github_logs.rs | not yet confirmed |
| R147 | docs/requirements/issue-0115-github-evidence-collection-and-hive-mind-trace-requirements.md | pre-2026-07 (undated); issue #115 | rust/tests/web/case-study-preservation.test.mjs | not yet confirmed |
| R148 | docs/requirements/issue-0115-github-evidence-collection-and-hive-mind-trace-requirements.md | pre-2026-07 (undated); issue #115 | rust/tests/unit/github_logs.rs; rust/tests/unit/docs_requirements.rs | not yet confirmed |
| R149 | docs/requirements/issue-0115-github-evidence-collection-and-hive-mind-trace-requirements.md | pre-2026-07 (undated); issue #115 | rust/tests/unit/github_logs.rs; rust/tests/integration/formal_ai_cli.rs | not yet confirmed |
| R150 | docs/requirements/issue-0063-cross-language-definition-fusion-requirements.md | pre-2026-07 (undated); issue #63 | issue-level coverage (not row-pinned): rust/tests/unit/specification/definition_fusion.rs | not yet confirmed |
| R151 | docs/requirements/issue-0063-cross-language-definition-fusion-requirements.md | pre-2026-07 (undated); issue #63 | issue-level coverage (not row-pinned): rust/tests/unit/specification/definition_fusion.rs | not yet confirmed |
| R152 | docs/requirements/issue-0063-cross-language-definition-fusion-requirements.md | pre-2026-07 (undated); issue #63 | issue-level coverage (not row-pinned): rust/tests/unit/specification/definition_fusion.rs | not yet confirmed |
| R153 | docs/requirements/issue-0063-cross-language-definition-fusion-requirements.md | pre-2026-07 (undated); issue #63 | issue-level coverage (not row-pinned): rust/tests/unit/specification/definition_fusion.rs | not yet confirmed |
| R154 | docs/requirements/issue-0063-cross-language-definition-fusion-requirements.md | pre-2026-07 (undated); issue #63 | rust/tests/unit/specification/definition_fusion.rs; rust/tests/e2e/tests/multilingual-chat.spec.js | not yet confirmed |
| R155 | docs/requirements/issue-0063-cross-language-definition-fusion-requirements.md | pre-2026-07 (undated); issue #63 | issue-level coverage (not row-pinned): rust/tests/unit/specification/definition_fusion.rs | not yet confirmed |
| R156 | docs/requirements/issue-0080-software-project-request-requirements.md | pre-2026-07 (undated); issue #80 | none recorded | not yet confirmed |
| R157 | docs/requirements/issue-0080-software-project-request-requirements.md | pre-2026-07 (undated); issue #80 | none recorded | not yet confirmed |
| R158 | docs/requirements/issue-0080-software-project-request-requirements.md | pre-2026-07 (undated); issue #80 | none recorded | not yet confirmed |
| R159 | docs/requirements/issue-0080-software-project-request-requirements.md | pre-2026-07 (undated); issue #80 | none recorded | not yet confirmed |
| R160 | docs/requirements/issue-0080-software-project-request-requirements.md | pre-2026-07 (undated); issue #80 | none recorded | not yet confirmed |
| R161 | docs/requirements/issue-0080-software-project-request-requirements.md | pre-2026-07 (undated); issue #80 | none recorded | not yet confirmed |
| R162 | docs/requirements/issue-0080-software-project-request-requirements.md | pre-2026-07 (undated); issue #80 | none recorded | not yet confirmed |
| R163 | docs/requirements/issue-0080-software-project-request-requirements.md | pre-2026-07 (undated); issue #80 | none recorded | not yet confirmed |
| R164 | docs/requirements/issue-0080-software-project-request-requirements.md | pre-2026-07 (undated); issue #80 | none recorded | not yet confirmed |
| R165 | docs/requirements/issue-0129-connectivity-diagnostics-requirements.md | pre-2026-07 (undated); issue #129 | rust/tests/e2e/tests/connectivity.spec.js; js/tests/index.html | not yet confirmed |
| R166 | docs/requirements/issue-0129-connectivity-diagnostics-requirements.md | pre-2026-07 (undated); issue #129 | rust/tests/e2e/tests/connectivity.spec.js; js/tests/connectivity.js | not yet confirmed |
| R167 | docs/requirements/issue-0129-connectivity-diagnostics-requirements.md | pre-2026-07 (undated); issue #129 | rust/tests/e2e/tests/connectivity.spec.js | not yet confirmed |
| R168 | docs/requirements/issue-0129-connectivity-diagnostics-requirements.md | pre-2026-07 (undated); issue #129 | rust/tests/e2e/tests/connectivity.spec.js | not yet confirmed |
| R169 | docs/requirements/issue-0129-connectivity-diagnostics-requirements.md | pre-2026-07 (undated); issue #129 | rust/tests/e2e/tests/connectivity.spec.js | not yet confirmed |
| R170 | docs/requirements/issue-0129-connectivity-diagnostics-requirements.md | pre-2026-07 (undated); issue #129 | rust/tests/e2e/tests/connectivity.spec.js | not yet confirmed |
| R171 | docs/requirements/issue-0129-connectivity-diagnostics-requirements.md | pre-2026-07 (undated); issue #129 | rust/tests/unit/ci-cd/workflow_release.rs | not yet confirmed |
| R172 | docs/requirements/issue-0129-connectivity-diagnostics-requirements.md | pre-2026-07 (undated); issue #129 | rust/tests/web/case-study-preservation.test.mjs | not yet confirmed |
| R173 | docs/requirements/issue-0127-structured-fact-query-reasoning-requirements.md | pre-2026-07 (undated); issue #127 | rust/tests/unit/specification/prompt_variations_facts.rs; rust/tests/web/issue-1172-live-fact-parity.test.mjs | not yet confirmed |
| R174 | docs/requirements/issue-0127-structured-fact-query-reasoning-requirements.md | pre-2026-07 (undated); issue #127 | rust/tests/web/issue-0127-fact-captures.test.mjs | not yet confirmed |
| R175 | docs/requirements/issue-0127-structured-fact-query-reasoning-requirements.md | pre-2026-07 (undated); issue #127 | rust/tests/web/issue-0127-fact-query-cache.test.mjs | not yet confirmed |
| R176 | docs/requirements/issue-0127-structured-fact-query-reasoning-requirements.md | pre-2026-07 (undated); issue #127 | rust/tests/web/issue-0127-fact-query-cache.test.mjs | not yet confirmed |
| R177 | docs/requirements/issue-0127-structured-fact-query-reasoning-requirements.md | pre-2026-07 (undated); issue #127 | rust/tests/web/issue-1172-live-fact-parity.test.mjs; rust/tests/web/issue-0127-fact-query-cache.test.mjs | not yet confirmed |
| R178 | docs/requirements/issue-0127-structured-fact-query-reasoning-requirements.md | pre-2026-07 (undated); issue #127 | rust/tests/web/issue-0127-fact-query-cache.test.mjs | not yet confirmed |
| R179 | docs/requirements/issue-0127-structured-fact-query-reasoning-requirements.md | pre-2026-07 (undated); issue #127 | rust/tests/unit/specification/prompt_variations_facts.rs | not yet confirmed |
| R180 | docs/requirements/issue-0127-structured-fact-query-reasoning-requirements.md | pre-2026-07 (undated); issue #127 | rust/tests/unit/specification/prompt_variations.rs; rust/tests/e2e/tests/multilingual-issue-27.spec.js | not yet confirmed |
| R181 | docs/requirements/issue-0133-duckduckgo-default-combined-ranking-and-expanded-provider-diagnostics.md | pre-2026-07 (undated); issue #133 | rust/tests/web/issue-0133-search-fusion.test.mjs | not yet confirmed |
| R182 | docs/requirements/issue-0133-duckduckgo-default-combined-ranking-and-expanded-provider-diagnostics.md | pre-2026-07 (undated); issue #133 | rust/tests/web/issue-0133-search-fusion.test.mjs | not yet confirmed |
| R183 | docs/requirements/issue-0133-duckduckgo-default-combined-ranking-and-expanded-provider-diagnostics.md | pre-2026-07 (undated); issue #133 | rust/tests/web/issue-0133-search-fusion.test.mjs | not yet confirmed |
| R184 | docs/requirements/issue-0133-duckduckgo-default-combined-ranking-and-expanded-provider-diagnostics.md | pre-2026-07 (undated); issue #133 | rust/tests/web/issue-0133-search-fusion.test.mjs; js/tests/connectivity.js | not yet confirmed |
| R185 | docs/requirements/issue-0133-duckduckgo-default-combined-ranking-and-expanded-provider-diagnostics.md | pre-2026-07 (undated); issue #133 | rust/tests/web/issue-0133-search-fusion.test.mjs; js/tests/connectivity.js | not yet confirmed |
| R186 | docs/requirements/issue-0133-duckduckgo-default-combined-ranking-and-expanded-provider-diagnostics.md | pre-2026-07 (undated); issue #133 | rust/tests/web/issue-0133-search-fusion.test.mjs | not yet confirmed |
| R187 | docs/requirements/issue-0133-duckduckgo-default-combined-ranking-and-expanded-provider-diagnostics.md | pre-2026-07 (undated); issue #133 | rust/tests/web/issue-0133-search-fusion.test.mjs | not yet confirmed |
| R188 | docs/requirements/issue-0133-duckduckgo-default-combined-ranking-and-expanded-provider-diagnostics.md | pre-2026-07 (undated); issue #133 | rust/tests/web/issue-0133-search-fusion.test.mjs | not yet confirmed |
| R189 | docs/requirements/issue-0133-duckduckgo-default-combined-ranking-and-expanded-provider-diagnostics.md | pre-2026-07 (undated); issue #133 | rust/tests/web/issue-0133-search-fusion.test.mjs | not yet confirmed |
| R190 | docs/requirements/issue-0133-duckduckgo-default-combined-ranking-and-expanded-provider-diagnostics.md | pre-2026-07 (undated); issue #133 | rust/tests/web/issue-0133-search-fusion.test.mjs | not yet confirmed |
| R191 | docs/requirements/issue-0133-duckduckgo-default-combined-ranking-and-expanded-provider-diagnostics.md | pre-2026-07 (undated); issue #133 | rust/tests/web/issue-0133-search-fusion.test.mjs | not yet confirmed |
| R192 | docs/requirements/issue-0133-duckduckgo-default-combined-ranking-and-expanded-provider-diagnostics.md | pre-2026-07 (undated); issue #133 | rust/tests/web/issue-0133-search-fusion.test.mjs | not yet confirmed |
| R193 | docs/requirements/issue-0133-duckduckgo-default-combined-ranking-and-expanded-provider-diagnostics.md | pre-2026-07 (undated); issue #133 | rust/tests/web/issue-0133-search-fusion.test.mjs | not yet confirmed |
| R194 | docs/requirements/issue-0133-duckduckgo-default-combined-ranking-and-expanded-provider-diagnostics.md | pre-2026-07 (undated); issue #133 | rust/tests/web/issue-0194-wasm-boundary.test.mjs | not yet confirmed |
| R195 | docs/requirements/issue-0159-hive-mind-lookup-and-curated-project-summarization.md | pre-2026-07 (undated); issue #159 | rust/tests/unit/specification/project_lookups.rs::russian_hive_mind_prompt_prefers_link_assistant_project | not yet confirmed |
| R196 | docs/requirements/issue-0159-hive-mind-lookup-and-curated-project-summarization.md | pre-2026-07 (undated); issue #159 | issue-level coverage (not row-pinned): rust/tests/unit/specification/project_lookups.rs:17 | not yet confirmed |
| R197 | docs/requirements/issue-0159-hive-mind-lookup-and-curated-project-summarization.md | pre-2026-07 (undated); issue #159 | rust/tests/unit/specification/summarization_pipeline.rs (mirrors rust/src/summarization/mod.rs::tests) | not yet confirmed |
| R198 | docs/requirements/issue-0159-hive-mind-lookup-and-curated-project-summarization.md | pre-2026-07 (undated); issue #159 | issue-level coverage (not row-pinned): rust/tests/unit/specification/project_lookups.rs:17 | not yet confirmed |
| R199 | docs/requirements/issue-0159-hive-mind-lookup-and-curated-project-summarization.md | pre-2026-07 (undated); issue #159 | issue-level coverage (not row-pinned): rust/tests/unit/specification/project_lookups.rs:17 | not yet confirmed |
| R200 | docs/requirements/issue-0159-hive-mind-lookup-and-curated-project-summarization.md | pre-2026-07 (undated); issue #159 | rust/tests/unit/specification/project_lookups.rs::curated_project_concept_prompt_routes_to_project_lookup | not yet confirmed |
| R201 | docs/requirements/issue-0159-hive-mind-lookup-and-curated-project-summarization.md | pre-2026-07 (undated); issue #159 | rust/tests/unit/specification/project_lookups.rs::curated_project_lookup_records_summarization_evidence | not yet confirmed |
| R202 | docs/requirements/issue-0159-hive-mind-lookup-and-curated-project-summarization.md | pre-2026-07 (undated); issue #159 | rust/tests/unit/specification/summarization_pipeline.rs::strip_markdown_noise_drops_badges_html_comments_and_code_blocks | not yet confirmed |
| R203 | docs/requirements/issue-0159-hive-mind-lookup-and-curated-project-summarization.md | pre-2026-07 (undated); issue #159 | rust/tests/unit/specification/summarization_pipeline.rs::formalize_dialog_biases_user_turns_above_assistant_turns | not yet confirmed |
| R204 | docs/requirements/issue-0159-hive-mind-lookup-and-curated-project-summarization.md | pre-2026-07 (undated); issue #159 | rust/tests/unit/specification/summarization_pipeline.rs::generate_chat_title_returns_five_or_fewer_words | not yet confirmed |
| R205 | docs/requirements/issue-0159-hive-mind-lookup-and-curated-project-summarization.md | pre-2026-07 (undated); issue #159 | rust/tests/unit/specification/project_lookups.rs::http_fetch_of_curated_github_url_describes_project_via_summarization | not yet confirmed |
| R206 | docs/requirements/issue-0159-hive-mind-lookup-and-curated-project-summarization.md | pre-2026-07 (undated); issue #159 | rust/tests/unit/specification/summarization_pipeline.rs::default_max_statements_is_thirty | not yet confirmed |
| R207 | docs/requirements/issue-0159-hive-mind-lookup-and-curated-project-summarization.md | pre-2026-07 (undated); issue #159 | rust/tests/unit/specification/summarization_pipeline.rs::summarization_mode_target_percent_matches_vision | not yet confirmed |
| R208 | docs/requirements/issue-0159-hive-mind-lookup-and-curated-project-summarization.md | pre-2026-07 (undated); issue #159 | rust/tests/unit/specification/project_lookups.rs::associative_project_promotion_can_be_disabled | not yet confirmed |
| R209 | docs/requirements/issue-0159-hive-mind-lookup-and-curated-project-summarization.md | pre-2026-07 (undated); issue #159 | issue-level coverage (not row-pinned): rust/tests/unit/specification/project_lookups.rs:17 | not yet confirmed |
| R210 | docs/requirements/issue-0162-calendar-weekday-reasoning.md | pre-2026-07 (undated); issue #162 | issue-level coverage (not row-pinned): rust/tests/unit/specification/reasoning_paths.rs:221 | not yet confirmed |
| R211 | docs/requirements/issue-0162-calendar-weekday-reasoning.md | pre-2026-07 (undated); issue #162 | issue-level coverage (not row-pinned): rust/tests/unit/specification/reasoning_paths.rs:221 | not yet confirmed |
| R212 | docs/requirements/issue-0162-calendar-weekday-reasoning.md | pre-2026-07 (undated); issue #162 | rust/tests/unit/specification/reasoning_paths.rs | not yet confirmed |
| R213 | docs/requirements/issue-0207-natural-translation-pipeline.md | pre-2026-07 (undated); issue #207 | rust/tests/unit/specification/translation_via_links.rs::russian_translate_how_are_you_prompt_returns_english_surface | not yet confirmed |
| R214 | docs/requirements/issue-0207-natural-translation-pipeline.md | pre-2026-07 (undated); issue #207 | rust/tests/unit/specification/translation_via_links.rs::russian_translate_how_are_you_prompt_returns_english_surface | not yet confirmed |
| R215 | docs/requirements/issue-0207-natural-translation-pipeline.md | pre-2026-07 (undated); issue #207 | rust/tests/unit/specification/translation_via_links.rs::translation_meaning_registry_covers_extended_phrases | not yet confirmed |
| R526-1 | docs/requirements/issue-0526-translation-quality-test.md | pre-2026-07 (undated); issue #526 | rust/tests/unit/specification/translation_round_trip.rs | not yet confirmed |
| R526-2 | docs/requirements/issue-0526-translation-quality-test.md | pre-2026-07 (undated); issue #526 | issue-level coverage (not row-pinned): rust/tests/unit/specification/translation_round_trip.rs | not yet confirmed |
| R526-3 | docs/requirements/issue-0526-translation-quality-test.md | pre-2026-07 (undated); issue #526 | issue-level coverage (not row-pinned): rust/tests/unit/specification/translation_round_trip.rs | not yet confirmed |
| R526-4 | docs/requirements/issue-0526-translation-quality-test.md | pre-2026-07 (undated); issue #526 | issue-level coverage (not row-pinned): rust/tests/unit/specification/translation_round_trip.rs | not yet confirmed |
| R526-5 | docs/requirements/issue-0526-translation-quality-test.md | pre-2026-07 (undated); issue #526 | issue-level coverage (not row-pinned): rust/tests/unit/specification/translation_round_trip.rs | not yet confirmed |
| R526-6 | docs/requirements/issue-0526-translation-quality-test.md | pre-2026-07 (undated); issue #526 | issue-level coverage (not row-pinned): rust/tests/unit/specification/translation_round_trip.rs | not yet confirmed |
| R890-1 | docs/requirements/issue-0890-formal-proof-program-translation.md | pre-2026-07 (undated); issue #890 | issue-level coverage (not row-pinned): rust/tests/unit/proof_language_independence.rs | not yet confirmed |
| R890-2 | docs/requirements/issue-0890-formal-proof-program-translation.md | pre-2026-07 (undated); issue #890 | issue-level coverage (not row-pinned): rust/tests/unit/proof_language_independence.rs | not yet confirmed |
| R890-3 | docs/requirements/issue-0890-formal-proof-program-translation.md | pre-2026-07 (undated); issue #890 | issue-level coverage (not row-pinned): rust/tests/unit/proof_language_independence.rs | not yet confirmed |
| R890-4 | docs/requirements/issue-0890-formal-proof-program-translation.md | pre-2026-07 (undated); issue #890 | issue-level coverage (not row-pinned): rust/tests/unit/proof_language_independence.rs | not yet confirmed |
| R890-5 | docs/requirements/issue-0890-formal-proof-program-translation.md | pre-2026-07 (undated); issue #890 | rust/tests/e2e/tests/proof-program-translation.spec.js | not yet confirmed |
| R890-6 | docs/requirements/issue-0890-formal-proof-program-translation.md | pre-2026-07 (undated); issue #890 | issue-level coverage (not row-pinned): rust/tests/unit/proof_language_independence.rs | not yet confirmed |
| R890-7 | docs/requirements/issue-0890-formal-proof-program-translation.md | pre-2026-07 (undated); issue #890 | issue-level coverage (not row-pinned): rust/tests/unit/proof_language_independence.rs | not yet confirmed |
| R498-1 | docs/requirements/issue-0498-google-trends-requirements.md | pre-2026-07 (undated); issue #498 | issue-level coverage (not row-pinned): rust/tests/unit/agentic-coding/issue_498_google_trends_learning.rs | not yet confirmed |
| R498-2 | docs/requirements/issue-0498-google-trends-requirements.md | pre-2026-07 (undated); issue #498 | issue-level coverage (not row-pinned): rust/tests/unit/agentic-coding/issue_498_google_trends_learning.rs | not yet confirmed |
| R498-3 | docs/requirements/issue-0498-google-trends-requirements.md | pre-2026-07 (undated); issue #498 | issue-level coverage (not row-pinned): rust/tests/unit/agentic-coding/issue_498_google_trends_learning.rs | not yet confirmed |
| R498-4 | docs/requirements/issue-0498-google-trends-requirements.md | pre-2026-07 (undated); issue #498 | issue-level coverage (not row-pinned): rust/tests/unit/agentic-coding/issue_498_google_trends_learning.rs | not yet confirmed |
| R498-5 | docs/requirements/issue-0498-google-trends-requirements.md | pre-2026-07 (undated); issue #498 | issue-level coverage (not row-pinned): rust/tests/unit/agentic-coding/issue_498_google_trends_learning.rs | not yet confirmed |
| R498-6 | docs/requirements/issue-0498-google-trends-requirements.md | pre-2026-07 (undated); issue #498 | issue-level coverage (not row-pinned): rust/tests/unit/agentic-coding/issue_498_google_trends_learning.rs | not yet confirmed |
| R498-7 | docs/requirements/issue-0498-google-trends-requirements.md | pre-2026-07 (undated); issue #498 | rust/tests/unit/agentic-coding/issue_498_google_trends_catalog.rs | not yet confirmed |
| R498-8 | docs/requirements/issue-0498-google-trends-requirements.md | pre-2026-07 (undated); issue #498 | issue-level coverage (not row-pinned): rust/tests/unit/agentic-coding/issue_498_google_trends_learning.rs | not yet confirmed |
| R498-9 | docs/requirements/issue-0498-google-trends-requirements.md | pre-2026-07 (undated); issue #498 | issue-level coverage (not row-pinned): rust/tests/unit/agentic-coding/issue_498_google_trends_learning.rs | not yet confirmed |
| R498-10 | docs/requirements/issue-0498-google-trends-requirements.md | pre-2026-07 (undated); issue #498 | issue-level coverage (not row-pinned): rust/tests/unit/agentic-coding/issue_498_google_trends_learning.rs | not yet confirmed |
| R527-1 | docs/requirements/issue-0527-question-generation-requirements.md | pre-2026-07 (undated); issue #527 | rust/tests/unit/docs_requirements/question_generation_documents.rs | not yet confirmed |
| R527-2 | docs/requirements/issue-0527-question-generation-requirements.md | pre-2026-07 (undated); issue #527 | rust/tests/unit/generated_question_ordering.rs | not yet confirmed |
| R527-3 | docs/requirements/issue-0527-question-generation-requirements.md | pre-2026-07 (undated); issue #527 | rust/tests/unit/generated_question_ordering.rs | not yet confirmed |
| R527-4 | docs/requirements/issue-0527-question-generation-requirements.md | pre-2026-07 (undated); issue #527 | rust/tests/unit/generated_question_ordering.rs | not yet confirmed |
| R527-5 | docs/requirements/issue-0527-question-generation-requirements.md | pre-2026-07 (undated); issue #527 | rust/tests/unit/generated_question_ordering.rs | not yet confirmed |
| R527-6 | docs/requirements/issue-0527-question-generation-requirements.md | pre-2026-07 (undated); issue #527 | rust/tests/unit/generated_question_ordering.rs | not yet confirmed |
| R527-7 | docs/requirements/issue-0527-question-generation-requirements.md | pre-2026-07 (undated); issue #527 | rust/tests/unit/generated_question_ordering.rs | not yet confirmed |
| R527-8 | docs/requirements/issue-0527-question-generation-requirements.md | pre-2026-07 (undated); issue #527 | rust/tests/unit/docs_requirements/question_generation_documents.rs | not yet confirmed |
| R527-9 | docs/requirements/issue-0527-question-generation-requirements.md | pre-2026-07 (undated); issue #527 | rust/tests/unit/agentic-coding/issue_527_question_catalog.rs | not yet confirmed |
| R527-10 | docs/requirements/issue-0527-question-generation-requirements.md | pre-2026-07 (undated); issue #527 | rust/tests/unit/agentic-coding/issue_527_question_catalog.rs | not yet confirmed |
| R527-11 | docs/requirements/issue-0527-question-generation-requirements.md | pre-2026-07 (undated); issue #527 | rust/tests/unit/agentic-coding/issue_527_question_catalog.rs | not yet confirmed |
| R527-12 | docs/requirements/issue-0527-question-generation-requirements.md | pre-2026-07 (undated); issue #527 | rust/tests/unit/agentic-coding/issue_527_question_catalog.rs | not yet confirmed |
| R216 | docs/requirements/issue-0187-current-day-calendar-prompt.md | pre-2026-07 (undated); issue #187 | rust/tests/unit/specification/reasoning_paths.rs | not yet confirmed |
| R217 | docs/requirements/issue-0187-current-day-calendar-prompt.md | pre-2026-07 (undated); issue #187 | rust/tests/unit/specification/reasoning_paths.rs | not yet confirmed |
| R218 | docs/requirements/issue-0187-current-day-calendar-prompt.md | pre-2026-07 (undated); issue #187 | rust/tests/unit/specification/reasoning_paths.rs; rust/tests/e2e/tests/multilingual-chat.spec.js | not yet confirmed |
| R219 | docs/requirements/issue-0187-current-day-calendar-prompt.md | pre-2026-07 (undated); issue #187 | rust/tests/web/issue-0078-issue-reporting.test.mjs; rust/tests/e2e/scripts/check-multilingual-intent-coverage.mjs | not yet confirmed |
| R220 | docs/requirements/issue-0195-docker-in-docker-telegram-runtime.md | pre-2026-07 (undated); issue #195 | rust/tests/unit/docker_runtime.rs | not yet confirmed |
| R221 | docs/requirements/issue-0195-docker-in-docker-telegram-runtime.md | pre-2026-07 (undated); issue #195 | issue-level coverage (not row-pinned): rust/tests/unit/specification/agent_isolation.rs | not yet confirmed |
| R222 | docs/requirements/issue-0195-docker-in-docker-telegram-runtime.md | pre-2026-07 (undated); issue #195 | issue-level coverage (not row-pinned): rust/tests/unit/specification/agent_isolation.rs | not yet confirmed |
| R223 | docs/requirements/issue-0195-docker-in-docker-telegram-runtime.md | pre-2026-07 (undated); issue #195 | issue-level coverage (not row-pinned): rust/tests/unit/specification/agent_isolation.rs | not yet confirmed |
| R224 | docs/requirements/issue-0195-docker-in-docker-telegram-runtime.md | pre-2026-07 (undated); issue #195 | rust/tests/unit/docs_requirements.rs | not yet confirmed |
| R225 | docs/requirements/issue-0195-docker-in-docker-telegram-runtime.md | pre-2026-07 (undated); issue #195 | issue-level coverage (not row-pinned): rust/tests/unit/specification/agent_isolation.rs | not yet confirmed |
| R195-7 | docs/requirements/issue-0195-docker-in-docker-telegram-runtime.md | PR for #1138 (issue #195 completion) | rust/tests/unit/issue_1138_execution_box.rs; rust/tests/unit/issue_1138_telegram_execution.rs | not yet confirmed |
| R226 | docs/requirements/issue-0196-permanent-memory-deletion-and-reset.md | pre-2026-07 (undated); issue #196 | rust/tests/unit/memory_maintenance.rs; rust/tests/integration/formal_ai_cli.rs | not yet confirmed |
| R227 | docs/requirements/issue-0196-permanent-memory-deletion-and-reset.md | pre-2026-07 (undated); issue #196 | rust/tests/unit/memory_maintenance.rs; rust/tests/integration/formal_ai_cli.rs; rust/tests/e2e/tests/multilingual-memory-settings.spec.js | not yet confirmed |
| R228 | docs/requirements/issue-0196-permanent-memory-deletion-and-reset.md | pre-2026-07 (undated); issue #196 | rust/tests/e2e/tests/multilingual-memory-settings.spec.js | not yet confirmed |
| R229 | docs/requirements/issue-0196-permanent-memory-deletion-and-reset.md | pre-2026-07 (undated); issue #196 | rust/tests/e2e/tests/multilingual-memory-settings.spec.js | not yet confirmed |
| R230 | docs/requirements/issue-0196-permanent-memory-deletion-and-reset.md | pre-2026-07 (undated); issue #196 | rust/tests/web/case-study-preservation.test.mjs | not yet confirmed |
| R231 | docs/requirements/issue-0278-native-doublets-store-default-requirements.md | pre-2026-07 (undated); issue #278 | rust/tests/source/source_tests/link_store/tests.rs; rust/tests/integration/issue_1069_link_cli_store.rs | not yet confirmed |
| R232 | docs/requirements/issue-0278-native-doublets-store-default-requirements.md | pre-2026-07 (undated); issue #278 | rust/tests/source/source_tests/link_store/tests.rs | not yet confirmed |
| R233 | docs/requirements/issue-0278-native-doublets-store-default-requirements.md | pre-2026-07 (undated); issue #278 | none recorded | not yet confirmed |
| R234 | docs/requirements/issue-0278-native-doublets-store-default-requirements.md | pre-2026-07 (undated); issue #278 | rust/tests/source/source_tests/link_store/tests.rs | not yet confirmed |
| R235 | docs/requirements/issue-0278-native-doublets-store-default-requirements.md | pre-2026-07 (undated); issue #278 | rust/tests/unit/docs_requirements.rs | not yet confirmed |
| R236 | docs/requirements/issue-0278-native-doublets-store-default-requirements.md | pre-2026-07 (undated); issue #278 | rust/tests/unit/docs_requirements.rs | not yet confirmed |
| R237 | docs/requirements/issue-0279-symbolic-probabilistic-reasoning.md | pre-2026-07 (undated); issue #279 | none recorded | not yet confirmed |
| R238 | docs/requirements/issue-0279-symbolic-probabilistic-reasoning.md | pre-2026-07 (undated); issue #279 | none recorded | not yet confirmed |
| R239 | docs/requirements/issue-0279-symbolic-probabilistic-reasoning.md | pre-2026-07 (undated); issue #279 | none recorded | not yet confirmed |
| R240 | docs/requirements/issue-0279-symbolic-probabilistic-reasoning.md | pre-2026-07 (undated); issue #279 | none recorded | not yet confirmed |
| R241 | docs/requirements/issue-0279-symbolic-probabilistic-reasoning.md | pre-2026-07 (undated); issue #279 | none recorded | not yet confirmed |
| R242 | docs/requirements/issue-0279-symbolic-probabilistic-reasoning.md | pre-2026-07 (undated); issue #279 | rust/tests/unit/specification/probabilistic_reasoning/evidence_core.rs | not yet confirmed |
| R243 | docs/requirements/issue-0283-generalized-natural-language-skill-compiler.md | pre-2026-07 (undated); issue #283 | issue-level coverage (not row-pinned): rust/tests/unit/specification/arbitrary_skill_compilation.rs | not yet confirmed |
| R244 | docs/requirements/issue-0283-generalized-natural-language-skill-compiler.md | pre-2026-07 (undated); issue #283 | issue-level coverage (not row-pinned): rust/tests/unit/specification/arbitrary_skill_compilation.rs | not yet confirmed |
| R245 | docs/requirements/issue-0283-generalized-natural-language-skill-compiler.md | pre-2026-07 (undated); issue #283 | issue-level coverage (not row-pinned): rust/tests/unit/specification/arbitrary_skill_compilation.rs | not yet confirmed |
| R246 | docs/requirements/issue-0327-cross-runtime-synthesis-parity.md | pre-2026-07 (undated); issue #327 | issue-level coverage (not row-pinned): rust/tests/unit/specification/synthesis.rs:39 | not yet confirmed |
| R247 | docs/requirements/issue-0327-cross-runtime-synthesis-parity.md | pre-2026-07 (undated); issue #327 | rust/tests/e2e/tests/cross-runtime-synthesis-parity.spec.js | not yet confirmed |
| R248 | docs/requirements/issue-0327-cross-runtime-synthesis-parity.md | pre-2026-07 (undated); issue #327 | issue-level coverage (not row-pinned): rust/tests/unit/specification/synthesis.rs:39 | not yet confirmed |
| R249 | docs/requirements/issue-0327-cross-runtime-synthesis-parity.md | pre-2026-07 (undated); issue #327 | rust/tests/web/issue-0327-wasm-synthesis-bridge.test.mjs | not yet confirmed |
| R250 | docs/requirements/issue-0244-vision-implementation-planning.md | pre-2026-07 (undated); issue #244 | rust/tests/web/issue-0244-vision-roadmap.test.mjs | not yet confirmed |
| R251 | docs/requirements/issue-0244-vision-implementation-planning.md | pre-2026-07 (undated); issue #244 | rust/tests/web/issue-0244-vision-roadmap.test.mjs | not yet confirmed |
| R252 | docs/requirements/issue-0244-vision-implementation-planning.md | pre-2026-07 (undated); issue #244 | rust/tests/web/case-study-preservation.test.mjs | not yet confirmed |
| R253 | docs/requirements/issue-0244-vision-implementation-planning.md | pre-2026-07 (undated); issue #244 | rust/tests/web/case-study-preservation.test.mjs | not yet confirmed |
| R254 | docs/requirements/issue-0244-vision-implementation-planning.md | pre-2026-07 (undated); issue #244 | rust/tests/web/issue-0244-vision-roadmap.test.mjs | not yet confirmed |
| R255 | docs/requirements/issue-0244-vision-implementation-planning.md | pre-2026-07 (undated); issue #244 | rust/tests/web/issue-0244-vision-roadmap.test.mjs | not yet confirmed |
| R256 | docs/requirements/issue-0349-reverse-sort-program-modification-roadmap.md | pre-2026-07 (undated); issue #349 | rust/tests/integration/issue_349_reverse_sort.rs::issue_349_reverse_sort_follow_up_must_not_be_unknown | not yet confirmed |
| R257 | docs/requirements/issue-0349-reverse-sort-program-modification-roadmap.md | pre-2026-07 (undated); issue #349 | rust/tests/unit/specification/code_generation_coreference.rs | not yet confirmed |
| R258 | docs/requirements/issue-0349-reverse-sort-program-modification-roadmap.md | pre-2026-07 (undated); issue #349 | rust/tests/unit/issue_936_substitution_compiler.rs | not yet confirmed |
| R259 | docs/requirements/issue-0349-reverse-sort-program-modification-roadmap.md | pre-2026-07 (undated); issue #349 | rust/tests/unit/specification/code_generation_program_modifiers.rs | not yet confirmed |
| R260 | docs/requirements/issue-0349-reverse-sort-program-modification-roadmap.md | pre-2026-07 (undated); issue #349 | rust/tests/integration/issue_349_reverse_sort.rs::issue_349_diagnostic_mode_emits_full_turn_5_reasoning_chain; rust/tests/e2e/tests/write-program-diagnostics.spec.js | not yet confirmed |
| R261 | docs/requirements/issue-0349-reverse-sort-program-modification-roadmap.md | pre-2026-07 (undated); issue #349 | rust/tests/web/issue-0361-cross-runtime-parity.test.mjs | not yet confirmed |
| R262 | docs/requirements/issue-0349-reverse-sort-program-modification-roadmap.md | pre-2026-07 (undated); issue #349 | rust/tests/unit/specification/coding_modification_benchmarks.rs::issue_362_multilingual_multi_turn_coding_modification_ratchet | not yet confirmed |
| R263 | docs/requirements/issue-0349-reverse-sort-program-modification-roadmap.md | pre-2026-07 (undated); issue #349 | rust/tests/e2e/tests/reasoning-first-report.spec.js | not yet confirmed |
| R264 | docs/requirements/issue-0349-reverse-sort-program-modification-roadmap.md | pre-2026-07 (undated); issue #349 | rust/tests/unit/specification/self_improvement.rs | not yet confirmed |
| R265 | docs/requirements/issue-0349-reverse-sort-program-modification-roadmap.md | pre-2026-07 (undated); issue #349 | rust/tests/web/issue-0365-roadmap-closure.test.mjs | not yet confirmed |
| R266 | docs/requirements/issue-0398-recursive-semantic-meta-language.md | pre-2026-07 (undated); issue #398 | issue-level coverage (not row-pinned): rust/tests/unit/seed/reference_closure.rs | not yet confirmed |
| R267 | docs/requirements/issue-0398-recursive-semantic-meta-language.md | pre-2026-07 (undated); issue #398 | issue-level coverage (not row-pinned): rust/tests/unit/seed/reference_closure.rs | not yet confirmed |
| R268 | docs/requirements/issue-0398-recursive-semantic-meta-language.md | pre-2026-07 (undated); issue #398 | issue-level coverage (not row-pinned): rust/tests/unit/seed/reference_closure.rs | not yet confirmed |
| R269 | docs/requirements/issue-0398-recursive-semantic-meta-language.md | pre-2026-07 (undated); issue #398 | issue-level coverage (not row-pinned): rust/tests/unit/seed/reference_closure.rs | not yet confirmed |
| R270 | docs/requirements/issue-0398-recursive-semantic-meta-language.md | pre-2026-07 (undated); issue #398 | issue-level coverage (not row-pinned): rust/tests/unit/seed/reference_closure.rs | not yet confirmed |
| R271 | docs/requirements/issue-0398-recursive-semantic-meta-language.md | pre-2026-07 (undated); issue #398 | rust/tests/source/source_tests/seed/meanings/tests.rs | not yet confirmed |
| R272 | docs/requirements/issue-0398-recursive-semantic-meta-language.md | pre-2026-07 (undated); issue #398 | issue-level coverage (not row-pinned): rust/tests/unit/seed/reference_closure.rs | not yet confirmed |
| R273 | docs/requirements/issue-0398-recursive-semantic-meta-language.md | pre-2026-07 (undated); issue #398 | issue-level coverage (not row-pinned): rust/tests/unit/seed/reference_closure.rs | not yet confirmed |
| R274 | docs/requirements/issue-0398-recursive-semantic-meta-language.md | pre-2026-07 (undated); issue #398 | issue-level coverage (not row-pinned): rust/tests/unit/seed/reference_closure.rs | not yet confirmed |
| R275 | docs/requirements/issue-0398-recursive-semantic-meta-language.md | pre-2026-07 (undated); issue #398 | rust/tests/source/seed/embedded.rs | not yet confirmed |
| R276 | docs/requirements/issue-0398-recursive-semantic-meta-language.md | pre-2026-07 (undated); issue #398 | issue-level coverage (not row-pinned): rust/tests/unit/seed/reference_closure.rs | not yet confirmed |
| R277 | docs/requirements/issue-0398-recursive-semantic-meta-language.md | pre-2026-07 (undated); issue #398 | issue-level coverage (not row-pinned): rust/tests/unit/seed/reference_closure.rs | not yet confirmed |
| R278 | docs/requirements/issue-0398-pr-review-standards-comment-4663407299.md | PR #399 (issue #398) | rust/tests/unit/data_files.rs | not yet confirmed |
| R279 | docs/requirements/issue-0398-pr-review-standards-comment-4663407299.md | PR #399 (issue #398) | rust/tests/unit/seed/overrides.rs | not yet confirmed |
| R280 | docs/requirements/issue-0398-pr-review-standards-comment-4663407299.md | PR #399 (issue #398) | issue-level coverage (not row-pinned): rust/tests/unit/seed/reference_closure.rs | not yet confirmed |
| R281 | docs/requirements/issue-0398-pr-review-standards-comment-4663407299.md | PR #399 (issue #398) | issue-level coverage (not row-pinned): rust/tests/unit/seed/reference_closure.rs | not yet confirmed |
| R282 | docs/requirements/issue-0398-pr-review-standards-comment-4663407299.md | PR #399 (issue #398) | issue-level coverage (not row-pinned): rust/tests/unit/seed/reference_closure.rs | not yet confirmed |
| R283 | docs/requirements/issue-0398-pr-review-standards-comment-4663407299.md | PR #399 (issue #398) | issue-level coverage (not row-pinned): rust/tests/unit/seed/reference_closure.rs | not yet confirmed |
| R284 | docs/requirements/issue-0398-pr-review-standards-comment-4668929105.md | pre-2026-07 (undated); issue #398 | rust/tests/unit/total_closure.rs | not yet confirmed |
| R285 | docs/requirements/issue-0398-pr-review-standards-comment-4668929105.md | pre-2026-07 (undated); issue #398 | issue-level coverage (not row-pinned): rust/tests/unit/seed/reference_closure.rs | not yet confirmed |
| R286 | docs/requirements/issue-0398-pr-review-standards-comment-4668929105.md | pre-2026-07 (undated); issue #398 | issue-level coverage (not row-pinned): rust/tests/unit/seed/reference_closure.rs | not yet confirmed |
| R287 | docs/requirements/issue-0398-pr-review-standards-comment-4668929105.md | pre-2026-07 (undated); issue #398 | issue-level coverage (not row-pinned): rust/tests/unit/seed/reference_closure.rs | not yet confirmed |
| R288 | docs/requirements/issue-0398-pr-review-standards-comment-4668929105.md | pre-2026-07 (undated); issue #398 | issue-level coverage (not row-pinned): rust/tests/unit/seed/reference_closure.rs | not yet confirmed |
| R289 | docs/requirements/issue-0412-pr-review-standards-comment-4674-knowledge-source-breadth.md | pre-2026-07 (undated); issue #412 | rust/tests/integration/issue_412_oracle_languages.rs; source_tests/solver_handler_oracle | not yet confirmed |
| R290 | docs/requirements/issue-0412-pr-review-standards-comment-4674-knowledge-source-breadth.md | pre-2026-07 (undated); issue #412 | issue-level coverage (not row-pinned): rust/tests/integration/issue_412_oracle_languages.rs | not yet confirmed |
| R291 | docs/requirements/issue-0412-pr-review-standards-comment-4674-knowledge-source-breadth.md | pre-2026-07 (undated); issue #412 | issue-level coverage (not row-pinned): rust/tests/integration/issue_412_oracle_languages.rs | not yet confirmed |
| R292 | docs/requirements/issue-0412-pr-review-standards-comment-4674-knowledge-source-breadth.md | pre-2026-07 (undated); issue #412 | issue-level coverage (not row-pinned): rust/tests/integration/issue_412_oracle_languages.rs | not yet confirmed |
| R293 | docs/requirements/issue-0408-text-and-code-editing-requirements.md | PR #416 (issue #408) | rust/tests/unit/specification/text_manipulation.rs | not yet confirmed |
| R294 | docs/requirements/issue-0408-text-and-code-editing-requirements.md | PR #416 (issue #408) | rust/tests/web/issue-0408-text-edit-parity.test.mjs; rust/tests/unit/specification/text_manipulation_benchmarks.rs | not yet confirmed |
| R295 | docs/requirements/issue-0408-text-and-code-editing-requirements.md | PR #416 (issue #408) | rust/tests/unit/specification/text_manipulation_benchmarks.rs::issue_408_text_code_edit_profile_passes_local_ratchet | not yet confirmed |
| R296 | docs/requirements/issue-0408-text-and-code-editing-requirements.md | PR #416 (issue #408) | rust/tests/unit/docs_requirements.rs::issue_408_text_edit_benchmark_scope_documents_are_traceable | not yet confirmed |
| R297 | docs/requirements/issue-0408-text-and-code-editing-requirements.md | PR #416 (issue #408) | none recorded | not yet confirmed |
| R298 | docs/requirements/issue-0451-symbolic-ai-reference-and-best-practices.md | PR #452 (issue #451) | rust/tests/unit/docs_requirements/symbolic_ai_reference.rs | not yet confirmed |
| R299 | docs/requirements/issue-0451-symbolic-ai-reference-and-best-practices.md | PR #452 (issue #451) | rust/tests/unit/docs_requirements/symbolic_ai_reference.rs | not yet confirmed |
| R300 | docs/requirements/issue-0451-symbolic-ai-reference-and-best-practices.md | PR #452 (issue #451) | rust/tests/unit/docs_requirements/symbolic_ai_reference.rs | not yet confirmed |
| R301 | docs/requirements/issue-0451-symbolic-ai-reference-and-best-practices.md | PR #452 (issue #451) | rust/tests/unit/docs_requirements/symbolic_ai_reference.rs | not yet confirmed |
| R302 | docs/requirements/issue-0451-symbolic-ai-reference-and-best-practices.md | PR #452 (issue #451) | rust/tests/unit/docs_requirements/symbolic_ai_reference.rs | not yet confirmed |
| R303 | docs/requirements/issue-0451-symbolic-ai-reference-and-best-practices.md | PR #452 (issue #451) | rust/tests/unit/docs_requirements/symbolic_ai_reference.rs | not yet confirmed |
| R304 | docs/requirements/issue-0451-symbolic-ai-reference-and-best-practices.md | PR #452 (issue #451) | rust/tests/unit/docs_requirements/symbolic_ai_reference.rs | not yet confirmed |
| R305 | docs/requirements/issue-0451-symbolic-ai-reference-and-best-practices.md | PR #452 (issue #451) | rust/tests/source/source_tests/proof_engine/decision/sat/tests.rs | not yet confirmed |
| R306 | docs/requirements/issue-0468-agentic-coding-mode.md | PR #469 (issue #468) | issue-level coverage (not row-pinned): rust/tests/integration/issue_716_agentic_execution.rs | not yet confirmed |
| R307 | docs/requirements/issue-0468-agentic-coding-mode.md | PR #469 (issue #468) | rust/tests/unit/agentic-coding/agentic_coding.rs | not yet confirmed |
| R308 | docs/requirements/issue-0468-agentic-coding-mode.md | PR #469 (issue #468) | issue-level coverage (not row-pinned): rust/tests/integration/issue_716_agentic_execution.rs | not yet confirmed |
| R309 | docs/requirements/issue-0468-agentic-coding-mode.md | PR #469 (issue #468) | issue-level coverage (not row-pinned): rust/tests/integration/issue_716_agentic_execution.rs | not yet confirmed |
| R310 | docs/requirements/issue-0468-agentic-coding-mode.md | PR #469 (issue #468) | rust/tests/unit/agentic-coding/agentic_coding.rs | not yet confirmed |
| R311 | docs/requirements/issue-0468-agentic-coding-mode.md | PR #469 (issue #468) | issue-level coverage (not row-pinned): rust/tests/integration/issue_716_agentic_execution.rs | not yet confirmed |
| R312 | docs/requirements/issue-0468-agentic-coding-mode.md | PR #469 (issue #468) | rust/tests/unit/agentic-coding/agentic_coding.rs | not yet confirmed |
| R313 | docs/requirements/issue-0468-agentic-coding-mode.md | PR #469 (issue #468) | issue-level coverage (not row-pinned): rust/tests/integration/issue_716_agentic_execution.rs | not yet confirmed |
| R314 | docs/requirements/issue-0468-agentic-coding-mode.md | PR #469 (issue #468); custom-task fallback repaired by #1138 B4 | rust/tests/unit/agentic-coding/agentic_coding.rs::a_custom_task_is_formalized_instead_of_the_seeded_fairy_tale; rust/tests/unit/agentic-coding/agentic_surfaces.rs | manually confirmed 2026-08-04 (audit): `formal-ai agent --help` run; offline `agent --silent --task ...` exit 0 — the 2026-08-04 finding "falls back to seeded fairy-tale KB rather than reflecting custom --task" is now pinned as a regression |
| R315 | docs/requirements/issue-0468-agentic-coding-mode.md | PR #469 (issue #468) | issue-level coverage (not row-pinned): rust/tests/integration/issue_716_agentic_execution.rs | not yet confirmed |
| R316 | docs/requirements/issue-0468-agentic-coding-mode.md | PR #469 (issue #468) | issue-level coverage (not row-pinned): rust/tests/integration/issue_716_agentic_execution.rs | not yet confirmed |
| R317 | docs/requirements/issue-0468-agentic-coding-mode.md | PR #469 (issue #468) | issue-level coverage (not row-pinned): rust/tests/integration/issue_716_agentic_execution.rs | not yet confirmed |
| R318 | docs/requirements/issue-0468-agentic-coding-mode.md | PR #469 (issue #468) | issue-level coverage (not row-pinned): rust/tests/integration/issue_716_agentic_execution.rs | not yet confirmed |
| R319 | docs/requirements/issue-0468-agentic-coding-mode.md | PR #469 (issue #468) | rust/tests/unit/agentic-coding/agentic_coding.rs; rust/tests/unit/agentic-coding/agentic_surfaces.rs | not yet confirmed |
| R320 | docs/requirements/issue-0438-prepared-telegram-docker-image.md | PR #470 (issue #438) | rust/tests/web/case-study-preservation.test.mjs | not yet confirmed |
| R321 | docs/requirements/issue-0438-prepared-telegram-docker-image.md | PR #470 (issue #438) | rust/tests/unit/ci-cd/release_publishing.rs | not yet confirmed |
| R322 | docs/requirements/issue-0438-prepared-telegram-docker-image.md | PR #470 (issue #438) | rust/tests/unit/docs_requirements.rs | not yet confirmed |
| R323 | docs/requirements/issue-0438-prepared-telegram-docker-image.md | PR #470 (issue #438) | rust/tests/unit/docker_runtime.rs | not yet confirmed |
| R324 | docs/requirements/issue-0438-prepared-telegram-docker-image.md | PR #470 (issue #438) | rust/tests/unit/docs_requirements.rs; rust/tests/unit/docker_runtime.rs | not yet confirmed |
| R325 | docs/requirements/issue-0438-prepared-telegram-docker-image.md | PR #470 (issue #438) | rust/tests/unit/ci-cd/release_publishing.rs | not yet confirmed |
| R326 | docs/requirements/issue-0438-prepared-telegram-docker-image.md | PR #470 (issue #438) | rust/tests/unit/docker_runtime.rs::compose_file_runs_prebuilt_telegram_image_with_minimum_configuration; rust/tests/unit/ci-cd/release_publishing.rs::release_workflow_publishes_prebuilt_ghcr_image_after_crate_is_visible_and_optional_docker_hub_mirror | not yet confirmed |
| R327 | docs/requirements/issue-0438-prepared-telegram-docker-image.md | PR #470 (issue #438) | desktop/scripts/service-control.test.mjs | not yet confirmed |
| R328 | docs/requirements/issue-0438-prepared-telegram-docker-image.md | PR #470 (issue #438) | rust/tests/unit/docker_runtime.rs | not yet confirmed |
| R329 | docs/requirements/issue-0438-prepared-telegram-docker-image.md | PR #470 (issue #438) | rust/tests/unit/specification/desktop_surface.rs::{desktop_service_control_starts_and_stops_prepared_containers,desktop_web_surface_exposes_one_click_service_controls}; rust/tests/unit/docker_runtime.rs::compose_file_offers_optional_openai_compatible_server_profile | not yet confirmed |
| R330 | docs/requirements/issue-0559-general-meta-algorithm.md | PR #560 (issue #559) | rust/tests/unit/specification/meta_frame.rs; rust/tests/unit/docs_requirements/general_meta_algorithm.rs::issue_559_problem_frame_is_traceable | not yet confirmed |
| R331 | docs/requirements/issue-0559-general-meta-algorithm.md | PR #560 (issue #559) | rust/tests/unit/specification/method_registry.rs; rust/tests/unit/specification/reasoning_paths.rs::selected_specialized_handler_is_recorded_as_a_meta_method | not yet confirmed |
| R332 | docs/requirements/issue-0559-general-meta-algorithm.md | PR #560 (issue #559) | rust/tests/unit/specification/meta_frame.rs; rust/tests/unit/specification/reasoning_loop.rs::handler_families_publish_loop_events_as_recursion_leaves | not yet confirmed |
| R333 | docs/requirements/issue-0559-general-meta-algorithm.md | PR #560 (issue #559) | rust/tests/unit/specification/meta_frame.rs; rust/tests/unit/docs_requirements/general_meta_algorithm.rs::issue_559_need_ledger_is_traceable | not yet confirmed |
| R334 | docs/requirements/issue-0559-general-meta-algorithm.md | PR #560 (issue #559) | rust/tests/unit/specification/solution_evidence.rs; rust/tests/unit/docs_requirements/general_meta_algorithm.rs::issue_559_solution_evidence_is_traceable | not yet confirmed |
| R335 | docs/requirements/issue-0559-general-meta-algorithm.md | PR #560 (issue #559) | rust/tests/unit/specification/recursive_core_recipe.rs; rust/tests/unit/docs_requirements/general_meta_algorithm.rs::issue_559_recursive_core_recipe_is_traceable | not yet confirmed |
| R336 | docs/requirements/issue-0559-general-meta-algorithm.md | PR #560 (issue #559) | rust/tests/unit/specification/route_method_alias.rs; rust/tests/unit/docs_requirements/general_meta_algorithm.rs::issue_559_route_method_alias_is_traceable | not yet confirmed |
| R337 | docs/requirements/issue-0559-general-meta-algorithm.md | PR #560 (issue #559) | rust/tests/unit/specification/meta_reasoning.rs; rust/tests/unit/docs_requirements/general_meta_algorithm.rs::issue_559_work_unit_reasoning_is_traceable | not yet confirmed |
| R338 | docs/requirements/issue-0559-general-meta-algorithm.md | PR #560 (issue #559) | rust/tests/unit/specification/meta_construction.rs; rust/tests/unit/docs_requirements/general_meta_algorithm.rs::issue_559_upward_construction_is_traceable | not yet confirmed |
| R339 | docs/requirements/issue-0559-general-meta-algorithm.md | PR #560 (issue #559) | rust/tests/unit/specification/selection.rs; rust/tests/unit/docs_requirements/general_meta_algorithm.rs::issue_559_selection_trace_is_traceable | not yet confirmed |
| R340 | docs/requirements/issue-0559-general-meta-algorithm.md | PR #560 (issue #559) | rust/tests/unit/specification/meta_self_improvement.rs; rust/tests/unit/docs_requirements/general_meta_algorithm.rs::issue_559_meta_self_improvement_is_traceable | not yet confirmed |
| R341 | docs/requirements/issue-0559-general-meta-algorithm.md | PR #560 (issue #559) | rust/tests/unit/specification/cue_lexicon.rs; rust/tests/unit/docs_requirements/general_meta_algorithm.rs::issue_559_cue_lexicon_is_traceable | not yet confirmed |
| R342 | docs/requirements/issue-0559-general-meta-algorithm.md | PR #560 (issue #559) | rust/tests/unit/specification/skill_ledger.rs; rust/tests/unit/docs_requirements/general_meta_algorithm.rs::issue_559_skill_ledger_is_traceable | not yet confirmed |
| R343 | docs/requirements/issue-0559-general-meta-algorithm.md | PR #560 (issue #559) | rust/tests/unit/specification/recipe_interpreter.rs; rust/tests/unit/docs_requirements/general_meta_algorithm.rs::issue_559_recipe_interpreter_is_traceable | not yet confirmed |
| R344 | docs/requirements/issue-0559-general-meta-algorithm.md | PR #560 (issue #559) | rust/tests/unit/seed/issue_699_handler_migration.rs; rust/tests/unit/specification/obligation_ledger.rs | not yet confirmed |
| R1138-B1-1 | docs/requirements/issue-1138-live-concept-lookup.md | PR for #1138 | rust/tests/unit/issue_1138_concept_lookup.rs | not yet confirmed |
| R1138-B1-2 | docs/requirements/issue-1138-live-concept-lookup.md | PR for #1138 | rust/tests/unit/issue_1138_concept_lookup.rs | not yet confirmed |
| R1138-B1-3 | docs/requirements/issue-1138-live-concept-lookup.md | PR for #1138 | rust/tests/unit/issue_1138_concept_lookup.rs | not yet confirmed |
| R1138-B1-4 | docs/requirements/issue-1138-live-concept-lookup.md | PR for #1138 | rust/tests/unit/issue_1138_universal_loop_lookup.rs | not yet confirmed |
| R1138-B1-5 | docs/requirements/issue-1138-live-concept-lookup.md | PR for #1138 | rust/tests/unit/issue_1138_concept_lookup.rs | not yet confirmed |
| R1138-B1-6 | docs/requirements/issue-1138-live-concept-lookup.md | PR for #1138 | rust/tests/unit/issue_1138_concept_lookup.rs | not yet confirmed |
| R1138-B1-7 | docs/requirements/issue-1138-live-concept-lookup.md | PR for #1138 | rust/tests/unit/issue_1138_universal_loop_lookup.rs | not yet confirmed |
| R1138-B1-8 | docs/requirements/issue-1138-live-concept-lookup.md | PR for #1138 | rust/tests/unit/issue_1138_concept_lookup.rs | not yet confirmed |
| R1138-B1-9 | docs/requirements/issue-1138-live-concept-lookup.md | PR for #1138 | rust/tests/unit/issue_1138_concept_lookup.rs; rust/tests/web/issue-1138-concept-lookup.test.mjs | not yet confirmed |
| R1138-B5-1 | docs/requirements/issue-1138-bottleneck-audit.md | PR for #1138 | rust/tests/unit/specification/execution_evidence.rs | not yet confirmed |
| R1138-B5-2 | docs/requirements/issue-1138-bottleneck-audit.md | PR for #1138 | rust/tests/unit/specification/obligation_ledger.rs | not yet confirmed |
| R1138-B5-3 | docs/requirements/issue-1138-bottleneck-audit.md | PR for #1138 | rust/tests/unit/issue_1138_obligation_evidence.rs | not yet confirmed |
| R1138-6-1 | docs/requirements/issue-1138-prerequisite-discovery.md | PR for #1138 | rust/tests/unit/prerequisite/issue_1138_toolchain_probe.rs; rust/tests/unit/prerequisite/issue_1138_prerequisite_need.rs | not yet confirmed |
| R1138-6-2 | docs/requirements/issue-1138-prerequisite-discovery.md | PR for #1138 | rust/tests/unit/issue_1138_surface_honesty.rs | not yet confirmed |
| R1138-6-3 | docs/requirements/issue-1138-prerequisite-discovery.md | PR for #1138 | rust/tests/unit/prerequisite/issue_1138_prerequisite_need.rs | not yet confirmed |
| R1138-6-4 | docs/requirements/issue-1138-prerequisite-discovery.md | PR for #1138 | rust/tests/unit/prerequisite/issue_1138_prerequisite_need.rs | not yet confirmed |
| R1138-6-5 | docs/requirements/issue-1138-prerequisite-discovery.md | PR for #1138 | rust/tests/unit/prerequisite/issue_1138_setup_publisher.rs | not yet confirmed |
| R1138-6-6 | docs/requirements/issue-1138-prerequisite-discovery.md | PR for #1138 | rust/tests/unit/prerequisite/issue_1138_install_scope.rs | not yet confirmed |
| R1138-6-7 | docs/requirements/issue-1138-prerequisite-discovery.md | PR for #1138 | rust/tests/unit/prerequisite/issue_1138_setup_publisher.rs; rust/tests/unit/prerequisite/issue_1138_install_scope.rs | not yet confirmed |
| R1138-6-8 | docs/requirements/issue-1138-prerequisite-discovery.md | PR for #1138 | rust/tests/unit/specification/prerequisite_recipe.rs; rust/tests/unit/prerequisite/issue_1138_prerequisite_need.rs | not yet confirmed |
| R1138-6-9 | docs/requirements/issue-1138-prerequisite-discovery.md | PR for #1138 | rust/tests/unit/issue_1138_execution_box.rs; rust/tests/unit/prerequisite/issue_1138_install_scope.rs | not yet confirmed |
| R1138-6-10 | docs/requirements/issue-1138-prerequisite-discovery.md | PR for #1138 | rust/tests/unit/issue_1138_execution_box.rs; rust/tests/unit/repository-workspace/issue_1138_named_tests.rs | not yet confirmed |
| R1138-6-11 | docs/requirements/issue-1138-prerequisite-discovery.md | PR for #1138 | rust/tests/unit/issue_1138_surface_honesty.rs; rust/tests/web/issue-1138-browser-runtime.test.mjs | not yet confirmed |
| R1138-6-12 | docs/requirements/issue-1138-prerequisite-discovery.md | PR for #1138 | rust/tests/unit/issue_1138_telegram_execution.rs; rust/tests/web/issue-1138-execution-parity.test.mjs | not yet confirmed |
| R1138-B2-1 | docs/requirements/issue-1138-composition-from-sources.md | PR for #1138 | rust/tests/unit/coding_discovery/procedure_text.rs | not yet confirmed |
| R1138-B2-2 | docs/requirements/issue-1138-composition-from-sources.md | PR for #1138 | rust/tests/unit/coding_discovery/program_ir.rs | not yet confirmed |
| R1138-B2-3 | docs/requirements/issue-1138-composition-from-sources.md | PR for #1138 | rust/tests/unit/coding_discovery/ir_lowering.rs; rust/tests/unit/coding_discovery/multilingual.rs | not yet confirmed |
| R1138-B2-4 | docs/requirements/issue-1138-composition-from-sources.md | PR for #1138 | rust/tests/unit/coding_discovery/fragment_catalog.rs | not yet confirmed |
| R1138-B2-5 | docs/requirements/issue-1138-composition-from-sources.md | PR for #1138 | rust/tests/unit/coding_discovery/ledger.rs | not yet confirmed |
| R1138-B2-6 | docs/requirements/issue-1138-composition-from-sources.md | PR for #1138 | data/benchmarks/external-results.lino (2026-09-17 full-suite rows); rust/tests/unit/specification/external_benchmarks.rs | not yet confirmed |
| R1138-B2-7 | docs/requirements/issue-1138-composition-from-sources.md | PR for #1138 | rust/tests/unit/coding_discovery/structural_composition.rs; rust/tests/unit/coding_discovery/no_memorization.rs | not yet confirmed |
| R1138-B2-8 | docs/requirements/issue-1138-composition-from-sources.md | PR for #1138 | data/seed/sources-registry.lino; rust/tests/unit/coding_discovery/oeis.rs; rust/tests/unit/coding_discovery/python_docs.rs | not yet confirmed |
| R1138-B4-1 | docs/requirements/issue-1138-formalization-depth.md | PR for #1138 | rust/tests/unit/issue_1138_formalization_depth.rs::an_unfamiliar_requirement_raises_a_need_for_every_unresolved_surface | not yet confirmed |
| R1138-B4-2 | docs/requirements/issue-1138-formalization-depth.md | PR for #1138 | rust/tests/unit/issue_1138_formalization_depth.rs::a_need_is_satisfied_by_the_registry_lookup_and_becomes_a_grounded_concept; rust/tests/unit/issue_1138_formalization_depth.rs::a_grounded_gloss_raises_its_own_needs_at_the_next_depth_and_stops_at_the_bound | not yet confirmed |
| R1138-B4-3 | docs/requirements/issue-1138-formalization-depth.md | PR for #1138 | rust/tests/unit/issue_1138_formalization_depth.rs::a_need_is_satisfied_by_the_registry_lookup_and_becomes_a_grounded_concept | not yet confirmed |
| R1138-B4-4 | docs/requirements/issue-1138-formalization-depth.md | PR for #1138 | rust/tests/unit/issue_1138_formalization_depth.rs::preserved_sentences_no_longer_satisfy_the_assertion_primitive; rust/tests/unit/issue_1138_formalization_depth.rs::a_document_with_an_unresolved_need_is_never_reported_as_covered | not yet confirmed |
| R1138-B4-5 | docs/requirements/issue-1138-formalization-depth.md | PR for #1138 | rust/tests/unit/issue_1138_formalization_depth.rs::an_extracted_procedure_enters_the_ledger_only_through_execution_and_review; rust/tests/unit/issue_1138_formalization_depth.rs::a_non_commercial_licensed_procedure_is_shown_but_refused_for_promotion | not yet confirmed |
| R1138-B4-6 | docs/requirements/issue-1138-formalization-depth.md | PR for #1138 | rust/tests/unit/issue_1138_formalization_depth.rs::the_same_requirement_in_five_languages_produces_one_concept_graph_identity | not yet confirmed |
| R1138-B4-7 | docs/requirements/issue-1138-formalization-depth.md | PR for #1138 | rust/tests/unit/issue_1138_segmentation.rs | not yet confirmed |
| R1138-B4-8 | docs/requirements/issue-1138-formalization-depth.md | PR for #1138 | rust/tests/unit/agentic-coding/agentic_coding.rs::a_custom_task_is_formalized_instead_of_the_seeded_fairy_tale; rust/tests/unit/agentic-coding/agentic_coding.rs::the_canonical_tale_still_formalizes_to_nine_primitives | not yet confirmed |
| R1138-B4-9 | docs/requirements/issue-1138-formalization-depth.md | PR for #1138 | rust/tests/unit/specification/meta_frame.rs::formalization_needs_join_the_universal_ledger_without_parallel_statuses | not yet confirmed |
| R1138-B4-10 | docs/requirements/issue-1138-formalization-depth.md | PR for #1138 | rust/tests/unit/issue_1138_formalization_depth.rs::an_offline_run_replays_the_committed_captures_and_reproduces_the_graph_identity | not yet confirmed |
| R345 | docs/requirements/issue-0563-repository-resource-summarization.md | PR #564 (issue #563) | rust/tests/unit/specification/summarization_pipeline.rs::repository_file_summary_recurses_into_markdown_embedded_grammars | not yet confirmed |
| R346 | docs/requirements/issue-0563-repository-resource-summarization.md | PR #564 (issue #563) | none recorded | not yet confirmed |
| R347 | docs/requirements/issue-0563-repository-resource-summarization.md | PR #564 (issue #563) | none recorded | not yet confirmed |
| R348 | docs/requirements/issue-0563-repository-resource-summarization.md | PR #564 (issue #563) | none recorded | not yet confirmed |
| R349 | docs/requirements/issue-0563-repository-resource-summarization.md | PR #564 (issue #563) | none recorded | not yet confirmed |
| R350 | docs/requirements/issue-0563-repository-resource-summarization.md | PR #564 (issue #563) | none recorded | not yet confirmed |
| R351 | docs/requirements/issue-0563-repository-resource-summarization.md | PR #564 (issue #563) | rust/tests/source/source_tests/summarization/mod/tests.rs::formalize_repository_file_rust_records_meta_language_and_symbols; rust/tests/unit/docs_requirements/repository_resource_summarization.rs::issue_563_repository_file_summarization_documents_are_traceable | not yet confirmed |
| R352 | docs/requirements/issue-0563-repository-resource-summarization.md | PR #564 (issue #563) | none recorded | not yet confirmed |
| R353 | docs/requirements/issue-0563-repository-resource-summarization.md | PR #564 (issue #563) | none recorded | not yet confirmed |
| R354 | docs/requirements/issue-0563-repository-resource-summarization.md | PR #564 (issue #563) | none recorded | not yet confirmed |
| R355 | docs/requirements/issue-0563-repository-resource-summarization.md | PR #564 (issue #563) | rust/tests/unit/specification/summarization_pipeline.rs::summarize_repository_resource_subsumes_file_summarization | not yet confirmed |
| R356 | docs/requirements/issue-0563-repository-resource-summarization.md | PR #564 (issue #563) | none recorded | not yet confirmed |
| R357 | docs/requirements/issue-0563-repository-resource-summarization.md | PR #564 (issue #563) | none recorded | not yet confirmed |
| R358 | docs/requirements/issue-0563-repository-resource-summarization.md | PR #564 (issue #563) | none recorded | not yet confirmed |
| R359 | docs/requirements/issue-0563-repository-resource-summarization.md | PR #564 (issue #563) | rust/tests/source/source_tests/summarization/mod/tests.rs::{summarize_repository_resource_topic_directory_is_identity_only,summarize_repository_resource_full_directory_recurses_into_nested_folder} | not yet confirmed |
| R360 | docs/requirements/issue-0492-release-badge-stability.md | PR #583 (issue #492) | rust/tests/unit/ci-cd/release_publishing.rs | not yet confirmed |
| R361 | docs/requirements/issue-0492-release-badge-stability.md | PR #583 (issue #492) | none recorded | not yet confirmed |
| R362 | docs/requirements/issue-0492-release-badge-stability.md | PR #583 (issue #492) | none recorded | not yet confirmed |
| R363 | docs/requirements/issue-0492-release-badge-stability.md | PR #583 (issue #492) | rust/tests/unit/ci-cd/release_publishing.rs | not yet confirmed |
| R364 | docs/requirements/issue-0492-release-badge-stability.md | PR #583 (issue #492) | rust/tests/web/case-study-preservation.test.mjs | not yet confirmed |
| R365 | docs/requirements/issue-0492-release-badge-stability.md | PR #583 (issue #492) | none recorded | not yet confirmed |
| R366 | docs/requirements/issue-0492-release-badge-stability.md | PR #583 (issue #492) | rust/tests/unit/docs_requirements/release_badge_stability.rs | not yet confirmed |
| R367 | docs/requirements/issue-0492-release-badge-stability.md | PR #583 (issue #492) | rust/tests/web/case-study-preservation.test.mjs | not yet confirmed |
| R368 | docs/requirements/issue-0492-release-badge-stability.md | PR #583 (issue #492) | rust/tests/unit/ci-cd/release_publishing.rs; rust/tests/unit/docs_requirements/release_badge_stability.rs | not yet confirmed |
| R369 | docs/requirements/issue-0492-release-badge-stability.md | PR #583 (issue #492) | rust/tests/unit/docs_requirements/release_badge_stability.rs | not yet confirmed |
| R499-1 | docs/requirements/issue-0499-learn-from-this-data-source-requirements.md | PR #641 (issue #499) | issue-level coverage (not row-pinned): rust/tests/unit/issue_499_learn_from_source.rs | not yet confirmed |
| R499-2 | docs/requirements/issue-0499-learn-from-this-data-source-requirements.md | PR #641 (issue #499) | rust/tests/unit/issue_499_learn_from_source.rs | not yet confirmed |
| R499-3 | docs/requirements/issue-0499-learn-from-this-data-source-requirements.md | PR #641 (issue #499) | issue-level coverage (not row-pinned): rust/tests/unit/issue_499_learn_from_source.rs | not yet confirmed |
| R499-4 | docs/requirements/issue-0499-learn-from-this-data-source-requirements.md | PR #641 (issue #499) | issue-level coverage (not row-pinned): rust/tests/unit/issue_499_learn_from_source.rs | not yet confirmed |
| R499-5 | docs/requirements/issue-0499-learn-from-this-data-source-requirements.md | PR #641 (issue #499) | issue-level coverage (not row-pinned): rust/tests/unit/issue_499_learn_from_source.rs | not yet confirmed |
| R499-6 | docs/requirements/issue-0499-learn-from-this-data-source-requirements.md | PR #641 (issue #499) | issue-level coverage (not row-pinned): rust/tests/unit/issue_499_learn_from_source.rs | not yet confirmed |
| R499-7 | docs/requirements/issue-0499-learn-from-this-data-source-requirements.md | PR #641 (issue #499) | issue-level coverage (not row-pinned): rust/tests/unit/issue_499_learn_from_source.rs | not yet confirmed |
| R499-8 | docs/requirements/issue-0499-learn-from-this-data-source-requirements.md | PR #641 (issue #499) | rust/tests/unit/issue_499_learn_from_source.rs; rust/tests/unit/docs_requirements/learn_from_data_source.rs | not yet confirmed |
| R370 | docs/requirements/issue-0538-detailed-meanings-and-words.md | PR #601 (issue #538) | rust/tests/unit/seed/word_surface_grammar.rs::tomato_surfaces_pin_their_grammatical_number | not yet confirmed |
| R371 | docs/requirements/issue-0538-detailed-meanings-and-words.md | PR #601 (issue #538) | rust/tests/unit/seed/word_surface_grammar.rs::tomato_surfaces_expose_part_of_speech_from_data | not yet confirmed |
| R372 | docs/requirements/issue-0538-detailed-meanings-and-words.md | PR #601 (issue #538) | rust/tests/unit/seed/word_surface_grammar.rs::every_tomato_surface_denotes_the_tomato_meaning | not yet confirmed |
| R373 | docs/requirements/issue-0538-detailed-meanings-and-words.md | PR #601 (issue #538) | rust/tests/unit/seed/word_surface_grammar.rs::tomato_singular_and_plural_are_distinct_forms_in_each_language | not yet confirmed |
| R374 | docs/requirements/issue-0538-detailed-meanings-and-words.md | PR #601 (issue #538) | rust/tests/unit/seed/semantic_grounding.rs | not yet confirmed |
| R375 | docs/requirements/issue-0538-detailed-meanings-and-words.md | PR #601 (issue #538) | none recorded | not yet confirmed |
| R376 | docs/requirements/issue-0538-detailed-meanings-and-words.md | PR #601 (issue #538) | rust/tests/unit/seed/word_surface_grammar.rs::grammatical_number_meanings_are_grounded_and_multilingual | not yet confirmed |
| R377 | docs/requirements/issue-0538-detailed-meanings-and-words.md | PR #601 (issue #538) | none recorded | not yet confirmed |
| R378 | docs/requirements/issue-0538-detailed-meanings-and-words.md | PR #601 (issue #538) | none recorded | not yet confirmed |
| R379 | docs/requirements/issue-0538-detailed-meanings-and-words.md | PR #601 (issue #538) | none recorded | not yet confirmed |
| R380 | docs/requirements/issue-0538-detailed-meanings-and-words.md | PR #601 (issue #538) | none recorded | not yet confirmed |
| R381 | docs/requirements/issue-0538-detailed-meanings-and-words.md | PR #601 (issue #538) | none recorded | not yet confirmed |
| R382 | docs/requirements/issue-0538-detailed-meanings-and-words.md | PR #601 (issue #538) | none recorded | not yet confirmed |
| R383 | docs/requirements/issue-0538-detailed-meanings-and-words.md | not delivered; the owning shard names it only as a follow-up (solution-plan R17 of issue #538, notes in `docs/vscode/extension.md`) and no open issue tracks it | none recorded | not yet confirmed |
| R384 | docs/requirements/issue-0538-detailed-meanings-and-words.md | PR #601 (issue #538) | none recorded | not yet confirmed |
| R385 | docs/requirements/issue-0538-detailed-meanings-and-words.md | PR #601 (issue #538) | rust/tests/unit/agentic-coding/issue_538_agentic.rs | not yet confirmed |
| R386 | docs/requirements/issue-0538-detailed-meanings-and-words.md | PR #601 (issue #538) | none recorded | not yet confirmed |
| R387 | docs/requirements/issue-0558-auto-learning.md | PR #637 (issue #558) | rust/tests/web/case-study-preservation.test.mjs | not yet confirmed |
| R388 | docs/requirements/issue-0558-auto-learning.md | PR #637 (issue #558) | rust/tests/web/case-study-preservation.test.mjs | not yet confirmed |
| R389 | docs/requirements/issue-0558-auto-learning.md | PR #637 (issue #558) | rust/tests/web/case-study-preservation.test.mjs | not yet confirmed |
| R390 | docs/requirements/issue-0558-auto-learning.md | PR #637 (issue #558) | rust/tests/web/case-study-preservation.test.mjs | not yet confirmed |
| R391 | docs/requirements/issue-0558-auto-learning.md | PR #637 (issue #558) | rust/tests/web/case-study-preservation.test.mjs | not yet confirmed |
| R392 | docs/requirements/issue-0558-auto-learning.md | PR #637 (issue #558) | rust/tests/unit/docs_requirements/auto_learning_case_study.rs; rust/tests/unit/mod.rs | not yet confirmed |
| R393 | docs/requirements/issue-0558-auto-learning.md | PR #637 (issue #558) | rust/tests/unit/agentic-coding/issue_558_self_healing.rs | not yet confirmed |
| R394 | docs/requirements/issue-0558-auto-learning.md | PR #637 (issue #558) | rust/tests/unit/agentic-coding/issue_558_self_healing.rs | not yet confirmed |
| R395 | docs/requirements/issue-0558-auto-learning.md | PR #637 (issue #558) | rust/tests/unit/agentic-coding/issue_558_self_healing.rs; rust/tests/integration/issue_558_self_healing.rs | not yet confirmed |
| R396 | docs/requirements/issue-0531-pattern-inference-research.md | PR #642 (issue #531) | none recorded | not yet confirmed |
| R397 | docs/requirements/issue-0531-pattern-inference-research.md | PR #642 (issue #531) | none recorded | not yet confirmed |
| R398 | docs/requirements/issue-0531-pattern-inference-research.md | PR #642 (issue #531) | none recorded | not yet confirmed |
| R399 | docs/requirements/issue-0531-pattern-inference-research.md | PR #642 (issue #531) | none recorded | not yet confirmed |
| R400 | docs/requirements/issue-0531-pattern-inference-research.md | PR #642 (issue #531) | none recorded | not yet confirmed |
| R401 | docs/requirements/issue-0531-pattern-inference-research.md | PR #642 (issue #531) | none recorded | not yet confirmed |
| R402 | docs/requirements/issue-0531-pattern-inference-research.md | PR #642 (issue #531) | rust/tests/unit/sequences_{store,symbols,converter,compression}.rs | not yet confirmed |
| R403 | docs/requirements/issue-0531-pattern-inference-research.md | PR #642 (issue #531) | rust/tests/unit/sequences_{patterns_1d,grid_2d,inference}.rs | not yet confirmed |
| R404 | docs/requirements/issue-0531-pattern-inference-research.md | PR #642 (issue #531) | rust/tests/unit/issue_531_concepts_probe.rs | not yet confirmed |
| R405 | docs/requirements/issue-0531-pattern-inference-research.md | PR #642 (issue #531) | rust/tests/unit/solver/issue_531_pattern_inference.rs | not yet confirmed |
| R406 | docs/requirements/issue-0531-pattern-inference-research.md | PR #642 (issue #531) | none recorded | not yet confirmed |
| R407 | docs/requirements/issue-0531-pattern-inference-research.md | PR #642 (issue #531) | rust/tests/unit/docs_requirements/pattern_inference_research.rs; rust/tests/unit/mod.rs | manually confirmed 2026-08-04 (audit): `npm --prefix desktop run smoke` passed (desktop/scripts/smoke.mjs) |
| R531-17 | docs/requirements/issue-0531-pattern-inference-research.md | PR #642 (issue #531) | none recorded | not yet confirmed |
| R531-18 | docs/requirements/issue-0531-pattern-inference-research.md | PR #642 (issue #531) | rust/tests/unit/issue_531_algorithm_discovery.rs | not yet confirmed |
| R531-19 | docs/requirements/issue-0531-pattern-inference-research.md | PR #642 (issue #531) | none recorded | not yet confirmed |
| R531-20 | docs/requirements/issue-0531-pattern-inference-research.md | PR #642 (issue #531) | none recorded | not yet confirmed |
| R531-21 | docs/requirements/issue-0531-pattern-inference-research.md | PR #642 (issue #531) | none recorded | not yet confirmed |
| R531-22 | docs/requirements/issue-0531-pattern-inference-research.md | PR #642 (issue #531) | none recorded | not yet confirmed |
| R531-23 | docs/requirements/issue-0531-pattern-inference-research.md | PR #642 (issue #531) | none recorded | not yet confirmed |
| R531-24 | docs/requirements/issue-0531-pattern-inference-research.md | PR #642 (issue #531) | none recorded | not yet confirmed |
| R531-25 | docs/requirements/issue-0531-pattern-inference-research.md | PR #642 (issue #531) | none recorded | not yet confirmed |
| R537 | docs/requirements/issue-0540-dreaming-memory-maintenance.md | PR #645 (issue #540) | rust/tests/unit/docs_requirements/dreaming_memory_maintenance.rs | not yet confirmed |
| R538 | docs/requirements/issue-0540-dreaming-memory-maintenance.md | PR #645 (issue #540) | none recorded | not yet confirmed |
| R539 | docs/requirements/issue-0540-dreaming-memory-maintenance.md | PR #645 (issue #540) | none recorded | not yet confirmed |
| R540 | docs/requirements/issue-0540-dreaming-memory-maintenance.md | PR #645 (issue #540) | none recorded | not yet confirmed |
| R541 | docs/requirements/issue-0540-dreaming-memory-maintenance.md | PR #645 (issue #540) | none recorded | not yet confirmed |
| R542 | docs/requirements/issue-0540-dreaming-memory-maintenance.md | PR #645 (issue #540) | none recorded | not yet confirmed |
| R543 | docs/requirements/issue-0540-dreaming-memory-maintenance.md | PR #645 (issue #540) | none recorded | not yet confirmed |
| R544 | docs/requirements/issue-0540-dreaming-memory-maintenance.md | PR #645 (issue #540) | none recorded | not yet confirmed |
| R545 | docs/requirements/issue-0540-dreaming-memory-maintenance.md | PR #645 (issue #540) | none recorded | not yet confirmed |
| R546 | docs/requirements/issue-0540-dreaming-memory-maintenance.md | PR #645 (issue #540) | none recorded | not yet confirmed |
| R547 | docs/requirements/issue-0540-dreaming-memory-maintenance.md | PR #645 (issue #540) | none recorded | not yet confirmed |
| R548 | docs/requirements/issue-0540-dreaming-memory-maintenance.md | PR #645 (issue #540) | rust/tests/unit/memory_maintenance.rs; rust/tests/unit/docs_requirements/dreaming_memory_maintenance.rs | manually confirmed 2026-08-04 (audit): `npm --prefix desktop run smoke` passed (desktop/scripts/smoke.mjs) |
| R408 | docs/requirements/issue-0540-dreaming-memory-maintenance.md | PR #645 (issue #540) | none recorded | not yet confirmed |
| R409 | docs/requirements/issue-0540-dreaming-memory-maintenance.md | PR #645 (issue #540) | none recorded | not yet confirmed |
| R410 | docs/requirements/issue-0540-dreaming-memory-maintenance.md | PR #645 (issue #540) | none recorded | not yet confirmed |
| R411 | docs/requirements/issue-0540-dreaming-memory-maintenance.md | PR #645 (issue #540) | none recorded | not yet confirmed |
| R412 | docs/requirements/issue-0540-dreaming-memory-maintenance.md | PR #645 (issue #540) | rust/tests/unit/specification/dreaming_meta_algorithm.rs | not yet confirmed |
| R413 | docs/requirements/issue-0540-dreaming-memory-maintenance.md | PR #645 (issue #540) | none recorded | not yet confirmed |
| R414 | docs/requirements/issue-0540-dreaming-memory-maintenance.md | PR #645 (issue #540) | none recorded | not yet confirmed |
| R415 | docs/requirements/issue-0540-dreaming-memory-maintenance.md | PR #645 (issue #540) | none recorded | not yet confirmed |
| R416 | docs/requirements/issue-0540-dreaming-memory-maintenance.md | PR #645 (issue #540) | none recorded | not yet confirmed |
| R417 | docs/requirements/issue-0540-dreaming-memory-maintenance.md | PR #645 (issue #540) | none recorded | not yet confirmed |
| R418 | docs/requirements/issue-0540-dreaming-memory-maintenance.md | PR #645 (issue #540) | none recorded | not yet confirmed |
| R419 | docs/requirements/issue-0540-dreaming-memory-maintenance.md | PR #645 (issue #540) | none recorded | not yet confirmed |
| R420 | docs/requirements/issue-0540-dreaming-memory-maintenance.md | PR #645 (issue #540) | none recorded | not yet confirmed |
| R421 | docs/requirements/issue-0540-dreaming-memory-maintenance.md | PR #645 (issue #540) | none recorded | not yet confirmed |
| R422 | docs/requirements/issue-0540-dreaming-memory-maintenance.md | PR #645 (issue #540) | rust/tests/unit/agentic-coding/issue_540_agent_cli.rs | not yet confirmed |
| R423 | docs/requirements/issue-0540-dreaming-memory-maintenance.md | PR #645 (issue #540) | rust/tests/unit/memory_learning.rs | not yet confirmed |
| R424 | docs/requirements/issue-0540-dreaming-memory-maintenance.md | PR #645 (issue #540) | none recorded | not yet confirmed |
| R425 | docs/requirements/issue-0701-auto-learning-adoption-gap.md | PR #645 (issue #540) | none recorded | not yet confirmed |
| R426 | docs/requirements/issue-0540-dreaming-memory-maintenance.md | PR #645 (issue #540) | none recorded | not yet confirmed |
| R427 | docs/requirements/issue-0540-dreaming-memory-maintenance.md | PR #645 (issue #540) | rust/tests/unit/dreaming_runtime.rs | not yet confirmed |
| R428 | docs/requirements/issue-0649-world-models-and-contexts.md | PR #675 (issue #649) | rust/tests/unit/docs_requirements/world_models_and_contexts.rs | not yet confirmed |
| R429 | docs/requirements/issue-0649-world-models-and-contexts.md | PR #675 (issue #649) | issue-level coverage (not row-pinned): rust/tests/unit/issue_702_world_model_dialog.rs:152 | not yet confirmed |
| R430 | docs/requirements/issue-0649-world-models-and-contexts.md | PR #675 (issue #649) | issue-level coverage (not row-pinned): rust/tests/unit/issue_702_world_model_dialog.rs:152 | not yet confirmed |
| R431 | docs/requirements/issue-0649-world-models-and-contexts.md | PR #675 (issue #649) | issue-level coverage (not row-pinned): rust/tests/unit/issue_702_world_model_dialog.rs:152 | not yet confirmed |
| R432 | docs/requirements/issue-0649-world-models-and-contexts.md | PR #675 (issue #649) | issue-level coverage (not row-pinned): rust/tests/unit/issue_702_world_model_dialog.rs:152 | not yet confirmed |
| R433 | docs/requirements/issue-0649-world-models-and-contexts.md | PR #675 (issue #649) | issue-level coverage (not row-pinned): rust/tests/unit/issue_702_world_model_dialog.rs:152 | not yet confirmed |
| R434 | docs/requirements/issue-0649-world-models-and-contexts.md | PR #675 (issue #649) | rust/tests/unit/docs_requirements/world_models_and_contexts.rs; rust/tests/unit/mod.rs | not yet confirmed |
| R435 | docs/requirements/issue-0482-nemotron-training-data-samples.md | PR #639 (issue #482) | issue-level coverage (not row-pinned): rust/tests/unit/specification/nemotron_training_samples.rs | not yet confirmed |
| R436 | docs/requirements/issue-0482-nemotron-training-data-samples.md | PR #639 (issue #482) | issue-level coverage (not row-pinned): rust/tests/unit/specification/nemotron_training_samples.rs | not yet confirmed |
| R437 | docs/requirements/issue-0482-nemotron-training-data-samples.md | PR #639 (issue #482) | issue-level coverage (not row-pinned): rust/tests/unit/specification/nemotron_training_samples.rs | not yet confirmed |
| R438 | docs/requirements/issue-0482-nemotron-training-data-samples.md | PR #639 (issue #482) | issue-level coverage (not row-pinned): rust/tests/unit/specification/nemotron_training_samples.rs | not yet confirmed |
| R439 | docs/requirements/issue-0482-nemotron-training-data-samples.md | PR #639 (issue #482) | issue-level coverage (not row-pinned): rust/tests/unit/specification/nemotron_training_samples.rs | not yet confirmed |
| R440 | docs/requirements/issue-0482-nemotron-training-data-samples.md | PR #639 (issue #482) | rust/tests/unit/specification/nemotron_training_samples.rs | not yet confirmed |
| R441 | docs/requirements/issue-0482-nemotron-training-data-samples.md | PR #639 (issue #482) | issue-level coverage (not row-pinned): rust/tests/unit/specification/nemotron_training_samples.rs | not yet confirmed |
| R442 | docs/requirements/issue-0482-nemotron-training-data-samples.md | PR #639 (issue #482) | issue-level coverage (not row-pinned): rust/tests/unit/specification/nemotron_training_samples.rs | not yet confirmed |
| R443 | docs/requirements/issue-0482-nemotron-training-data-samples.md | PR #639 (issue #482) | issue-level coverage (not row-pinned): rust/tests/unit/specification/nemotron_training_samples.rs | not yet confirmed |
| R444 | docs/requirements/issue-0482-nemotron-training-data-samples.md | PR #639 (issue #482) | rust/tests/unit/docs_requirements/nemotron_training_documents.rs; rust/tests/unit/mod.rs | not yet confirmed |
| R445 | docs/requirements/issue-0686-associative-knowledge-networks-learning.md | PR #689 (issue #686) | rust/tests/unit/docs_requirements/associative_knowledge_networks.rs | not yet confirmed |
| R446 | docs/requirements/issue-0686-associative-knowledge-networks-learning.md | PR #689 (issue #686) | issue-level coverage (not row-pinned): rust/tests/unit/memory/issue_686_associative_persistence.rs | not yet confirmed |
| R447 | docs/requirements/issue-0686-associative-knowledge-networks-learning.md | PR #689 (issue #686) | issue-level coverage (not row-pinned): rust/tests/unit/memory/issue_686_associative_persistence.rs | not yet confirmed |
| R448 | docs/requirements/issue-0686-associative-knowledge-networks-learning.md | PR #689 (issue #686) | issue-level coverage (not row-pinned): rust/tests/unit/memory/issue_686_associative_persistence.rs | not yet confirmed |
| R449 | docs/requirements/issue-0686-associative-knowledge-networks-learning.md | PR #689 (issue #686) | issue-level coverage (not row-pinned): rust/tests/unit/memory/issue_686_associative_persistence.rs | not yet confirmed |
| R450 | docs/requirements/issue-0686-associative-knowledge-networks-learning.md | PR #689 (issue #686) | rust/tests/unit/memory/issue_686_associative_persistence.rs | not yet confirmed |
| R451 | docs/requirements/issue-0686-associative-knowledge-networks-learning.md | PR #689 (issue #686) | issue-level coverage (not row-pinned): rust/tests/unit/memory/issue_686_associative_persistence.rs | not yet confirmed |
| R452 | docs/requirements/issue-0686-associative-knowledge-networks-learning.md | PR #689 (issue #686) | rust/tests/unit/docs_requirements/associative_knowledge_networks.rs; rust/tests/unit/mod.rs | not yet confirmed |
| R453 | docs/requirements/issue-0686-associative-knowledge-networks-learning.md | PR #689 (issue #686) | issue-level coverage (not row-pinned): rust/tests/unit/memory/issue_686_associative_persistence.rs | not yet confirmed |
| R454 | docs/requirements/issue-0686-associative-knowledge-networks-learning.md | PR #689 (issue #686) | issue-level coverage (not row-pinned): rust/tests/unit/memory/issue_686_associative_persistence.rs | not yet confirmed |
| R455 | docs/requirements/issue-0686-associative-knowledge-networks-learning.md | PR #689 (issue #686) | issue-level coverage (not row-pinned): rust/tests/unit/memory/issue_686_associative_persistence.rs | not yet confirmed |
| R456 | docs/requirements/issue-0686-associative-knowledge-networks-learning.md | PR #689 (issue #686) | issue-level coverage (not row-pinned): rust/tests/unit/memory/issue_686_associative_persistence.rs | not yet confirmed |
| R457 | docs/requirements/issue-0686-associative-knowledge-networks-learning.md | PR #689 (issue #686) | issue-level coverage (not row-pinned): rust/tests/unit/memory/issue_686_associative_persistence.rs | not yet confirmed |
| R458 | docs/requirements/issue-0686-associative-knowledge-networks-learning.md | PR #689 (issue #686) | issue-level coverage (not row-pinned): rust/tests/unit/memory/issue_686_associative_persistence.rs | not yet confirmed |
| R459 | docs/requirements/issue-0656-benchmark-gated-promotion-protocol.md | PR #690 (issue #656) | issue-level coverage (not row-pinned): rust/tests/integration/issue_656_improve.rs | not yet confirmed |
| R460 | docs/requirements/issue-0656-benchmark-gated-promotion-protocol.md | PR #690 (issue #656) | issue-level coverage (not row-pinned): rust/tests/integration/issue_656_improve.rs | not yet confirmed |
| R461 | docs/requirements/issue-0656-benchmark-gated-promotion-protocol.md | PR #690 (issue #656) | issue-level coverage (not row-pinned): rust/tests/integration/issue_656_improve.rs | not yet confirmed |
| R462 | docs/requirements/issue-0656-benchmark-gated-promotion-protocol.md | PR #690 (issue #656) | issue-level coverage (not row-pinned): rust/tests/integration/issue_656_improve.rs | not yet confirmed |
| R463 | docs/requirements/issue-0656-benchmark-gated-promotion-protocol.md | PR #690 (issue #656) | rust/tests/integration/issue_656_improve.rs | not yet confirmed |
| R464 | docs/requirements/issue-0656-benchmark-gated-promotion-protocol.md | PR #690 (issue #656) | issue-level coverage (not row-pinned): rust/tests/integration/issue_656_improve.rs | not yet confirmed |
| R465 | docs/requirements/issue-0656-benchmark-gated-promotion-protocol.md | PR #690 (issue #656) | rust/tests/unit/docs_requirements/benchmark_gated_promotion.rs | not yet confirmed |
| R466 | docs/requirements/issue-0656-benchmark-gated-promotion-protocol.md | PR #690 (issue #656) | issue-level coverage (not row-pinned): rust/tests/integration/issue_656_improve.rs | not yet confirmed |
| R467 | docs/requirements/issue-0656-benchmark-gated-promotion-protocol.md | PR #690 (issue #656) | issue-level coverage (not row-pinned): rust/tests/integration/issue_656_improve.rs | not yet confirmed |
| R468 | docs/requirements/issue-0656-benchmark-gated-promotion-protocol.md | PR #690 (issue #656) | issue-level coverage (not row-pinned): rust/tests/integration/issue_656_improve.rs | not yet confirmed |
| R469 | docs/requirements/issue-0656-benchmark-gated-promotion-protocol.md | PR #690 (issue #656) | issue-level coverage (not row-pinned): rust/tests/integration/issue_656_improve.rs | not yet confirmed |
| R470 | docs/requirements/issue-0656-benchmark-gated-promotion-protocol.md | PR #690 (issue #656) | rust/tests/integration/issue_656_improve.rs | not yet confirmed |
| R471 | docs/requirements/issue-0656-benchmark-gated-promotion-protocol.md | PR #690 (issue #656) | issue-level coverage (not row-pinned): rust/tests/integration/issue_656_improve.rs | not yet confirmed |
| R472 | docs/requirements/issue-0656-benchmark-gated-promotion-protocol.md | PR #690 (issue #656) | rust/tests/integration/issue_656_improve.rs | not yet confirmed |
| R473 | docs/requirements/issue-0657-release-self-hosting-metric.md | PR #735 (issue #657) | issue-level coverage (not row-pinned): rust/tests/unit/agentic-coding/issue_657_self_hosting_learning.rs | not yet confirmed |
| R474 | docs/requirements/issue-0657-release-self-hosting-metric.md | PR #735 (issue #657) | issue-level coverage (not row-pinned): rust/tests/unit/agentic-coding/issue_657_self_hosting_learning.rs | not yet confirmed |
| R475 | docs/requirements/issue-0657-release-self-hosting-metric.md | PR #735 (issue #657) | issue-level coverage (not row-pinned): rust/tests/unit/agentic-coding/issue_657_self_hosting_learning.rs | not yet confirmed |
| R476 | docs/requirements/issue-0657-release-self-hosting-metric.md | PR #735 (issue #657) | issue-level coverage (not row-pinned): rust/tests/unit/agentic-coding/issue_657_self_hosting_learning.rs | not yet confirmed |
| R477 | docs/requirements/issue-0657-release-self-hosting-metric.md | PR #735 (issue #657) | issue-level coverage (not row-pinned): rust/tests/unit/agentic-coding/issue_657_self_hosting_learning.rs | not yet confirmed |
| R478 | docs/requirements/issue-0657-release-self-hosting-metric.md | PR #735 (issue #657) | issue-level coverage (not row-pinned): rust/tests/unit/agentic-coding/issue_657_self_hosting_learning.rs | not yet confirmed |
| R479 | docs/requirements/issue-0657-release-self-hosting-metric.md | PR #735 (issue #657) | issue-level coverage (not row-pinned): rust/tests/unit/agentic-coding/issue_657_self_hosting_learning.rs | not yet confirmed |
| R549 | docs/requirements/issue-0657-release-self-hosting-metric.md | PR #735 (issue #657) | issue-level coverage (not row-pinned): rust/tests/unit/agentic-coding/issue_657_self_hosting_learning.rs | not yet confirmed |
| R480 | docs/requirements/issue-0673-workspace-self-ast-census.md | PR #807 (issue #673) | issue-level coverage (not row-pinned): rust/tests/unit/issue_673_self_ast_census.rs | not yet confirmed |
| R481 | docs/requirements/issue-0673-workspace-self-ast-census.md | PR #807 (issue #673) | issue-level coverage (not row-pinned): rust/tests/unit/issue_673_self_ast_census.rs | not yet confirmed |
| R482 | docs/requirements/issue-0673-workspace-self-ast-census.md | PR #807 (issue #673) | issue-level coverage (not row-pinned): rust/tests/unit/issue_673_self_ast_census.rs | not yet confirmed |
| R483 | docs/requirements/issue-0673-workspace-self-ast-census.md | PR #807 (issue #673) | issue-level coverage (not row-pinned): rust/tests/unit/issue_673_self_ast_census.rs | not yet confirmed |
| R701-1 | docs/requirements/issue-0701-auto-learning-adoption-gap.md | PR #817 (issue #701) | issue-level coverage (not row-pinned): rust/tests/unit/issue_701_learning_adoption.rs | not yet confirmed |
| R701-2 | docs/requirements/issue-0701-auto-learning-adoption-gap.md | PR #817 (issue #701) | issue-level coverage (not row-pinned): rust/tests/unit/issue_701_learning_adoption.rs | not yet confirmed |
| R701-3 | docs/requirements/issue-0701-auto-learning-adoption-gap.md | PR #817 (issue #701) | issue-level coverage (not row-pinned): rust/tests/unit/issue_701_learning_adoption.rs | not yet confirmed |
| R701-4 | docs/requirements/issue-0701-auto-learning-adoption-gap.md | PR #817 (issue #701) | rust/tests/unit/issue_701_dreaming_amendment_class.rs | not yet confirmed |
| R701-5 | docs/requirements/issue-0701-auto-learning-adoption-gap.md | PR #817 (issue #701) | issue-level coverage (not row-pinned): rust/tests/unit/issue_701_learning_adoption.rs | not yet confirmed |
| R701-6 | docs/requirements/issue-0701-auto-learning-adoption-gap.md | PR #817 (issue #701) | issue-level coverage (not row-pinned): rust/tests/unit/issue_701_learning_adoption.rs | not yet confirmed |
| R550 | docs/requirements/issue-0674-arbitrary-natural-language-programs.md | PR #815 (issue #674) | none recorded | not yet confirmed |
| R551 | docs/requirements/issue-0674-arbitrary-natural-language-programs.md | PR #815 (issue #674) | none recorded | not yet confirmed |
| R552 | docs/requirements/issue-0674-arbitrary-natural-language-programs.md | PR #815 (issue #674) | none recorded | not yet confirmed |
| R553 | docs/requirements/issue-0674-arbitrary-natural-language-programs.md | PR #815 (issue #674) | none recorded | not yet confirmed |
| R554 | docs/requirements/issue-0674-arbitrary-natural-language-programs.md | PR #815 (issue #674) | none recorded | not yet confirmed |
| R555 | docs/requirements/issue-0674-arbitrary-natural-language-programs.md | PR #815 (issue #674) | none recorded | not yet confirmed |
| R556 | docs/requirements/issue-0674-arbitrary-natural-language-programs.md | PR #815 (issue #674) | none recorded | not yet confirmed |
| R557 | docs/requirements/issue-0674-arbitrary-natural-language-programs.md | PR #815 (issue #674) | none recorded | not yet confirmed |
| R558 | docs/requirements/issue-0674-arbitrary-natural-language-programs.md | PR #815 (issue #674) | none recorded | not yet confirmed |
| R528 | docs/requirements/issue-0698-real-external-benchmark-harness.md | PR #816 (issue #698) | rust/tests/unit/specification/external_benchmarks.rs | not yet confirmed |
| R529 | docs/requirements/issue-0698-real-external-benchmark-harness.md | PR #816 (issue #698) | rust/tests/unit/specification/external_benchmarks.rs | not yet confirmed |
| R530 | docs/requirements/issue-0698-real-external-benchmark-harness.md | PR #816 (issue #698) | rust/tests/unit/specification/external_benchmarks.rs | not yet confirmed |
| R531 | docs/requirements/issue-0698-real-external-benchmark-harness.md | PR #816 (issue #698) | rust/tests/unit/specification/external_benchmarks.rs | not yet confirmed |
| R532 | docs/requirements/issue-0698-real-external-benchmark-harness.md | PR #816 (issue #698) | rust/tests/unit/specification/external_benchmarks.rs | not yet confirmed |
| R533 | docs/requirements/issue-0698-real-external-benchmark-harness.md | PR #816 (issue #698) | rust/tests/unit/specification/external_benchmarks.rs | not yet confirmed |
| R534 | docs/requirements/issue-0698-real-external-benchmark-harness.md | PR #816 (issue #698) | none recorded | not yet confirmed |
| R535 | docs/requirements/issue-0698-real-external-benchmark-harness.md | PR #816 (issue #698) | rust/tests/unit/agentic-coding/issue_698_external_benchmark_learning.rs | not yet confirmed |
| R702-1 | docs/requirements/issue-0702-dialogue-world-model.md | PR #675 (issue #702) | none recorded | not yet confirmed |
| R702-2 | docs/requirements/issue-0702-dialogue-world-model.md | PR #675 (issue #702) | none recorded | not yet confirmed |
| R702-3 | docs/requirements/issue-0702-dialogue-world-model.md | PR #675 (issue #702) | none recorded | not yet confirmed |
| R702-4 | docs/requirements/issue-0702-dialogue-world-model.md | PR #675 (issue #702) | none recorded | not yet confirmed |
| R702-5 | docs/requirements/issue-0702-dialogue-world-model.md | PR #675 (issue #702) | none recorded | not yet confirmed |
| R702-6 | docs/requirements/issue-0702-dialogue-world-model.md | PR #675 (issue #702) | none recorded | not yet confirmed |
| R702-7 | docs/requirements/issue-0702-dialogue-world-model.md | PR #675 (issue #702) | none recorded | not yet confirmed |
| R702-8 | docs/requirements/issue-0702-dialogue-world-model.md | PR #675 (issue #702) | none recorded | not yet confirmed |
| R702-9 | docs/requirements/issue-0702-dialogue-world-model.md | PR #675 (issue #702) | none recorded | not yet confirmed |
| R702-10 | docs/requirements/issue-0702-dialogue-world-model.md | PR #675 (issue #702) | rust/tests/unit/docs_requirements/dialogue_world_model.rs | not yet confirmed |
| R702-11 | docs/requirements/issue-0702-dialogue-world-model.md | PR #675 (issue #702) | none recorded | not yet confirmed |
| R702-12 | docs/requirements/issue-0702-dialogue-world-model.md | PR #675 (issue #702) | none recorded | not yet confirmed |
| R702-13 | docs/requirements/issue-0702-dialogue-world-model.md | PR #675 (issue #702) | none recorded | not yet confirmed |
| R702-14 | docs/requirements/issue-0702-dialogue-world-model.md | PR #675 (issue #702) | none recorded | not yet confirmed |
| R702-15 | docs/requirements/issue-0702-dialogue-world-model.md | PR #675 (issue #702) | none recorded | not yet confirmed |
| R702-16 | docs/requirements/issue-0702-dialogue-world-model.md | PR #675 (issue #702) | none recorded | not yet confirmed |
| R703-1 | docs/requirements/issue-0703-external-agent-orchestration.md | PR #876 (issue #703) | rust/tests/integration/issue_703_orchestration.rs | not yet confirmed |
| R703-2 | docs/requirements/issue-0703-external-agent-orchestration.md | PR #876 (issue #703) | none recorded | not yet confirmed |
| R703-3 | docs/requirements/issue-0703-external-agent-orchestration.md | PR #876 (issue #703) | none recorded | not yet confirmed |
| R703-4 | docs/requirements/issue-0703-external-agent-orchestration.md | PR #876 (issue #703) | none recorded | not yet confirmed |
| R703-5 | docs/requirements/issue-0703-external-agent-orchestration.md | PR #876 (issue #703) | none recorded | not yet confirmed |
| R703-6 | docs/requirements/issue-0703-external-agent-orchestration.md | PR #876 (issue #703) | none recorded | not yet confirmed |
| R703-7 | docs/requirements/issue-0703-external-agent-orchestration.md | PR #876 (issue #703) | none recorded | not yet confirmed |
| R703-8 | docs/requirements/issue-0703-external-agent-orchestration.md | PR #876 (issue #703) | none recorded | not yet confirmed |
| R703-9 | docs/requirements/issue-0703-external-agent-orchestration.md | PR #876 (issue #703) | none recorded | not yet confirmed |
| R703-10 | docs/requirements/issue-0703-external-agent-orchestration.md | PR #876 (issue #703) | none recorded | not yet confirmed |
| R703-11 | docs/requirements/issue-0703-external-agent-orchestration.md | PR #876 (issue #703) | none recorded | not yet confirmed |
| R703-12 | docs/requirements/issue-0703-external-agent-orchestration.md | PR #876 (issue #703) | none recorded | not yet confirmed |
| R484 | docs/requirements/issue-0834-legal-compliance-self-audit.md | PR #837 (issue #834) | rust/tests/unit/docs_requirements/legal_compliance_self_audit.rs | not yet confirmed |
| R485 | docs/requirements/issue-0834-legal-compliance-self-audit.md | PR #837 (issue #834) | rust/tests/unit/docs_requirements/legal_compliance_self_audit.rs | not yet confirmed |
| R486 | docs/requirements/issue-0834-legal-compliance-self-audit.md | PR #837 (issue #834) | rust/tests/unit/docs_requirements/legal_compliance_self_audit.rs | not yet confirmed |
| R487 | docs/requirements/issue-0834-legal-compliance-self-audit.md | PR #837 (issue #834) | rust/tests/unit/docs_requirements/legal_compliance_self_audit.rs | not yet confirmed |
| R488 | docs/requirements/issue-0834-legal-compliance-self-audit.md | PR #837 (issue #834) | rust/tests/unit/docs_requirements/legal_compliance_self_audit.rs | not yet confirmed |
| R489 | docs/requirements/issue-0834-legal-compliance-self-audit.md | PR #837 (issue #834) | rust/tests/unit/docs_requirements/legal_compliance_self_audit.rs | not yet confirmed |
| R490 | docs/requirements/issue-0834-legal-compliance-self-audit.md | PR #837 (issue #834) | rust/tests/unit/docs_requirements/legal_compliance_self_audit.rs | not yet confirmed |
| R491 | docs/requirements/issue-0834-legal-compliance-self-audit.md | PR #837 (issue #834) | rust/tests/unit/docs_requirements/legal_compliance_self_audit.rs | not yet confirmed |
| R492 | docs/requirements/issue-0834-legal-compliance-self-audit.md | PR #837 (issue #834) | rust/tests/unit/docs_requirements/legal_compliance_self_audit.rs | not yet confirmed |
| R493 | docs/requirements/issue-0834-legal-compliance-self-audit.md | PR #837 (issue #834) | rust/tests/unit/docs_requirements/legal_compliance_self_audit.rs | not yet confirmed |
| R494 | docs/requirements/issue-0839-full-conversation-issue-reports.md | pre-2026-07 (undated); issue #839 | issue-level coverage (not row-pinned): rust/tests/integration/issue_839_context_export.rs | not yet confirmed |
| R495 | docs/requirements/issue-0839-full-conversation-issue-reports.md | pre-2026-07 (undated); issue #839 | issue-level coverage (not row-pinned): rust/tests/integration/issue_839_context_export.rs | not yet confirmed |
| R496 | docs/requirements/issue-0839-full-conversation-issue-reports.md | pre-2026-07 (undated); issue #839 | issue-level coverage (not row-pinned): rust/tests/integration/issue_839_context_export.rs | not yet confirmed |
| R497 | docs/requirements/issue-0839-full-conversation-issue-reports.md | pre-2026-07 (undated); issue #839 | rust/tests/integration/issue_839_report_parity.rs | not yet confirmed |
| R498 | docs/requirements/issue-0839-full-conversation-issue-reports.md | pre-2026-07 (undated); issue #839 | issue-level coverage (not row-pinned): rust/tests/integration/issue_839_context_export.rs | not yet confirmed |
| R499 | docs/requirements/issue-0839-full-conversation-issue-reports.md | pre-2026-07 (undated); issue #839 | issue-level coverage (not row-pinned): rust/tests/integration/issue_839_context_export.rs | not yet confirmed |
| R500 | docs/requirements/issue-0839-full-conversation-issue-reports.md | pre-2026-07 (undated); issue #839 | issue-level coverage (not row-pinned): rust/tests/integration/issue_839_context_export.rs | not yet confirmed |
| R501 | docs/requirements/issue-0844-statement-level-merging-into-a-context.md | PR #855 (issue #844) | issue-level coverage (not row-pinned): rust/tests/unit/issue_844_statement_merge.rs | not yet confirmed |
| R502 | docs/requirements/issue-0844-statement-level-merging-into-a-context.md | PR #855 (issue #844) | issue-level coverage (not row-pinned): rust/tests/unit/issue_844_statement_merge.rs | not yet confirmed |
| R503 | docs/requirements/issue-0844-statement-level-merging-into-a-context.md | PR #855 (issue #844) | issue-level coverage (not row-pinned): rust/tests/unit/issue_844_statement_merge.rs | not yet confirmed |
| R504 | docs/requirements/issue-0844-statement-level-merging-into-a-context.md | PR #855 (issue #844) | issue-level coverage (not row-pinned): rust/tests/unit/issue_844_statement_merge.rs | not yet confirmed |
| R505 | docs/requirements/issue-0844-statement-level-merging-into-a-context.md | PR #855 (issue #844) | issue-level coverage (not row-pinned): rust/tests/unit/issue_844_statement_merge.rs | not yet confirmed |
| R506 | docs/requirements/issue-0844-statement-level-merging-into-a-context.md | PR #855 (issue #844) | issue-level coverage (not row-pinned): rust/tests/unit/issue_844_statement_merge.rs | not yet confirmed |
| R507 | docs/requirements/issue-0844-statement-level-merging-into-a-context.md | PR #855 (issue #844) | issue-level coverage (not row-pinned): rust/tests/unit/issue_844_statement_merge.rs | not yet confirmed |
| R508 | docs/requirements/issue-0844-statement-level-merging-into-a-context.md | PR #855 (issue #844) | issue-level coverage (not row-pinned): rust/tests/unit/issue_844_statement_merge.rs | not yet confirmed |
| R509 | docs/requirements/issue-0844-statement-level-merging-into-a-context.md | PR #855 (issue #844) | issue-level coverage (not row-pinned): rust/tests/unit/issue_844_statement_merge.rs | not yet confirmed |
| R510 | docs/requirements/issue-0844-statement-level-merging-into-a-context.md | PR #855 (issue #844) | rust/tests/unit/docs_requirements/statement_merging.rs | not yet confirmed |
| R847-1 | docs/requirements/issue-0847-task-decomposition-as-a-working-task.md | PR #857 (issue #847) | issue-level coverage (not row-pinned): rust/tests/unit/solver/issue_847_task_decomposition.rs | not yet confirmed |
| R847-2 | docs/requirements/issue-0847-task-decomposition-as-a-working-task.md | PR #857 (issue #847) | issue-level coverage (not row-pinned): rust/tests/unit/solver/issue_847_task_decomposition.rs | not yet confirmed |
| R847-3 | docs/requirements/issue-0847-task-decomposition-as-a-working-task.md | PR #857 (issue #847) | issue-level coverage (not row-pinned): rust/tests/unit/solver/issue_847_task_decomposition.rs | not yet confirmed |
| R847-4 | docs/requirements/issue-0847-task-decomposition-as-a-working-task.md | PR #857 (issue #847) | issue-level coverage (not row-pinned): rust/tests/unit/solver/issue_847_task_decomposition.rs | not yet confirmed |
| R847-5 | docs/requirements/issue-0847-task-decomposition-as-a-working-task.md | PR #857 (issue #847) | issue-level coverage (not row-pinned): rust/tests/unit/solver/issue_847_task_decomposition.rs | not yet confirmed |
| R847-6 | docs/requirements/issue-0847-task-decomposition-as-a-working-task.md | PR #857 (issue #847) | issue-level coverage (not row-pinned): rust/tests/unit/solver/issue_847_task_decomposition.rs | not yet confirmed |
| R847-7 | docs/requirements/issue-0847-task-decomposition-as-a-working-task.md | PR #857 (issue #847) | issue-level coverage (not row-pinned): rust/tests/unit/solver/issue_847_task_decomposition.rs | not yet confirmed |
| R847-8 | docs/requirements/issue-0847-task-decomposition-as-a-working-task.md | PR #857 (issue #847) | rust/tests/unit/specification/task_decomposition.rs; rust/tests/unit/solver/issue_847_task_decomposition.rs | not yet confirmed |
| R848-1 | docs/requirements/issue-0848-executable-coding-tasks.md | PR #897 (issue #848) | none recorded | not yet confirmed |
| R848-2 | docs/requirements/issue-0848-executable-coding-tasks.md | PR #897 (issue #848) | none recorded | not yet confirmed |
| R848-3 | docs/requirements/issue-0848-executable-coding-tasks.md | PR #897 (issue #848) | none recorded | not yet confirmed |
| R848-4 | docs/requirements/issue-0848-executable-coding-tasks.md | PR #897 (issue #848) | none recorded | not yet confirmed |
| R848-5 | docs/requirements/issue-0848-executable-coding-tasks.md | PR #897 (issue #848) | none recorded | not yet confirmed |
| R848-6 | docs/requirements/issue-0848-executable-coding-tasks.md | PR #897 (issue #848) | none recorded | not yet confirmed |
| R848-7 | docs/requirements/issue-0848-executable-coding-tasks.md | PR #897 (issue #848) | none recorded | not yet confirmed |
| R848-8 | docs/requirements/issue-0848-executable-coding-tasks.md | PR #897 (issue #848) | none recorded | not yet confirmed |
| R848-9 | docs/requirements/issue-0848-executable-coding-tasks.md | PR #897 (issue #848) | none recorded | not yet confirmed |
| R848-10 | docs/requirements/issue-0848-executable-coding-tasks.md | PR #897 (issue #848) | none recorded | not yet confirmed |
| R706-1 | docs/requirements/issue-0706-any-language-protocol.md | PR #880 (issue #706) | issue-level coverage (not row-pinned): rust/tests/unit/issue_706_any_language.rs | not yet confirmed |
| R706-2 | docs/requirements/issue-0706-any-language-protocol.md | PR #880 (issue #706) | issue-level coverage (not row-pinned): rust/tests/unit/issue_706_any_language.rs | not yet confirmed |
| R706-3 | docs/requirements/issue-0706-any-language-protocol.md | PR #880 (issue #706) | issue-level coverage (not row-pinned): rust/tests/unit/issue_706_any_language.rs | not yet confirmed |
| R706-4 | docs/requirements/issue-0706-any-language-protocol.md | PR #880 (issue #706) | issue-level coverage (not row-pinned): rust/tests/unit/issue_706_any_language.rs | not yet confirmed |
| R706-5 | docs/requirements/issue-0706-any-language-protocol.md | PR #880 (issue #706) | rust/tests/unit/issue_706_any_language.rs | not yet confirmed |
| R706-6 | docs/requirements/issue-0706-any-language-protocol.md | PR #880 (issue #706) | issue-level coverage (not row-pinned): rust/tests/unit/issue_706_any_language.rs | not yet confirmed |
| R706-7 | docs/requirements/issue-0706-any-language-protocol.md | PR #880 (issue #706) | issue-level coverage (not row-pinned): rust/tests/unit/issue_706_any_language.rs | not yet confirmed |
| R706-8 | docs/requirements/issue-0706-any-language-protocol.md | PR #880 (issue #706) | issue-level coverage (not row-pinned): rust/tests/unit/issue_706_any_language.rs | not yet confirmed |
| R706-9 | docs/requirements/issue-0706-any-language-protocol.md | PR #880 (issue #706) | issue-level coverage (not row-pinned): rust/tests/unit/issue_706_any_language.rs | not yet confirmed |
| R858-1 | docs/requirements/issue-0858-claude-code-returning-user-recap.md | PR #899 (issue #858) | issue-level coverage (not row-pinned): rust/tests/unit/claude_code_away_recap.rs | not yet confirmed |
| R858-2 | docs/requirements/issue-0858-claude-code-returning-user-recap.md | PR #899 (issue #858) | issue-level coverage (not row-pinned): rust/tests/unit/claude_code_away_recap.rs | not yet confirmed |
| R858-3 | docs/requirements/issue-0858-claude-code-returning-user-recap.md | PR #899 (issue #858) | issue-level coverage (not row-pinned): rust/tests/unit/claude_code_away_recap.rs | not yet confirmed |
| R858-4 | docs/requirements/issue-0858-claude-code-returning-user-recap.md | PR #899 (issue #858) | issue-level coverage (not row-pinned): rust/tests/unit/claude_code_away_recap.rs | not yet confirmed |
| R858-5 | docs/requirements/issue-0858-claude-code-returning-user-recap.md | PR #899 (issue #858) | issue-level coverage (not row-pinned): rust/tests/unit/claude_code_away_recap.rs | not yet confirmed |
| R858-6 | docs/requirements/issue-0858-claude-code-returning-user-recap.md | PR #899 (issue #858) | issue-level coverage (not row-pinned): rust/tests/unit/claude_code_away_recap.rs | not yet confirmed |
| R708-1 | docs/requirements/issue-0708-bounded-natural-language-memory-programs.md | PR #883 (issue #708) | issue-level coverage (not row-pinned): rust/tests/integration/memory_query.rs | not yet confirmed |
| R708-2 | docs/requirements/issue-0708-bounded-natural-language-memory-programs.md | PR #883 (issue #708) | issue-level coverage (not row-pinned): rust/tests/integration/memory_query.rs | not yet confirmed |
| R708-3 | docs/requirements/issue-0708-bounded-natural-language-memory-programs.md | PR #883 (issue #708) | issue-level coverage (not row-pinned): rust/tests/integration/memory_query.rs | not yet confirmed |
| R708-4 | docs/requirements/issue-0708-bounded-natural-language-memory-programs.md | PR #883 (issue #708) | issue-level coverage (not row-pinned): rust/tests/integration/memory_query.rs | not yet confirmed |
| R708-5 | docs/requirements/issue-0708-bounded-natural-language-memory-programs.md | PR #883 (issue #708) | issue-level coverage (not row-pinned): rust/tests/integration/memory_query.rs | not yet confirmed |
| R708-6 | docs/requirements/issue-0708-bounded-natural-language-memory-programs.md | PR #883 (issue #708) | issue-level coverage (not row-pinned): rust/tests/integration/memory_query.rs | not yet confirmed |
| R708-7 | docs/requirements/issue-0708-bounded-natural-language-memory-programs.md | PR #883 (issue #708) | issue-level coverage (not row-pinned): rust/tests/integration/memory_query.rs | not yet confirmed |
| R708-8 | docs/requirements/issue-0708-bounded-natural-language-memory-programs.md | PR #883 (issue #708) | rust/tests/e2e/tests/browser-memory-programs.spec.js | not yet confirmed |
| R708-9 | docs/requirements/issue-0708-bounded-natural-language-memory-programs.md | PR #883 (issue #708) | issue-level coverage (not row-pinned): rust/tests/integration/memory_query.rs | not yet confirmed |
| R709-1 | docs/requirements/issue-0709-multi-source-search-fusion.md | pre-2026-07 (undated); issue #709 | rust/tests/unit/issue_709_search_fusion.rs | not yet confirmed |
| R709-2 | docs/requirements/issue-0709-multi-source-search-fusion.md | pre-2026-07 (undated); issue #709 | rust/tests/unit/issue_709_search_fusion.rs | not yet confirmed |
| R709-3 | docs/requirements/issue-0709-multi-source-search-fusion.md | pre-2026-07 (undated); issue #709 | rust/tests/unit/issue_709_search_fusion.rs | not yet confirmed |
| R709-4 | docs/requirements/issue-0709-multi-source-search-fusion.md | pre-2026-07 (undated); issue #709 | rust/tests/unit/issue_709_search_fusion.rs, rust/tests/e2e/tests/ranked-provenance-answers.spec.js | not yet confirmed |
| R709-5 | docs/requirements/issue-0709-multi-source-search-fusion.md | pre-2026-07 (undated); issue #709 | rust/tests/unit/issue_709_search_fusion.rs | not yet confirmed |
| R710-D1 | docs/requirements/issue-0710-dynamic-coding-discovery.md | delivered 2026-09-15; PR #888 (issue #710) | rust/tests/unit/coding_discovery/no_memorization.rs | not yet confirmed |
| R710-D2 | docs/requirements/issue-0710-dynamic-coding-discovery.md | delivered 2026-09-15; PR #888 (issue #710); full-suite measurement 2026-09-17 (PR #1139) | rust/tests/unit/specification/external_benchmarks.rs; first-20 upstream run recorded in docs/case-studies/issue-710/README.md; full-suite rows in data/benchmarks/external-results.lino | measured 2026-09-17 with `benchmark run --suite humaneval --slice 164`, cold-offline and with `--online`; rows recorded in data/benchmarks/external-results.lino |
| R710-D3 | docs/requirements/issue-0710-dynamic-coding-discovery.md | delivered 2026-09-15; PR #888 (issue #710); full-suite measurement 2026-09-18 (PR #1139) | rust/tests/unit/specification/external_benchmarks.rs; first-20 upstream run recorded in docs/case-studies/issue-710/README.md; full-suite rows in data/benchmarks/external-results.lino | measured 2026-09-18 with `benchmark run --suite mbpp --slice 500`, cold-offline and with `--online`; rows recorded in data/benchmarks/external-results.lino |
| R710-D4 | docs/requirements/issue-0710-dynamic-coding-discovery.md | delivered 2026-09-15; PR #888 (issue #710) | rust/tests/unit/coding_discovery/routing.rs | not yet confirmed |
| R710-D5 | docs/requirements/issue-0710-dynamic-coding-discovery.md | delivered 2026-09-15; PR #888 (issue #710) | rust/tests/unit/specification/external_benchmarks.rs | not yet confirmed |
| R710-D6 | docs/requirements/issue-0710-dynamic-coding-discovery.md | delivered 2026-09-15; PR #888 (issue #710) | rust/tests/unit/coding_discovery/python_docs.rs; rust/tests/unit/coding_discovery/wikifunctions.rs; rust/tests/unit/coding_discovery/rosetta.rs | not yet confirmed |
| R710-D7 | docs/requirements/issue-0710-dynamic-coding-discovery.md | delivered 2026-09-15; PR #888 (issue #710) | rust/tests/unit/coding_discovery/python_docs.rs; rust/tests/unit/coding_discovery/wikifunctions.rs; rust/tests/unit/coding_discovery/rosetta.rs | not yet confirmed |
| R710-D8 | docs/requirements/issue-0710-dynamic-coding-discovery.md | delivered 2026-09-15; PR #888 (issue #710) | rust/tests/unit/coding_discovery/concepts.rs | not yet confirmed |
| R710-D9 | docs/requirements/issue-0710-dynamic-coding-discovery.md | delivered 2026-09-15; PR #888 (issue #710) | rust/tests/unit/coding_discovery/composition.rs | not yet confirmed |
| R710-D10 | docs/requirements/issue-0710-dynamic-coding-discovery.md | delivered 2026-09-15; PR #888 (issue #710) | rust/tests/unit/coding_discovery/ledger.rs | not yet confirmed |
| R710-D11 | docs/requirements/issue-0710-dynamic-coding-discovery.md | delivered 2026-09-15; PR #888 (issue #710) | rust/tests/unit/coding_discovery/multilingual.rs | not yet confirmed |
| R710-D12 | docs/requirements/issue-0710-dynamic-coding-discovery.md | delivered 2026-09-15; PR #888 (issue #710) | rust/tests/unit/coding_discovery/multilingual.rs | not yet confirmed |
| R710-D13 | docs/requirements/issue-0710-dynamic-coding-discovery.md | delivered 2026-09-15; PR #888 (issue #710) | rust/tests/unit/coding_discovery/rosetta.rs | not yet confirmed |
| R710-D14 | docs/requirements/issue-0710-dynamic-coding-discovery.md | delivered 2026-09-15; PR #888 (issue #710) | rust/tests/unit/specification/external_benchmarks.rs | not yet confirmed |
| R710-D15 | docs/requirements/issue-0710-dynamic-coding-discovery.md | delivered 2026-09-15; PR #888 (issue #710) | rust/tests/unit/docs_benchmarks.rs | not yet confirmed |
| R710-D16 | docs/requirements/issue-0710-dynamic-coding-discovery.md | delivered 2026-09-15; PR #888 (issue #710) | rust/tests/unit/specification/self_hosting_metric.rs | not yet confirmed |
| R1137-1 | docs/requirements/issue-1137-pre-merge-four-client-routing-replay.md | delivered 2026-09-15; PR #888 (issue #1137) | rust/tests/unit/ci-cd/issue_1137_agentic_routing_replay.rs::agentic_routing_changes_enable_full_four_client_replay_on_pull_requests; scripts/detect-code-changes.rs::tests::agentic_source_changes_request_the_four_client_replay | not yet confirmed — CI on the final PR head is the unattended proof |
| R1137-2 | docs/requirements/issue-1137-pre-merge-four-client-routing-replay.md | delivered 2026-09-15; PR #888 (issue #1137) | rust/tests/unit/ci-cd/issue_1137_agentic_routing_replay.rs::the_full_replay_still_exercises_each_supported_client; experiments/agent_cli_e2e/run_issue_781.sh | not yet confirmed — CI on the final PR head is the unattended proof |
| R1137-3 | docs/requirements/issue-1137-pre-merge-four-client-routing-replay.md | delivered 2026-09-15; PR #888 (issue #1137) | rust/tests/unit/ci-cd/mod.rs (compiles scripts/detect-code-changes.rs into the unit binary, so its tests::agentic_source_changes_request_the_four_client_replay runs there: rust/src/coding/composition.rs and docs/agentic_coding/README.md stay outside the boundary), rust/tests/unit/ci-cd/issue_1137_agentic_routing_replay.rs | not yet confirmed |
| R835-1 | docs/requirements/issue-0835-multi-jurisdiction-file-legal-risk-assessment.md | PR #900 (issue #835) | issue-level coverage (not row-pinned): rust/tests/unit/issue_835_file_legality.rs | not yet confirmed |
| R835-2 | docs/requirements/issue-0835-multi-jurisdiction-file-legal-risk-assessment.md | PR #900 (issue #835) | issue-level coverage (not row-pinned): rust/tests/unit/issue_835_file_legality.rs | not yet confirmed |
| R835-3 | docs/requirements/issue-0835-multi-jurisdiction-file-legal-risk-assessment.md | PR #900 (issue #835) | issue-level coverage (not row-pinned): rust/tests/unit/issue_835_file_legality.rs | not yet confirmed |
| R835-4 | docs/requirements/issue-0835-multi-jurisdiction-file-legal-risk-assessment.md | PR #900 (issue #835) | issue-level coverage (not row-pinned): rust/tests/unit/issue_835_file_legality.rs | not yet confirmed |
| R835-5 | docs/requirements/issue-0835-multi-jurisdiction-file-legal-risk-assessment.md | PR #900 (issue #835) | issue-level coverage (not row-pinned): rust/tests/unit/issue_835_file_legality.rs | not yet confirmed |
| R835-6 | docs/requirements/issue-0835-multi-jurisdiction-file-legal-risk-assessment.md | PR #900 (issue #835) | issue-level coverage (not row-pinned): rust/tests/unit/issue_835_file_legality.rs | not yet confirmed |
| R835-7 | docs/requirements/issue-0835-multi-jurisdiction-file-legal-risk-assessment.md | PR #900 (issue #835) | issue-level coverage (not row-pinned): rust/tests/unit/issue_835_file_legality.rs | not yet confirmed |
| R835-8 | docs/requirements/issue-0835-multi-jurisdiction-file-legal-risk-assessment.md | PR #900 (issue #835) | issue-level coverage (not row-pinned): rust/tests/unit/issue_835_file_legality.rs | not yet confirmed |
| R835-9 | docs/requirements/issue-0835-multi-jurisdiction-file-legal-risk-assessment.md | PR #900 (issue #835) | issue-level coverage (not row-pinned): rust/tests/unit/issue_835_file_legality.rs | not yet confirmed |
| R864-1 | docs/requirements/issue-0864-proactive-failure-report-invitations.md | PR #910 (issue #864) | issue-level coverage (not row-pinned): rust/tests/unit/agentic-coding/proactive_failure_reports.rs | not yet confirmed |
| R864-2 | docs/requirements/issue-0864-proactive-failure-report-invitations.md | PR #910 (issue #864) | issue-level coverage (not row-pinned): rust/tests/unit/agentic-coding/proactive_failure_reports.rs | not yet confirmed |
| R864-3 | docs/requirements/issue-0864-proactive-failure-report-invitations.md | PR #910 (issue #864) | issue-level coverage (not row-pinned): rust/tests/unit/agentic-coding/proactive_failure_reports.rs | not yet confirmed |
| R864-4 | docs/requirements/issue-0864-proactive-failure-report-invitations.md | PR #910 (issue #864) | issue-level coverage (not row-pinned): rust/tests/unit/agentic-coding/proactive_failure_reports.rs | not yet confirmed |
| R864-5 | docs/requirements/issue-0864-proactive-failure-report-invitations.md | PR #910 (issue #864) | issue-level coverage (not row-pinned): rust/tests/unit/agentic-coding/proactive_failure_reports.rs | not yet confirmed |

[Continued traceability rows](requirements-traceability-continuation.md).
