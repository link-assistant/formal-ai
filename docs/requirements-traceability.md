# Requirements Traceability

Companion table for [REQUIREMENTS.md](../REQUIREMENTS.md). Per konard's
2026-08-04 standing requirement, every tracked requirement must record (1)
when it was delivered in code, (2) the automated test that pins it, and (3)
manual test confirmation. Populated from the 2026-08-04 requirement audit
(944 audited requirement records, `docs/case-studies/issue-914/` lineage).

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
| R1 | docs/requirements/preamble-requirements-for-issue-1.md | pre-2026-07 (undated) | none recorded | not yet confirmed |
| R2 | docs/requirements/preamble-requirements-for-issue-1.md | pre-2026-07 (undated) | none recorded | not yet confirmed |
| R3 | docs/requirements/preamble-requirements-for-issue-1.md | pre-2026-07 (undated) | none recorded | not yet confirmed |
| R4 | docs/requirements/preamble-requirements-for-issue-1.md | pre-2026-07 (undated) | none recorded | not yet confirmed |
| R5 | docs/requirements/preamble-requirements-for-issue-1.md | pre-2026-07 (undated) | none recorded | not yet confirmed |
| R6 | docs/requirements/preamble-requirements-for-issue-1.md | pre-2026-07 (undated) | none recorded | not yet confirmed |
| R7 | docs/requirements/preamble-requirements-for-issue-1.md | pre-2026-07 (undated) | none recorded | not yet confirmed |
| R8 | docs/requirements/preamble-requirements-for-issue-1.md | pre-2026-07 (undated) | none recorded | not yet confirmed |
| R9 | docs/requirements/preamble-requirements-for-issue-1.md | pre-2026-07 (undated) | none recorded | manually confirmed 2026-08-04 (audit): README `chat --prompt "Hi"` en greeting run via built binary |
| R10 | docs/requirements/preamble-requirements-for-issue-1.md | pre-2026-07 (undated) | none recorded | manually confirmed 2026-08-04 (audit): README `chat --prompt "Write me hello world program in Rust"` run, JSON chat format checked |
| R11 | docs/requirements/preamble-requirements-for-issue-1.md | pre-2026-07 (undated) | none recorded | not yet confirmed |
| R12 | docs/requirements/preamble-requirements-for-issue-1.md | pre-2026-07 (undated) | none recorded | not yet confirmed |
| R13 | docs/requirements/preamble-requirements-for-issue-1.md | pre-2026-07 (undated) | none recorded | manually confirmed 2026-08-04 (audit): `formal-ai serve` started, `/v1/chat/completions` and `/v1/responses` called, server stopped cleanly |
| R14 | docs/requirements/preamble-requirements-for-issue-1.md | pre-2026-07 (undated) | none recorded | not yet confirmed |
| R15 | docs/requirements/preamble-requirements-for-issue-1.md | pre-2026-07 (undated) | none recorded | not yet confirmed |
| R16 | docs/requirements/preamble-requirements-for-issue-1.md | pre-2026-07 (undated) | none recorded | not yet confirmed |
| R17 | docs/requirements/preamble-requirements-for-issue-1.md | pre-2026-07 (undated) | none recorded | manually confirmed 2026-08-04 (audit): `npm --prefix desktop run smoke` passed |
| R18 | docs/requirements/preamble-requirements-for-issue-1.md | pre-2026-07 (undated) | none recorded | not yet confirmed |
| R19 | docs/requirements/preamble-requirements-for-issue-1.md | pre-2026-07 (undated) | none recorded | not yet confirmed |
| R20 | docs/requirements/preamble-requirements-for-issue-1.md | pre-2026-07 (undated) | none recorded | not yet confirmed |
| R21 | docs/requirements/preamble-requirements-for-issue-1.md | pre-2026-07 (undated) | none recorded | not yet confirmed |
| R22 | docs/requirements/preamble-requirements-for-issue-1.md | pre-2026-07 (undated) | none recorded | manually confirmed 2026-08-04 (audit): README greetings run in en/ru/hi/zh (Hello/Привет/नमस्ते/你好) |
| R23 | docs/requirements/preamble-requirements-for-issue-1.md | pre-2026-07 (undated) | none recorded | not yet confirmed |
| R24 | docs/requirements/preamble-requirements-for-issue-1.md | pre-2026-07 (undated) | none recorded | not yet confirmed |
| R25 | docs/requirements/preamble-requirements-for-issue-1.md | pre-2026-07 (undated) | none recorded | not yet confirmed |
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
| R71 | docs/requirements/issue-0012-holistic-vision-requirements.md | pre-2026-07 (undated); issue #12 | rust/tests/unit/specification/ | not yet confirmed |
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
| R93 | docs/requirements/issue-0016-multilingual-wikipedia-and-append-only-memory-requirements.md | pre-2026-07 (undated); issue #16 | rust/tests/e2e/tests/multilingual.spec.js; rust/tests/e2e/playwright.local.config.js | not yet confirmed |
| R94 | docs/requirements/issue-0016-multilingual-wikipedia-and-append-only-memory-requirements.md | pre-2026-07 (undated); issue #16 | issue-level coverage (not row-pinned): rust/tests/unit/specification/multilingual.rs | not yet confirmed |
| R95 | docs/requirements/issue-0016-multilingual-wikipedia-and-append-only-memory-requirements.md | pre-2026-07 (undated); issue #16 | rust/tests/e2e/tests/multilingual.spec.js | not yet confirmed |
| R96 | docs/requirements/issue-0016-multilingual-wikipedia-and-append-only-memory-requirements.md | pre-2026-07 (undated); issue #16 | issue-level coverage (not row-pinned): rust/tests/unit/specification/multilingual.rs | not yet confirmed |
| R97 | docs/requirements/issue-0016-multilingual-wikipedia-and-append-only-memory-requirements.md | pre-2026-07 (undated); issue #16 | issue-level coverage (not row-pinned): rust/tests/unit/specification/multilingual.rs | not yet confirmed |
| R98 | docs/requirements/issue-0016-multilingual-wikipedia-and-append-only-memory-requirements.md | pre-2026-07 (undated); issue #16 | issue-level coverage (not row-pinned): rust/tests/unit/specification/multilingual.rs | not yet confirmed |
| R99 | docs/requirements/issue-0016-multilingual-wikipedia-and-append-only-memory-requirements.md | pre-2026-07 (undated); issue #16 | issue-level coverage (not row-pinned): rust/tests/unit/specification/multilingual.rs | not yet confirmed |
| R100 | docs/requirements/issue-0016-multilingual-wikipedia-and-append-only-memory-requirements.md | pre-2026-07 (undated); issue #16 | issue-level coverage (not row-pinned): rust/tests/unit/specification/multilingual.rs | not yet confirmed |
| R101 | docs/requirements/issue-0016-follow-up-universal-data-seed-across-every-interface-pr-17-reopen.md | PR #17 (issue #16) | issue-level coverage (not row-pinned): rust/tests/unit/specification/multilingual.rs | not yet confirmed |
| R102 | docs/requirements/issue-0016-follow-up-universal-data-seed-across-every-interface-pr-17-reopen.md | PR #17 (issue #16) | issue-level coverage (not row-pinned): rust/tests/unit/specification/multilingual.rs | not yet confirmed |
| R103 | docs/requirements/issue-0016-follow-up-universal-data-seed-across-every-interface-pr-17-reopen.md | PR #17 (issue #16) | seed::tests | not yet confirmed |
| R104 | docs/requirements/issue-0016-follow-up-universal-data-seed-across-every-interface-pr-17-reopen.md | PR #17 (issue #16) | seed::tests::bundle_round_trips_through_parse_bundle; seed::tests::parse_bundle_recovers_intent_routing_via_inner_parser | not yet confirmed |
| R105 | docs/requirements/issue-0016-follow-up-universal-data-seed-across-every-interface-pr-17-reopen.md | PR #17 (issue #16) | rust/tests/e2e/playwright.local.config.js | not yet confirmed |
| R106 | docs/requirements/issue-0016-follow-up-universal-data-seed-across-every-interface-pr-17-reopen.md | PR #17 (issue #16) | issue-level coverage (not row-pinned): rust/tests/unit/specification/multilingual.rs | not yet confirmed |
| R107 | docs/requirements/issue-0016-follow-up-universal-data-seed-across-every-interface-pr-17-reopen.md | PR #17 (issue #16) | issue-level coverage (not row-pinned): rust/tests/unit/specification/multilingual.rs | not yet confirmed |
| R108 | docs/requirements/issue-0016-follow-up-universal-data-seed-across-every-interface-pr-17-reopen.md | PR #17 (issue #16) | issue-level coverage (not row-pinned): rust/tests/unit/specification/multilingual.rs | not yet confirmed |
| R109 | docs/requirements/issue-0018-full-memory-export-import-requirements.md | pre-2026-07 (undated); issue #18 | issue-level coverage (not row-pinned): rust/tests/e2e/tests/issue-672-migration-replay.spec.js | not yet confirmed |
| R110 | docs/requirements/issue-0018-full-memory-export-import-requirements.md | pre-2026-07 (undated); issue #18 | issue-level coverage (not row-pinned): rust/tests/e2e/tests/issue-672-migration-replay.spec.js | not yet confirmed |
| R111 | docs/requirements/issue-0018-full-memory-export-import-requirements.md | pre-2026-07 (undated); issue #18 | issue-level coverage (not row-pinned): rust/tests/e2e/tests/issue-672-migration-replay.spec.js | not yet confirmed |
| R112 | docs/requirements/issue-0078-shorter-issue-reporting-requirements.md | pre-2026-07 (undated); issue #18 | issue-level coverage (not row-pinned): rust/tests/e2e/tests/issue-672-migration-replay.spec.js | not yet confirmed |
| R113 | docs/requirements/issue-0018-full-memory-export-import-requirements.md | pre-2026-07 (undated); issue #18 | issue-level coverage (not row-pinned): rust/tests/e2e/tests/issue-672-migration-replay.spec.js | not yet confirmed |
| R114 | docs/requirements/issue-0018-full-memory-export-import-requirements.md | pre-2026-07 (undated); issue #18 | rust/tests/e2e/tests/multilingual.spec.js; memory::tests::full_memory_round_trip_* | not yet confirmed |
| R115 | docs/requirements/issue-0078-shorter-issue-reporting-requirements.md | pre-2026-07 (undated); issue #78 | none recorded | not yet confirmed |
| R116 | docs/requirements/issue-0078-shorter-issue-reporting-requirements.md | pre-2026-07 (undated); issue #78 | rust/tests/e2e/tests/demo.spec.js | not yet confirmed |
| R117 | docs/requirements/issue-0078-shorter-issue-reporting-requirements.md | pre-2026-07 (undated); issue #78 | none recorded | not yet confirmed |
| R118 | docs/requirements/issue-0078-shorter-issue-reporting-requirements.md | pre-2026-07 (undated); issue #78 | none recorded | not yet confirmed |
| R119 | docs/requirements/issue-0078-shorter-issue-reporting-requirements.md | pre-2026-07 (undated); issue #78 | rust/tests/e2e/tests/multilingual.spec.js; rust/tests/e2e/tests/demo.spec.js | not yet confirmed |
| R120 | docs/requirements/issue-0096-calculator-delegation-requirements.md | pre-2026-07 (undated); issue #96 | none recorded | manually confirmed 2026-08-04 (audit): README arithmetic examples run in en (`8% of $50`) and ru (currency conversion) |
| R121 | docs/requirements/issue-0096-calculator-delegation-requirements.md | pre-2026-07 (undated); issue #96 | none recorded | not yet confirmed |
| R122 | docs/requirements/issue-0096-calculator-delegation-requirements.md | pre-2026-07 (undated); issue #96 | rust/tests/unit/specification/calculator_delegation.rs | not yet confirmed |
| R123 | docs/requirements/issue-0096-calculator-delegation-requirements.md | pre-2026-07 (undated); issue #96 | none recorded | not yet confirmed |
| R124 | docs/requirements/issue-0096-calculator-delegation-requirements.md | pre-2026-07 (undated); issue #96 | rust/tests/unit/specification/calculator_delegation.rs | not yet confirmed |
| R125 | docs/requirements/issue-0096-calculator-delegation-requirements.md | pre-2026-07 (undated); issue #96 | none recorded | not yet confirmed |
| R126 | docs/requirements/issue-0096-calculator-delegation-requirements.md | pre-2026-07 (undated); issue #96 | none recorded | not yet confirmed |
| R127 | docs/requirements/issue-0096-calculator-delegation-requirements.md | pre-2026-07 (undated); issue #96 | none recorded | not yet confirmed |
| R128 | docs/requirements/issue-0096-calculator-delegation-requirements.md | pre-2026-07 (undated); issue #96 | none recorded | not yet confirmed |
| R129 | docs/requirements/issue-0103-test-matrix-and-architecture-requirements.md | pre-2026-07 (undated); issue #103 | rust/tests/unit/specification/prompt_variations.rs; rust/tests/unit/specification/chat_surface.rs | not yet confirmed |
| R130 | docs/requirements/issue-0103-test-matrix-and-architecture-requirements.md | pre-2026-07 (undated); issue #103 | rust/tests/unit/specification/prompt_variations.rs; rust/tests/unit/specification/multilingual.rs | not yet confirmed |
| R131 | docs/requirements/issue-0103-test-matrix-and-architecture-requirements.md | pre-2026-07 (undated); issue #103 | issue-level coverage (not row-pinned): rust/tests/unit/specification/prompt_variations.rs | not yet confirmed |
| R132 | docs/requirements/issue-0103-test-matrix-and-architecture-requirements.md | pre-2026-07 (undated); issue #103 | rust/tests/unit/specification/prompt_variations.rs | not yet confirmed |
| R133 | docs/requirements/issue-0103-test-matrix-and-architecture-requirements.md | pre-2026-07 (undated); issue #103 | issue-level coverage (not row-pinned): rust/tests/unit/specification/prompt_variations.rs | not yet confirmed |
| R134 | docs/requirements/issue-0103-test-matrix-and-architecture-requirements.md | pre-2026-07 (undated); issue #103 | issue-level coverage (not row-pinned): rust/tests/unit/specification/prompt_variations.rs | not yet confirmed |
| R135 | docs/requirements/issue-0103-test-matrix-and-architecture-requirements.md | pre-2026-07 (undated); issue #103 | issue-level coverage (not row-pinned): rust/tests/unit/specification/prompt_variations.rs | not yet confirmed |
| R136 | docs/requirements/issue-0103-test-matrix-and-architecture-requirements.md | pre-2026-07 (undated); issue #103 | issue-level coverage (not row-pinned): rust/tests/unit/specification/prompt_variations.rs | not yet confirmed |
| R137 | docs/requirements/issue-0117-lino-i18n-catalog-requirements.md | pre-2026-07 (undated); issue #117 | none recorded | not yet confirmed |
| R138 | docs/requirements/issue-0117-lino-i18n-catalog-requirements.md | pre-2026-07 (undated); issue #117 | none recorded | not yet confirmed |
| R139 | docs/requirements/issue-0117-lino-i18n-catalog-requirements.md | pre-2026-07 (undated); issue #117 | none recorded | not yet confirmed |
| R140 | docs/requirements/issue-0117-lino-i18n-catalog-requirements.md | pre-2026-07 (undated); issue #117 | rust/tests/e2e/scripts/check-i18n-catalog.mjs; npm run --prefix rust/tests/e2e check:i18n | not yet confirmed |
| R141 | docs/requirements/issue-0117-lino-i18n-catalog-requirements.md | pre-2026-07 (undated); issue #117 | rust/tests/e2e/tests/demo.spec.js | not yet confirmed |
| R142 | docs/requirements/issue-0117-lino-i18n-catalog-requirements.md | pre-2026-07 (undated); issue #117 | none recorded | not yet confirmed |
| R143 | docs/requirements/issue-0115-github-evidence-collection-and-hive-mind-trace-requirements.md | pre-2026-07 (undated); issue #115 | none recorded | not yet confirmed |
| R144 | docs/requirements/issue-0115-github-evidence-collection-and-hive-mind-trace-requirements.md | pre-2026-07 (undated); issue #115 | none recorded | not yet confirmed |
| R145 | docs/requirements/issue-0115-github-evidence-collection-and-hive-mind-trace-requirements.md | pre-2026-07 (undated); issue #115 | none recorded | not yet confirmed |
| R146 | docs/requirements/issue-0115-github-evidence-collection-and-hive-mind-trace-requirements.md | pre-2026-07 (undated); issue #115 | none recorded | not yet confirmed |
| R147 | docs/requirements/issue-0115-github-evidence-collection-and-hive-mind-trace-requirements.md | pre-2026-07 (undated); issue #115 | none recorded | not yet confirmed |
| R148 | docs/requirements/issue-0115-github-evidence-collection-and-hive-mind-trace-requirements.md | pre-2026-07 (undated); issue #115 | none recorded | not yet confirmed |
| R149 | docs/requirements/issue-0115-github-evidence-collection-and-hive-mind-trace-requirements.md | pre-2026-07 (undated); issue #115 | rust/tests/unit/github_logs.rs; rust/tests/integration/formal_ai_cli.rs | not yet confirmed |
| R150 | docs/requirements/issue-0063-cross-language-definition-fusion-requirements.md | pre-2026-07 (undated); issue #63 | issue-level coverage (not row-pinned): rust/tests/unit/specification/definition_fusion.rs | not yet confirmed |
| R151 | docs/requirements/issue-0063-cross-language-definition-fusion-requirements.md | pre-2026-07 (undated); issue #63 | issue-level coverage (not row-pinned): rust/tests/unit/specification/definition_fusion.rs | not yet confirmed |
| R152 | docs/requirements/issue-0063-cross-language-definition-fusion-requirements.md | pre-2026-07 (undated); issue #63 | issue-level coverage (not row-pinned): rust/tests/unit/specification/definition_fusion.rs | not yet confirmed |
| R153 | docs/requirements/issue-0063-cross-language-definition-fusion-requirements.md | pre-2026-07 (undated); issue #63 | issue-level coverage (not row-pinned): rust/tests/unit/specification/definition_fusion.rs | not yet confirmed |
| R154 | docs/requirements/issue-0063-cross-language-definition-fusion-requirements.md | pre-2026-07 (undated); issue #63 | rust/tests/unit/specification/definition_fusion.rs; rust/tests/e2e/tests/multilingual.spec.js | not yet confirmed |
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
| R165 | docs/requirements/issue-0129-connectivity-diagnostics-requirements.md | pre-2026-07 (undated); issue #129 | js/tests/index.html | not yet confirmed |
| R166 | docs/requirements/issue-0129-connectivity-diagnostics-requirements.md | pre-2026-07 (undated); issue #129 | js/tests/connectivity.js | not yet confirmed |
| R167 | docs/requirements/issue-0129-connectivity-diagnostics-requirements.md | pre-2026-07 (undated); issue #129 | issue-level coverage (not row-pinned): rust/tests/connectivity.js | not yet confirmed |
| R168 | docs/requirements/issue-0129-connectivity-diagnostics-requirements.md | pre-2026-07 (undated); issue #129 | issue-level coverage (not row-pinned): rust/tests/connectivity.js | not yet confirmed |
| R169 | docs/requirements/issue-0129-connectivity-diagnostics-requirements.md | pre-2026-07 (undated); issue #129 | issue-level coverage (not row-pinned): rust/tests/connectivity.js | not yet confirmed |
| R170 | docs/requirements/issue-0129-connectivity-diagnostics-requirements.md | pre-2026-07 (undated); issue #129 | rust/tests/e2e/tests/connectivity.spec.js | not yet confirmed |
| R171 | docs/requirements/issue-0129-connectivity-diagnostics-requirements.md | pre-2026-07 (undated); issue #129 | rust/tests/unit/ci-cd/workflow_release.rs | not yet confirmed |
| R172 | docs/requirements/issue-0129-connectivity-diagnostics-requirements.md | pre-2026-07 (undated); issue #129 | issue-level coverage (not row-pinned): rust/tests/connectivity.js | not yet confirmed |
| R173 | docs/requirements/issue-0127-structured-fact-query-reasoning-requirements.md | pre-2026-07 (undated); issue #127 | none recorded | not yet confirmed |
| R174 | docs/requirements/issue-0127-structured-fact-query-reasoning-requirements.md | pre-2026-07 (undated); issue #127 | none recorded | not yet confirmed |
| R175 | docs/requirements/issue-0127-structured-fact-query-reasoning-requirements.md | pre-2026-07 (undated); issue #127 | none recorded | not yet confirmed |
| R176 | docs/requirements/issue-0127-structured-fact-query-reasoning-requirements.md | pre-2026-07 (undated); issue #127 | none recorded | not yet confirmed |
| R177 | docs/requirements/issue-0127-structured-fact-query-reasoning-requirements.md | pre-2026-07 (undated); issue #127 | none recorded | not yet confirmed |
| R178 | docs/requirements/issue-0127-structured-fact-query-reasoning-requirements.md | pre-2026-07 (undated); issue #127 | none recorded | not yet confirmed |
| R179 | docs/requirements/issue-0127-structured-fact-query-reasoning-requirements.md | pre-2026-07 (undated); issue #127 | none recorded | not yet confirmed |
| R180 | docs/requirements/issue-0127-structured-fact-query-reasoning-requirements.md | pre-2026-07 (undated); issue #127 | rust/tests/unit/specification/prompt_variations.rs; rust/tests/e2e/tests/multilingual.spec.js | not yet confirmed |
| R181 | docs/requirements/issue-0133-duckduckgo-default-combined-ranking-and-expanded-provider-diagnostics.md | pre-2026-07 (undated); issue #133 | issue-level coverage (not row-pinned): rust/tests/connectivity.js | not yet confirmed |
| R182 | docs/requirements/issue-0133-duckduckgo-default-combined-ranking-and-expanded-provider-diagnostics.md | pre-2026-07 (undated); issue #133 | issue-level coverage (not row-pinned): rust/tests/connectivity.js | not yet confirmed |
| R183 | docs/requirements/issue-0133-duckduckgo-default-combined-ranking-and-expanded-provider-diagnostics.md | pre-2026-07 (undated); issue #133 | issue-level coverage (not row-pinned): rust/tests/connectivity.js | not yet confirmed |
| R184 | docs/requirements/issue-0133-duckduckgo-default-combined-ranking-and-expanded-provider-diagnostics.md | pre-2026-07 (undated); issue #133 | js/tests/connectivity.js | not yet confirmed |
| R185 | docs/requirements/issue-0133-duckduckgo-default-combined-ranking-and-expanded-provider-diagnostics.md | pre-2026-07 (undated); issue #133 | js/tests/connectivity.js | not yet confirmed |
| R186 | docs/requirements/issue-0133-duckduckgo-default-combined-ranking-and-expanded-provider-diagnostics.md | pre-2026-07 (undated); issue #133 | issue-level coverage (not row-pinned): rust/tests/connectivity.js | not yet confirmed |
| R187 | docs/requirements/issue-0133-duckduckgo-default-combined-ranking-and-expanded-provider-diagnostics.md | pre-2026-07 (undated); issue #133 | issue-level coverage (not row-pinned): rust/tests/connectivity.js | not yet confirmed |
| R188 | docs/requirements/issue-0133-duckduckgo-default-combined-ranking-and-expanded-provider-diagnostics.md | pre-2026-07 (undated); issue #133 | issue-level coverage (not row-pinned): rust/tests/connectivity.js | not yet confirmed |
| R189 | docs/requirements/issue-0133-duckduckgo-default-combined-ranking-and-expanded-provider-diagnostics.md | pre-2026-07 (undated); issue #133 | issue-level coverage (not row-pinned): rust/tests/connectivity.js | not yet confirmed |
| R190 | docs/requirements/issue-0133-duckduckgo-default-combined-ranking-and-expanded-provider-diagnostics.md | pre-2026-07 (undated); issue #133 | issue-level coverage (not row-pinned): rust/tests/connectivity.js | not yet confirmed |
| R191 | docs/requirements/issue-0133-duckduckgo-default-combined-ranking-and-expanded-provider-diagnostics.md | pre-2026-07 (undated); issue #133 | issue-level coverage (not row-pinned): rust/tests/connectivity.js | not yet confirmed |
| R192 | docs/requirements/issue-0133-duckduckgo-default-combined-ranking-and-expanded-provider-diagnostics.md | pre-2026-07 (undated); issue #133 | issue-level coverage (not row-pinned): rust/tests/connectivity.js | not yet confirmed |
| R193 | docs/requirements/issue-0133-duckduckgo-default-combined-ranking-and-expanded-provider-diagnostics.md | pre-2026-07 (undated); issue #133 | issue-level coverage (not row-pinned): rust/tests/connectivity.js | not yet confirmed |
| R194 | docs/requirements/issue-0133-duckduckgo-default-combined-ranking-and-expanded-provider-diagnostics.md | pre-2026-07 (undated); issue #133 | issue-level coverage (not row-pinned): rust/tests/connectivity.js | not yet confirmed |
| R195 | docs/requirements/issue-0159-hive-mind-lookup-and-curated-project-summarization.md | pre-2026-07 (undated); issue #159 | rust/tests/unit/specification/project_lookups.rs::russian_hive_mind_prompt_prefers_link_assistant_project | not yet confirmed |
| R196 | docs/requirements/issue-0159-hive-mind-lookup-and-curated-project-summarization.md | pre-2026-07 (undated); issue #159 | issue-level coverage (not row-pinned): rust/tests/unit/specification/project_lookups.rs:17 | not yet confirmed |
| R197 | docs/requirements/issue-0159-hive-mind-lookup-and-curated-project-summarization.md | pre-2026-07 (undated); issue #159 | rust/src/summarization/mod.rs::tests | not yet confirmed |
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
| R890-1 | docs/requirements/issue-0890-formal-proof-program-translation.md | pre-2026-07 (undated); issue #890 | issue-level coverage (not row-pinned): rust/tests/unit/issue_890.rs | not yet confirmed |
| R890-2 | docs/requirements/issue-0890-formal-proof-program-translation.md | pre-2026-07 (undated); issue #890 | issue-level coverage (not row-pinned): rust/tests/unit/issue_890.rs | not yet confirmed |
| R890-3 | docs/requirements/issue-0890-formal-proof-program-translation.md | pre-2026-07 (undated); issue #890 | issue-level coverage (not row-pinned): rust/tests/unit/issue_890.rs | not yet confirmed |
| R890-4 | docs/requirements/issue-0890-formal-proof-program-translation.md | pre-2026-07 (undated); issue #890 | issue-level coverage (not row-pinned): rust/tests/unit/issue_890.rs | not yet confirmed |
| R890-5 | docs/requirements/issue-0890-formal-proof-program-translation.md | pre-2026-07 (undated); issue #890 | rust/tests/e2e/tests/issue-890.spec.js | not yet confirmed |
| R890-6 | docs/requirements/issue-0890-formal-proof-program-translation.md | pre-2026-07 (undated); issue #890 | issue-level coverage (not row-pinned): rust/tests/unit/issue_890.rs | not yet confirmed |
| R890-7 | docs/requirements/issue-0890-formal-proof-program-translation.md | pre-2026-07 (undated); issue #890 | issue-level coverage (not row-pinned): rust/tests/unit/issue_890.rs | not yet confirmed |
| R498-1 | docs/requirements/issue-0498-google-trends-requirements.md | pre-2026-07 (undated); issue #498 | issue-level coverage (not row-pinned): rust/tests/unit/issue_498_google_trends_learning.rs | not yet confirmed |
| R498-2 | docs/requirements/issue-0498-google-trends-requirements.md | pre-2026-07 (undated); issue #498 | issue-level coverage (not row-pinned): rust/tests/unit/issue_498_google_trends_learning.rs | not yet confirmed |
| R498-3 | docs/requirements/issue-0498-google-trends-requirements.md | pre-2026-07 (undated); issue #498 | issue-level coverage (not row-pinned): rust/tests/unit/issue_498_google_trends_learning.rs | not yet confirmed |
| R498-4 | docs/requirements/issue-0498-google-trends-requirements.md | pre-2026-07 (undated); issue #498 | issue-level coverage (not row-pinned): rust/tests/unit/issue_498_google_trends_learning.rs | not yet confirmed |
| R498-5 | docs/requirements/issue-0498-google-trends-requirements.md | pre-2026-07 (undated); issue #498 | issue-level coverage (not row-pinned): rust/tests/unit/issue_498_google_trends_learning.rs | not yet confirmed |
| R498-6 | docs/requirements/issue-0498-google-trends-requirements.md | pre-2026-07 (undated); issue #498 | issue-level coverage (not row-pinned): rust/tests/unit/issue_498_google_trends_learning.rs | not yet confirmed |
| R498-7 | docs/requirements/issue-0498-google-trends-requirements.md | pre-2026-07 (undated); issue #498 | rust/tests/unit/issue_498_google_trends_catalog.rs | not yet confirmed |
| R498-8 | docs/requirements/issue-0498-google-trends-requirements.md | pre-2026-07 (undated); issue #498 | issue-level coverage (not row-pinned): rust/tests/unit/issue_498_google_trends_learning.rs | not yet confirmed |
| R498-9 | docs/requirements/issue-0498-google-trends-requirements.md | pre-2026-07 (undated); issue #498 | issue-level coverage (not row-pinned): rust/tests/unit/issue_498_google_trends_learning.rs | not yet confirmed |
| R498-10 | docs/requirements/issue-0498-google-trends-requirements.md | pre-2026-07 (undated); issue #498 | issue-level coverage (not row-pinned): rust/tests/unit/issue_498_google_trends_learning.rs | not yet confirmed |
| R527-1 | docs/requirements/issue-0527-question-generation-requirements.md | pre-2026-07 (undated); issue #527 | issue-level coverage (not row-pinned): rust/tests/unit/docs_requirements_issue_527.rs | not yet confirmed |
| R527-2 | docs/requirements/issue-0527-question-generation-requirements.md | pre-2026-07 (undated); issue #527 | issue-level coverage (not row-pinned): rust/tests/unit/docs_requirements_issue_527.rs | not yet confirmed |
| R527-3 | docs/requirements/issue-0527-question-generation-requirements.md | pre-2026-07 (undated); issue #527 | issue-level coverage (not row-pinned): rust/tests/unit/docs_requirements_issue_527.rs | not yet confirmed |
| R527-4 | docs/requirements/issue-0527-question-generation-requirements.md | pre-2026-07 (undated); issue #527 | issue-level coverage (not row-pinned): rust/tests/unit/docs_requirements_issue_527.rs | not yet confirmed |
| R527-5 | docs/requirements/issue-0527-question-generation-requirements.md | pre-2026-07 (undated); issue #527 | issue-level coverage (not row-pinned): rust/tests/unit/docs_requirements_issue_527.rs | not yet confirmed |
| R527-6 | docs/requirements/issue-0527-question-generation-requirements.md | pre-2026-07 (undated); issue #527 | issue-level coverage (not row-pinned): rust/tests/unit/docs_requirements_issue_527.rs | not yet confirmed |
| R527-7 | docs/requirements/issue-0527-question-generation-requirements.md | pre-2026-07 (undated); issue #527 | issue-level coverage (not row-pinned): rust/tests/unit/docs_requirements_issue_527.rs | not yet confirmed |
| R527-8 | docs/requirements/issue-0527-question-generation-requirements.md | pre-2026-07 (undated); issue #527 | issue-level coverage (not row-pinned): rust/tests/unit/docs_requirements_issue_527.rs | not yet confirmed |
| R527-9 | docs/requirements/issue-0527-question-generation-requirements.md | pre-2026-07 (undated); issue #527 | rust/tests/unit/issue_527_question_catalog.rs | not yet confirmed |
| R527-10 | docs/requirements/issue-0527-question-generation-requirements.md | pre-2026-07 (undated); issue #527 | issue-level coverage (not row-pinned): rust/tests/unit/docs_requirements_issue_527.rs | not yet confirmed |
| R527-11 | docs/requirements/issue-0527-question-generation-requirements.md | pre-2026-07 (undated); issue #527 | issue-level coverage (not row-pinned): rust/tests/unit/docs_requirements_issue_527.rs | not yet confirmed |
| R527-12 | docs/requirements/issue-0527-question-generation-requirements.md | pre-2026-07 (undated); issue #527 | rust/tests/unit/issue_527_question_catalog.rs | not yet confirmed |
| R216 | docs/requirements/issue-0187-current-day-calendar-prompt.md | pre-2026-07 (undated); issue #187 | none recorded | not yet confirmed |
| R217 | docs/requirements/issue-0187-current-day-calendar-prompt.md | pre-2026-07 (undated); issue #187 | none recorded | not yet confirmed |
| R218 | docs/requirements/issue-0187-current-day-calendar-prompt.md | pre-2026-07 (undated); issue #187 | rust/tests/unit/specification/reasoning_paths.rs; rust/tests/e2e/tests/multilingual.spec.js | not yet confirmed |
| R219 | docs/requirements/issue-0187-current-day-calendar-prompt.md | pre-2026-07 (undated); issue #187 | rust/tests/e2e/scripts/check-multilingual-intent-coverage.mjs | not yet confirmed |
| R220 | docs/requirements/issue-0195-docker-in-docker-telegram-runtime.md | pre-2026-07 (undated); issue #195 | rust/tests/unit/docker_runtime.rs | not yet confirmed |
| R221 | docs/requirements/issue-0195-docker-in-docker-telegram-runtime.md | pre-2026-07 (undated); issue #195 | issue-level coverage (not row-pinned): rust/tests/unit/specification/agent_isolation.rs | not yet confirmed |
| R222 | docs/requirements/issue-0195-docker-in-docker-telegram-runtime.md | pre-2026-07 (undated); issue #195 | issue-level coverage (not row-pinned): rust/tests/unit/specification/agent_isolation.rs | not yet confirmed |
| R223 | docs/requirements/issue-0195-docker-in-docker-telegram-runtime.md | pre-2026-07 (undated); issue #195 | issue-level coverage (not row-pinned): rust/tests/unit/specification/agent_isolation.rs | not yet confirmed |
| R224 | docs/requirements/issue-0195-docker-in-docker-telegram-runtime.md | pre-2026-07 (undated); issue #195 | rust/tests/unit/docs_requirements.rs | not yet confirmed |
| R225 | docs/requirements/issue-0195-docker-in-docker-telegram-runtime.md | pre-2026-07 (undated); issue #195 | issue-level coverage (not row-pinned): rust/tests/unit/specification/agent_isolation.rs | not yet confirmed |
| R195-7 | docs/requirements/issue-0195-docker-in-docker-telegram-runtime.md | PR for #1138 (issue #195 completion) | rust/tests/unit/issue_1138_execution_box.rs; rust/tests/unit/issue_1138_telegram_execution.rs | not yet confirmed |
| R226 | docs/requirements/issue-0196-permanent-memory-deletion-and-reset.md | pre-2026-07 (undated); issue #196 | none recorded | not yet confirmed |
| R227 | docs/requirements/issue-0196-permanent-memory-deletion-and-reset.md | pre-2026-07 (undated); issue #196 | none recorded | not yet confirmed |
| R228 | docs/requirements/issue-0196-permanent-memory-deletion-and-reset.md | pre-2026-07 (undated); issue #196 | none recorded | not yet confirmed |
| R229 | docs/requirements/issue-0196-permanent-memory-deletion-and-reset.md | pre-2026-07 (undated); issue #196 | none recorded | not yet confirmed |
| R230 | docs/requirements/issue-0196-permanent-memory-deletion-and-reset.md | pre-2026-07 (undated); issue #196 | none recorded | not yet confirmed |
| R231 | docs/requirements/issue-0278-native-doublets-store-default-requirements.md | pre-2026-07 (undated); issue #278 | none recorded | not yet confirmed |
| R232 | docs/requirements/issue-0278-native-doublets-store-default-requirements.md | pre-2026-07 (undated); issue #278 | none recorded | not yet confirmed |
| R233 | docs/requirements/issue-0278-native-doublets-store-default-requirements.md | pre-2026-07 (undated); issue #278 | none recorded | not yet confirmed |
| R234 | docs/requirements/issue-0278-native-doublets-store-default-requirements.md | pre-2026-07 (undated); issue #278 | none recorded | not yet confirmed |
| R235 | docs/requirements/issue-0278-native-doublets-store-default-requirements.md | pre-2026-07 (undated); issue #278 | none recorded | not yet confirmed |
| R236 | docs/requirements/issue-0278-native-doublets-store-default-requirements.md | pre-2026-07 (undated); issue #278 | none recorded | not yet confirmed |
| R237 | docs/requirements/issue-0279-symbolic-probabilistic-reasoning.md | pre-2026-07 (undated); issue #279 | none recorded | not yet confirmed |
| R238 | docs/requirements/issue-0279-symbolic-probabilistic-reasoning.md | pre-2026-07 (undated); issue #279 | none recorded | not yet confirmed |
| R239 | docs/requirements/issue-0279-symbolic-probabilistic-reasoning.md | pre-2026-07 (undated); issue #279 | none recorded | not yet confirmed |
| R240 | docs/requirements/issue-0279-symbolic-probabilistic-reasoning.md | pre-2026-07 (undated); issue #279 | none recorded | not yet confirmed |
| R241 | docs/requirements/issue-0279-symbolic-probabilistic-reasoning.md | pre-2026-07 (undated); issue #279 | none recorded | not yet confirmed |
| R242 | docs/requirements/issue-0279-symbolic-probabilistic-reasoning.md | pre-2026-07 (undated); issue #279 | none recorded | not yet confirmed |
| R243 | docs/requirements/issue-0283-generalized-natural-language-skill-compiler.md | pre-2026-07 (undated); issue #283 | issue-level coverage (not row-pinned): rust/tests/unit/specification/arbitrary_skill_compilation.rs | not yet confirmed |
| R244 | docs/requirements/issue-0283-generalized-natural-language-skill-compiler.md | pre-2026-07 (undated); issue #283 | issue-level coverage (not row-pinned): rust/tests/unit/specification/arbitrary_skill_compilation.rs | not yet confirmed |
| R245 | docs/requirements/issue-0283-generalized-natural-language-skill-compiler.md | pre-2026-07 (undated); issue #283 | issue-level coverage (not row-pinned): rust/tests/unit/specification/arbitrary_skill_compilation.rs | not yet confirmed |
| R246 | docs/requirements/issue-0327-cross-runtime-synthesis-parity.md | pre-2026-07 (undated); issue #327 | issue-level coverage (not row-pinned): rust/tests/unit/specification/synthesis.rs:39 | not yet confirmed |
| R247 | docs/requirements/issue-0327-cross-runtime-synthesis-parity.md | pre-2026-07 (undated); issue #327 | rust/tests/e2e/tests/issue-327.spec.js | not yet confirmed |
| R248 | docs/requirements/issue-0327-cross-runtime-synthesis-parity.md | pre-2026-07 (undated); issue #327 | issue-level coverage (not row-pinned): rust/tests/unit/specification/synthesis.rs:39 | not yet confirmed |
| R249 | docs/requirements/issue-0327-cross-runtime-synthesis-parity.md | pre-2026-07 (undated); issue #327 | issue-level coverage (not row-pinned): rust/tests/unit/specification/synthesis.rs:39 | not yet confirmed |
| R250 | docs/requirements/issue-0244-vision-implementation-planning.md | pre-2026-07 (undated); issue #244 | none recorded | not yet confirmed |
| R251 | docs/requirements/issue-0244-vision-implementation-planning.md | pre-2026-07 (undated); issue #244 | none recorded | not yet confirmed |
| R252 | docs/requirements/issue-0244-vision-implementation-planning.md | pre-2026-07 (undated); issue #244 | none recorded | not yet confirmed |
| R253 | docs/requirements/issue-0244-vision-implementation-planning.md | pre-2026-07 (undated); issue #244 | none recorded | not yet confirmed |
| R254 | docs/requirements/issue-0244-vision-implementation-planning.md | pre-2026-07 (undated); issue #244 | none recorded | not yet confirmed |
| R255 | docs/requirements/issue-0244-vision-implementation-planning.md | pre-2026-07 (undated); issue #244 | none recorded | not yet confirmed |
| R256 | docs/requirements/issue-0349-reverse-sort-program-modification-roadmap.md | pre-2026-07 (undated); issue #349 | rust/tests/integration/issue_349_reverse_sort.rs::issue_349_reverse_sort_follow_up_must_not_be_unknown | not yet confirmed |
| R257 | docs/requirements/issue-0349-reverse-sort-program-modification-roadmap.md | pre-2026-07 (undated); issue #349 | rust/tests/unit/specification/code_generation_coreference.rs | not yet confirmed |
| R258 | docs/requirements/issue-0349-reverse-sort-program-modification-roadmap.md | pre-2026-07 (undated); issue #349 | none recorded | not yet confirmed |
| R259 | docs/requirements/issue-0349-reverse-sort-program-modification-roadmap.md | pre-2026-07 (undated); issue #349 | rust/tests/unit/specification/code_generation_program_modifiers.rs | not yet confirmed |
| R260 | docs/requirements/issue-0349-reverse-sort-program-modification-roadmap.md | pre-2026-07 (undated); issue #349 | rust/tests/integration/issue_349_reverse_sort.rs::issue_349_diagnostic_mode_emits_full_turn_5_reasoning_chain; rust/tests/e2e/tests/issue-360.spec.js | not yet confirmed |
| R261 | docs/requirements/issue-0349-reverse-sort-program-modification-roadmap.md | pre-2026-07 (undated); issue #349 | none recorded | not yet confirmed |
| R262 | docs/requirements/issue-0349-reverse-sort-program-modification-roadmap.md | pre-2026-07 (undated); issue #349 | rust/tests/unit/specification/coding_modification_benchmarks.rs::issue_362_multilingual_multi_turn_coding_modification_ratchet | not yet confirmed |
| R263 | docs/requirements/issue-0349-reverse-sort-program-modification-roadmap.md | pre-2026-07 (undated); issue #349 | rust/tests/e2e/tests/issue-363.spec.js | not yet confirmed |
| R264 | docs/requirements/issue-0349-reverse-sort-program-modification-roadmap.md | pre-2026-07 (undated); issue #349 | rust/tests/unit/specification/self_improvement.rs | not yet confirmed |
| R265 | docs/requirements/issue-0349-reverse-sort-program-modification-roadmap.md | pre-2026-07 (undated); issue #349 | none recorded | not yet confirmed |
| R266 | docs/requirements/issue-0398-recursive-semantic-meta-language.md | pre-2026-07 (undated); issue #398 | issue-level coverage (not row-pinned): rust/tests/unit/reference_closure.rs | not yet confirmed |
| R267 | docs/requirements/issue-0398-recursive-semantic-meta-language.md | pre-2026-07 (undated); issue #398 | issue-level coverage (not row-pinned): rust/tests/unit/reference_closure.rs | not yet confirmed |
| R268 | docs/requirements/issue-0398-recursive-semantic-meta-language.md | pre-2026-07 (undated); issue #398 | issue-level coverage (not row-pinned): rust/tests/unit/reference_closure.rs | not yet confirmed |
| R269 | docs/requirements/issue-0398-recursive-semantic-meta-language.md | pre-2026-07 (undated); issue #398 | issue-level coverage (not row-pinned): rust/tests/unit/reference_closure.rs | not yet confirmed |
| R270 | docs/requirements/issue-0398-recursive-semantic-meta-language.md | pre-2026-07 (undated); issue #398 | issue-level coverage (not row-pinned): rust/tests/unit/reference_closure.rs | not yet confirmed |
| R271 | docs/requirements/issue-0398-recursive-semantic-meta-language.md | pre-2026-07 (undated); issue #398 | issue-level coverage (not row-pinned): rust/tests/unit/reference_closure.rs | not yet confirmed |
| R272 | docs/requirements/issue-0398-recursive-semantic-meta-language.md | pre-2026-07 (undated); issue #398 | issue-level coverage (not row-pinned): rust/tests/unit/reference_closure.rs | not yet confirmed |
| R273 | docs/requirements/issue-0398-recursive-semantic-meta-language.md | pre-2026-07 (undated); issue #398 | issue-level coverage (not row-pinned): rust/tests/unit/reference_closure.rs | not yet confirmed |
| R274 | docs/requirements/issue-0398-recursive-semantic-meta-language.md | pre-2026-07 (undated); issue #398 | issue-level coverage (not row-pinned): rust/tests/unit/reference_closure.rs | not yet confirmed |
| R275 | docs/requirements/issue-0398-recursive-semantic-meta-language.md | pre-2026-07 (undated); issue #398 | rust/tests/source/seed/embedded.rs | not yet confirmed |
| R276 | docs/requirements/issue-0398-recursive-semantic-meta-language.md | pre-2026-07 (undated); issue #398 | issue-level coverage (not row-pinned): rust/tests/unit/reference_closure.rs | not yet confirmed |
| R277 | docs/requirements/issue-0398-recursive-semantic-meta-language.md | pre-2026-07 (undated); issue #398 | issue-level coverage (not row-pinned): rust/tests/unit/reference_closure.rs | not yet confirmed |
| R278 | docs/requirements/issue-0398-pr-review-standards-comment-4663407299.md | PR #399 (issue #398) | rust/tests/unit/data_files.rs | not yet confirmed |
| R279 | docs/requirements/issue-0398-pr-review-standards-comment-4663407299.md | PR #399 (issue #398) | rust/tests/unit/overrides.rs | not yet confirmed |
| R280 | docs/requirements/issue-0398-pr-review-standards-comment-4663407299.md | PR #399 (issue #398) | issue-level coverage (not row-pinned): rust/tests/unit/reference_closure.rs | not yet confirmed |
| R281 | docs/requirements/issue-0398-pr-review-standards-comment-4663407299.md | PR #399 (issue #398) | issue-level coverage (not row-pinned): rust/tests/unit/reference_closure.rs | not yet confirmed |
| R282 | docs/requirements/issue-0398-pr-review-standards-comment-4663407299.md | PR #399 (issue #398) | issue-level coverage (not row-pinned): rust/tests/unit/reference_closure.rs | not yet confirmed |
| R283 | docs/requirements/issue-0398-pr-review-standards-comment-4663407299.md | PR #399 (issue #398) | issue-level coverage (not row-pinned): rust/tests/unit/reference_closure.rs | not yet confirmed |
| R284 | docs/requirements/issue-0398-pr-review-standards-comment-4668929105.md | pre-2026-07 (undated); issue #398 | rust/tests/unit/total_closure.rs | not yet confirmed |
| R285 | docs/requirements/issue-0398-pr-review-standards-comment-4668929105.md | pre-2026-07 (undated); issue #398 | issue-level coverage (not row-pinned): rust/tests/unit/reference_closure.rs | not yet confirmed |
| R286 | docs/requirements/issue-0398-pr-review-standards-comment-4668929105.md | pre-2026-07 (undated); issue #398 | issue-level coverage (not row-pinned): rust/tests/unit/reference_closure.rs | not yet confirmed |
| R287 | docs/requirements/issue-0398-pr-review-standards-comment-4668929105.md | pre-2026-07 (undated); issue #398 | issue-level coverage (not row-pinned): rust/tests/unit/reference_closure.rs | not yet confirmed |
| R288 | docs/requirements/issue-0398-pr-review-standards-comment-4668929105.md | pre-2026-07 (undated); issue #398 | issue-level coverage (not row-pinned): rust/tests/unit/reference_closure.rs | not yet confirmed |
| R289 | docs/requirements/issue-0412-pr-review-standards-comment-4674-knowledge-source-breadth.md | pre-2026-07 (undated); issue #412 | rust/tests/integration/issue_412_oracle_languages.rs; source_tests/solver_handler_oracle | not yet confirmed |
| R290 | docs/requirements/issue-0412-pr-review-standards-comment-4674-knowledge-source-breadth.md | pre-2026-07 (undated); issue #412 | issue-level coverage (not row-pinned): rust/tests/integration/issue_412_oracle_languages.rs | not yet confirmed |
| R291 | docs/requirements/issue-0412-pr-review-standards-comment-4674-knowledge-source-breadth.md | pre-2026-07 (undated); issue #412 | issue-level coverage (not row-pinned): rust/tests/integration/issue_412_oracle_languages.rs | not yet confirmed |
| R292 | docs/requirements/issue-0412-pr-review-standards-comment-4674-knowledge-source-breadth.md | pre-2026-07 (undated); issue #412 | issue-level coverage (not row-pinned): rust/tests/integration/issue_412_oracle_languages.rs | not yet confirmed |
| R293 | docs/requirements/issue-0408-text-and-code-editing-requirements.md | PR #416 (issue #408) | rust/tests/unit/specification/text_manipulation.rs | not yet confirmed |
| R294 | docs/requirements/issue-0408-text-and-code-editing-requirements.md | PR #416 (issue #408) | none recorded | not yet confirmed |
| R295 | docs/requirements/issue-0408-text-and-code-editing-requirements.md | PR #416 (issue #408) | rust/tests/unit/specification/text_manipulation_benchmarks.rs::issue_408_text_code_edit_profile_passes_local_ratchet | not yet confirmed |
| R296 | docs/requirements/issue-0408-text-and-code-editing-requirements.md | PR #416 (issue #408) | rust/tests/unit/docs_requirements.rs::issue_408_text_edit_benchmark_scope_documents_are_traceable | not yet confirmed |
| R297 | docs/requirements/issue-0408-text-and-code-editing-requirements.md | PR #416 (issue #408) | none recorded | not yet confirmed |
| R298 | docs/requirements/issue-0451-symbolic-ai-reference-and-best-practices.md | PR #452 (issue #451) | issue-level coverage (not row-pinned): rust/tests/unit/docs_requirements_issue_451.rs | not yet confirmed |
| R299 | docs/requirements/issue-0451-symbolic-ai-reference-and-best-practices.md | PR #452 (issue #451) | issue-level coverage (not row-pinned): rust/tests/unit/docs_requirements_issue_451.rs | not yet confirmed |
| R300 | docs/requirements/issue-0451-symbolic-ai-reference-and-best-practices.md | PR #452 (issue #451) | issue-level coverage (not row-pinned): rust/tests/unit/docs_requirements_issue_451.rs | not yet confirmed |
| R301 | docs/requirements/issue-0451-symbolic-ai-reference-and-best-practices.md | PR #452 (issue #451) | issue-level coverage (not row-pinned): rust/tests/unit/docs_requirements_issue_451.rs | not yet confirmed |
| R302 | docs/requirements/issue-0451-symbolic-ai-reference-and-best-practices.md | PR #452 (issue #451) | issue-level coverage (not row-pinned): rust/tests/unit/docs_requirements_issue_451.rs | not yet confirmed |
| R303 | docs/requirements/issue-0451-symbolic-ai-reference-and-best-practices.md | PR #452 (issue #451) | issue-level coverage (not row-pinned): rust/tests/unit/docs_requirements_issue_451.rs | not yet confirmed |
| R304 | docs/requirements/issue-0451-symbolic-ai-reference-and-best-practices.md | PR #452 (issue #451) | issue-level coverage (not row-pinned): rust/tests/unit/docs_requirements_issue_451.rs | not yet confirmed |
| R305 | docs/requirements/issue-0451-symbolic-ai-reference-and-best-practices.md | PR #452 (issue #451) | rust/tests/source/source_tests/proof_engine/decision/{sat,boolean}/tests.rs | not yet confirmed |
| R306 | docs/requirements/issue-0468-agentic-coding-mode.md | PR #469 (issue #468) | issue-level coverage (not row-pinned): rust/tests/integration/issue_716_agentic_execution.rs | not yet confirmed |
| R307 | docs/requirements/issue-0468-agentic-coding-mode.md | PR #469 (issue #468) | rust/tests/unit/agentic_coding.rs | not yet confirmed |
| R308 | docs/requirements/issue-0468-agentic-coding-mode.md | PR #469 (issue #468) | issue-level coverage (not row-pinned): rust/tests/integration/issue_716_agentic_execution.rs | not yet confirmed |
| R309 | docs/requirements/issue-0468-agentic-coding-mode.md | PR #469 (issue #468) | issue-level coverage (not row-pinned): rust/tests/integration/issue_716_agentic_execution.rs | not yet confirmed |
| R310 | docs/requirements/issue-0468-agentic-coding-mode.md | PR #469 (issue #468) | rust/tests/unit/agentic_coding.rs | not yet confirmed |
| R311 | docs/requirements/issue-0468-agentic-coding-mode.md | PR #469 (issue #468) | issue-level coverage (not row-pinned): rust/tests/integration/issue_716_agentic_execution.rs | not yet confirmed |
| R312 | docs/requirements/issue-0468-agentic-coding-mode.md | PR #469 (issue #468) | rust/tests/unit/agentic_coding.rs | not yet confirmed |
| R313 | docs/requirements/issue-0468-agentic-coding-mode.md | PR #469 (issue #468) | issue-level coverage (not row-pinned): rust/tests/integration/issue_716_agentic_execution.rs | not yet confirmed |
| R314 | docs/requirements/issue-0468-agentic-coding-mode.md | PR #469 (issue #468); custom-task fallback repaired by #1138 B4 | rust/tests/unit/agentic_coding.rs::a_custom_task_is_formalized_instead_of_the_seeded_fairy_tale; rust/tests/unit/agentic_surfaces.rs | manually confirmed 2026-08-04 (audit): `formal-ai agent --help` run; offline `agent --silent --task ...` exit 0 — the 2026-08-04 finding "falls back to seeded fairy-tale KB rather than reflecting custom --task" is now pinned as a regression |
| R315 | docs/requirements/issue-0468-agentic-coding-mode.md | PR #469 (issue #468) | issue-level coverage (not row-pinned): rust/tests/integration/issue_716_agentic_execution.rs | not yet confirmed |
| R316 | docs/requirements/issue-0468-agentic-coding-mode.md | PR #469 (issue #468) | issue-level coverage (not row-pinned): rust/tests/integration/issue_716_agentic_execution.rs | not yet confirmed |
| R317 | docs/requirements/issue-0468-agentic-coding-mode.md | PR #469 (issue #468) | issue-level coverage (not row-pinned): rust/tests/integration/issue_716_agentic_execution.rs | not yet confirmed |
| R318 | docs/requirements/issue-0468-agentic-coding-mode.md | PR #469 (issue #468) | issue-level coverage (not row-pinned): rust/tests/integration/issue_716_agentic_execution.rs | not yet confirmed |
| R319 | docs/requirements/issue-0468-agentic-coding-mode.md | PR #469 (issue #468) | rust/tests/unit/agentic_coding.rs; rust/tests/unit/agentic_surfaces.rs | not yet confirmed |
| R320 | docs/requirements/issue-0438-prepared-telegram-docker-image.md | PR #470 (issue #438) | none recorded | not yet confirmed |
| R321 | docs/requirements/issue-0438-prepared-telegram-docker-image.md | PR #470 (issue #438) | none recorded | not yet confirmed |
| R322 | docs/requirements/issue-0438-prepared-telegram-docker-image.md | PR #470 (issue #438) | none recorded | not yet confirmed |
| R323 | docs/requirements/issue-0438-prepared-telegram-docker-image.md | PR #470 (issue #438) | none recorded | not yet confirmed |
| R324 | docs/requirements/issue-0438-prepared-telegram-docker-image.md | PR #470 (issue #438) | none recorded | not yet confirmed |
| R325 | docs/requirements/issue-0438-prepared-telegram-docker-image.md | PR #470 (issue #438) | none recorded | not yet confirmed |
| R326 | docs/requirements/issue-0438-prepared-telegram-docker-image.md | PR #470 (issue #438) | rust/tests/unit/docker_runtime.rs::compose_file_runs_prebuilt_telegram_image_with_minimum_configuration; rust/tests/unit/ci-cd/release_publishing.rs::release_workflow_publishes_prebuilt_ghcr_image_after_crate_is_visible_and_optional_docker_hub_mirror | not yet confirmed |
| R327 | docs/requirements/issue-0438-prepared-telegram-docker-image.md | PR #470 (issue #438) | none recorded | not yet confirmed |
| R328 | docs/requirements/issue-0438-prepared-telegram-docker-image.md | PR #470 (issue #438) | none recorded | not yet confirmed |
| R329 | docs/requirements/issue-0438-prepared-telegram-docker-image.md | PR #470 (issue #438) | rust/tests/unit/specification/desktop_surface.rs::{desktop_service_control_starts_and_stops_prepared_containers,desktop_web_surface_exposes_one_click_service_controls}; rust/tests/unit/docker_runtime.rs::compose_file_offers_optional_openai_compatible_server_profile | not yet confirmed |
| R330 | docs/requirements/issue-0559-general-meta-algorithm.md | PR #560 (issue #559) | rust/tests/unit/specification/meta_frame.rs; rust/tests/unit/docs_requirements_issue_559.rs::issue_559_problem_frame_is_traceable | not yet confirmed |
| R331 | docs/requirements/issue-0559-general-meta-algorithm.md | PR #560 (issue #559) | rust/tests/unit/specification/method_registry.rs; rust/tests/unit/specification/reasoning_paths.rs::selected_specialized_handler_is_recorded_as_a_meta_method | not yet confirmed |
| R332 | docs/requirements/issue-0559-general-meta-algorithm.md | PR #560 (issue #559) | rust/tests/unit/specification/meta_frame.rs; rust/tests/unit/specification/reasoning_loop.rs::handler_families_publish_loop_events_as_recursion_leaves | not yet confirmed |
| R333 | docs/requirements/issue-0559-general-meta-algorithm.md | PR #560 (issue #559) | rust/tests/unit/specification/meta_frame.rs; rust/tests/unit/docs_requirements_issue_559.rs::issue_559_need_ledger_is_traceable | not yet confirmed |
| R334 | docs/requirements/issue-0559-general-meta-algorithm.md | PR #560 (issue #559) | rust/tests/unit/specification/solution_evidence.rs; rust/tests/unit/docs_requirements_issue_559.rs::issue_559_solution_evidence_is_traceable | not yet confirmed |
| R335 | docs/requirements/issue-0559-general-meta-algorithm.md | PR #560 (issue #559) | rust/tests/unit/specification/recursive_core_recipe.rs; rust/tests/unit/docs_requirements_issue_559.rs::issue_559_recursive_core_recipe_is_traceable | not yet confirmed |
| R336 | docs/requirements/issue-0559-general-meta-algorithm.md | PR #560 (issue #559) | rust/tests/unit/specification/route_method_alias.rs; rust/tests/unit/docs_requirements_issue_559.rs::issue_559_route_method_alias_is_traceable | not yet confirmed |
| R337 | docs/requirements/issue-0559-general-meta-algorithm.md | PR #560 (issue #559) | rust/tests/unit/specification/meta_reasoning.rs; rust/tests/unit/docs_requirements_issue_559.rs::issue_559_work_unit_reasoning_is_traceable | not yet confirmed |
| R338 | docs/requirements/issue-0559-general-meta-algorithm.md | PR #560 (issue #559) | rust/tests/unit/specification/meta_construction.rs; rust/tests/unit/docs_requirements_issue_559.rs::issue_559_upward_construction_is_traceable | not yet confirmed |
| R339 | docs/requirements/issue-0559-general-meta-algorithm.md | PR #560 (issue #559) | rust/tests/unit/specification/selection.rs; rust/tests/unit/docs_requirements_issue_559.rs::issue_559_selection_trace_is_traceable | not yet confirmed |
| R340 | docs/requirements/issue-0559-general-meta-algorithm.md | PR #560 (issue #559) | rust/tests/unit/specification/meta_self_improvement.rs; rust/tests/unit/docs_requirements_issue_559.rs::issue_559_meta_self_improvement_is_traceable | not yet confirmed |
| R341 | docs/requirements/issue-0559-general-meta-algorithm.md | PR #560 (issue #559) | rust/tests/unit/specification/cue_lexicon.rs; rust/tests/unit/docs_requirements_issue_559.rs::issue_559_cue_lexicon_is_traceable | not yet confirmed |
| R342 | docs/requirements/issue-0559-general-meta-algorithm.md | PR #560 (issue #559) | rust/tests/unit/specification/skill_ledger.rs; rust/tests/unit/docs_requirements_issue_559.rs::issue_559_skill_ledger_is_traceable | not yet confirmed |
| R343 | docs/requirements/issue-0559-general-meta-algorithm.md | PR #560 (issue #559) | rust/tests/unit/specification/recipe_interpreter.rs; rust/tests/unit/docs_requirements_issue_559.rs::issue_559_recipe_interpreter_is_traceable | not yet confirmed |
| R344 | docs/requirements/issue-0559-general-meta-algorithm.md | PR #560 (issue #559) | rust/tests/unit/issue_699_handler_migration.rs; rust/tests/unit/specification/obligation_ledger.rs | not yet confirmed |
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
| R1138-6-1 | docs/requirements/issue-1138-prerequisite-discovery.md | PR for #1138 | rust/tests/unit/issue_1138_toolchain_probe.rs; rust/tests/unit/issue_1138_prerequisite_need.rs | not yet confirmed |
| R1138-6-2 | docs/requirements/issue-1138-prerequisite-discovery.md | PR for #1138 | rust/tests/unit/issue_1138_surface_honesty.rs | not yet confirmed |
| R1138-6-3 | docs/requirements/issue-1138-prerequisite-discovery.md | PR for #1138 | rust/tests/unit/issue_1138_prerequisite_need.rs | not yet confirmed |
| R1138-6-4 | docs/requirements/issue-1138-prerequisite-discovery.md | PR for #1138 | rust/tests/unit/issue_1138_prerequisite_need.rs | not yet confirmed |
| R1138-6-5 | docs/requirements/issue-1138-prerequisite-discovery.md | PR for #1138 | rust/tests/unit/issue_1138_setup_publisher.rs | not yet confirmed |
| R1138-6-6 | docs/requirements/issue-1138-prerequisite-discovery.md | PR for #1138 | rust/tests/unit/issue_1138_install_scope.rs | not yet confirmed |
| R1138-6-7 | docs/requirements/issue-1138-prerequisite-discovery.md | PR for #1138 | rust/tests/unit/issue_1138_setup_publisher.rs; rust/tests/unit/issue_1138_install_scope.rs | not yet confirmed |
| R1138-6-8 | docs/requirements/issue-1138-prerequisite-discovery.md | PR for #1138 | rust/tests/unit/specification/prerequisite_recipe.rs; rust/tests/unit/issue_1138_prerequisite_need.rs | not yet confirmed |
| R1138-6-9 | docs/requirements/issue-1138-prerequisite-discovery.md | PR for #1138 | rust/tests/unit/issue_1138_execution_box.rs; rust/tests/unit/issue_1138_install_scope.rs | not yet confirmed |
| R1138-6-10 | docs/requirements/issue-1138-prerequisite-discovery.md | PR for #1138 | rust/tests/unit/issue_1138_execution_box.rs; rust/tests/unit/issue_1138_named_tests.rs | not yet confirmed |
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
| R1138-B4-8 | docs/requirements/issue-1138-formalization-depth.md | PR for #1138 | rust/tests/unit/agentic_coding.rs::a_custom_task_is_formalized_instead_of_the_seeded_fairy_tale; rust/tests/unit/agentic_coding.rs::the_canonical_tale_still_formalizes_to_nine_primitives | not yet confirmed |
| R1138-B4-9 | docs/requirements/issue-1138-formalization-depth.md | PR for #1138 | rust/tests/unit/specification/meta_frame.rs::formalization_needs_join_the_universal_ledger_without_parallel_statuses | not yet confirmed |
| R1138-B4-10 | docs/requirements/issue-1138-formalization-depth.md | PR for #1138 | rust/tests/unit/issue_1138_formalization_depth.rs::an_offline_run_replays_the_committed_captures_and_reproduces_the_graph_identity | not yet confirmed |
| R345 | docs/requirements/issue-0563-repository-resource-summarization.md | PR #564 (issue #563) | rust/tests/unit/specification/summarization_pipeline.rs::repository_file_summary_recurses_into_markdown_embedded_grammars | not yet confirmed |
| R346 | docs/requirements/issue-0563-repository-resource-summarization.md | PR #564 (issue #563) | none recorded | not yet confirmed |
| R347 | docs/requirements/issue-0563-repository-resource-summarization.md | PR #564 (issue #563) | none recorded | not yet confirmed |
| R348 | docs/requirements/issue-0563-repository-resource-summarization.md | PR #564 (issue #563) | none recorded | not yet confirmed |
| R349 | docs/requirements/issue-0563-repository-resource-summarization.md | PR #564 (issue #563) | none recorded | not yet confirmed |
| R350 | docs/requirements/issue-0563-repository-resource-summarization.md | PR #564 (issue #563) | none recorded | not yet confirmed |
| R351 | docs/requirements/issue-0563-repository-resource-summarization.md | PR #564 (issue #563) | rust/tests/source/source_tests/summarization/mod/tests.rs::formalize_repository_file_rust_records_meta_language_and_symbols; rust/tests/unit/docs_requirements_issue_563.rs::issue_563_repository_file_summarization_documents_are_traceable | not yet confirmed |
| R352 | docs/requirements/issue-0563-repository-resource-summarization.md | PR #564 (issue #563) | none recorded | not yet confirmed |
| R353 | docs/requirements/issue-0563-repository-resource-summarization.md | PR #564 (issue #563) | none recorded | not yet confirmed |
| R354 | docs/requirements/issue-0563-repository-resource-summarization.md | PR #564 (issue #563) | none recorded | not yet confirmed |
| R355 | docs/requirements/issue-0563-repository-resource-summarization.md | PR #564 (issue #563) | rust/tests/unit/specification/summarization_pipeline.rs::summarize_repository_resource_subsumes_file_summarization | not yet confirmed |
| R356 | docs/requirements/issue-0563-repository-resource-summarization.md | PR #564 (issue #563) | none recorded | not yet confirmed |
| R357 | docs/requirements/issue-0563-repository-resource-summarization.md | PR #564 (issue #563) | none recorded | not yet confirmed |
| R358 | docs/requirements/issue-0563-repository-resource-summarization.md | PR #564 (issue #563) | none recorded | not yet confirmed |
| R359 | docs/requirements/issue-0563-repository-resource-summarization.md | PR #564 (issue #563) | rust/tests/source/source_tests/summarization/mod/tests.rs::{summarize_repository_resource_topic_directory_is_identity_only,summarize_repository_resource_full_directory_recurses_into_nested_folder} | not yet confirmed |
| R360 | docs/requirements/issue-0492-release-badge-stability.md | PR #583 (issue #492) | none recorded | not yet confirmed |
| R361 | docs/requirements/issue-0492-release-badge-stability.md | PR #583 (issue #492) | none recorded | not yet confirmed |
| R362 | docs/requirements/issue-0492-release-badge-stability.md | PR #583 (issue #492) | none recorded | not yet confirmed |
| R363 | docs/requirements/issue-0492-release-badge-stability.md | PR #583 (issue #492) | none recorded | not yet confirmed |
| R364 | docs/requirements/issue-0492-release-badge-stability.md | PR #583 (issue #492) | none recorded | not yet confirmed |
| R365 | docs/requirements/issue-0492-release-badge-stability.md | PR #583 (issue #492) | none recorded | not yet confirmed |
| R366 | docs/requirements/issue-0492-release-badge-stability.md | PR #583 (issue #492) | none recorded | not yet confirmed |
| R367 | docs/requirements/issue-0492-release-badge-stability.md | PR #583 (issue #492) | none recorded | not yet confirmed |
| R368 | docs/requirements/issue-0492-release-badge-stability.md | PR #583 (issue #492) | none recorded | not yet confirmed |
| R369 | docs/requirements/issue-0492-release-badge-stability.md | PR #583 (issue #492) | none recorded | not yet confirmed |
| R499-1 | docs/requirements/issue-0499-learn-from-this-data-source-requirements.md | PR #641 (issue #499) | issue-level coverage (not row-pinned): rust/tests/unit/issue_499_learn_from_source.rs | not yet confirmed |
| R499-2 | docs/requirements/issue-0499-learn-from-this-data-source-requirements.md | PR #641 (issue #499) | rust/tests/unit/issue_499_learn_from_source.rs | not yet confirmed |
| R499-3 | docs/requirements/issue-0499-learn-from-this-data-source-requirements.md | PR #641 (issue #499) | issue-level coverage (not row-pinned): rust/tests/unit/issue_499_learn_from_source.rs | not yet confirmed |
| R499-4 | docs/requirements/issue-0499-learn-from-this-data-source-requirements.md | PR #641 (issue #499) | issue-level coverage (not row-pinned): rust/tests/unit/issue_499_learn_from_source.rs | not yet confirmed |
| R499-5 | docs/requirements/issue-0499-learn-from-this-data-source-requirements.md | PR #641 (issue #499) | issue-level coverage (not row-pinned): rust/tests/unit/issue_499_learn_from_source.rs | not yet confirmed |
| R499-6 | docs/requirements/issue-0499-learn-from-this-data-source-requirements.md | PR #641 (issue #499) | issue-level coverage (not row-pinned): rust/tests/unit/issue_499_learn_from_source.rs | not yet confirmed |
| R499-7 | docs/requirements/issue-0499-learn-from-this-data-source-requirements.md | PR #641 (issue #499) | issue-level coverage (not row-pinned): rust/tests/unit/issue_499_learn_from_source.rs | not yet confirmed |
| R499-8 | docs/requirements/issue-0499-learn-from-this-data-source-requirements.md | PR #641 (issue #499) | rust/tests/unit/issue_499_learn_from_source.rs; rust/tests/unit/docs_requirements_issue_499.rs | not yet confirmed |
| R370 | docs/requirements/issue-0538-detailed-meanings-and-words.md | PR #601 (issue #538) | rust/tests/unit/issue_538.rs::tomato_surfaces_pin_their_grammatical_number | not yet confirmed |
| R371 | docs/requirements/issue-0538-detailed-meanings-and-words.md | PR #601 (issue #538) | rust/tests/unit/issue_538.rs::tomato_surfaces_expose_part_of_speech_from_data | not yet confirmed |
| R372 | docs/requirements/issue-0538-detailed-meanings-and-words.md | PR #601 (issue #538) | rust/tests/unit/issue_538.rs::every_tomato_surface_denotes_the_tomato_meaning | not yet confirmed |
| R373 | docs/requirements/issue-0538-detailed-meanings-and-words.md | PR #601 (issue #538) | rust/tests/unit/issue_538.rs::tomato_singular_and_plural_are_distinct_forms_in_each_language | not yet confirmed |
| R374 | docs/requirements/issue-0538-detailed-meanings-and-words.md | PR #601 (issue #538) | rust/tests/unit/semantic_grounding.rs | not yet confirmed |
| R375 | docs/requirements/issue-0538-detailed-meanings-and-words.md | PR #601 (issue #538) | none recorded | not yet confirmed |
| R376 | docs/requirements/issue-0538-detailed-meanings-and-words.md | PR #601 (issue #538) | rust/tests/unit/issue_538.rs::grammatical_number_meanings_are_grounded_and_multilingual | not yet confirmed |
| R377 | docs/requirements/issue-0538-detailed-meanings-and-words.md | PR #601 (issue #538) | none recorded | not yet confirmed |
| R378 | docs/requirements/issue-0538-detailed-meanings-and-words.md | PR #601 (issue #538) | none recorded | not yet confirmed |
| R379 | docs/requirements/issue-0538-detailed-meanings-and-words.md | PR #601 (issue #538) | none recorded | not yet confirmed |
| R380 | docs/requirements/issue-0538-detailed-meanings-and-words.md | PR #601 (issue #538) | none recorded | not yet confirmed |
| R381 | docs/requirements/issue-0538-detailed-meanings-and-words.md | PR #601 (issue #538) | none recorded | not yet confirmed |
| R382 | docs/requirements/issue-0538-detailed-meanings-and-words.md | PR #601 (issue #538) | none recorded | not yet confirmed |
| R383 | docs/requirements/issue-0538-detailed-meanings-and-words.md | not delivered; the owning shard names it only as a follow-up (solution-plan R17 of issue #538, notes in `docs/vscode/extension.md`) and no open issue tracks it | none recorded | not yet confirmed |
| R384 | docs/requirements/issue-0538-detailed-meanings-and-words.md | PR #601 (issue #538) | none recorded | not yet confirmed |
| R385 | docs/requirements/issue-0538-detailed-meanings-and-words.md | PR #601 (issue #538) | rust/tests/unit/issue_538_agentic.rs | not yet confirmed |
| R386 | docs/requirements/issue-0538-detailed-meanings-and-words.md | PR #601 (issue #538) | none recorded | not yet confirmed |
| R387 | docs/requirements/issue-0558-auto-learning.md | PR #637 (issue #558) | none recorded | not yet confirmed |
| R388 | docs/requirements/issue-0558-auto-learning.md | PR #637 (issue #558) | none recorded | not yet confirmed |
| R389 | docs/requirements/issue-0558-auto-learning.md | PR #637 (issue #558) | none recorded | not yet confirmed |
| R390 | docs/requirements/issue-0558-auto-learning.md | PR #637 (issue #558) | none recorded | not yet confirmed |
| R391 | docs/requirements/issue-0558-auto-learning.md | PR #637 (issue #558) | none recorded | not yet confirmed |
| R392 | docs/requirements/issue-0558-auto-learning.md | PR #637 (issue #558) | rust/tests/unit/docs_requirements_issue_558.rs; rust/tests/unit/mod.rs | not yet confirmed |
| R393 | docs/requirements/issue-0558-auto-learning.md | PR #637 (issue #558) | rust/tests/unit/issue_558_self_healing.rs | not yet confirmed |
| R394 | docs/requirements/issue-0558-auto-learning.md | PR #637 (issue #558) | rust/tests/unit/issue_558_self_healing.rs | not yet confirmed |
| R395 | docs/requirements/issue-0558-auto-learning.md | PR #637 (issue #558) | rust/tests/unit/issue_558_self_healing.rs; rust/tests/integration/issue_558_self_healing.rs | not yet confirmed |
| R396 | docs/requirements/issue-0531-pattern-inference-research.md | PR #642 (issue #531) | none recorded | not yet confirmed |
| R397 | docs/requirements/issue-0531-pattern-inference-research.md | PR #642 (issue #531) | none recorded | not yet confirmed |
| R398 | docs/requirements/issue-0531-pattern-inference-research.md | PR #642 (issue #531) | none recorded | not yet confirmed |
| R399 | docs/requirements/issue-0531-pattern-inference-research.md | PR #642 (issue #531) | none recorded | not yet confirmed |
| R400 | docs/requirements/issue-0531-pattern-inference-research.md | PR #642 (issue #531) | none recorded | not yet confirmed |
| R401 | docs/requirements/issue-0531-pattern-inference-research.md | PR #642 (issue #531) | none recorded | not yet confirmed |
| R402 | docs/requirements/issue-0531-pattern-inference-research.md | PR #642 (issue #531) | rust/tests/unit/sequences_{store,symbols,converter,compression}.rs | not yet confirmed |
| R403 | docs/requirements/issue-0531-pattern-inference-research.md | PR #642 (issue #531) | rust/tests/unit/sequences_{patterns_1d,grid_2d,inference}.rs | not yet confirmed |
| R404 | docs/requirements/issue-0531-pattern-inference-research.md | PR #642 (issue #531) | rust/tests/unit/issue_531_concepts_probe.rs | not yet confirmed |
| R405 | docs/requirements/issue-0531-pattern-inference-research.md | PR #642 (issue #531) | rust/tests/unit/issue_531_pattern_inference.rs | not yet confirmed |
| R406 | docs/requirements/issue-0531-pattern-inference-research.md | PR #642 (issue #531) | none recorded | not yet confirmed |
| R407 | docs/requirements/issue-0531-pattern-inference-research.md | PR #642 (issue #531) | rust/tests/unit/docs_requirements_issue_531.rs; rust/tests/unit/mod.rs | manually confirmed 2026-08-04 (audit): `npm --prefix desktop run smoke` passed (desktop/scripts/smoke.mjs) |
| R531-17 | docs/requirements/issue-0531-pattern-inference-research.md | PR #642 (issue #531) | none recorded | not yet confirmed |
| R531-18 | docs/requirements/issue-0531-pattern-inference-research.md | PR #642 (issue #531) | rust/tests/unit/issue_531_algorithm_discovery.rs | not yet confirmed |
| R531-19 | docs/requirements/issue-0531-pattern-inference-research.md | PR #642 (issue #531) | none recorded | not yet confirmed |
| R531-20 | docs/requirements/issue-0531-pattern-inference-research.md | PR #642 (issue #531) | none recorded | not yet confirmed |
| R531-21 | docs/requirements/issue-0531-pattern-inference-research.md | PR #642 (issue #531) | none recorded | not yet confirmed |
| R531-22 | docs/requirements/issue-0531-pattern-inference-research.md | PR #642 (issue #531) | none recorded | not yet confirmed |
| R531-23 | docs/requirements/issue-0531-pattern-inference-research.md | PR #642 (issue #531) | none recorded | not yet confirmed |
| R531-24 | docs/requirements/issue-0531-pattern-inference-research.md | PR #642 (issue #531) | none recorded | not yet confirmed |
| R531-25 | docs/requirements/issue-0531-pattern-inference-research.md | PR #642 (issue #531) | none recorded | not yet confirmed |
| R537 | docs/requirements/issue-0540-dreaming-memory-maintenance.md | PR #645 (issue #540) | rust/tests/unit/docs_requirements_issue_540.rs | not yet confirmed |
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
| R548 | docs/requirements/issue-0540-dreaming-memory-maintenance.md | PR #645 (issue #540) | rust/tests/unit/memory_maintenance.rs; rust/tests/unit/docs_requirements_issue_540.rs | manually confirmed 2026-08-04 (audit): `npm --prefix desktop run smoke` passed (desktop/scripts/smoke.mjs) |
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
| R422 | docs/requirements/issue-0540-dreaming-memory-maintenance.md | PR #645 (issue #540) | rust/tests/unit/issue_540_agent_cli.rs | not yet confirmed |
| R423 | docs/requirements/issue-0540-dreaming-memory-maintenance.md | PR #645 (issue #540) | rust/tests/unit/memory_learning.rs | not yet confirmed |
| R424 | docs/requirements/issue-0540-dreaming-memory-maintenance.md | PR #645 (issue #540) | none recorded | not yet confirmed |
| R425 | docs/requirements/issue-0701-auto-learning-adoption-gap.md | PR #645 (issue #540) | none recorded | not yet confirmed |
| R426 | docs/requirements/issue-0540-dreaming-memory-maintenance.md | PR #645 (issue #540) | none recorded | not yet confirmed |
| R427 | docs/requirements/issue-0540-dreaming-memory-maintenance.md | PR #645 (issue #540) | rust/tests/unit/dreaming_runtime.rs | not yet confirmed |
| R428 | docs/requirements/issue-0649-world-models-and-contexts.md | PR #675 (issue #649) | rust/tests/unit/docs_requirements_issue_649.rs | not yet confirmed |
| R429 | docs/requirements/issue-0649-world-models-and-contexts.md | PR #675 (issue #649) | issue-level coverage (not row-pinned): rust/tests/unit/issue_702_world_model_dialog.rs:152 | not yet confirmed |
| R430 | docs/requirements/issue-0649-world-models-and-contexts.md | PR #675 (issue #649) | issue-level coverage (not row-pinned): rust/tests/unit/issue_702_world_model_dialog.rs:152 | not yet confirmed |
| R431 | docs/requirements/issue-0649-world-models-and-contexts.md | PR #675 (issue #649) | issue-level coverage (not row-pinned): rust/tests/unit/issue_702_world_model_dialog.rs:152 | not yet confirmed |
| R432 | docs/requirements/issue-0649-world-models-and-contexts.md | PR #675 (issue #649) | issue-level coverage (not row-pinned): rust/tests/unit/issue_702_world_model_dialog.rs:152 | not yet confirmed |
| R433 | docs/requirements/issue-0649-world-models-and-contexts.md | PR #675 (issue #649) | issue-level coverage (not row-pinned): rust/tests/unit/issue_702_world_model_dialog.rs:152 | not yet confirmed |
| R434 | docs/requirements/issue-0649-world-models-and-contexts.md | PR #675 (issue #649) | rust/tests/unit/docs_requirements_issue_649.rs; rust/tests/unit/mod.rs | not yet confirmed |
| R435 | docs/requirements/issue-0482-nemotron-training-data-samples.md | PR #639 (issue #482) | issue-level coverage (not row-pinned): rust/tests/unit/specification/nemotron_training_samples.rs | not yet confirmed |
| R436 | docs/requirements/issue-0482-nemotron-training-data-samples.md | PR #639 (issue #482) | issue-level coverage (not row-pinned): rust/tests/unit/specification/nemotron_training_samples.rs | not yet confirmed |
| R437 | docs/requirements/issue-0482-nemotron-training-data-samples.md | PR #639 (issue #482) | issue-level coverage (not row-pinned): rust/tests/unit/specification/nemotron_training_samples.rs | not yet confirmed |
| R438 | docs/requirements/issue-0482-nemotron-training-data-samples.md | PR #639 (issue #482) | issue-level coverage (not row-pinned): rust/tests/unit/specification/nemotron_training_samples.rs | not yet confirmed |
| R439 | docs/requirements/issue-0482-nemotron-training-data-samples.md | PR #639 (issue #482) | issue-level coverage (not row-pinned): rust/tests/unit/specification/nemotron_training_samples.rs | not yet confirmed |
| R440 | docs/requirements/issue-0482-nemotron-training-data-samples.md | PR #639 (issue #482) | rust/tests/unit/specification/nemotron_training_samples.rs | not yet confirmed |
| R441 | docs/requirements/issue-0482-nemotron-training-data-samples.md | PR #639 (issue #482) | issue-level coverage (not row-pinned): rust/tests/unit/specification/nemotron_training_samples.rs | not yet confirmed |
| R442 | docs/requirements/issue-0482-nemotron-training-data-samples.md | PR #639 (issue #482) | issue-level coverage (not row-pinned): rust/tests/unit/specification/nemotron_training_samples.rs | not yet confirmed |
| R443 | docs/requirements/issue-0482-nemotron-training-data-samples.md | PR #639 (issue #482) | issue-level coverage (not row-pinned): rust/tests/unit/specification/nemotron_training_samples.rs | not yet confirmed |
| R444 | docs/requirements/issue-0482-nemotron-training-data-samples.md | PR #639 (issue #482) | rust/tests/unit/docs_requirements_issue_482.rs; rust/tests/unit/mod.rs | not yet confirmed |
| R445 | docs/requirements/issue-0686-associative-knowledge-networks-learning.md | PR #689 (issue #686) | rust/tests/unit/docs_requirements_issue_686.rs | not yet confirmed |
| R446 | docs/requirements/issue-0686-associative-knowledge-networks-learning.md | PR #689 (issue #686) | issue-level coverage (not row-pinned): rust/tests/unit/issue_686_associative_persistence.rs | not yet confirmed |
| R447 | docs/requirements/issue-0686-associative-knowledge-networks-learning.md | PR #689 (issue #686) | issue-level coverage (not row-pinned): rust/tests/unit/issue_686_associative_persistence.rs | not yet confirmed |
| R448 | docs/requirements/issue-0686-associative-knowledge-networks-learning.md | PR #689 (issue #686) | issue-level coverage (not row-pinned): rust/tests/unit/issue_686_associative_persistence.rs | not yet confirmed |
| R449 | docs/requirements/issue-0686-associative-knowledge-networks-learning.md | PR #689 (issue #686) | issue-level coverage (not row-pinned): rust/tests/unit/issue_686_associative_persistence.rs | not yet confirmed |
| R450 | docs/requirements/issue-0686-associative-knowledge-networks-learning.md | PR #689 (issue #686) | rust/tests/unit/issue_686_associative_persistence.rs | not yet confirmed |
| R451 | docs/requirements/issue-0686-associative-knowledge-networks-learning.md | PR #689 (issue #686) | issue-level coverage (not row-pinned): rust/tests/unit/issue_686_associative_persistence.rs | not yet confirmed |
| R452 | docs/requirements/issue-0686-associative-knowledge-networks-learning.md | PR #689 (issue #686) | rust/tests/unit/docs_requirements_issue_686.rs; rust/tests/unit/mod.rs | not yet confirmed |
| R453 | docs/requirements/issue-0686-associative-knowledge-networks-learning.md | PR #689 (issue #686) | issue-level coverage (not row-pinned): rust/tests/unit/issue_686_associative_persistence.rs | not yet confirmed |
| R454 | docs/requirements/issue-0686-associative-knowledge-networks-learning.md | PR #689 (issue #686) | issue-level coverage (not row-pinned): rust/tests/unit/issue_686_associative_persistence.rs | not yet confirmed |
| R455 | docs/requirements/issue-0686-associative-knowledge-networks-learning.md | PR #689 (issue #686) | issue-level coverage (not row-pinned): rust/tests/unit/issue_686_associative_persistence.rs | not yet confirmed |
| R456 | docs/requirements/issue-0686-associative-knowledge-networks-learning.md | PR #689 (issue #686) | issue-level coverage (not row-pinned): rust/tests/unit/issue_686_associative_persistence.rs | not yet confirmed |
| R457 | docs/requirements/issue-0686-associative-knowledge-networks-learning.md | PR #689 (issue #686) | issue-level coverage (not row-pinned): rust/tests/unit/issue_686_associative_persistence.rs | not yet confirmed |
| R458 | docs/requirements/issue-0686-associative-knowledge-networks-learning.md | PR #689 (issue #686) | issue-level coverage (not row-pinned): rust/tests/unit/issue_686_associative_persistence.rs | not yet confirmed |
| R459 | docs/requirements/issue-0656-benchmark-gated-promotion-protocol.md | PR #690 (issue #656) | issue-level coverage (not row-pinned): rust/tests/integration/issue_656_improve.rs | not yet confirmed |
| R460 | docs/requirements/issue-0656-benchmark-gated-promotion-protocol.md | PR #690 (issue #656) | issue-level coverage (not row-pinned): rust/tests/integration/issue_656_improve.rs | not yet confirmed |
| R461 | docs/requirements/issue-0656-benchmark-gated-promotion-protocol.md | PR #690 (issue #656) | issue-level coverage (not row-pinned): rust/tests/integration/issue_656_improve.rs | not yet confirmed |
| R462 | docs/requirements/issue-0656-benchmark-gated-promotion-protocol.md | PR #690 (issue #656) | issue-level coverage (not row-pinned): rust/tests/integration/issue_656_improve.rs | not yet confirmed |
| R463 | docs/requirements/issue-0656-benchmark-gated-promotion-protocol.md | PR #690 (issue #656) | rust/tests/integration/issue_656_improve.rs | not yet confirmed |
| R464 | docs/requirements/issue-0656-benchmark-gated-promotion-protocol.md | PR #690 (issue #656) | issue-level coverage (not row-pinned): rust/tests/integration/issue_656_improve.rs | not yet confirmed |
| R465 | docs/requirements/issue-0656-benchmark-gated-promotion-protocol.md | PR #690 (issue #656) | rust/tests/unit/docs_requirements_issue_656.rs | not yet confirmed |
| R466 | docs/requirements/issue-0656-benchmark-gated-promotion-protocol.md | PR #690 (issue #656) | issue-level coverage (not row-pinned): rust/tests/integration/issue_656_improve.rs | not yet confirmed |
| R467 | docs/requirements/issue-0656-benchmark-gated-promotion-protocol.md | PR #690 (issue #656) | issue-level coverage (not row-pinned): rust/tests/integration/issue_656_improve.rs | not yet confirmed |
| R468 | docs/requirements/issue-0656-benchmark-gated-promotion-protocol.md | PR #690 (issue #656) | issue-level coverage (not row-pinned): rust/tests/integration/issue_656_improve.rs | not yet confirmed |
| R469 | docs/requirements/issue-0656-benchmark-gated-promotion-protocol.md | PR #690 (issue #656) | issue-level coverage (not row-pinned): rust/tests/integration/issue_656_improve.rs | not yet confirmed |
| R470 | docs/requirements/issue-0656-benchmark-gated-promotion-protocol.md | PR #690 (issue #656) | issue-level coverage (not row-pinned): rust/tests/integration/issue_656_improve.rs | not yet confirmed |
| R471 | docs/requirements/issue-0656-benchmark-gated-promotion-protocol.md | PR #690 (issue #656) | issue-level coverage (not row-pinned): rust/tests/integration/issue_656_improve.rs | not yet confirmed |
| R472 | docs/requirements/issue-0656-benchmark-gated-promotion-protocol.md | PR #690 (issue #656) | issue-level coverage (not row-pinned): rust/tests/integration/issue_656_improve.rs | not yet confirmed |
| R473 | docs/requirements/issue-0657-release-self-hosting-metric.md | PR #735 (issue #657) | issue-level coverage (not row-pinned): rust/tests/unit/issue_657_self_hosting_learning.rs | not yet confirmed |
| R474 | docs/requirements/issue-0657-release-self-hosting-metric.md | PR #735 (issue #657) | issue-level coverage (not row-pinned): rust/tests/unit/issue_657_self_hosting_learning.rs | not yet confirmed |
| R475 | docs/requirements/issue-0657-release-self-hosting-metric.md | PR #735 (issue #657) | issue-level coverage (not row-pinned): rust/tests/unit/issue_657_self_hosting_learning.rs | not yet confirmed |
| R476 | docs/requirements/issue-0657-release-self-hosting-metric.md | PR #735 (issue #657) | issue-level coverage (not row-pinned): rust/tests/unit/issue_657_self_hosting_learning.rs | not yet confirmed |
| R477 | docs/requirements/issue-0657-release-self-hosting-metric.md | PR #735 (issue #657) | issue-level coverage (not row-pinned): rust/tests/unit/issue_657_self_hosting_learning.rs | not yet confirmed |
| R478 | docs/requirements/issue-0657-release-self-hosting-metric.md | PR #735 (issue #657) | issue-level coverage (not row-pinned): rust/tests/unit/issue_657_self_hosting_learning.rs | not yet confirmed |
| R479 | docs/requirements/issue-0657-release-self-hosting-metric.md | PR #735 (issue #657) | issue-level coverage (not row-pinned): rust/tests/unit/issue_657_self_hosting_learning.rs | not yet confirmed |
| R549 | docs/requirements/issue-0657-release-self-hosting-metric.md | PR #735 (issue #657) | issue-level coverage (not row-pinned): rust/tests/unit/issue_657_self_hosting_learning.rs | not yet confirmed |
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
| R528 | docs/requirements/issue-0698-real-external-benchmark-harness.md | PR #816 (issue #698) | none recorded | not yet confirmed |
| R529 | docs/requirements/issue-0698-real-external-benchmark-harness.md | PR #816 (issue #698) | none recorded | not yet confirmed |
| R530 | docs/requirements/issue-0698-real-external-benchmark-harness.md | PR #816 (issue #698) | none recorded | not yet confirmed |
| R531 | docs/requirements/issue-0698-real-external-benchmark-harness.md | PR #816 (issue #698) | none recorded | not yet confirmed |
| R532 | docs/requirements/issue-0698-real-external-benchmark-harness.md | PR #816 (issue #698) | none recorded | not yet confirmed |
| R533 | docs/requirements/issue-0698-real-external-benchmark-harness.md | PR #816 (issue #698) | none recorded | not yet confirmed |
| R534 | docs/requirements/issue-0698-real-external-benchmark-harness.md | PR #816 (issue #698) | none recorded | not yet confirmed |
| R535 | docs/requirements/issue-0698-real-external-benchmark-harness.md | PR #816 (issue #698) | none recorded | not yet confirmed |
| R702-1 | docs/requirements/issue-0702-dialogue-world-model.md | PR #675 (issue #702) | none recorded | not yet confirmed |
| R702-2 | docs/requirements/issue-0702-dialogue-world-model.md | PR #675 (issue #702) | none recorded | not yet confirmed |
| R702-3 | docs/requirements/issue-0702-dialogue-world-model.md | PR #675 (issue #702) | none recorded | not yet confirmed |
| R702-4 | docs/requirements/issue-0702-dialogue-world-model.md | PR #675 (issue #702) | none recorded | not yet confirmed |
| R702-5 | docs/requirements/issue-0702-dialogue-world-model.md | PR #675 (issue #702) | none recorded | not yet confirmed |
| R702-6 | docs/requirements/issue-0702-dialogue-world-model.md | PR #675 (issue #702) | none recorded | not yet confirmed |
| R702-7 | docs/requirements/issue-0702-dialogue-world-model.md | PR #675 (issue #702) | none recorded | not yet confirmed |
| R702-8 | docs/requirements/issue-0702-dialogue-world-model.md | PR #675 (issue #702) | none recorded | not yet confirmed |
| R702-9 | docs/requirements/issue-0702-dialogue-world-model.md | PR #675 (issue #702) | none recorded | not yet confirmed |
| R702-10 | docs/requirements/issue-0702-dialogue-world-model.md | PR #675 (issue #702) | rust/tests/unit/docs_requirements_issue_702.rs | not yet confirmed |
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
| R484 | docs/requirements/issue-0834-legal-compliance-self-audit.md | PR #837 (issue #834) | issue-level coverage (not row-pinned): rust/tests/unit/docs_requirements_issue_834.rs | not yet confirmed |
| R485 | docs/requirements/issue-0834-legal-compliance-self-audit.md | PR #837 (issue #834) | issue-level coverage (not row-pinned): rust/tests/unit/docs_requirements_issue_834.rs | not yet confirmed |
| R486 | docs/requirements/issue-0834-legal-compliance-self-audit.md | PR #837 (issue #834) | issue-level coverage (not row-pinned): rust/tests/unit/docs_requirements_issue_834.rs | not yet confirmed |
| R487 | docs/requirements/issue-0834-legal-compliance-self-audit.md | PR #837 (issue #834) | issue-level coverage (not row-pinned): rust/tests/unit/docs_requirements_issue_834.rs | not yet confirmed |
| R488 | docs/requirements/issue-0834-legal-compliance-self-audit.md | PR #837 (issue #834) | issue-level coverage (not row-pinned): rust/tests/unit/docs_requirements_issue_834.rs | not yet confirmed |
| R489 | docs/requirements/issue-0834-legal-compliance-self-audit.md | PR #837 (issue #834) | issue-level coverage (not row-pinned): rust/tests/unit/docs_requirements_issue_834.rs | not yet confirmed |
| R490 | docs/requirements/issue-0834-legal-compliance-self-audit.md | PR #837 (issue #834) | issue-level coverage (not row-pinned): rust/tests/unit/docs_requirements_issue_834.rs | not yet confirmed |
| R491 | docs/requirements/issue-0834-legal-compliance-self-audit.md | PR #837 (issue #834) | issue-level coverage (not row-pinned): rust/tests/unit/docs_requirements_issue_834.rs | not yet confirmed |
| R492 | docs/requirements/issue-0834-legal-compliance-self-audit.md | PR #837 (issue #834) | issue-level coverage (not row-pinned): rust/tests/unit/docs_requirements_issue_834.rs | not yet confirmed |
| R493 | docs/requirements/issue-0834-legal-compliance-self-audit.md | PR #837 (issue #834) | rust/tests/unit/docs_requirements_issue_834.rs | not yet confirmed |
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
| R510 | docs/requirements/issue-0844-statement-level-merging-into-a-context.md | PR #855 (issue #844) | rust/tests/unit/docs_requirements_issue_844.rs | not yet confirmed |
| R847-1 | docs/requirements/issue-0847-task-decomposition-as-a-working-task.md | PR #857 (issue #847) | issue-level coverage (not row-pinned): rust/tests/unit/issue_847_task_decomposition.rs | not yet confirmed |
| R847-2 | docs/requirements/issue-0847-task-decomposition-as-a-working-task.md | PR #857 (issue #847) | issue-level coverage (not row-pinned): rust/tests/unit/issue_847_task_decomposition.rs | not yet confirmed |
| R847-3 | docs/requirements/issue-0847-task-decomposition-as-a-working-task.md | PR #857 (issue #847) | issue-level coverage (not row-pinned): rust/tests/unit/issue_847_task_decomposition.rs | not yet confirmed |
| R847-4 | docs/requirements/issue-0847-task-decomposition-as-a-working-task.md | PR #857 (issue #847) | issue-level coverage (not row-pinned): rust/tests/unit/issue_847_task_decomposition.rs | not yet confirmed |
| R847-5 | docs/requirements/issue-0847-task-decomposition-as-a-working-task.md | PR #857 (issue #847) | issue-level coverage (not row-pinned): rust/tests/unit/issue_847_task_decomposition.rs | not yet confirmed |
| R847-6 | docs/requirements/issue-0847-task-decomposition-as-a-working-task.md | PR #857 (issue #847) | issue-level coverage (not row-pinned): rust/tests/unit/issue_847_task_decomposition.rs | not yet confirmed |
| R847-7 | docs/requirements/issue-0847-task-decomposition-as-a-working-task.md | PR #857 (issue #847) | issue-level coverage (not row-pinned): rust/tests/unit/issue_847_task_decomposition.rs | not yet confirmed |
| R847-8 | docs/requirements/issue-0847-task-decomposition-as-a-working-task.md | PR #857 (issue #847) | rust/tests/unit/specification/task_decomposition.rs; rust/tests/unit/issue_847_task_decomposition.rs | not yet confirmed |
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
| R706-5 | docs/requirements/issue-0706-any-language-protocol.md | PR #880 (issue #706) | rust/tests/e2e/scripts/check-language-{test-coverage,change-parity}.mjs | not yet confirmed |
| R706-6 | docs/requirements/issue-0706-any-language-protocol.md | PR #880 (issue #706) | issue-level coverage (not row-pinned): rust/tests/unit/issue_706_any_language.rs | not yet confirmed |
| R706-7 | docs/requirements/issue-0706-any-language-protocol.md | PR #880 (issue #706) | issue-level coverage (not row-pinned): rust/tests/unit/issue_706_any_language.rs | not yet confirmed |
| R706-8 | docs/requirements/issue-0706-any-language-protocol.md | PR #880 (issue #706) | issue-level coverage (not row-pinned): rust/tests/unit/issue_706_any_language.rs | not yet confirmed |
| R706-9 | docs/requirements/issue-0706-any-language-protocol.md | PR #880 (issue #706) | issue-level coverage (not row-pinned): rust/tests/unit/issue_706_any_language.rs | not yet confirmed |
| R858-1 | docs/requirements/issue-0858-claude-code-returning-user-recap.md | PR #899 (issue #858) | issue-level coverage (not row-pinned): rust/tests/unit/issue_858.rs | not yet confirmed |
| R858-2 | docs/requirements/issue-0858-claude-code-returning-user-recap.md | PR #899 (issue #858) | issue-level coverage (not row-pinned): rust/tests/unit/issue_858.rs | not yet confirmed |
| R858-3 | docs/requirements/issue-0858-claude-code-returning-user-recap.md | PR #899 (issue #858) | issue-level coverage (not row-pinned): rust/tests/unit/issue_858.rs | not yet confirmed |
| R858-4 | docs/requirements/issue-0858-claude-code-returning-user-recap.md | PR #899 (issue #858) | issue-level coverage (not row-pinned): rust/tests/unit/issue_858.rs | not yet confirmed |
| R858-5 | docs/requirements/issue-0858-claude-code-returning-user-recap.md | PR #899 (issue #858) | issue-level coverage (not row-pinned): rust/tests/unit/issue_858.rs | not yet confirmed |
| R858-6 | docs/requirements/issue-0858-claude-code-returning-user-recap.md | PR #899 (issue #858) | issue-level coverage (not row-pinned): rust/tests/unit/issue_858.rs | not yet confirmed |
| R708-1 | docs/requirements/issue-0708-bounded-natural-language-memory-programs.md | PR #883 (issue #708) | issue-level coverage (not row-pinned): rust/tests/integration/memory_query.rs | not yet confirmed |
| R708-2 | docs/requirements/issue-0708-bounded-natural-language-memory-programs.md | PR #883 (issue #708) | issue-level coverage (not row-pinned): rust/tests/integration/memory_query.rs | not yet confirmed |
| R708-3 | docs/requirements/issue-0708-bounded-natural-language-memory-programs.md | PR #883 (issue #708) | issue-level coverage (not row-pinned): rust/tests/integration/memory_query.rs | not yet confirmed |
| R708-4 | docs/requirements/issue-0708-bounded-natural-language-memory-programs.md | PR #883 (issue #708) | issue-level coverage (not row-pinned): rust/tests/integration/memory_query.rs | not yet confirmed |
| R708-5 | docs/requirements/issue-0708-bounded-natural-language-memory-programs.md | PR #883 (issue #708) | issue-level coverage (not row-pinned): rust/tests/integration/memory_query.rs | not yet confirmed |
| R708-6 | docs/requirements/issue-0708-bounded-natural-language-memory-programs.md | PR #883 (issue #708) | issue-level coverage (not row-pinned): rust/tests/integration/memory_query.rs | not yet confirmed |
| R708-7 | docs/requirements/issue-0708-bounded-natural-language-memory-programs.md | PR #883 (issue #708) | issue-level coverage (not row-pinned): rust/tests/integration/memory_query.rs | not yet confirmed |
| R708-8 | docs/requirements/issue-0708-bounded-natural-language-memory-programs.md | PR #883 (issue #708) | rust/tests/e2e/tests/issue-708.spec.js | not yet confirmed |
| R708-9 | docs/requirements/issue-0708-bounded-natural-language-memory-programs.md | PR #883 (issue #708) | issue-level coverage (not row-pinned): rust/tests/integration/memory_query.rs | not yet confirmed |
| R709-1 | docs/requirements/issue-0709-multi-source-search-fusion.md | pre-2026-07 (undated); issue #709 | none recorded | not yet confirmed |
| R709-2 | docs/requirements/issue-0709-multi-source-search-fusion.md | pre-2026-07 (undated); issue #709 | none recorded | not yet confirmed |
| R709-3 | docs/requirements/issue-0709-multi-source-search-fusion.md | pre-2026-07 (undated); issue #709 | none recorded | not yet confirmed |
| R709-4 | docs/requirements/issue-0709-multi-source-search-fusion.md | pre-2026-07 (undated); issue #709 | none recorded | not yet confirmed |
| R709-5 | docs/requirements/issue-0709-multi-source-search-fusion.md | pre-2026-07 (undated); issue #709 | none recorded | not yet confirmed |
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
| R710-D15 | docs/requirements/issue-0710-dynamic-coding-discovery.md | delivered 2026-09-15; PR #888 (issue #710) | rust/tests/unit/docs_requirements/benchmarks.rs::latest_external_rows_are_published_from_the_ledger | not yet confirmed |
| R710-D16 | docs/requirements/issue-0710-dynamic-coding-discovery.md | delivered 2026-09-15; PR #888 (issue #710) | rust/tests/unit/specification/self_hosting_metric.rs | not yet confirmed |
| R1137-1 | docs/requirements/issue-1137-pre-merge-four-client-routing-replay.md | delivered 2026-09-15; PR #888 (issue #1137) | rust/tests/unit/ci-cd/issue_1137_agentic_routing_replay.rs::agentic_routing_changes_enable_full_four_client_replay_on_pull_requests; scripts/detect-code-changes.rs::tests::agentic_source_changes_request_the_four_client_replay | not yet confirmed — CI on the final PR head is the unattended proof |
| R1137-2 | docs/requirements/issue-1137-pre-merge-four-client-routing-replay.md | delivered 2026-09-15; PR #888 (issue #1137) | rust/tests/unit/ci-cd/issue_1137_agentic_routing_replay.rs::the_full_replay_still_exercises_each_supported_client; experiments/agent_cli_e2e/run_issue_781.sh | not yet confirmed — CI on the final PR head is the unattended proof |
| R1137-3 | docs/requirements/issue-1137-pre-merge-four-client-routing-replay.md | delivered 2026-09-15; PR #888 (issue #1137) | scripts/detect-code-changes.rs::tests::agentic_source_changes_request_the_four_client_replay | not yet confirmed |
| R835-1 | docs/requirements/issue-0835-multi-jurisdiction-file-legal-risk-assessment.md | PR #900 (issue #835) | issue-level coverage (not row-pinned): rust/tests/unit/issue_835_file_legality.rs | not yet confirmed |
| R835-2 | docs/requirements/issue-0835-multi-jurisdiction-file-legal-risk-assessment.md | PR #900 (issue #835) | issue-level coverage (not row-pinned): rust/tests/unit/issue_835_file_legality.rs | not yet confirmed |
| R835-3 | docs/requirements/issue-0835-multi-jurisdiction-file-legal-risk-assessment.md | PR #900 (issue #835) | issue-level coverage (not row-pinned): rust/tests/unit/issue_835_file_legality.rs | not yet confirmed |
| R835-4 | docs/requirements/issue-0835-multi-jurisdiction-file-legal-risk-assessment.md | PR #900 (issue #835) | issue-level coverage (not row-pinned): rust/tests/unit/issue_835_file_legality.rs | not yet confirmed |
| R835-5 | docs/requirements/issue-0835-multi-jurisdiction-file-legal-risk-assessment.md | PR #900 (issue #835) | issue-level coverage (not row-pinned): rust/tests/unit/issue_835_file_legality.rs | not yet confirmed |
| R835-6 | docs/requirements/issue-0835-multi-jurisdiction-file-legal-risk-assessment.md | PR #900 (issue #835) | issue-level coverage (not row-pinned): rust/tests/unit/issue_835_file_legality.rs | not yet confirmed |
| R835-7 | docs/requirements/issue-0835-multi-jurisdiction-file-legal-risk-assessment.md | PR #900 (issue #835) | issue-level coverage (not row-pinned): rust/tests/unit/issue_835_file_legality.rs | not yet confirmed |
| R835-8 | docs/requirements/issue-0835-multi-jurisdiction-file-legal-risk-assessment.md | PR #900 (issue #835) | issue-level coverage (not row-pinned): rust/tests/unit/issue_835_file_legality.rs | not yet confirmed |
| R835-9 | docs/requirements/issue-0835-multi-jurisdiction-file-legal-risk-assessment.md | PR #900 (issue #835) | issue-level coverage (not row-pinned): rust/tests/unit/issue_835_file_legality.rs | not yet confirmed |
| R864-1 | docs/requirements/issue-0864-proactive-failure-report-invitations.md | PR #910 (issue #864) | issue-level coverage (not row-pinned): rust/tests/unit/issue_864.rs | not yet confirmed |
| R864-2 | docs/requirements/issue-0864-proactive-failure-report-invitations.md | PR #910 (issue #864) | issue-level coverage (not row-pinned): rust/tests/unit/issue_864.rs | not yet confirmed |
| R864-3 | docs/requirements/issue-0864-proactive-failure-report-invitations.md | PR #910 (issue #864) | issue-level coverage (not row-pinned): rust/tests/unit/issue_864.rs | not yet confirmed |
| R864-4 | docs/requirements/issue-0864-proactive-failure-report-invitations.md | PR #910 (issue #864) | issue-level coverage (not row-pinned): rust/tests/unit/issue_864.rs | not yet confirmed |
| R864-5 | docs/requirements/issue-0864-proactive-failure-report-invitations.md | PR #910 (issue #864) | issue-level coverage (not row-pinned): rust/tests/unit/issue_864.rs | not yet confirmed |
| R914-1 | docs/requirements/issue-0914-vision-implementation-planning-coding-first.md | pre-2026-07 (undated); issue #914 | issue-level coverage (not row-pinned): rust/tests/unit/docs_requirements_issue_914.rs | not yet confirmed |
| R914-2 | docs/requirements/issue-0914-vision-implementation-planning-coding-first.md | pre-2026-07 (undated); issue #914 | issue-level coverage (not row-pinned): rust/tests/unit/docs_requirements_issue_914.rs | not yet confirmed |
| R914-3 | docs/requirements/issue-0914-vision-implementation-planning-coding-first.md | pre-2026-07 (undated); issue #914 | issue-level coverage (not row-pinned): rust/tests/unit/docs_requirements_issue_914.rs | not yet confirmed |
| R914-4 | docs/requirements/issue-0914-vision-implementation-planning-coding-first.md | pre-2026-07 (undated); issue #914 | issue-level coverage (not row-pinned): rust/tests/unit/docs_requirements_issue_914.rs | not yet confirmed |
| R914-5 | docs/requirements/issue-0914-vision-implementation-planning-coding-first.md | partial: E70 delivered general natural-formal translation for the seeded FOL statement slice (PR #984, issue #917); E75 delivered method learning (PR #1005, issue #922); broader coverage remains incremental seed growth | issue-level coverage (not row-pinned): rust/tests/unit/docs_requirements_issue_914.rs | not yet confirmed |
| R914-6 | docs/requirements/issue-0914-vision-implementation-planning-coding-first.md | partial: enforced by the #918 minimal-core boundary (PR #986) and the open handler-migration continuation #959 | issue-level coverage (not row-pinned): rust/tests/unit/docs_requirements_issue_914.rs | not yet confirmed |
| R914-7 | docs/requirements/issue-0914-vision-implementation-planning-coding-first.md | pre-2026-07 (undated); issue #914 | issue-level coverage (not row-pinned): rust/tests/unit/docs_requirements_issue_914.rs | not yet confirmed |
| R914-8 | docs/requirements/issue-0914-vision-implementation-planning-coding-first.md | delivered 2026-09-15; PR #888 (issue #710) | rust/tests/unit/coding_discovery/composition.rs | not yet confirmed |
| R914-9 | docs/requirements/issue-0914-vision-implementation-planning-coding-first.md | delivered 2026-09-15; PR #888 (issue #710) | rust/tests/unit/issue_1138_learned_items_change_answers.rs | not yet confirmed |
| R914-10 | docs/requirements/issue-0914-vision-implementation-planning-coding-first.md | delivered 2026-08-14; PR #1003 (issue #920) | rust/tests/unit/issue_920_question_necessity.rs | not yet confirmed |
| R914-11 | docs/requirements/issue-0914-vision-implementation-planning-coding-first.md | delivered 2026-08-18; PR #1004 (issue #921) | rust/tests/unit/issue_921.rs | not yet confirmed |
| R914-12 | docs/requirements/issue-0914-vision-implementation-planning-coding-first.md | pre-2026-07 (undated); issue #914 | issue-level coverage (not row-pinned): rust/tests/unit/docs_requirements_issue_914.rs | not yet confirmed |
| R914-13 | docs/requirements/issue-0914-vision-implementation-planning-coding-first.md | pre-2026-07 (undated); issue #914 | issue-level coverage (not row-pinned): rust/tests/unit/docs_requirements_issue_914.rs | not yet confirmed |
| R914-14 | docs/requirements/issue-0914-vision-implementation-planning-coding-first.md | pre-2026-07 (undated); issue #914 | issue-level coverage (not row-pinned): rust/tests/unit/docs_requirements_issue_914.rs | not yet confirmed |
| R914-15 | docs/requirements/issue-0914-vision-implementation-planning-coding-first.md | pre-2026-07 (undated); issue #914 | issue-level coverage (not row-pinned): rust/tests/unit/docs_requirements_issue_914.rs | not yet confirmed |
| R891-1 | docs/requirements/issue-0891-equation-corpus-ratchet.md | PR #968 (issue #891) | rust/tests/unit/specification/equation_corpus.rs; rust/tests/unit/docs_requirements_issue_891.rs | manually probed 2026-08-04 with `cargo run --example issue_891_equation_probe`; raw engine output kept in docs/case-studies/issue-891/raw-data/production-solver-probe.tsv |
| R891-2 | docs/requirements/issue-0891-equation-corpus-ratchet.md | PR #968 (issue #891) | rust/tests/unit/specification/equation_corpus.rs; rust/tests/unit/docs_requirements_issue_891.rs | manually probed 2026-08-04 with `cargo run --example issue_891_equation_probe`; raw engine output kept in docs/case-studies/issue-891/raw-data/production-solver-probe.tsv |
| R891-3 | docs/requirements/issue-0891-equation-corpus-ratchet.md | PR #968 (issue #891) | rust/tests/unit/specification/equation_corpus.rs; rust/tests/unit/docs_requirements_issue_891.rs | manually probed 2026-08-04 with `cargo run --example issue_891_equation_probe`; raw engine output kept in docs/case-studies/issue-891/raw-data/production-solver-probe.tsv |
| R891-4 | docs/requirements/issue-0891-equation-corpus-ratchet.md | PR #968 (issue #891) | rust/tests/unit/specification/equation_corpus.rs; rust/tests/unit/docs_requirements_issue_891.rs | manually probed 2026-08-04 with `cargo run --example issue_891_equation_probe`; raw engine output kept in docs/case-studies/issue-891/raw-data/production-solver-probe.tsv |
| R891-5 | docs/requirements/issue-0891-equation-corpus-ratchet.md | PR #968 (issue #891) | rust/tests/unit/specification/equation_corpus.rs; rust/tests/unit/docs_requirements_issue_891.rs | manually probed 2026-08-04 with `cargo run --example issue_891_equation_probe`; raw engine output kept in docs/case-studies/issue-891/raw-data/production-solver-probe.tsv |
| R891-6 | docs/requirements/issue-0891-equation-corpus-ratchet.md | PR #968 (issue #891) | rust/tests/unit/specification/equation_corpus.rs; rust/tests/unit/docs_requirements_issue_891.rs | manually probed 2026-08-04 with `cargo run --example issue_891_equation_probe`; raw engine output kept in docs/case-studies/issue-891/raw-data/production-solver-probe.tsv |
| R909-1 | docs/requirements/issue-0909-headless-ready-global-client-configuration.md | delivered 2026-08-04; issue #909 | rust/tests/integration/with_formal_ai_headless_global.rs:45 | manually confirmed 2026-08-04: `formal-ai with --global gemini` into a throwaway HOME wrote `.gemini/settings.json` with `security.auth.selectedType`, `--undo` removed it |
| R909-2 | docs/requirements/issue-0909-headless-ready-global-client-configuration.md | delivered 2026-08-04; issue #909 | rust/tests/integration/with_formal_ai_headless_global.rs:147 | manually confirmed 2026-08-04: `formal-ai with --global qwen` wrote `OPENAI_API_KEY`, `OPENAI_BASE_URL`, and `OPENAI_MODEL` into `~/.profile` |
| R909-3 | docs/requirements/issue-0909-headless-ready-global-client-configuration.md | delivered 2026-08-04; issue #909 | rust/tests/integration/with_formal_ai_headless_global.rs:171 | manually confirmed 2026-08-04: `experiments/issue-909-headless-config-gaps.sh` reported every headless requirement present |
| R909-4 | docs/requirements/issue-0909-headless-ready-global-client-configuration.md | delivered 2026-08-04; issue #909 | rust/tests/unit/docs_requirements_issue_909.rs:13 | not yet confirmed |
| R909-5 | docs/requirements/issue-0909-headless-ready-global-client-configuration.md | delivered 2026-08-04; issue #909 | issue-level coverage (not row-pinned): rust/tests/unit/docs_requirements_issue_909.rs | manually confirmed 2026-08-04: script run against the debug binary, exit 0 |
| R909-6 | docs/requirements/issue-0909-headless-ready-global-client-configuration.md | delivered 2026-08-04; issue #909 | rust/tests/integration/with_formal_ai_headless_global.rs:308 | manually confirmed 2026-08-04: `--global --all --verify` sweep into a throwaway HOME satisfied every registry-declared requirement |
| R909-7 | docs/requirements/issue-0909-headless-ready-global-client-configuration.md | delivered 2026-08-06; issue #909 review | rust/tests/unit/total_closure.rs:25 | manually confirmed 2026-08-06: `experiments/issue-909-seed-shard-conflict-blast-radius.sh` dirtied exactly 1 of 16 shards at each of four sort positions, against 11 of 11 before the fix |
| R893-1 | docs/requirements/issue-0893-iterative-summarization-validation-and-the-80-quality-ratchet.md | PR #970 (issue #893) | rust/tests/unit/specification/issue_893_summarization_validation.rs; rust/tests/unit/docs_requirements_issue_893.rs | measured 2026-08-05 with `cargo run --release --example issue_893_measure`; raw run kept in docs/case-studies/issue-893/raw-data/ |
| R893-2 | docs/requirements/issue-0893-iterative-summarization-validation-and-the-80-quality-ratchet.md | PR #970 (issue #893) | rust/tests/unit/specification/issue_893_summarization_validation.rs; rust/tests/unit/docs_requirements_issue_893.rs | measured 2026-08-05 with `cargo run --release --example issue_893_measure`; raw run kept in docs/case-studies/issue-893/raw-data/ |
| R893-3 | docs/requirements/issue-0893-iterative-summarization-validation-and-the-80-quality-ratchet.md | PR #970 (issue #893) | rust/tests/unit/specification/issue_893_summarization_validation.rs; rust/tests/unit/docs_requirements_issue_893.rs | committed baseline data/summarization/quality-baseline.lino, re-measured by `formal-ai summarization ratchet` |
| R893-4 | docs/requirements/issue-0893-iterative-summarization-validation-and-the-80-quality-ratchet.md | PR #970 (issue #893) | rust/tests/unit/specification/issue_893_summarization_validation.rs; rust/tests/unit/docs_requirements_issue_893.rs | measured 2026-08-05; embedded-grammar blocks counted against an independent CommonMark fence scanner |
| R893-5 | docs/requirements/issue-0893-iterative-summarization-validation-and-the-80-quality-ratchet.md | PR #970 (issue #893) | rust/tests/unit/specification/issue_893_summarization_validation.rs; rust/tests/unit/docs_requirements_issue_893.rs | the four compression failures and the `<version>` grounding defect found by the 600-file sweep are recorded in docs/case-studies/issue-893/README.md rather than tuned away |
| R536 | docs/requirements/doctrine-standing-doctrine-compiled-logic-interfacing-only-javascript-2026-08-04.md | doctrine adopted 2026-08-04 | none yet — enforcement tracked in #934/#951/#952/#953 | n/a |
| R894-1 | docs/requirements/issue-0894-ci-template-upstream-filings.md | PR #971 (issue #894) | rust/tests/unit/docs_requirements_issue_894.rs | revalidated 2026-08-05 against the four template default branches; commands and verbatim output kept in docs/case-studies/issue-894/raw-data/revalidation-greps.txt and revalidation-greps-2.txt |
| R894-2 | docs/requirements/issue-0894-ci-template-upstream-filings.md | PR #971 (issue #894) | rust/tests/unit/docs_requirements_issue_894.rs | eight issues filed upstream 2026-08-05; bodies and API snapshot kept in docs/case-studies/issue-894/raw-data/ |
| R894-3 | docs/requirements/issue-0894-ci-template-upstream-filings.md | PR #971 (issue #894) | rust/tests/unit/docs_requirements_issue_894.rs | ledger rendered and links opened 2026-08-05 in docs/case-studies/issue-479/template-comparison/REPORT.md |
| R894-4 | docs/requirements/issue-0894-ci-template-upstream-filings.md | PR #971 (issue #894) | rust/tests/unit/docs_requirements_issue_894.rs | falsified 2026-08-05 by deleting a filing URL from the ledger and observing the test fail |
| R980-1 | docs/requirements/issue-0980-default-branch-ci-false-results.md | PR #981 (issue #980) | rust/tests/unit/ci-cd/issue_980.rs | manually confirmed 2026-08-08 by downloading all seven referenced workflow logs and matching each run timestamp and SHA; findings preserved in dev/log/issues/980/pulls/981/ |
| R980-2 | docs/requirements/issue-0980-default-branch-ci-false-results.md | PR #981 (issue #980) | rust/tests/unit/ci-cd/issue_980.rs; rust/tests/e2e/tests/issue-282.spec.js; rust/tests/e2e/tests/issue-541-permissions-cold-start.spec.js | manually confirmed 2026-08-08: opener parity passed 12/12 repeated cases and permission replay passed 9/9 repeated cases |
| R980-3 | docs/requirements/issue-0980-default-branch-ci-false-results.md | PR #981 (issue #980) | rust/tests/unit/ci-cd/issue_980.rs | manually confirmed 2026-08-08 against complete tracked trees at rust c867f78, js 7b70923, and python 98d6dca; snapshots and control indexes preserved in the evidence bundle |
| R980-4 | docs/requirements/issue-0980-default-branch-ci-false-results.md | PR #981 (issue #980) | rust/tests/unit/ci-cd/issue_980.rs | falsified 2026-08-08 by running the regression gates before the fixes; the formatting and isolation guards failed, then all three passed after the fixes |
| R973-1 | docs/requirements/issue-0973-automated-solve-session-evidence.md | PR #974 (issue #973) | rust/tests/issue_973_solve_flags.rs::the_live_self_coding_entry_point_attaches_logs_and_runs_verbose; rust/tests/issue_973_solve_flags.rs::every_published_solve_invocation_carries_both_evidence_flags | falsified 2026-08-05 by removing `--attach-logs` from rust/examples/self-coding/run.sh and observing both tests fail |
| R973-2 | docs/requirements/issue-0973-automated-solve-session-evidence.md | PR #974 (issue #973) | rust/tests/issue_973_solve_flags.rs::contributing_explains_why_both_flags_are_load_bearing | falsified 2026-08-05 by removing `--verbose` from the CONTRIBUTING.md command and observing the scan fail at CONTRIBUTING.md:115 |
| R973-3 | docs/requirements/issue-0973-automated-solve-session-evidence.md | PR #974 (issue #973) | rust/tests/issue_973_solve_flags.rs::every_published_solve_invocation_carries_both_evidence_flags; rust/tests/issue_973_solve_flags.rs::the_case_study_records_the_unrecoverable_failure_and_the_fix | not yet confirmed beyond the two falsification runs above |
| R1021-1 | docs/requirements/issue-1021-full-range-coding-and-contribution-artifacts.md | PR #1027 (issue #1021), 2026-08-19 | rust/tests/unit/issue_1021_closed_circle.rs::closed_circle_session_replays | not yet confirmed |
| R1021-2 | docs/requirements/issue-1021-full-range-coding-and-contribution-artifacts.md | PR #1027 (issue #1021), 2026-08-19 | rust/tests/unit/issue_1021_behaviour_range.rs (whole module -- every case is a held-out paraphrase or an unseen word order) | falsified-then-confirmed 2026-08-19 for the listing rule: Spanish was registered in data/seed/languages.lino but had no listing vocabulary, and supplying it in data/seed/shell-intents.lino made four held-out Spanish word orders route with no Rust change -- generalization by data, recorded as finding 11; falsified again 2026-08-20 while covering R1021-31 -- `contains_token` tested word boundaries with `is_ascii_alphanumeric`, so the `c` of the Spanish `codigo` matched the alias of the language C and every Spanish request mentioning code was answered as a C program (docs/case-studies/issue-1021/logs/spanish-code-boundary-before.log), fixed by making the boundary a property of letters and pinned by rust/tests/unit/issue_1021_behaviour_range.rs::a_one_letter_alias_does_not_match_inside_an_accented_word -- recorded as finding 20 |
| R1021-3 | docs/requirements/issue-1021-full-range-coding-and-contribution-artifacts.md | PR #1027 (issue #1021), 2026-08-19 | rust/tests/unit/issue_1021_behaviour_range.rs::a_bare_command_is_the_request | not yet confirmed |
| R1021-4 | docs/requirements/issue-1021-full-range-coding-and-contribution-artifacts.md | PR #1027 (issue #1021), 2026-08-19 | rust/tests/unit/issue_1021_behaviour_range.rs::a_command_naming_noun_is_not_an_argument; ::a_command_naming_noun_is_stripped_for_every_command | not yet confirmed |
| R1021-5 | docs/requirements/issue-1021-full-range-coding-and-contribution-artifacts.md | PR #1027 (issue #1021), 2026-08-19 | rust/tests/unit/issue_1021_behaviour_range.rs::a_prose_listing_request_routes_to_ls_in_any_word_order; ::listing_parts_alone_do_not_make_a_listing_request | measured 2026-08-19 with `cargo run --example issue_1021_spanish_probe`; output kept in docs/case-studies/issue-1021/logs/spanish-listing-routing-after.log; four held-out Spanish word orders route to `ls` from seed data alone, and `lista los procesos en ejecucion` still does not, so the parts still have to combine -- see finding 11 |
| R1021-6 | docs/requirements/issue-1021-full-range-coding-and-contribution-artifacts.md | PR #1027 (issue #1021), 2026-08-20 | rust/tests/unit/issue_1021_behaviour_range.rs::a_named_exercise_is_answered_as_a_program; ::a_named_exercise_is_not_a_file_operation; ::the_stdin_answer_prints_the_input_it_was_verified_against; ::a_task_that_reads_no_input_keeps_its_plain_run_command | measured 2026-08-20 with `cargo run --example issue_1021_copy_stdin_harness`, which writes each answer's program to a scratch workspace, runs the check and run commands it printed, and compares the output against the fixture piped in; output kept in docs/case-studies/issue-1021/logs/copy-stdin-harness.log -- 10 of the 13 languages passed end to end and 3 were skipped for a toolchain absent from this machine (tsc, dotnet, scalac), 0 failed; routing recorded in docs/case-studies/issue-1021/logs/named-exercise-routing-after.log |
| R1021-7 | docs/requirements/issue-1021-full-range-coding-and-contribution-artifacts.md | PR #1027 (issue #1021), 2026-08-20 | rust/tests/unit/issue_1021_behaviour_range.rs::a_named_exercise_is_not_a_file_operation; ::a_named_exercise_is_answered_as_a_program | measured 2026-08-20 with `cargo run --example issue_1021_named_exercise_probe`; output kept in docs/case-studies/issue-1021/logs/named-exercise-routing-after.log -- `Execute https://rosettacode.org/wiki/Copy_stdin_to_stdout in Rust` is answered with the verified Rust program rather than reaching web search |
| R1021-8 | docs/requirements/issue-1021-full-range-coding-and-contribution-artifacts.md | PR #1027 (issue #1021), 2026-08-20 | rust/tests/unit/issue_1021_behaviour_range.rs::a_framework_named_coding_request_is_answered_in_that_framework; ::php_is_answered_from_the_catalog_like_every_catalogued_language; rust/tests/source/source_tests/coding/catalog/mod/framework_targets.rs (whole module) | measured 2026-08-20 with `bash experiments/issue-1021-laravel/run.sh` (the composed Artisan command run inside a real `composer create-project laravel/laravel` application: Laravel Framework 13.26.1 on PHP 8.3.31) and with `node experiments/issue-1021-laravel/worker_check.mjs`, which loads the 26 browser worker shards and the seed lexicon and confirms the mirror resolves the reported prompt in all four reported natural languages to `laravel` while `write me some PHP code` still resolves to `php`; the same harness's last two of seventeen assertions record what the mirror does not carry -- eleven tasks against the engine's twelve, without `copy_stdin_to_stdout` (finding 21) |
| R1021-9 | docs/requirements/issue-1021-full-range-coding-and-contribution-artifacts.md | PR #1027 (issue #1021), 2026-08-19 | rust/tests/unit/issue_1021_behaviour_range.rs::a_move_between_absolute_paths_is_performed; ::a_traversing_move_is_not_performed | not yet confirmed |
| R1021-10 | docs/requirements/issue-1021-full-range-coding-and-contribution-artifacts.md | PR #1027 (issue #1021), 2026-08-19 | rust/tests/unit/issue_1021_write_path.rs::filing_an_issue_is_refused_in_both_states | not yet confirmed |
| R1021-11 | docs/requirements/issue-1021-full-range-coding-and-contribution-artifacts.md | PR #1027 (issue #1021), 2026-08-20 | rust/tests/unit/issue_1021_write_path.rs::the_ladder_has_both_rungs_and_an_opt_in_to_climb_the_first; ::a_command_the_operator_named_is_not_the_ladders_business; `experiments/issue_916_write_effect_ladder/test_ladder.py` (`SandboxResetTests`, `ExpectedCommandTests`, `MutatingLadderDatasetTests` -- 30 judge tests, no server needed) | measured 2026-08-20 against the real release binary: `experiments/issue_916_write_effect_ladder/run_write_effect_ladder.sh` reports 16/16 rungs green including `824.L1`-`824.L5`, and the same run against the committed baseline reports `baseline 11/11 -> now 16/16`, so the ratchet moved up rather than sideways; log kept in docs/case-studies/issue-1021/logs/write-effect-ladder-after.log |
| R1021-12 | docs/requirements/issue-1021-full-range-coding-and-contribution-artifacts.md | PR #1027 (issue #1021), 2026-08-20 | rust/tests/unit/issue_1021_recoverable_memory.rs::a_version_that_does_not_compile_leaves_the_previous_one_in_place; ::the_compile_failure_is_a_real_compiler_diagnostic; ::a_rollback_removes_a_file_the_candidate_added; ::a_failed_version_falls_back_to_the_last_adopted_one_not_to_the_first; ::a_candidate_that_edits_a_baseline_test_is_rolled_back_before_it_is_scored | not yet confirmed |
| R1021-13 | docs/requirements/issue-1021-full-range-coding-and-contribution-artifacts.md | PR #1027 (issue #1021), 2026-08-20 | rust/tests/unit/issue_1021_bounded_autonomy.rs::a_loop_that_never_resolves_stops_at_the_limit_and_asks; ::the_question_repeats_until_the_operator_answers_it; ::granting_more_time_resumes_the_run_from_where_it_stopped; ::the_default_limit_is_the_hour_the_issue_names; ::full_trust_does_not_arrive_with_the_full_autonomous_mode | not yet confirmed |
| R1021-14 | docs/requirements/issue-1021-full-range-coding-and-contribution-artifacts.md | PR #1027 (issue #1021), 2026-08-19 | none -- not delivered; no current open tracker | n/a -- not delivered; the release ledger is non-zero, but no qualifying run is a `solve` run |
| R1021-15 | docs/requirements/issue-1021-full-range-coding-and-contribution-artifacts.md | PR #1027 (issue #1021), 2026-08-19 | rust/tests/unit/issue_1021_contribution_artifacts.rs::a_composed_fragment_is_one_the_changelog_gate_accepts; ::every_seeded_bump_and_category_composes_its_own_heading | not yet confirmed |
| R1021-16 | docs/requirements/issue-1021-full-range-coding-and-contribution-artifacts.md | PR #1027 (issue #1021), 2026-08-19 | rust/tests/unit/issue_1021_contribution_artifacts.rs::a_composed_body_closes_its_issue_by_the_gates_own_rules; rust/tests/unit/issue_1021_closed_circle.rs::the_artifacts_satisfy_the_gates_that_read_them | not yet confirmed |
| R1021-17 | docs/requirements/issue-1021-full-range-coding-and-contribution-artifacts.md | PR #1027 (issue #1021), 2026-08-19 | rust/tests/unit/issue_1021_contribution_artifacts.rs::the_generator_composes_prose_without_containing_any | not yet confirmed |
| R1021-18 | docs/requirements/issue-1021-full-range-coding-and-contribution-artifacts.md | PR #1027 (issue #1021), 2026-08-19 | rust/tests/unit/issue_1021_behaviour_range.rs (whole module) | not yet confirmed |
| R1021-19 | docs/requirements/issue-1021-full-range-coding-and-contribution-artifacts.md | PR #1027 (issue #1021), 2026-08-19 | rust/tests/unit/issue_1021_closed_circle.rs::the_committed_process_artifacts_are_generator_output | not yet confirmed |
| R1021-20 | docs/requirements/issue-1021-full-range-coding-and-contribution-artifacts.md | PR #1027 (issue #1021), 2026-08-19 | rust/tests/unit/issue_1021_write_path.rs (whole module -- both states driven from one process) | not yet confirmed |
| R1021-21 | docs/requirements/issue-1021-full-range-coding-and-contribution-artifacts.md | PR #1027 (issue #1021), 2026-08-19 | rust/tests/unit/issue_1021_closed_circle.rs::closed_circle_session_replays | not yet confirmed |
| R1021-22 | docs/requirements/issue-1021-full-range-coding-and-contribution-artifacts.md | PR #1027 (issue #1021), 2026-08-19 | none -- not achieved; the run itself is the remaining gap | n/a -- not achieved |
| R1021-23 | docs/requirements/issue-1021-full-range-coding-and-contribution-artifacts.md | PR #1027 (issue #1021), 2026-08-19 | rust/tests/unit/issue_918.rs::minimal_core_ledger_covers_every_recursive_handler_source; ::coding_path_has_complete_metadata_and_every_other_gap_is_data | not yet confirmed |
| R1021-24 | docs/requirements/issue-1021-full-range-coding-and-contribution-artifacts.md | PR #1027 (issue #1021), 2026-08-19 | rust/tests/unit/docs_requirements_issue_1021.rs | not yet confirmed |
| R1021-25 | docs/requirements/issue-1021-full-range-coding-and-contribution-artifacts.md | PR #1027 (issue #1021), 2026-08-19 | none -- the probes are examples, run by hand | run 2026-08-19; output preserved under docs/case-studies/issue-1021/logs/ |
| R1021-26 | docs/requirements/issue-1021-full-range-coding-and-contribution-artifacts.md | PR #1027 (issue #1021), 2026-08-19 | rust/tests/unit/ci-cd/issue_1021.rs::the_codex_pin_names_the_upstream_defect_and_the_bisect_that_would_lift_it | measured 2026-08-19: `python3 experiments/issue_1021_codex_tui_version/codex_trust_dialog_probe.py 0.148.0 enter-now` leaves a bare `codex` on its trust dialog after 20 s while 0.147.0 clears it; filed as openai/codex#39487 and pinned in .github/workflows/release.yml -- see finding 12 |
| R1021-27 | docs/requirements/issue-1021-full-range-coding-and-contribution-artifacts.md | PR #1027 (issue #1021), 2026-08-19 | rust/tests/unit/docs_requirements_issue_1021.rs | not yet confirmed |
| R1021-28 | docs/requirements/issue-1021-full-range-coding-and-contribution-artifacts.md | PR #1027 (issue #1021), 2026-08-19 | rust/tests/unit/ci-cd/issue_1021.rs::a_stalled_mirror_is_killed_at_its_own_deadline_and_the_next_attempt_succeeds; ::every_budgeted_retry_in_a_workflow_fits_the_budget_it_runs_under | measured 2026-08-19 against a stand-in `apt-get` that stalls, refuses and recovers; the stall observed in CI is preserved in docs/case-studies/issue-1021/logs/xvfb-install-budget-terminated.log -- see finding 14 |
| R1021-29 | docs/requirements/issue-1021-full-range-coding-and-contribution-artifacts.md | PR #1027 (issue #1021), 2026-08-19 | rust/tests/integration/issue_703_orchestration_followup.rs::timeout_terminates_descendant_processes | measured 2026-08-19: five consecutive local runs pass in ~2.1s each; mutation-verified by spawning the descendant with `process_group(0)`, which the test reports as `still running 5s after the agent timed out`; the CI failure it answers is preserved in docs/case-studies/issue-1021/logs/descendant-timeout-macos-slice8.log -- see finding 15 |
| R1021-30 | docs/requirements/issue-1021-full-range-coding-and-contribution-artifacts.md | PR #1027 (issue #1021), 2026-08-19 | rust/tests/unit/ci-cd/issue_1021.rs::the_deadline_exits_124_and_kills_the_whole_stalled_tree; ::a_command_that_beats_its_deadline_keeps_its_own_status; ::no_committed_script_reaches_for_a_timeout_binary_macos_does_not_have; ::the_deadline_never_expires_before_the_time_it_was_given | measured 2026-08-19: the eleven `ci_cd::issue_1021` tests pass locally in 3.8s; mutation-verified three times -- signalling only the root of the tree leaves the stalled child alive, restoring `timeout "$attempt_seconds"` fails the guard at `scripts/apt-install-with-retry.sh:90`, and reading elapsed time from bash's `SECONDS` alone fails the accuracy test with `a 3s deadline expired after 2.480484781s`. Measured accuracy (`experiments/issue-1021-deadline-precision/measure.sh`): 3.5s on a 3s deadline, 10.8s on a 10s one, never early. Confirmed on macOS 2026-08-19: all eleven tests pass on the macOS core slices of run 32294252392, including slices 15/16 and 16/16, the two that reported `timeout: command not found` in run 32282461075 (`docs/case-studies/issue-1021/logs/macos-deadline-tests-green.log`); the lower-bound test measured 3.835s against its 3s deadline there. See findings 16, 17 and 18 |
| R1021-31 | docs/requirements/issue-1021-full-range-coding-and-contribution-artifacts.md | PR #1027 (issue #1021), 2026-08-20 | rust/tests/unit/issue_1021_behaviour_range.rs::a_coding_request_naming_no_language_is_a_coding_request; ::the_languageless_coding_request_is_answered_in_its_own_language; ::asking_for_code_is_a_coding_request_whatever_the_asking_verb; ::an_asking_verb_alone_is_not_a_coding_request | measured 2026-08-20 with `cargo run --example issue_1021_languageless_probe`; output kept in docs/case-studies/issue-1021/logs/languageless-request-after.log -- the reported bare request is answered with the question about the language rather than with search results, and each of the four subjects-beyond-the-artefact prompts still routes elsewhere; measured again 2026-08-20 with `cargo run --example issue_1021_languageless_followup`, which asks the same bare request and then answers the question it comes back with -- output kept in docs/case-studies/issue-1021/logs/languageless-followup.log, showing the follow-up turn is answered from the catalog rather than asked again (finding 6) |
| R1021-32 | docs/requirements/issue-1021-full-range-coding-and-contribution-artifacts.md | PR #1027 (issue #1021), 2026-08-20 | rust/tests/unit/issue_1021_verified_move.rs::a_move_expands_into_preconditions_preparation_action_and_postconditions; ::a_copy_declares_the_same_shape_with_a_source_that_survives; ::a_destination_in_the_working_directory_prepares_the_working_directory; ::a_command_with_no_declared_effect_is_not_a_recipe; ::a_requested_move_runs_its_checks_around_the_action; ::a_move_onto_an_occupied_destination_stops_before_it_acts; ::a_move_of_a_missing_source_stops_on_the_first_check; rust/tests/integration/issue_749_shell_routing.rs::whole_shell_task_matrix_routes_without_web_search | measured 2026-08-20 end to end against a real filesystem by ladder rungs `824.L1`-`824.L5`, all green; falsified 2026-08-20 before the helpers were updated -- driving the recipe made `rust/tests/unit/issue_749_shell_routing.rs` assert `test -e a.txt` where it expected `cp a.txt b.txt`, which is the observable difference between issuing a command and carrying it out, and both unit matrices and the HTTP matrix now assert the whole recipe rather than its first step |
| R1021-33 | docs/requirements/issue-1021-full-range-coding-and-contribution-artifacts.md | PR #1027 (issue #1021), 2026-08-21 | rust/tests/unit/ci-cd/issue_1021.rs::the_crate_is_on_edition_2024_and_the_judge_compiles_the_same_edition; ::nothing_in_the_tree_reaches_for_a_nightly_toolchain; rust/tests/unit/ci-cd/issue_1014.rs::unix_agent_runner_uses_command_streams_exact_argv_api (the `=0.16.0` pin the refresh had to leave alone); and the whole suite, which is what a dependency refresh is actually tested by | measured 2026-08-21: `cargo build --all-features`, `cargo clippy --all-targets --all-features` and `cargo test --all-features` are green on the refreshed tree, as are the docs.rs profile gate (`check_docs_rs_dependency_profile`), the JavaScript manifest gate (`check_javascript_dependencies`) and `cargo audit`; falsified 2026-08-21 before the code moved -- `sha2` 0.11 produced ten `LowerHex is not satisfied` errors in nine files, which is what distinguishes a dependency that was upgraded from one whose number was changed (finding 28); the stable-only guard was mutation-verified by pointing one workflow's toolchain action at a non-stable channel and watching it fail. Re-measured 2026-08-21 against the registries rather than against the diff, with `python3 experiments/issue-1021-dependency-freshness/check.py`: 32 crates and 30 npm specs, 0 behind newest stable, 5 floating `@link-assistant/` specs skipped by rule. That run is what caught the one miss -- the `browser-commander` override had been taken to 0.16.0 when 0.16.1 had been the newest stable since six hours after it, now corrected and bundle-measured to the same 11,827,516 bytes (finding 31) -- and the tool exists because two hand checks each gave a wrong answer, one by reading a renamed crate's manifest key and one by reading npm's `latest` tag instead of the version list. Manual confirmation of the refreshed tree on a clean runner: not yet confirmed -- CI on this branch is the first unattended run of it |
| R1021-34 | docs/requirements/issue-1021-full-range-coding-and-contribution-artifacts.md | PR #1027 (issue #1021), 2026-08-21 | scripts/check-web-archive.test.mjs -- four tests naming the report shapes the single-heading lookup got wrong: a timeout with no errors section, failures spread across several failing sections, an unrecognised category, and a report with nothing but healthy sections; rust/tests/unit/ci-cd/issue_1017.rs::link_report_parser_is_unit_tested_before_it_is_trusted keeps the workflow running them ahead of lychee, though note it only checks that the file exists and is non-empty, which is why it could not have caught this | falsified 2026-08-21 by running the four new tests against the previous parser: all four fail and the three pre-existing ones pass, which is what shows the gap was the report shape rather than the assertions. Reproduced from the real artefact with `node experiments/issue-1021-link-checker-false-positive/reproduce.mjs`, replaying the report captured verbatim from run 32454084765: 17 URLs reported broken and 16 of them links lychee had classified as healthy redirects before the fix, 1 and 0 after. Measured 2026-08-21: the two links CI timed out on both answer 200 from here in three consecutive requests each (0.67/0.62/0.51s for rowanzellers.com/hellaswag/, 4.34/0.76/0.82s for the Anthropic CLI-usage page), and run 32455788384 checked the same 1285 links with 0 timeouts and passed, so the verdict turned on the timeout and not on the links. Manual confirmation on a runner: not yet confirmed -- the failing shape needs a run in which some link happens to time out, which cannot be summoned on demand; what is confirmed is that the parser now returns the timeout alone from that exact report |
| R1021-35 | docs/requirements/issue-1021-full-range-coding-and-contribution-artifacts.md | PR #1027 (issue #1021), 2026-08-21 | rust/tests/integration/issue_1021_client_preflight.rs::the_anthropic_hello_probe_is_answered_under_the_base_path_a_client_is_given; ::every_published_base_path_answers_a_reachability_probe; ::the_hello_probe_does_not_answer_for_paths_it_does_not_own; and the `E2E (t3code)` and `E2E (claude)` legs of `.github/workflows/agentic-cli-matrix.yml`, which are the gates that reported both failures | falsified 2026-08-21 against the unpatched `rust/src/server.rs`: the hello-probe test fails with ``assertion `left == right` failed: HEAD /api/hello  left: 404  right: 200`` while the other two pass before and after, which is what makes them regression guards for behaviour that already held rather than tests written to match the fix. The upstream endpoint was measured rather than inferred -- `https://api.anthropic.com/api/hello` answers `200` and `{"message": "hello"}` to `GET` and `200` with a 0-byte body to `HEAD`, unauthenticated -- and the mechanism was read verbatim out of the shipped `@anthropic-ai/claude-code-linux-x64` binary, where `preconnectFired` and the `/api/hello` warm-up occur 4 and 1 times in 2.1.238 and 0 and 0 times in 2.1.215. The `t3code` half was reproduced locally under Node 22.23.2: 0.0.28 lists `start serve auth project connect`, 0.0.33 adds `pair` and `service`, and both new subcommands' help text was read to confirm neither opens a prompt path before the contract was re-recorded. Manual confirmation on a runner: not yet confirmed -- the matrix run on the commit carrying this fix is the first unattended check of it |
| R1021-36 | docs/requirements/issue-1021-full-range-coding-and-contribution-artifacts.md | PR #1027 (issue #1021), 2026-08-21 | rust/tests/unit/ci-cd/issue_1021.rs::a_budget_that_expires_reports_the_compiler_cache_counters; ::a_budget_warning_reports_the_counters_without_touching_the_result; ::a_budget_that_wraps_no_compiler_reports_no_counters | falsified 2026-08-21 against the previous `scripts/run-with-budget-warning.sh`: the two positive tests fail there and the negative one passes on both sides, as a guard for already-correct silence should. The finding behind it was measured rather than assumed -- `experiments/issue_1021_compile_rate_compare.py` matched 480 crates against themselves across the red and green job logs of the same shard on the same lockfile and found the red run 2.4x-2.6x slower at every decile, uniformly; the re-run of the identical commit then passed in 620s of the 1200s budget against the green run's 838s, so one piece of work was observed at 620s, 838s and terminated-at-1200s. The dead `macOS-cargo-*` restore was ruled out by measurement too, at 19s and 20s of download against a 1200s budget. Manual confirmation on a runner: partly confirmed -- the silence is, the counters are not. Job 96736754559 ran the same shard at 603s of its 1200s budget, below the warning threshold, and `grep -c '[budget]'` over its log returns 0, so a healthy step stays quiet on a real runner. The counters themselves have still only been printed by the stand-in sccache the tests drive, because no CI step has blown its budget since |
| R1021-37 | docs/requirements/issue-1021-full-range-coding-and-contribution-artifacts.md | PR #1027 (issue #1021), 2026-08-21 | rust/tests/unit/ci-cd/codeql_sink_heuristics.rs::no_function_parameter_is_named_after_a_hard_coded_cryptographic_sink; ::no_logging_macro_is_handed_a_name_that_reads_as_account_information; ::the_scan_skips_the_same_directories_the_codeql_config_ignores; and the nine tests that pin the reader itself, because a guard that silently parses nothing passes for the wrong reason; ::a_parameter_named_salt_is_found_and_the_seed_that_replaced_it_is_not; ::a_salt_in_a_comment_or_a_string_declares_nothing; ::a_multi_line_signature_reports_the_line_of_each_parameter; ::self_and_nested_types_do_not_produce_spurious_names; ::patterns_are_stripped_down_to_the_binding_name; ::an_inline_capture_is_read_out_of_the_format_string; ::placeholders_that_capture_nothing_are_not_mistaken_for_bindings; ::digests_and_hashes_are_not_read_as_account_information; ::a_substring_that_is_not_a_word_is_not_account_information | falsified 2026-08-21 with `bash experiments/issue-1021-codeql-name-heuristics/falsify.sh`, which reverts the renames and runs the guard against the tree that produced the alerts: it names `rust/src/translation/selection.rs:324` and `:352`, `rust/tests/source/translation/selection.rs:297` and `:325`, and `rust/src/cli_improve.rs:84`, which are the exact two mechanisms behind the 99 alerts, and 12 of 12 pass once the renames are back. The heuristics were read at the source rather than inferred from the alert text -- `HeuristicSinks` in the upstream `HardcodedCryptographicValueExtensions.qll` and `HeuristicNames::nameIndicatesSensitiveData` in `SensitiveDataHeuristics.qll` -- and the guard deliberately anchors account names on word segments where upstream matches substrings, because the upstream form would flag `accounted_for` in `rust/examples/issue_559_meta_core.rs`, which CodeQL itself does not report; `a_substring_that_is_not_a_word_is_not_account_information` pins the deviation. Measured 2026-08-21: the branch's 101 open alerts are identical to `main`'s on rule, severity and path, so nothing here was introduced by this pull request -- the check attributed them because 1299 files change (`docs/case-studies/issue-1021/logs/codeql-name-heuristic-alerts.log`). Manual confirmation on a runner: confirmed. The CodeQL analysis of 6149a639f, the commit carrying the rename, is green on both legs -- `CodeQL (rust)` job 96784436271 and `CodeQL (actions)` job 96784436191 -- and the aggregate `CodeQL` check that had reported *99 new alerts including 98 critical severity security vulnerabilities* now reports *No new alerts in code changed by this pull request* (run 96784677379). Open alerts on the branch went 101 to 2 with 0 critical, while `main` still carries all 101, which is what shows the count moved because the code changed rather than because the query did; the 2 that remain are the `rust/cleartext-logging` alerts on the real Agent CLI session ids in `rust/tests/unit/docs_requirements_issue_917.rs` and `_918.rs`, expected and left open by design. Nothing was dismissed and no alert was suppressed (`docs/case-studies/issue-1021/logs/codeql-name-heuristic-alerts.log`). The same run found what the rename had left behind: four `data/meta/self-ast/` census documents keyed by content id went stale, failing the `Check self-AST census freshness` step and `issue_673_self_ast_census::committed_census_documents_match_what_the_sources_render`, and `cargo run --example regenerate_self_ast_census` rewrote exactly those four |
| R1073-1 | docs/requirements/issue-1073-reasoning-standard.md | PR #1074 (issue #1073), 2026-09-04 | rust/tests/unit/issue_1073_reasoning_standard.rs::depth_floor_enumerates_every_gate_even_for_a_trivial_episode; ::the_depth_floor_holds_for_the_smallest_request_the_pipeline_can_formalize; rust/tests/unit/specification/reasoning_standard_meta_algorithm.rs::the_meta_core_runs_the_audit_with_no_mode_in_front_of_it; rust/tests/unit/specification/meta_construction.rs::both_directions_are_the_default_depth_floor; rust/tests/unit/specification/selection.rs::record_is_the_default_and_off_emits_no_artifact; rust/tests/unit/specification/skill_ledger.rs::accumulate_is_the_default_and_off_records_nothing | measured 2026-09-04 with `cargo run --example dump_reasoning_standard_audit` (docs/case-studies/issue-1073/logs/reasoning-standard-audit.log): the trivial request `"hi"` -- which triggers no world claim, no source, no conclusion and no action -- still has all seven declared gates enumerated, six reporting `not_triggered` with the trigger that was false and `instruction_formalization` reporting `violated` with the two sources it is missing named, and closes at `not_confirmed_not_refuted` with its blockers named |
| R1073-2 | docs/requirements/issue-1073-reasoning-standard.md | PR #1074 (issue #1073), 2026-09-04 | rust/tests/unit/issue_1073_reasoning_standard.rs::gathered_instructions_are_compiled_into_checkable_steps | measured 2026-09-04 (docs/case-studies/issue-1073/logs/reasoning-standard-audit.log): the trivial request's `instruction_formalization` gate fires on its task class alone and reports `violated` with `courtesy:no_instructions_gathered` and `instruction_sources:0:required:2`, while the reference dialog, which gathered them, reports `satisfied` |
| R1073-3 | docs/requirements/issue-1073-reasoning-standard.md | PR #1074 (issue #1073), 2026-09-04 | rust/tests/unit/issue_1073_reasoning_standard.rs::primary_documentation_is_required_by_default | measured 2026-09-04 (docs/case-studies/issue-1073/logs/reasoning-standard-audit.log): the reference dialog's `documentation_default` gate reports `satisfied`; nothing in the episode had to ask for documentation to be consulted |
| R1073-4 | docs/requirements/issue-1073-reasoning-standard.md | PR #1074 (issue #1073), 2026-09-04 | rust/tests/unit/issue_1073_reasoning_standard.rs::source_trust_is_derived_from_the_primacy_chain | measured 2026-09-04: all thirteen sources in `data/seed/sources-registry.lino` derive the tier they previously asserted, and the four that asserted nothing (`wikidata`, `wiktionary`, `wordnet`, `wikipedia`) now derive `independent_corroboration` through `DerivationReason::NamedUpstreamChain` rather than through the old silent `_` fallback |
| R1073-5 | docs/requirements/issue-1073-reasoning-standard.md | PR #1074 (issue #1073), 2026-09-04 | rust/tests/unit/issue_1073_reasoning_standard.rs::conclusions_need_varied_refutations_before_they_may_be_leaned_toward | measured 2026-09-04 (docs/case-studies/issue-1073/logs/reasoning-standard-audit.log): the reference dialog's `refutation_variety` gate reports `satisfied`, and the trivial request, which reached no conclusion, is blocked at `impulse_08ba5f07b55ec3da:no_conclusion_recorded` instead of leaning toward one |
| R1073-6 | docs/requirements/issue-1073-reasoning-standard.md | PR #1074 (issue #1073), 2026-09-04 | rust/tests/unit/issue_1073_reasoning_standard.rs::the_standard_is_a_formal_procedure_that_replays_without_a_model; rust/tests/unit/specification/reasoning_standard_meta_algorithm.rs (five grounding tests over data/meta/reasoning-standard-recipe.lino) | measured 2026-09-04 (docs/case-studies/issue-1073/logs/reasoning-standard-audit.log): both ledgers are produced by `standard()` reading `data/meta/reasoning-standard.lino` and `audit()` evaluating it, with no model in the loop and no network call |
| R1073-7 | docs/requirements/issue-1073-reasoning-standard.md | PR #1074 (issue #1073), 2026-09-04 | rust/tests/unit/issue_1073_reasoning_standard.rs::the_reference_dialog_passes_and_each_adopted_behaviour_is_load_bearing | measured 2026-09-04 with `cargo run --example dump_reasoning_standard_audit` (docs/case-studies/issue-1073/logs/reasoning-standard-audit.log): the reference episode clears all seven gates with verdict `confirmed` |
| R1085-1 | docs/requirements/issue-1085-the-links-network-is-not-the-system-that-reasons.md | issue-1085 branch, 2026-09-08 | superseded: the kernel/non-kernel split and the Rust-line ceiling are withdrawn (see REQUIREMENTS.md R1085-1) | withdrawn |
| R1085-4 | docs/requirements/issue-1085-the-links-network-is-not-the-system-that-reasons.md | issue-1085 branch, 2026-09-08 | rust/tests/unit/specification/self_hosting_metric.rs (fixtures carry Formal-AI-Model); rust/tests/unit/specification/self_hosting_metric/retraction.rs | not yet confirmed |
| R1085-5 | docs/requirements/issue-1085-the-links-network-is-not-the-system-that-reasons.md | issue-1085 branch, 2026-09-08 | rust/tests/unit/specification/self_hosting_metric.rs::captured_artifacts_and_lockfiles_do_not_move_the_metric | not yet confirmed |
| R1085-6 | docs/requirements/issue-1085-the-links-network-is-not-the-system-that-reasons.md | issue-1085 branch, 2026-09-08 | rust/tests/unit/specification/self_hosting_metric.rs::release_target_ratchets_and_records_each_self_authored_pull_request | not yet confirmed |
| R1085-7 | docs/requirements/issue-1085-the-links-network-is-not-the-system-that-reasons.md | issue-1085 branch, 2026-09-08 | scripts/self-hosting-replay.rs (exercised by self-development-status.yml) | not yet confirmed |
| R1085-8 | docs/requirements/issue-1085-the-links-network-is-not-the-system-that-reasons.md | issue-1085 branch, 2026-09-08 | rust/tests/unit/ci-cd/issue_1014.rs::an_ineligible_cycle_is_reported_red_without_gating_the_release; rust/tests/unit/specification/self_hosting_metric.rs::release_pipeline_and_ledger_remain_pinned_to_the_metric | not yet confirmed |
| R1085-9 | docs/requirements/issue-1085-the-links-network-is-not-the-system-that-reasons.md | issue-1085 branch, 2026-09-08 | experiments/issue_1028_agent_cli_ladder/verify-node.sh (uncompilable_leaf_change) | not yet confirmed |
| R1085-10 | docs/requirements/issue-1085-the-links-network-is-not-the-system-that-reasons.md | delivered 2026-09-18: every curated 13/13 citation carries the upstream rows beside it (VISION, ROADMAP pillars 25/26 and the #922 section, README Measured Today, ARCHITECTURE D5.4 paragraph) | rust/tests/unit/docs_benchmarks.rs::latest_external_rows_are_published_from_the_ledger; ::curated_pass_ratios_publish_an_upstream_comparison_beside_them | not yet confirmed |
| R1085-12 | docs/requirements/issue-1085-the-links-network-is-not-the-system-that-reasons.md | issue-1085 branch, 2026-09-08 | rust/tests/unit/ci-cd/issue_1081/release_preflight.rs::the_crates_io_token_is_never_judged_by_the_cookie_only_me_endpoint | manual: crates.io /api/v1/me answered 403 to a bogus token and to no token alike on 2026-09-08; source `AuthCheck::only_cookie()` |
| R1085-13 | docs/requirements/issue-1085-the-links-network-is-not-the-system-that-reasons.md | issue-1085 branch, 2026-09-08 | rust/tests/unit/ci-cd/issue_1012.rs; rust/tests/unit/ci-cd/issue_1081.rs (budget share) | not yet confirmed |
| R97-R100 | docs/requirements/issue-0016-follow-up-universal-data-seed-across-every-interface-pr-17-reopen.md | none recorded | none recorded | not yet confirmed |
| R278-R281 | docs/requirements/issue-0398-pr-review-standards-comment-4663407299.md | none recorded | none recorded | not yet confirmed |
| R453-M1 | docs/requirements/issue-0453-moonshot-splitting.md | none recorded | rust/tests/unit/issue_1138_selection_heuristics.rs | not yet confirmed |
| R453-M2 | docs/requirements/issue-0453-moonshot-splitting.md | none recorded | none recorded | not yet confirmed |
| R453-M3 | docs/requirements/issue-0453-moonshot-splitting.md | none recorded | none recorded | not yet confirmed |
| R453-M4 | docs/requirements/issue-0453-moonshot-splitting.md | none recorded | none recorded | not yet confirmed |
| R491-1 | docs/requirements/issue-0491-least-action-continuation.md | none recorded | none recorded | not yet confirmed |
| R491-C1 | docs/requirements/issue-0491-least-action-continuation.md | none recorded | none recorded | not yet confirmed |
| R491-C2 | docs/requirements/issue-0491-least-action-continuation.md | none recorded | none recorded | not yet confirmed |
| R491-C3 | docs/requirements/issue-0491-least-action-continuation.md | none recorded | none recorded | not yet confirmed |
| R491-C4 | docs/requirements/issue-0491-least-action-continuation.md | none recorded | none recorded | not yet confirmed |
| R531-01 | docs/requirements/issue-0531-pattern-inference-research.md | none recorded | none recorded | not yet confirmed |
| R480-R483 | docs/requirements/issue-0538-detailed-meanings-and-words.md | none recorded | none recorded | not yet confirmed |
| R558-01 | docs/requirements/issue-0558-auto-learning.md | none recorded | rust/tests/unit/issue_558_self_healing.rs | not yet confirmed |
| R558-05 | docs/requirements/issue-0558-auto-learning.md | none recorded | rust/tests/unit/issue_558_self_healing.rs | not yet confirmed |
| R558-12 | docs/requirements/issue-0558-auto-learning.md | none recorded | none recorded | not yet confirmed |
| R345-R354 | docs/requirements/issue-0563-repository-resource-summarization.md | none recorded | none recorded | not yet confirmed |
| R355-R359 | docs/requirements/issue-0563-repository-resource-summarization.md | none recorded | none recorded | not yet confirmed |
| R649-01 | docs/requirements/issue-0649-world-models-and-contexts.md | none recorded | none recorded | not yet confirmed |
| R649-14 | docs/requirements/issue-0649-world-models-and-contexts.md | none recorded | none recorded | not yet confirmed |
| R649-15 | docs/requirements/issue-0649-world-models-and-contexts.md | none recorded | none recorded | not yet confirmed |
| R649-19 | docs/requirements/issue-0649-world-models-and-contexts.md | none recorded | none recorded | not yet confirmed |
| R686-01 | docs/requirements/issue-0686-associative-knowledge-networks-learning.md | none recorded | none recorded | not yet confirmed |
| R686-18 | docs/requirements/issue-0686-associative-knowledge-networks-learning.md | none recorded | none recorded | not yet confirmed |
| R705-1 | docs/requirements/issue-0705-anticipatory-dreaming.md | none recorded | none recorded | not yet confirmed |
| R705-2 | docs/requirements/issue-0705-anticipatory-dreaming.md | none recorded | none recorded | not yet confirmed |
| R705-3 | docs/requirements/issue-0705-anticipatory-dreaming.md | none recorded | none recorded | not yet confirmed |
| R705-4 | docs/requirements/issue-0705-anticipatory-dreaming.md | none recorded | none recorded | not yet confirmed |
| R705-5 | docs/requirements/issue-0705-anticipatory-dreaming.md | none recorded | none recorded | not yet confirmed |
| R705-6 | docs/requirements/issue-0705-anticipatory-dreaming.md | none recorded | none recorded | not yet confirmed |
| R705-7 | docs/requirements/issue-0705-anticipatory-dreaming.md | none recorded | none recorded | not yet confirmed |
| R705-8 | docs/requirements/issue-0705-anticipatory-dreaming.md | none recorded | rust/tests/unit/issue_705_anticipation.rs | not yet confirmed |
| R710-01 | docs/requirements/issue-0710-dropped-requirements-re-verification.md | none recorded | none recorded | not yet confirmed |
| R710-02 | docs/requirements/issue-0710-dropped-requirements-re-verification.md | none recorded | none recorded | not yet confirmed |
| R710-03 | docs/requirements/issue-0710-dropped-requirements-re-verification.md | none recorded | none recorded | not yet confirmed |
| R710-04 | docs/requirements/issue-0710-dropped-requirements-re-verification.md | none recorded | none recorded | not yet confirmed |
| R710-05 | docs/requirements/issue-0710-dropped-requirements-re-verification.md | none recorded | none recorded | not yet confirmed |
| R710-06 | docs/requirements/issue-0710-dropped-requirements-re-verification.md | none recorded | none recorded | not yet confirmed |
| R710-07 | docs/requirements/issue-0710-dropped-requirements-re-verification.md | none recorded | none recorded | not yet confirmed |
| R710-08 | docs/requirements/issue-0710-dropped-requirements-re-verification.md | none recorded | none recorded | not yet confirmed |
| R710-09 | docs/requirements/issue-0710-dropped-requirements-re-verification.md | none recorded | none recorded | not yet confirmed |
| R710-10 | docs/requirements/issue-0710-dropped-requirements-re-verification.md | none recorded | none recorded | not yet confirmed |
| R710-11 | docs/requirements/issue-0710-dropped-requirements-re-verification.md | none recorded | none recorded | not yet confirmed |
| R710-12 | docs/requirements/issue-0710-dropped-requirements-re-verification.md | none recorded | none recorded | not yet confirmed |
| R710-13 | docs/requirements/issue-0710-dropped-requirements-re-verification.md | none recorded | none recorded | not yet confirmed |
| R710-14 | docs/requirements/issue-0710-dropped-requirements-re-verification.md | none recorded | none recorded | not yet confirmed |
| R710-15 | docs/requirements/issue-0710-dropped-requirements-re-verification.md | none recorded | none recorded | not yet confirmed |
| R710-16 | docs/requirements/issue-0710-dropped-requirements-re-verification.md | none recorded | none recorded | not yet confirmed |
| R710-17 | docs/requirements/issue-0710-dropped-requirements-re-verification.md | none recorded | none recorded | not yet confirmed |
| R710-18 | docs/requirements/issue-0710-dropped-requirements-re-verification.md | none recorded | none recorded | not yet confirmed |
| R710-19 | docs/requirements/issue-0710-dropped-requirements-re-verification.md | none recorded | none recorded | not yet confirmed |
| R710-20 | docs/requirements/issue-0710-dropped-requirements-re-verification.md | none recorded | none recorded | not yet confirmed |
| R710-21 | docs/requirements/issue-0710-dropped-requirements-re-verification.md | none recorded | none recorded | not yet confirmed |
| R710-22 | docs/requirements/issue-0710-dropped-requirements-re-verification.md | none recorded | none recorded | not yet confirmed |
| R710-23 | docs/requirements/issue-0710-dropped-requirements-re-verification.md | none recorded | none recorded | not yet confirmed |
| R710-24 | docs/requirements/issue-0710-dropped-requirements-re-verification.md | none recorded | none recorded | not yet confirmed |
| R710-25 | docs/requirements/issue-0710-dropped-requirements-re-verification.md | none recorded | none recorded | not yet confirmed |
| R710-26 | docs/requirements/issue-0710-dropped-requirements-re-verification.md | none recorded | none recorded | not yet confirmed |
| R710-27 | docs/requirements/issue-0710-dropped-requirements-re-verification.md | none recorded | none recorded | not yet confirmed |
| R710-28 | docs/requirements/issue-0710-dropped-requirements-re-verification.md | none recorded | none recorded | not yet confirmed |
| R710-29 | docs/requirements/issue-0710-dropped-requirements-re-verification.md | none recorded | none recorded | not yet confirmed |
| R710-30 | docs/requirements/issue-0710-dropped-requirements-re-verification.md | none recorded | none recorded | not yet confirmed |
| R710-31 | docs/requirements/issue-0710-dropped-requirements-re-verification.md | none recorded | none recorded | not yet confirmed |
| R710-32 | docs/requirements/issue-0710-dropped-requirements-re-verification.md | none recorded | none recorded | not yet confirmed |
| R710-D17 | docs/requirements/issue-0710-dynamic-coding-discovery.md | none recorded | rust/tests/unit/coding_discovery/ledger.rs | not yet confirmed |
| R710-R1 | docs/requirements/issue-0710-repository-and-retention-continuation.md | none recorded | none recorded | not yet confirmed |
| R710-R2 | docs/requirements/issue-0710-repository-and-retention-continuation.md | none recorded | none recorded | not yet confirmed |
| R710-R3 | docs/requirements/issue-0710-repository-and-retention-continuation.md | none recorded | none recorded | not yet confirmed |
| R710-R4 | docs/requirements/issue-0710-repository-and-retention-continuation.md | none recorded | none recorded | not yet confirmed |
| R710-R5 | docs/requirements/issue-0710-repository-and-retention-continuation.md | none recorded | none recorded | not yet confirmed |
| R710-R6 | docs/requirements/issue-0710-repository-and-retention-continuation.md | none recorded | none recorded | not yet confirmed |
| R710-R7 | docs/requirements/issue-0710-repository-and-retention-continuation.md | none recorded | none recorded | not yet confirmed |
| R710-R8 | docs/requirements/issue-0710-repository-and-retention-continuation.md | none recorded | none recorded | not yet confirmed |
| R710-R9 | docs/requirements/issue-0710-repository-and-retention-continuation.md | none recorded | none recorded | not yet confirmed |
| R710-R10 | docs/requirements/issue-0710-repository-and-retention-continuation.md | none recorded | none recorded | not yet confirmed |
| R802-1 | docs/requirements/issue-0802-hypothesis-search.md | none recorded | none recorded | not yet confirmed |
| R802-2 | docs/requirements/issue-0802-hypothesis-search.md | none recorded | none recorded | not yet confirmed |
| R802-3 | docs/requirements/issue-0802-hypothesis-search.md | none recorded | none recorded | not yet confirmed |
| R802-4 | docs/requirements/issue-0802-hypothesis-search.md | none recorded | none recorded | not yet confirmed |
| R56kfQp | docs/requirements/issue-0848-executable-coding-tasks.md | none recorded | none recorded | not yet confirmed |
| R873-1 | docs/requirements/issue-0873-research-driven-unknown-recovery.md | none recorded | rust/tests/unit/issue_873.rs | not yet confirmed |
| R873-2 | docs/requirements/issue-0873-research-driven-unknown-recovery.md | none recorded | none recorded | not yet confirmed |
| R873-3 | docs/requirements/issue-0873-research-driven-unknown-recovery.md | none recorded | none recorded | not yet confirmed |
| R873-4 | docs/requirements/issue-0873-research-driven-unknown-recovery.md | none recorded | none recorded | not yet confirmed |
| R873-5 | docs/requirements/issue-0873-research-driven-unknown-recovery.md | none recorded | none recorded | not yet confirmed |
| R873-6 | docs/requirements/issue-0873-research-driven-unknown-recovery.md | none recorded | none recorded | not yet confirmed |
| R873-7 | docs/requirements/issue-0873-research-driven-unknown-recovery.md | none recorded | none recorded | not yet confirmed |
| R873-8 | docs/requirements/issue-0873-research-driven-unknown-recovery.md | none recorded | none recorded | not yet confirmed |
| R873-9 | docs/requirements/issue-0873-research-driven-unknown-recovery.md | none recorded | none recorded | not yet confirmed |
| R873-10 | docs/requirements/issue-0873-research-driven-unknown-recovery.md | none recorded | none recorded | not yet confirmed |
| R895-1 | docs/requirements/issue-0895-coverage-publication-and-ratchet.md | none recorded | none recorded | not yet confirmed |
| R895-2 | docs/requirements/issue-0895-coverage-publication-and-ratchet.md | none recorded | none recorded | not yet confirmed |
| R895-3 | docs/requirements/issue-0895-coverage-publication-and-ratchet.md | none recorded | none recorded | not yet confirmed |
| R895-4 | docs/requirements/issue-0895-coverage-publication-and-ratchet.md | none recorded | none recorded | not yet confirmed |
| R895-5 | docs/requirements/issue-0895-coverage-publication-and-ratchet.md | none recorded | none recorded | not yet confirmed |
| R901-1 | docs/requirements/issue-0901-triz-contradictions.md | none recorded | rust/tests/unit/issue_1138_selection_heuristics.rs | not yet confirmed |
| R901-2 | docs/requirements/issue-0901-triz-contradictions.md | none recorded | none recorded | not yet confirmed |
| R901-3 | docs/requirements/issue-0901-triz-contradictions.md | none recorded | none recorded | not yet confirmed |
| R901-4 | docs/requirements/issue-0901-triz-contradictions.md | none recorded | rust/tests/unit/issue_1138_selection_heuristics.rs | not yet confirmed |
| R901-5 | docs/requirements/issue-0901-triz-contradictions.md | none recorded | none recorded | not yet confirmed |
| R916-08a | docs/requirements/issue-0909-headless-ready-global-client-configuration.md | none recorded | none recorded | not yet confirmed |
| R916-08b | docs/requirements/issue-0909-headless-ready-global-client-configuration.md | none recorded | none recorded | not yet confirmed |
| R917-1 | docs/requirements/issue-0917-general-natural-formal-translation.md | none recorded | none recorded | not yet confirmed |
| R917-2 | docs/requirements/issue-0917-general-natural-formal-translation.md | none recorded | none recorded | not yet confirmed |
| R917-3 | docs/requirements/issue-0917-general-natural-formal-translation.md | none recorded | none recorded | not yet confirmed |
| R917-4 | docs/requirements/issue-0917-general-natural-formal-translation.md | none recorded | none recorded | not yet confirmed |
| R917-5 | docs/requirements/issue-0917-general-natural-formal-translation.md | none recorded | none recorded | not yet confirmed |
| R917-6 | docs/requirements/issue-0917-general-natural-formal-translation.md | none recorded | none recorded | not yet confirmed |
| R917-7 | docs/requirements/issue-0917-general-natural-formal-translation.md | none recorded | none recorded | not yet confirmed |
| R918-1 | docs/requirements/issue-0918-minimal-core-boundary-and-seed-metadata-audit.md | none recorded | none recorded | not yet confirmed |
| R918-2 | docs/requirements/issue-0918-minimal-core-boundary-and-seed-metadata-audit.md | none recorded | none recorded | not yet confirmed |
| R918-3 | docs/requirements/issue-0918-minimal-core-boundary-and-seed-metadata-audit.md | none recorded | none recorded | not yet confirmed |
| R918-4 | docs/requirements/issue-0918-minimal-core-boundary-and-seed-metadata-audit.md | none recorded | none recorded | not yet confirmed |
| R918-5 | docs/requirements/issue-0918-minimal-core-boundary-and-seed-metadata-audit.md | none recorded | none recorded | not yet confirmed |
| R918-6 | docs/requirements/issue-0918-minimal-core-boundary-and-seed-metadata-audit.md | none recorded | none recorded | not yet confirmed |
| R919-1 | docs/requirements/issue-0919-research-driven-coding-procedures.md | none recorded | rust/tests/unit/issue_919.rs | not yet confirmed |
| R919-2 | docs/requirements/issue-0919-research-driven-coding-procedures.md | none recorded | none recorded | not yet confirmed |
| R919-3 | docs/requirements/issue-0919-research-driven-coding-procedures.md | none recorded | none recorded | not yet confirmed |
| R919-4 | docs/requirements/issue-0919-research-driven-coding-procedures.md | none recorded | none recorded | not yet confirmed |
| R919-5 | docs/requirements/issue-0919-research-driven-coding-procedures.md | none recorded | none recorded | not yet confirmed |
| R919-6 | docs/requirements/issue-0919-research-driven-coding-procedures.md | none recorded | none recorded | not yet confirmed |
| R916-09 | docs/requirements/issue-0921-hive-mind-full-circle-integration-gate.md | none recorded | rust/tests/unit/issue_907.rs | not yet confirmed |
| R916-10 | docs/requirements/issue-0921-hive-mind-full-circle-integration-gate.md | none recorded | rust/tests/unit/issue_907.rs | not yet confirmed |
| R921-1 | docs/requirements/issue-0921-hive-mind-full-circle-integration-gate.md | none recorded | none recorded | not yet confirmed |
| R921-2 | docs/requirements/issue-0921-hive-mind-full-circle-integration-gate.md | none recorded | none recorded | not yet confirmed |
| R921-3 | docs/requirements/issue-0921-hive-mind-full-circle-integration-gate.md | none recorded | none recorded | not yet confirmed |
| R921-4 | docs/requirements/issue-0921-hive-mind-full-circle-integration-gate.md | none recorded | none recorded | not yet confirmed |
| R921-5 | docs/requirements/issue-0921-hive-mind-full-circle-integration-gate.md | none recorded | none recorded | not yet confirmed |
| R921-6 | docs/requirements/issue-0921-hive-mind-full-circle-integration-gate.md | none recorded | rust/tests/unit/issue_907.rs | not yet confirmed |
| R921-7 | docs/requirements/issue-0921-hive-mind-full-circle-integration-gate.md | none recorded | rust/tests/unit/issue_904.rs | not yet confirmed |
| R921-8 | docs/requirements/issue-0921-hive-mind-full-circle-integration-gate.md | none recorded | none recorded | not yet confirmed |
| R922-1 | docs/requirements/issue-0922-method-learning-from-experience.md | none recorded | none recorded | not yet confirmed |
| R922-2 | docs/requirements/issue-0922-method-learning-from-experience.md | none recorded | none recorded | not yet confirmed |
| R922-3 | docs/requirements/issue-0922-method-learning-from-experience.md | none recorded | none recorded | not yet confirmed |
| R922-4 | docs/requirements/issue-0922-method-learning-from-experience.md | none recorded | none recorded | not yet confirmed |
| R922-5 | docs/requirements/issue-0922-method-learning-from-experience.md | none recorded | none recorded | not yet confirmed |
| R922-6 | docs/requirements/issue-0922-method-learning-from-experience.md | none recorded | none recorded | not yet confirmed |
| R923-1 | docs/requirements/issue-0923-formal-reasoning-coverage-growth.md | none recorded | none recorded | not yet confirmed |
| R923-2 | docs/requirements/issue-0923-formal-reasoning-coverage-growth.md | none recorded | none recorded | not yet confirmed |
| R923-3 | docs/requirements/issue-0923-formal-reasoning-coverage-growth.md | none recorded | none recorded | not yet confirmed |
| R923-4 | docs/requirements/issue-0923-formal-reasoning-coverage-growth.md | none recorded | none recorded | not yet confirmed |
| R923-5 | docs/requirements/issue-0923-formal-reasoning-coverage-growth.md | none recorded | none recorded | not yet confirmed |
| R924-1 | docs/requirements/issue-0924-formal-ai-self-development-loop.md | none recorded | none recorded | not yet confirmed |
| R924-2 | docs/requirements/issue-0924-formal-ai-self-development-loop.md | none recorded | none recorded | not yet confirmed |
| R924-3 | docs/requirements/issue-0924-formal-ai-self-development-loop.md | none recorded | none recorded | not yet confirmed |
| R924-4 | docs/requirements/issue-0924-formal-ai-self-development-loop.md | none recorded | none recorded | not yet confirmed |
| R924-5 | docs/requirements/issue-0924-formal-ai-self-development-loop.md | none recorded | none recorded | not yet confirmed |
| R924-6 | docs/requirements/issue-0924-formal-ai-self-development-loop.md | none recorded | none recorded | not yet confirmed |
| R924-7 | docs/requirements/issue-0924-formal-ai-self-development-loop.md | none recorded | none recorded | not yet confirmed |
| R924-8 | docs/requirements/issue-0924-formal-ai-self-development-loop.md | none recorded | none recorded | not yet confirmed |
| R924-9 | docs/requirements/issue-0924-formal-ai-self-development-loop.md | none recorded | none recorded | not yet confirmed |
| R924-10 | docs/requirements/issue-0924-formal-ai-self-development-loop.md | none recorded | none recorded | not yet confirmed |
| R924-11 | docs/requirements/issue-0924-formal-ai-self-development-loop.md | none recorded | none recorded | not yet confirmed |
| R931-1 | docs/requirements/issue-0931-local-transports.md | none recorded | none recorded | not yet confirmed |
| R931-2 | docs/requirements/issue-0931-local-transports.md | none recorded | none recorded | not yet confirmed |
| R931-3 | docs/requirements/issue-0931-local-transports.md | none recorded | none recorded | not yet confirmed |
| R931-4 | docs/requirements/issue-0931-local-transports.md | none recorded | none recorded | not yet confirmed |
| R931-5 | docs/requirements/issue-0931-local-transports.md | none recorded | none recorded | not yet confirmed |
| R931-6 | docs/requirements/issue-0931-local-transports.md | none recorded | none recorded | not yet confirmed |
| R931-7 | docs/requirements/issue-0931-local-transports.md | none recorded | none recorded | not yet confirmed |
| R931-8 | docs/requirements/issue-0931-local-transports.md | none recorded | none recorded | not yet confirmed |
| R931-9 | docs/requirements/issue-0931-local-transports.md | none recorded | none recorded | not yet confirmed |
| R931-10 | docs/requirements/issue-0931-local-transports.md | none recorded | none recorded | not yet confirmed |
| R931-11 | docs/requirements/issue-0931-local-transports.md | none recorded | none recorded | not yet confirmed |
| R931-12 | docs/requirements/issue-0931-local-transports.md | none recorded | none recorded | not yet confirmed |
| R932-1 | docs/requirements/issue-0932-box-language-projects.md | none recorded | none recorded | not yet confirmed |
| R932-2 | docs/requirements/issue-0932-box-language-projects.md | none recorded | none recorded | not yet confirmed |
| R932-3 | docs/requirements/issue-0932-box-language-projects.md | none recorded | none recorded | not yet confirmed |
| R932-4 | docs/requirements/issue-0932-box-language-projects.md | none recorded | rust/tests/unit/ci-cd/issue_932.rs | not yet confirmed |
| R932-5 | docs/requirements/issue-0932-box-language-projects.md | none recorded | rust/tests/integration/issue_932_box_language_projects.rs | not yet confirmed |
| R932-6 | docs/requirements/issue-0932-box-language-projects.md | none recorded | none recorded | not yet confirmed |
| R932-7 | docs/requirements/issue-0932-box-language-projects.md | none recorded | rust/tests/unit/issue_932_box_language_projects.rs | not yet confirmed |
| R932-8 | docs/requirements/issue-0932-box-language-projects.md | none recorded | none recorded | not yet confirmed |
| R932-9 | docs/requirements/issue-0932-box-language-projects.md | none recorded | none recorded | not yet confirmed |
| R932-10 | docs/requirements/issue-0932-box-language-projects.md | none recorded | none recorded | not yet confirmed |
| R932-11 | docs/requirements/issue-0932-box-language-projects.md | none recorded | none recorded | not yet confirmed |
| R932-12 | docs/requirements/issue-0932-box-language-projects.md | none recorded | rust/tests/unit/issue_932_self_authoring.rs | not yet confirmed |
| R932-13 | docs/requirements/issue-0932-box-language-projects.md | none recorded | rust/tests/unit/installation_conversion.rs | not yet confirmed |
| R234-2 | docs/requirements/issue-0933-conversational-variation-floor.md | none recorded | rust/tests/unit/conversational_variations.rs | not yet confirmed |
| R933-1 | docs/requirements/issue-0933-conversational-variation-floor.md | none recorded | none recorded | not yet confirmed |
| R933-2 | docs/requirements/issue-0933-conversational-variation-floor.md | none recorded | none recorded | not yet confirmed |
| R933-3 | docs/requirements/issue-0933-conversational-variation-floor.md | none recorded | none recorded | not yet confirmed |
| R933-4 | docs/requirements/issue-0933-conversational-variation-floor.md | none recorded | none recorded | not yet confirmed |
| R933-5 | docs/requirements/issue-0933-conversational-variation-floor.md | none recorded | none recorded | not yet confirmed |
| R933-6 | docs/requirements/issue-0933-conversational-variation-floor.md | none recorded | none recorded | not yet confirmed |
| R933-7 | docs/requirements/issue-0933-conversational-variation-floor.md | none recorded | none recorded | not yet confirmed |
| R933-8 | docs/requirements/issue-0933-conversational-variation-floor.md | none recorded | none recorded | not yet confirmed |
| R933-9 | docs/requirements/issue-0933-conversational-variation-floor.md | none recorded | none recorded | not yet confirmed |
| R933-10 | docs/requirements/issue-0933-conversational-variation-floor.md | none recorded | none recorded | not yet confirmed |
| R933-11 | docs/requirements/issue-0933-conversational-variation-floor.md | none recorded | rust/tests/unit/conversational_variations.rs | not yet confirmed |
| R933-12 | docs/requirements/issue-0933-conversational-variation-floor.md | none recorded | rust/tests/unit/issue_933_answer_parity.rs | not yet confirmed |
| R933-13 | docs/requirements/issue-0933-conversational-variation-floor.md | none recorded | rust/tests/unit/issue_933_self_authoring.rs | not yet confirmed |
| R933-14 | docs/requirements/issue-0933-conversational-variation-floor.md | none recorded | none recorded | not yet confirmed |
| R936-1 | docs/requirements/issue-0936-substitution-rule-compilation.md | none recorded | none recorded | not yet confirmed |
| R936-2 | docs/requirements/issue-0936-substitution-rule-compilation.md | none recorded | none recorded | not yet confirmed |
| R936-3 | docs/requirements/issue-0936-substitution-rule-compilation.md | none recorded | none recorded | not yet confirmed |
| R936-4 | docs/requirements/issue-0936-substitution-rule-compilation.md | none recorded | none recorded | not yet confirmed |
| R936-5 | docs/requirements/issue-0936-substitution-rule-compilation.md | none recorded | none recorded | not yet confirmed |
| R936-6 | docs/requirements/issue-0936-substitution-rule-compilation.md | none recorded | none recorded | not yet confirmed |
| R936-7 | docs/requirements/issue-0936-substitution-rule-compilation.md | none recorded | none recorded | not yet confirmed |
| R222-1 | docs/requirements/issue-0960-enforcing-recorded-but-unenforced-conventions.md | none recorded | none recorded | not yet confirmed |
| R234-4 | docs/requirements/issue-0960-enforcing-recorded-but-unenforced-conventions.md | none recorded | none recorded | not yet confirmed |
| R960-1 | docs/requirements/issue-0960-enforcing-recorded-but-unenforced-conventions.md | none recorded | rust/tests/unit/data_files.rs | not yet confirmed |
| R960-2 | docs/requirements/issue-0960-enforcing-recorded-but-unenforced-conventions.md | none recorded | none recorded | not yet confirmed |
| R960-3 | docs/requirements/issue-0960-enforcing-recorded-but-unenforced-conventions.md | none recorded | rust/tests/unit/total_closure.rs | not yet confirmed |
| R960-4 | docs/requirements/issue-0960-enforcing-recorded-but-unenforced-conventions.md | none recorded | rust/tests/unit/assistant_name.rs | not yet confirmed |
| R960-5 | docs/requirements/issue-0960-enforcing-recorded-but-unenforced-conventions.md | none recorded | none recorded | not yet confirmed |
| R960-6 | docs/requirements/issue-0960-enforcing-recorded-but-unenforced-conventions.md | none recorded | none recorded | not yet confirmed |
| R961-1 | docs/requirements/issue-0961-macos-ci-parity.md | none recorded | none recorded | not yet confirmed |
| R961-2 | docs/requirements/issue-0961-macos-ci-parity.md | none recorded | none recorded | not yet confirmed |
| R961-3 | docs/requirements/issue-0961-macos-ci-parity.md | none recorded | rust/tests/integration/pty.rs | not yet confirmed |
| R961-4 | docs/requirements/issue-0961-macos-ci-parity.md | none recorded | none recorded | not yet confirmed |
| R961-5 | docs/requirements/issue-0961-macos-ci-parity.md | none recorded | none recorded | not yet confirmed |
| R961-6 | docs/requirements/issue-0961-macos-ci-parity.md | none recorded | rust/tests/issue_961_macos_portability.rs | not yet confirmed |
| R982-1 | docs/requirements/issue-0982-persisted-memory-compatibility-contract.md | none recorded | none recorded | not yet confirmed |
| R982-2 | docs/requirements/issue-0982-persisted-memory-compatibility-contract.md | none recorded | none recorded | not yet confirmed |
| R982-3 | docs/requirements/issue-0982-persisted-memory-compatibility-contract.md | none recorded | none recorded | not yet confirmed |
| R982-4 | docs/requirements/issue-0982-persisted-memory-compatibility-contract.md | none recorded | none recorded | not yet confirmed |
| R982-5 | docs/requirements/issue-0982-persisted-memory-compatibility-contract.md | none recorded | none recorded | not yet confirmed |
| R982-6 | docs/requirements/issue-0982-persisted-memory-compatibility-contract.md | none recorded | none recorded | not yet confirmed |
| R982-7 | docs/requirements/issue-0982-persisted-memory-compatibility-contract.md | none recorded | none recorded | not yet confirmed |
| R982-8 | docs/requirements/issue-0982-persisted-memory-compatibility-contract.md | none recorded | none recorded | not yet confirmed |
| R982-9 | docs/requirements/issue-0982-persisted-memory-compatibility-contract.md | none recorded | none recorded | not yet confirmed |
| R982-10 | docs/requirements/issue-0982-persisted-memory-compatibility-contract.md | none recorded | none recorded | not yet confirmed |
| R982-11 | docs/requirements/issue-0982-persisted-memory-compatibility-contract.md | none recorded | none recorded | not yet confirmed |
| R991-1 | docs/requirements/issue-0991-dynamic-multi-source-how-to-synthesis.md | none recorded | rust/tests/unit/issue_991_how_to_synthesis.rs | not yet confirmed |
| R991-2 | docs/requirements/issue-0991-dynamic-multi-source-how-to-synthesis.md | none recorded | rust/tests/integration/issue_991_how_to_http.rs | not yet confirmed |
| R991-3 | docs/requirements/issue-0991-dynamic-multi-source-how-to-synthesis.md | none recorded | none recorded | not yet confirmed |
| R991-4 | docs/requirements/issue-0991-dynamic-multi-source-how-to-synthesis.md | none recorded | none recorded | not yet confirmed |
| R991-5 | docs/requirements/issue-0991-dynamic-multi-source-how-to-synthesis.md | none recorded | none recorded | not yet confirmed |
| R991-6 | docs/requirements/issue-0991-dynamic-multi-source-how-to-synthesis.md | none recorded | none recorded | not yet confirmed |
| R991-7 | docs/requirements/issue-0991-dynamic-multi-source-how-to-synthesis.md | none recorded | rust/tests/unit/issue_991_how_to_synthesis.rs | not yet confirmed |
| R991-8 | docs/requirements/issue-0991-dynamic-multi-source-how-to-synthesis.md | none recorded | none recorded | not yet confirmed |
| R991-9 | docs/requirements/issue-0991-dynamic-multi-source-how-to-synthesis.md | none recorded | rust/tests/unit/issue_991_incremental_decomposition.rs | not yet confirmed |
| R1012-1 | docs/requirements/issue-1012-complete-ci-cd-diagnostic-audit.md | none recorded | none recorded | not yet confirmed |
| R1012-2 | docs/requirements/issue-1012-complete-ci-cd-diagnostic-audit.md | none recorded | rust/tests/unit/ci-cd/issue_1012.rs | not yet confirmed |
| R1012-3 | docs/requirements/issue-1012-complete-ci-cd-diagnostic-audit.md | none recorded | none recorded | not yet confirmed |
| R1012-4 | docs/requirements/issue-1012-complete-ci-cd-diagnostic-audit.md | none recorded | none recorded | not yet confirmed |
| R1012-5 | docs/requirements/issue-1012-complete-ci-cd-diagnostic-audit.md | none recorded | none recorded | not yet confirmed |
| R1012-6 | docs/requirements/issue-1012-complete-ci-cd-diagnostic-audit.md | none recorded | none recorded | not yet confirmed |
| R1012-7 | docs/requirements/issue-1012-complete-ci-cd-diagnostic-audit.md | none recorded | none recorded | not yet confirmed |
| R1012-8 | docs/requirements/issue-1012-complete-ci-cd-diagnostic-audit.md | none recorded | none recorded | not yet confirmed |
| R1012-9 | docs/requirements/issue-1012-complete-ci-cd-diagnostic-audit.md | none recorded | none recorded | not yet confirmed |
| R1014-1 | docs/requirements/issue-1014-complete-ci-cd-diagnostic-audit.md | none recorded | none recorded | not yet confirmed |
| R1014-2 | docs/requirements/issue-1014-complete-ci-cd-diagnostic-audit.md | none recorded | rust/tests/unit/ci-cd/issue_1014.rs | not yet confirmed |
| R1014-3 | docs/requirements/issue-1014-complete-ci-cd-diagnostic-audit.md | none recorded | none recorded | not yet confirmed |
| R1014-4 | docs/requirements/issue-1014-complete-ci-cd-diagnostic-audit.md | none recorded | none recorded | not yet confirmed |
| R1014-5 | docs/requirements/issue-1014-complete-ci-cd-diagnostic-audit.md | none recorded | none recorded | not yet confirmed |
| R1014-6 | docs/requirements/issue-1014-complete-ci-cd-diagnostic-audit.md | none recorded | none recorded | not yet confirmed |
| R1014-7 | docs/requirements/issue-1014-complete-ci-cd-diagnostic-audit.md | none recorded | none recorded | not yet confirmed |
| R1014-8 | docs/requirements/issue-1014-complete-ci-cd-diagnostic-audit.md | none recorded | none recorded | not yet confirmed |
| R1014-9 | docs/requirements/issue-1014-complete-ci-cd-diagnostic-audit.md | none recorded | none recorded | not yet confirmed |
| R1017-1 | docs/requirements/issue-1017-ci-cd-false-results-sweep.md | none recorded | none recorded | not yet confirmed |
| R1017-2 | docs/requirements/issue-1017-ci-cd-false-results-sweep.md | none recorded | rust/tests/unit/ci-cd/issue_1017.rs | not yet confirmed |
| R1017-3 | docs/requirements/issue-1017-ci-cd-false-results-sweep.md | none recorded | none recorded | not yet confirmed |
| R1017-4 | docs/requirements/issue-1017-ci-cd-false-results-sweep.md | none recorded | none recorded | not yet confirmed |
| R1017-5 | docs/requirements/issue-1017-ci-cd-false-results-sweep.md | none recorded | none recorded | not yet confirmed |
| R1017-6 | docs/requirements/issue-1017-ci-cd-false-results-sweep.md | none recorded | none recorded | not yet confirmed |
| R1017-7 | docs/requirements/issue-1017-ci-cd-false-results-sweep.md | none recorded | none recorded | not yet confirmed |
| R1017-8 | docs/requirements/issue-1017-ci-cd-false-results-sweep.md | none recorded | none recorded | not yet confirmed |
| R1017-9 | docs/requirements/issue-1017-ci-cd-false-results-sweep.md | none recorded | none recorded | not yet confirmed |
| R1017-10 | docs/requirements/issue-1017-ci-cd-false-results-sweep.md | none recorded | none recorded | not yet confirmed |
| R1017-11 | docs/requirements/issue-1017-ci-cd-false-results-sweep.md | none recorded | rust/tests/source/agent.rs | not yet confirmed |
| R1017-12 | docs/requirements/issue-1017-ci-cd-false-results-sweep.md | none recorded | none recorded | not yet confirmed |
| R379-clean | docs/requirements/issue-1021-full-range-coding-and-contribution-artifacts.md | none recorded | none recorded | not yet confirmed |
| R1085-2 | docs/requirements/issue-1085-the-links-network-is-not-the-system-that-reasons.md | none recorded | rust/tests/unit/issue_1085_seed_links.rs | not yet confirmed |
| R1085-3 | docs/requirements/issue-1085-the-links-network-is-not-the-system-that-reasons.md | none recorded | rust/tests/unit/issue_1085_requirement_resolution.rs | not yet confirmed |
| R1085-11 | docs/requirements/issue-1085-the-links-network-is-not-the-system-that-reasons.md | none recorded | rust/tests/unit/issue_1085_upstream_frontier.rs | not yet confirmed |
| R1085-14 | docs/requirements/issue-1085-the-links-network-is-not-the-system-that-reasons.md | none recorded | none recorded | not yet confirmed |
| R1085-15 | docs/requirements/issue-1085-the-links-network-is-not-the-system-that-reasons.md | none recorded | none recorded | not yet confirmed |
| R1085-16 | docs/requirements/issue-1085-the-links-network-is-not-the-system-that-reasons.md | none recorded | none recorded | not yet confirmed |
| R1085-17 | docs/requirements/issue-1085-the-links-network-is-not-the-system-that-reasons.md | none recorded | none recorded | not yet confirmed |
| R1138-B7-1 | docs/requirements/issue-1138-learning-effects.md | none recorded | rust/tests/unit/issue_1138_learned_items_change_answers.rs | not yet confirmed |
| R1138-B7-2 | docs/requirements/issue-1138-learning-effects.md | none recorded | rust/tests/unit/issue_1138_learning_ratchet.rs | not yet confirmed |
| R1138-B7-3 | docs/requirements/issue-1138-learning-effects.md | none recorded | rust/tests/unit/issue_1138_learning_ratchet.rs | not yet confirmed |
| R1138-B7-4 | docs/requirements/issue-1138-learning-effects.md | none recorded | rust/tests/unit/issue_1138_learning_ratchet.rs | not yet confirmed |
| R1138-3-1 | docs/requirements/issue-1138-repository-workspace-protocol.md | none recorded | rust/tests/unit/issue_1138_repository_workspace.rs | not yet confirmed |
| R1138-3-2 | docs/requirements/issue-1138-repository-workspace-protocol.md | none recorded | rust/tests/unit/issue_1138_locate_targets.rs | not yet confirmed |
| R1138-3-3 | docs/requirements/issue-1138-repository-workspace-protocol.md | none recorded | rust/tests/unit/issue_1138_named_tests.rs | not yet confirmed |
| R1138-3-4 | docs/requirements/issue-1138-repository-workspace-protocol.md | none recorded | rust/tests/unit/issue_1138_repository_workspace.rs | not yet confirmed |
| R1138-3-5 | docs/requirements/issue-1138-repository-workspace-protocol.md | none recorded | rust/tests/unit/issue_1138_repository_workspace.rs | not yet confirmed |
| R1138-3-6 | docs/requirements/issue-1138-repository-workspace-protocol.md | none recorded | rust/tests/unit/issue_1138_command_allowlist.rs | not yet confirmed |
| R1138-3-7 | docs/requirements/issue-1138-repository-workspace-protocol.md | none recorded | rust/tests/unit/issue_1138_solve_cli.rs | not yet confirmed |
| R1138-3-8 | docs/requirements/issue-1138-repository-workspace-protocol.md | none recorded | rust/tests/unit/issue_1138_repository_workspace.rs | not yet confirmed |
| R1138-3-9 | docs/requirements/issue-1138-repository-workspace-protocol.md | none recorded | rust/tests/unit/specification/repository_workspace_protocol.rs | not yet confirmed |
| R1138-B12-1 | docs/requirements/issue-1138-selection-heuristics.md | none recorded | rust/tests/unit/issue_1138_selection_heuristics.rs | not yet confirmed |
| R1138-B12-2 | docs/requirements/issue-1138-selection-heuristics.md | none recorded | rust/tests/unit/issue_1138_selection_heuristics.rs | not yet confirmed |
| R1138-B12-3 | docs/requirements/issue-1138-selection-heuristics.md | none recorded | rust/tests/unit/issue_1138_selection_heuristics.rs | not yet confirmed |
| R1138-B12-4 | docs/requirements/issue-1138-selection-heuristics.md | none recorded | rust/tests/unit/issue_1138_selection_heuristics.rs | not yet confirmed |
| R1138-B8-1 | docs/requirements/issue-1138-verifiable-task-routing.md | none recorded | rust/tests/unit/verifiable_task/identity.rs | not yet confirmed |
| R1138-B8-2 | docs/requirements/issue-1138-verifiable-task-routing.md | none recorded | rust/tests/unit/verifiable_task/recognition.rs | not yet confirmed |
| R1138-B8-3 | docs/requirements/issue-1138-verifiable-task-routing.md | none recorded | rust/tests/unit/verifiable_task/execution.rs | not yet confirmed |
| R1138-B8-4 | docs/requirements/issue-1138-verifiable-task-routing.md | none recorded | rust/tests/unit/verifiable_task/execution.rs | not yet confirmed |
| R1138-B8-5 | docs/requirements/issue-1138-verifiable-task-routing.md | none recorded | rust/tests/unit/verifiable_task/ledger.rs | not yet confirmed |
| R1138-B8-6 | docs/requirements/issue-1138-verifiable-task-routing.md | none recorded | rust/tests/unit/verifiable_task/ledger.rs | not yet confirmed |
| R1138-B8-7 | docs/requirements/issue-1138-verifiable-task-routing.md | none recorded | rust/tests/unit/verifiable_task/recognition.rs | not yet confirmed |
| R1138-B8-8 | docs/requirements/issue-1138-verifiable-task-routing.md | none recorded | rust/tests/unit/verifiable_task/ratchets.rs | not yet confirmed |
| R1138-B8-9 | docs/requirements/issue-1138-verifiable-task-routing.md | none recorded | rust/tests/unit/verifiable_task/rendering.rs | not yet confirmed |
