# Requirements Traceability

Companion table for [REQUIREMENTS.md](../REQUIREMENTS.md). Per konard's
2026-08-04 standing requirement, every tracked requirement must record (1)
when it was delivered in code, (2) the automated test that pins it, and (3)
manual test confirmation. Populated from the 2026-08-04 requirement audit
(944 audited requirement records, `docs/case-studies/issue-914/` lineage).

Honesty rules applied throughout: `not yet confirmed` and `none recorded`
mean exactly that -- absence of a record, not failure. `not delivered --
tracked in #NNN` rows point at the batch #930-#962 trackers or pre-existing
open issues. Manual-confirmation entries are cited **only** for the specific
checks the 2026-08-04 audit test battery actually ran by hand (see
`docs/case-studies/issue-914/` audit report); every other row is honestly
"not yet confirmed" even where the automated suite passes. CI enforcement
of this table's freshness is tracked by E105 (#957); this is the initial
data population, not an automated generator wired into CI yet.

The ID collisions this table originally preserved were resolved by issue
#964 (executed in PR #997): the later duplicate blocks were renumbered to
fresh IDs — the issue-540 R396-R407 became R537-R548, the issue-657 R480
became R549, and the issue-674 R501-R509 became R550-R558. The `Line`
column still records the REQUIREMENTS.md line each row was audited at in
2026-08-04 and is therefore historical, not live.

| ID | Line | Delivered | Automated test | Manual confirmation |
| --- | --- | --- | --- | --- |
| R1 | 14 | pre-2026-07 (undated) | none recorded | not yet confirmed |
| R2 | 15 | pre-2026-07 (undated) | none recorded | not yet confirmed |
| R3 | 16 | pre-2026-07 (undated) | none recorded | not yet confirmed |
| R4 | 17 | pre-2026-07 (undated) | none recorded | not yet confirmed |
| R5 | 18 | pre-2026-07 (undated) | none recorded | not yet confirmed |
| R6 | 19 | pre-2026-07 (undated) | none recorded | not yet confirmed |
| R7 | 20 | pre-2026-07 (undated) | none recorded | not yet confirmed |
| R8 | 21 | pre-2026-07 (undated) | none recorded | not yet confirmed |
| R9 | 22 | pre-2026-07 (undated) | none recorded | manually confirmed 2026-08-04 (audit): README `chat --prompt "Hi"` en greeting run via built binary |
| R10 | 23 | pre-2026-07 (undated) | none recorded | manually confirmed 2026-08-04 (audit): README `chat --prompt "Write me hello world program in Rust"` run, JSON chat format checked |
| R11 | 24 | pre-2026-07 (undated) | none recorded | not yet confirmed |
| R12 | 25 | pre-2026-07 (undated) | none recorded | not yet confirmed |
| R13 | 26 | pre-2026-07 (undated) | none recorded | manually confirmed 2026-08-04 (audit): `formal-ai serve` started, `/v1/chat/completions` and `/v1/responses` called, server stopped cleanly |
| R14 | 27 | pre-2026-07 (undated) | none recorded | not yet confirmed |
| R15 | 28 | pre-2026-07 (undated) | none recorded | not yet confirmed |
| R16 | 29 | pre-2026-07 (undated) | none recorded | not yet confirmed |
| R17 | 30 | pre-2026-07 (undated) | none recorded | manually confirmed 2026-08-04 (audit): `npm --prefix desktop run smoke` passed |
| R18 | 31 | pre-2026-07 (undated) | none recorded | not yet confirmed |
| R19 | 32 | pre-2026-07 (undated) | none recorded | not yet confirmed |
| R20 | 33 | pre-2026-07 (undated) | none recorded | not yet confirmed |
| R21 | 34 | pre-2026-07 (undated) | none recorded | not yet confirmed |
| R22 | 35 | pre-2026-07 (undated) | none recorded | manually confirmed 2026-08-04 (audit): README greetings run in en/ru/hi/zh (Hello/Привет/नमस्ते/你好) |
| R23 | 36 | pre-2026-07 (undated) | none recorded | not yet confirmed |
| R24 | 37 | pre-2026-07 (undated) | none recorded | not yet confirmed |
| R25 | 38 | pre-2026-07 (undated) | none recorded | not yet confirmed |
| R26 | 50 | pre-2026-07 (undated); issue #6 | issue-level coverage (not row-pinned): tests/e2e/tests/demo.spec.js:80 | not yet confirmed |
| R27 | 51 | pre-2026-07 (undated); issue #6 | issue-level coverage (not row-pinned): tests/e2e/tests/demo.spec.js:80 | not yet confirmed |
| R28 | 52 | pre-2026-07 (undated); issue #6 | issue-level coverage (not row-pinned): tests/e2e/tests/demo.spec.js:80 | not yet confirmed |
| R29 | 53 | pre-2026-07 (undated); issue #6 | issue-level coverage (not row-pinned): tests/e2e/tests/demo.spec.js:80 | not yet confirmed |
| R30 | 54 | pre-2026-07 (undated); issue #6 | issue-level coverage (not row-pinned): tests/e2e/tests/demo.spec.js:80 | not yet confirmed |
| R31 | 62 | pre-2026-07 (undated); issue #8 | issue-level coverage (not row-pinned): tests/unit/specification/agent_isolation.rs | not yet confirmed |
| R32 | 63 | pre-2026-07 (undated); issue #8 | issue-level coverage (not row-pinned): tests/unit/specification/agent_isolation.rs | not yet confirmed |
| R33 | 64 | pre-2026-07 (undated); issue #8 | issue-level coverage (not row-pinned): tests/unit/specification/agent_isolation.rs | not yet confirmed |
| R34 | 65 | pre-2026-07 (undated); issue #8 | issue-level coverage (not row-pinned): tests/unit/specification/agent_isolation.rs | not yet confirmed |
| R35 | 66 | pre-2026-07 (undated); issue #8 | issue-level coverage (not row-pinned): tests/unit/specification/agent_isolation.rs | not yet confirmed |
| R36 | 67 | pre-2026-07 (undated); issue #8 | issue-level coverage (not row-pinned): tests/unit/specification/agent_isolation.rs | not yet confirmed |
| R37 | 68 | pre-2026-07 (undated); issue #8 | issue-level coverage (not row-pinned): tests/unit/specification/agent_isolation.rs | not yet confirmed |
| R38 | 69 | pre-2026-07 (undated); issue #8 | issue-level coverage (not row-pinned): tests/unit/specification/agent_isolation.rs | not yet confirmed |
| R39 | 70 | pre-2026-07 (undated); issue #8 | issue-level coverage (not row-pinned): tests/unit/specification/agent_isolation.rs | not yet confirmed |
| R40 | 71 | pre-2026-07 (undated); issue #8 | issue-level coverage (not row-pinned): tests/unit/specification/agent_isolation.rs | not yet confirmed |
| R41 | 72 | pre-2026-07 (undated); issue #8 | issue-level coverage (not row-pinned): tests/unit/specification/agent_isolation.rs | not yet confirmed |
| R42 | 73 | pre-2026-07 (undated); issue #8 | issue-level coverage (not row-pinned): tests/unit/specification/agent_isolation.rs | not yet confirmed |
| R43 | 74 | pre-2026-07 (undated); issue #8 | issue-level coverage (not row-pinned): tests/unit/specification/agent_isolation.rs | not yet confirmed |
| R44 | 75 | pre-2026-07 (undated); issue #8 | issue-level coverage (not row-pinned): tests/unit/specification/agent_isolation.rs | not yet confirmed |
| R45 | 76 | pre-2026-07 (undated); issue #8 | issue-level coverage (not row-pinned): tests/unit/specification/agent_isolation.rs | not yet confirmed |
| R46 | 77 | pre-2026-07 (undated); issue #8 | issue-level coverage (not row-pinned): tests/unit/specification/agent_isolation.rs | not yet confirmed |
| R47 | 78 | pre-2026-07 (undated); issue #8 | issue-level coverage (not row-pinned): tests/unit/specification/agent_isolation.rs | not yet confirmed |
| R48 | 86 | pre-2026-07 (undated); issue #10 | issue-level coverage (not row-pinned): tests/e2e/tests/demo.spec.js:443 | not yet confirmed |
| R49 | 87 | pre-2026-07 (undated); issue #10 | issue-level coverage (not row-pinned): tests/e2e/tests/demo.spec.js:443 | not yet confirmed |
| R50 | 88 | pre-2026-07 (undated); issue #10 | issue-level coverage (not row-pinned): tests/e2e/tests/demo.spec.js:443 | not yet confirmed |
| R51 | 89 | pre-2026-07 (undated); issue #10 | issue-level coverage (not row-pinned): tests/e2e/tests/demo.spec.js:443 | not yet confirmed |
| R52 | 90 | pre-2026-07 (undated); issue #10 | issue-level coverage (not row-pinned): tests/e2e/tests/demo.spec.js:443 | not yet confirmed |
| R53 | 91 | pre-2026-07 (undated); issue #10 | issue-level coverage (not row-pinned): tests/e2e/tests/demo.spec.js:443 | not yet confirmed |
| R54 | 92 | pre-2026-07 (undated); issue #10 | issue-level coverage (not row-pinned): tests/e2e/tests/demo.spec.js:443 | not yet confirmed |
| R55 | 100 | pre-2026-07 (undated); issue #12 | issue-level coverage (not row-pinned): tests/unit/docs_requirements.rs | not yet confirmed |
| R56 | 101 | pre-2026-07 (undated); issue #12 | issue-level coverage (not row-pinned): tests/unit/docs_requirements.rs | not yet confirmed |
| R57 | 102 | pre-2026-07 (undated); issue #12 | issue-level coverage (not row-pinned): tests/unit/docs_requirements.rs | not yet confirmed |
| R58 | 103 | pre-2026-07 (undated); issue #12 | issue-level coverage (not row-pinned): tests/unit/docs_requirements.rs | not yet confirmed |
| R59 | 104 | pre-2026-07 (undated); issue #12 | issue-level coverage (not row-pinned): tests/unit/docs_requirements.rs | not yet confirmed |
| R60 | 105 | pre-2026-07 (undated); issue #12 | issue-level coverage (not row-pinned): tests/unit/docs_requirements.rs | not yet confirmed |
| R61 | 106 | pre-2026-07 (undated); issue #12 | issue-level coverage (not row-pinned): tests/unit/docs_requirements.rs | manually confirmed 2026-08-04 (audit): `chat --prompt "What is 8% of $50?"` run twice, outputs byte-identical (cmp) |
| R62 | 107 | pre-2026-07 (undated); issue #12 | issue-level coverage (not row-pinned): tests/unit/docs_requirements.rs | not yet confirmed |
| R63 | 108 | pre-2026-07 (undated); issue #12 | issue-level coverage (not row-pinned): tests/unit/docs_requirements.rs | not yet confirmed |
| R64 | 109 | pre-2026-07 (undated); issue #12 | issue-level coverage (not row-pinned): tests/unit/docs_requirements.rs | not yet confirmed |
| R65 | 110 | pre-2026-07 (undated); issue #12 | issue-level coverage (not row-pinned): tests/unit/docs_requirements.rs | not yet confirmed |
| R66 | 111 | pre-2026-07 (undated); issue #12 | tests/unit/specification/reasoning_loop.rs | not yet confirmed |
| R67 | 112 | pre-2026-07 (undated); issue #12 | tests/unit/specification/source_cache.rs | not yet confirmed |
| R68 | 113 | pre-2026-07 (undated); issue #12 | issue-level coverage (not row-pinned): tests/unit/docs_requirements.rs | not yet confirmed |
| R69 | 114 | pre-2026-07 (undated); issue #12 | issue-level coverage (not row-pinned): tests/unit/docs_requirements.rs | not yet confirmed |
| R70 | 115 | pre-2026-07 (undated); issue #12 | tests/unit/docs_requirements.rs | not yet confirmed |
| R71 | 116 | pre-2026-07 (undated); issue #12 | tests/unit/specification/ | not yet confirmed |
| R72 | 117 | pre-2026-07 (undated); issue #12 | tests/unit/specification/reasoning_loop.rs | not yet confirmed |
| R73 | 118 | pre-2026-07 (undated); issue #12 | issue-level coverage (not row-pinned): tests/unit/docs_requirements.rs | not yet confirmed |
| R74 | 119 | pre-2026-07 (undated); issue #12 | issue-level coverage (not row-pinned): tests/unit/docs_requirements.rs | not yet confirmed |
| R75 | 120 | pre-2026-07 (undated); issue #12 | issue-level coverage (not row-pinned): tests/unit/docs_requirements.rs | not yet confirmed |
| R76 | 121 | pre-2026-07 (undated); issue #12 | issue-level coverage (not row-pinned): tests/unit/docs_requirements.rs | not yet confirmed |
| R77 | 122 | pre-2026-07 (undated); issue #12 | tests/unit/specification/transparent_state.rs | not yet confirmed |
| R78 | 123 | pre-2026-07 (undated); issue #12 | issue-level coverage (not row-pinned): tests/unit/docs_requirements.rs | not yet confirmed |
| R79 | 124 | pre-2026-07 (undated); issue #12 | tests/unit/specification/source_cache.rs | not yet confirmed |
| R80 | 125 | pre-2026-07 (undated); issue #12 | tests/unit/specification/agent_isolation.rs | not yet confirmed |
| R81 | 126 | pre-2026-07 (undated); issue #12 | tests/unit/specification/network_visualization.rs | not yet confirmed |
| R82 | 127 | pre-2026-07 (undated); issue #12 | tests/unit/specification/reasoning_loop.rs::answers_are_repeatable_for_the_same_prompt | not yet confirmed |
| R83 | 138 | pre-2026-07 (undated); issue #14 | issue-level coverage (not row-pinned): tests/unit/specification/conversation_history.rs | not yet confirmed |
| R84 | 139 | pre-2026-07 (undated); issue #14 | issue-level coverage (not row-pinned): tests/unit/specification/conversation_history.rs | not yet confirmed |
| R85 | 140 | pre-2026-07 (undated); issue #14 | tests/unit/specification/reasoning_paths.rs::arithmetic_* | not yet confirmed |
| R86 | 141 | pre-2026-07 (undated); issue #14 | tests/unit/specification/reasoning_paths.rs::concept_lookup_* | not yet confirmed |
| R87 | 142 | pre-2026-07 (undated); issue #14 | tests/unit/specification/reasoning_paths.rs::solve_with_history_* | not yet confirmed |
| R88 | 143 | pre-2026-07 (undated); issue #14 | tests/unit/specification/reasoning_paths.rs::javascript_* | not yet confirmed |
| R89 | 144 | pre-2026-07 (undated); issue #14 | issue-level coverage (not row-pinned): tests/unit/specification/conversation_history.rs | not yet confirmed |
| R90 | 156 | pre-2026-07 (undated); issue #16 | issue-level coverage (not row-pinned): tests/unit/specification/multilingual.rs | not yet confirmed |
| R91 | 157 | pre-2026-07 (undated); issue #16 | issue-level coverage (not row-pinned): tests/unit/specification/multilingual.rs | not yet confirmed |
| R92 | 158 | pre-2026-07 (undated); issue #16 | issue-level coverage (not row-pinned): tests/unit/specification/multilingual.rs | not yet confirmed |
| R93 | 159 | pre-2026-07 (undated); issue #16 | tests/e2e/tests/multilingual.spec.js; tests/e2e/playwright.local.config.js | not yet confirmed |
| R94 | 160 | pre-2026-07 (undated); issue #16 | issue-level coverage (not row-pinned): tests/unit/specification/multilingual.rs | not yet confirmed |
| R95 | 161 | pre-2026-07 (undated); issue #16 | tests/e2e/tests/multilingual.spec.js | not yet confirmed |
| R96 | 162 | pre-2026-07 (undated); issue #16 | issue-level coverage (not row-pinned): tests/unit/specification/multilingual.rs | not yet confirmed |
| R97 | 163 | pre-2026-07 (undated); issue #16 | issue-level coverage (not row-pinned): tests/unit/specification/multilingual.rs | not yet confirmed |
| R98 | 164 | pre-2026-07 (undated); issue #16 | issue-level coverage (not row-pinned): tests/unit/specification/multilingual.rs | not yet confirmed |
| R99 | 165 | pre-2026-07 (undated); issue #16 | issue-level coverage (not row-pinned): tests/unit/specification/multilingual.rs | not yet confirmed |
| R100 | 166 | pre-2026-07 (undated); issue #16 | issue-level coverage (not row-pinned): tests/unit/specification/multilingual.rs | not yet confirmed |
| R101 | 180 | PR #17 (issue #16) | issue-level coverage (not row-pinned): tests/unit/specification/multilingual.rs | not yet confirmed |
| R102 | 181 | PR #17 (issue #16) | issue-level coverage (not row-pinned): tests/unit/specification/multilingual.rs | not yet confirmed |
| R103 | 182 | PR #17 (issue #16) | seed::tests | not yet confirmed |
| R104 | 183 | PR #17 (issue #16) | seed::tests::bundle_round_trips_through_parse_bundle; seed::tests::parse_bundle_recovers_intent_routing_via_inner_parser | not yet confirmed |
| R105 | 184 | PR #17 (issue #16) | tests/e2e/playwright.local.config.js | not yet confirmed |
| R106 | 185 | PR #17 (issue #16) | issue-level coverage (not row-pinned): tests/unit/specification/multilingual.rs | not yet confirmed |
| R107 | 186 | PR #17 (issue #16) | issue-level coverage (not row-pinned): tests/unit/specification/multilingual.rs | not yet confirmed |
| R108 | 187 | PR #17 (issue #16) | issue-level coverage (not row-pinned): tests/unit/specification/multilingual.rs | not yet confirmed |
| R109 | 199 | pre-2026-07 (undated); issue #18 | issue-level coverage (not row-pinned): tests/e2e/tests/issue-672-migration-replay.spec.js | not yet confirmed |
| R110 | 200 | pre-2026-07 (undated); issue #18 | issue-level coverage (not row-pinned): tests/e2e/tests/issue-672-migration-replay.spec.js | not yet confirmed |
| R111 | 201 | pre-2026-07 (undated); issue #18 | issue-level coverage (not row-pinned): tests/e2e/tests/issue-672-migration-replay.spec.js | not yet confirmed |
| R112 | 202 | pre-2026-07 (undated); issue #18 | issue-level coverage (not row-pinned): tests/e2e/tests/issue-672-migration-replay.spec.js | not yet confirmed |
| R113 | 203 | pre-2026-07 (undated); issue #18 | issue-level coverage (not row-pinned): tests/e2e/tests/issue-672-migration-replay.spec.js | not yet confirmed |
| R114 | 204 | pre-2026-07 (undated); issue #18 | tests/e2e/tests/multilingual.spec.js; memory::tests::full_memory_round_trip_* | not yet confirmed |
| R115 | 218 | pre-2026-07 (undated); issue #78 | none recorded | not yet confirmed |
| R116 | 219 | pre-2026-07 (undated); issue #78 | tests/e2e/tests/demo.spec.js | not yet confirmed |
| R117 | 220 | pre-2026-07 (undated); issue #78 | none recorded | not yet confirmed |
| R118 | 221 | pre-2026-07 (undated); issue #78 | none recorded | not yet confirmed |
| R119 | 222 | pre-2026-07 (undated); issue #78 | tests/e2e/tests/multilingual.spec.js; tests/e2e/tests/demo.spec.js | not yet confirmed |
| R120 | 233 | pre-2026-07 (undated); issue #96 | none recorded | manually confirmed 2026-08-04 (audit): README arithmetic examples run in en (`8% of $50`) and ru (currency conversion) |
| R121 | 234 | pre-2026-07 (undated); issue #96 | none recorded | not yet confirmed |
| R122 | 235 | pre-2026-07 (undated); issue #96 | tests/unit/specification/calculator_delegation.rs | not yet confirmed |
| R123 | 236 | pre-2026-07 (undated); issue #96 | none recorded | not yet confirmed |
| R124 | 237 | pre-2026-07 (undated); issue #96 | tests/unit/specification/calculator_delegation.rs | not yet confirmed |
| R125 | 238 | pre-2026-07 (undated); issue #96 | none recorded | not yet confirmed |
| R126 | 239 | pre-2026-07 (undated); issue #96 | none recorded | not yet confirmed |
| R127 | 240 | pre-2026-07 (undated); issue #96 | none recorded | not yet confirmed |
| R128 | 241 | pre-2026-07 (undated); issue #96 | none recorded | not yet confirmed |
| R129 | 256 | pre-2026-07 (undated); issue #103 | tests/unit/specification/prompt_variations.rs; tests/unit/specification/chat_surface.rs | not yet confirmed |
| R130 | 257 | pre-2026-07 (undated); issue #103 | tests/unit/specification/prompt_variations.rs; tests/unit/specification/multilingual.rs | not yet confirmed |
| R131 | 258 | pre-2026-07 (undated); issue #103 | issue-level coverage (not row-pinned): tests/unit/specification/prompt_variations.rs | not yet confirmed |
| R132 | 259 | pre-2026-07 (undated); issue #103 | tests/unit/specification/prompt_variations.rs | not yet confirmed |
| R133 | 260 | pre-2026-07 (undated); issue #103 | issue-level coverage (not row-pinned): tests/unit/specification/prompt_variations.rs | not yet confirmed |
| R134 | 261 | pre-2026-07 (undated); issue #103 | issue-level coverage (not row-pinned): tests/unit/specification/prompt_variations.rs | not yet confirmed |
| R135 | 262 | pre-2026-07 (undated); issue #103 | issue-level coverage (not row-pinned): tests/unit/specification/prompt_variations.rs | not yet confirmed |
| R136 | 263 | pre-2026-07 (undated); issue #103 | issue-level coverage (not row-pinned): tests/unit/specification/prompt_variations.rs | not yet confirmed |
| R137 | 275 | pre-2026-07 (undated); issue #117 | none recorded | not yet confirmed |
| R138 | 276 | pre-2026-07 (undated); issue #117 | none recorded | not yet confirmed |
| R139 | 277 | pre-2026-07 (undated); issue #117 | none recorded | not yet confirmed |
| R140 | 278 | pre-2026-07 (undated); issue #117 | tests/e2e/scripts/check-i18n-catalog.mjs; npm run --prefix tests/e2e check:i18n | not yet confirmed |
| R141 | 279 | pre-2026-07 (undated); issue #117 | tests/e2e/tests/demo.spec.js | not yet confirmed |
| R142 | 280 | pre-2026-07 (undated); issue #117 | none recorded | not yet confirmed |
| R143 | 294 | pre-2026-07 (undated); issue #115 | none recorded | not yet confirmed |
| R144 | 295 | pre-2026-07 (undated); issue #115 | none recorded | not yet confirmed |
| R145 | 296 | pre-2026-07 (undated); issue #115 | none recorded | not yet confirmed |
| R146 | 297 | pre-2026-07 (undated); issue #115 | none recorded | not yet confirmed |
| R147 | 298 | pre-2026-07 (undated); issue #115 | none recorded | not yet confirmed |
| R148 | 299 | pre-2026-07 (undated); issue #115 | none recorded | not yet confirmed |
| R149 | 300 | pre-2026-07 (undated); issue #115 | tests/unit/github_logs.rs; tests/integration/formal_ai_cli.rs | not yet confirmed |
| R150 | 314 | pre-2026-07 (undated); issue #63 | issue-level coverage (not row-pinned): tests/unit/specification/definition_fusion.rs | not yet confirmed |
| R151 | 315 | pre-2026-07 (undated); issue #63 | issue-level coverage (not row-pinned): tests/unit/specification/definition_fusion.rs | not yet confirmed |
| R152 | 316 | pre-2026-07 (undated); issue #63 | issue-level coverage (not row-pinned): tests/unit/specification/definition_fusion.rs | not yet confirmed |
| R153 | 317 | pre-2026-07 (undated); issue #63 | issue-level coverage (not row-pinned): tests/unit/specification/definition_fusion.rs | not yet confirmed |
| R154 | 318 | pre-2026-07 (undated); issue #63 | tests/unit/specification/definition_fusion.rs; tests/e2e/tests/multilingual.spec.js | not yet confirmed |
| R155 | 319 | pre-2026-07 (undated); issue #63 | issue-level coverage (not row-pinned): tests/unit/specification/definition_fusion.rs | not yet confirmed |
| R156 | 332 | pre-2026-07 (undated); issue #80 | none recorded | not yet confirmed |
| R157 | 333 | pre-2026-07 (undated); issue #80 | none recorded | not yet confirmed |
| R158 | 334 | pre-2026-07 (undated); issue #80 | none recorded | not yet confirmed |
| R159 | 335 | pre-2026-07 (undated); issue #80 | none recorded | not yet confirmed |
| R160 | 336 | pre-2026-07 (undated); issue #80 | none recorded | not yet confirmed |
| R161 | 337 | pre-2026-07 (undated); issue #80 | none recorded | not yet confirmed |
| R162 | 338 | pre-2026-07 (undated); issue #80 | none recorded | not yet confirmed |
| R163 | 339 | pre-2026-07 (undated); issue #80 | none recorded | not yet confirmed |
| R164 | 340 | pre-2026-07 (undated); issue #80 | none recorded | not yet confirmed |
| R165 | 351 | pre-2026-07 (undated); issue #129 | src/web/tests/index.html | not yet confirmed |
| R166 | 352 | pre-2026-07 (undated); issue #129 | src/web/tests/connectivity.js | not yet confirmed |
| R167 | 353 | pre-2026-07 (undated); issue #129 | issue-level coverage (not row-pinned): tests/connectivity.js | not yet confirmed |
| R168 | 354 | pre-2026-07 (undated); issue #129 | issue-level coverage (not row-pinned): tests/connectivity.js | not yet confirmed |
| R169 | 355 | pre-2026-07 (undated); issue #129 | issue-level coverage (not row-pinned): tests/connectivity.js | not yet confirmed |
| R170 | 356 | pre-2026-07 (undated); issue #129 | tests/e2e/tests/connectivity.spec.js | not yet confirmed |
| R171 | 357 | pre-2026-07 (undated); issue #129 | tests/unit/ci-cd/workflow_release.rs | not yet confirmed |
| R172 | 358 | pre-2026-07 (undated); issue #129 | issue-level coverage (not row-pinned): tests/connectivity.js | not yet confirmed |
| R173 | 371 | pre-2026-07 (undated); issue #127 | none recorded | not yet confirmed |
| R174 | 372 | pre-2026-07 (undated); issue #127 | none recorded | not yet confirmed |
| R175 | 373 | pre-2026-07 (undated); issue #127 | none recorded | not yet confirmed |
| R176 | 374 | pre-2026-07 (undated); issue #127 | none recorded | not yet confirmed |
| R177 | 375 | pre-2026-07 (undated); issue #127 | none recorded | not yet confirmed |
| R178 | 376 | pre-2026-07 (undated); issue #127 | none recorded | not yet confirmed |
| R179 | 377 | pre-2026-07 (undated); issue #127 | none recorded | not yet confirmed |
| R180 | 378 | pre-2026-07 (undated); issue #127 | tests/unit/specification/prompt_variations.rs; tests/e2e/tests/multilingual.spec.js | not yet confirmed |
| R181 | 393 | pre-2026-07 (undated); issue #133 | issue-level coverage (not row-pinned): tests/connectivity.js | not yet confirmed |
| R182 | 394 | pre-2026-07 (undated); issue #133 | issue-level coverage (not row-pinned): tests/connectivity.js | not yet confirmed |
| R183 | 395 | pre-2026-07 (undated); issue #133 | issue-level coverage (not row-pinned): tests/connectivity.js | not yet confirmed |
| R184 | 396 | pre-2026-07 (undated); issue #133 | src/web/tests/connectivity.js | not yet confirmed |
| R185 | 397 | pre-2026-07 (undated); issue #133 | src/web/tests/connectivity.js | not yet confirmed |
| R186 | 398 | pre-2026-07 (undated); issue #133 | issue-level coverage (not row-pinned): tests/connectivity.js | not yet confirmed |
| R187 | 399 | pre-2026-07 (undated); issue #133 | issue-level coverage (not row-pinned): tests/connectivity.js | not yet confirmed |
| R188 | 400 | pre-2026-07 (undated); issue #133 | issue-level coverage (not row-pinned): tests/connectivity.js | not yet confirmed |
| R189 | 401 | pre-2026-07 (undated); issue #133 | issue-level coverage (not row-pinned): tests/connectivity.js | not yet confirmed |
| R190 | 402 | pre-2026-07 (undated); issue #133 | issue-level coverage (not row-pinned): tests/connectivity.js | not yet confirmed |
| R191 | 403 | pre-2026-07 (undated); issue #133 | issue-level coverage (not row-pinned): tests/connectivity.js | not yet confirmed |
| R192 | 404 | pre-2026-07 (undated); issue #133 | issue-level coverage (not row-pinned): tests/connectivity.js | not yet confirmed |
| R193 | 405 | pre-2026-07 (undated); issue #133 | issue-level coverage (not row-pinned): tests/connectivity.js | not yet confirmed |
| R194 | 406 | pre-2026-07 (undated); issue #133 | issue-level coverage (not row-pinned): tests/connectivity.js | not yet confirmed |
| R195 | 424 | pre-2026-07 (undated); issue #159 | tests/unit/specification/project_lookups.rs::russian_hive_mind_prompt_prefers_link_assistant_project | not yet confirmed |
| R196 | 425 | pre-2026-07 (undated); issue #159 | issue-level coverage (not row-pinned): tests/unit/specification/project_lookups.rs:17 | not yet confirmed |
| R197 | 426 | pre-2026-07 (undated); issue #159 | src/summarization/mod.rs::tests | not yet confirmed |
| R198 | 427 | pre-2026-07 (undated); issue #159 | issue-level coverage (not row-pinned): tests/unit/specification/project_lookups.rs:17 | not yet confirmed |
| R199 | 428 | pre-2026-07 (undated); issue #159 | issue-level coverage (not row-pinned): tests/unit/specification/project_lookups.rs:17 | not yet confirmed |
| R200 | 429 | pre-2026-07 (undated); issue #159 | tests/unit/specification/project_lookups.rs::curated_project_concept_prompt_routes_to_project_lookup | not yet confirmed |
| R201 | 430 | pre-2026-07 (undated); issue #159 | tests/unit/specification/project_lookups.rs::curated_project_lookup_records_summarization_evidence | not yet confirmed |
| R202 | 431 | pre-2026-07 (undated); issue #159 | tests/unit/specification/summarization_pipeline.rs::strip_markdown_noise_drops_badges_html_comments_and_code_blocks | not yet confirmed |
| R203 | 432 | pre-2026-07 (undated); issue #159 | tests/unit/specification/summarization_pipeline.rs::formalize_dialog_biases_user_turns_above_assistant_turns | not yet confirmed |
| R204 | 433 | pre-2026-07 (undated); issue #159 | tests/unit/specification/summarization_pipeline.rs::generate_chat_title_returns_five_or_fewer_words | not yet confirmed |
| R205 | 434 | pre-2026-07 (undated); issue #159 | tests/unit/specification/project_lookups.rs::http_fetch_of_curated_github_url_describes_project_via_summarization | not yet confirmed |
| R206 | 435 | pre-2026-07 (undated); issue #159 | tests/unit/specification/summarization_pipeline.rs::default_max_statements_is_thirty | not yet confirmed |
| R207 | 436 | pre-2026-07 (undated); issue #159 | tests/unit/specification/summarization_pipeline.rs::summarization_mode_target_percent_matches_vision | not yet confirmed |
| R208 | 437 | pre-2026-07 (undated); issue #159 | tests/unit/specification/project_lookups.rs::associative_project_promotion_can_be_disabled | not yet confirmed |
| R209 | 438 | pre-2026-07 (undated); issue #159 | issue-level coverage (not row-pinned): tests/unit/specification/project_lookups.rs:17 | not yet confirmed |
| R210 | 450 | pre-2026-07 (undated); issue #162 | issue-level coverage (not row-pinned): tests/unit/specification/reasoning_paths.rs:221 | not yet confirmed |
| R211 | 451 | pre-2026-07 (undated); issue #162 | issue-level coverage (not row-pinned): tests/unit/specification/reasoning_paths.rs:221 | not yet confirmed |
| R212 | 452 | pre-2026-07 (undated); issue #162 | tests/unit/specification/reasoning_paths.rs | not yet confirmed |
| R213 | 471 | pre-2026-07 (undated); issue #207 | tests/unit/specification/translation_via_links.rs::russian_translate_how_are_you_prompt_returns_english_surface | not yet confirmed |
| R214 | 472 | pre-2026-07 (undated); issue #207 | tests/unit/specification/translation_via_links.rs::russian_translate_how_are_you_prompt_returns_english_surface | not yet confirmed |
| R215 | 473 | pre-2026-07 (undated); issue #207 | tests/unit/specification/translation_via_links.rs::translation_meaning_registry_covers_extended_phrases | not yet confirmed |
| R526-1 | 485 | pre-2026-07 (undated); issue #526 | tests/unit/specification/translation_round_trip.rs | not yet confirmed |
| R526-2 | 486 | pre-2026-07 (undated); issue #526 | issue-level coverage (not row-pinned): tests/unit/specification/translation_round_trip.rs | not yet confirmed |
| R526-3 | 487 | pre-2026-07 (undated); issue #526 | issue-level coverage (not row-pinned): tests/unit/specification/translation_round_trip.rs | not yet confirmed |
| R526-4 | 488 | pre-2026-07 (undated); issue #526 | issue-level coverage (not row-pinned): tests/unit/specification/translation_round_trip.rs | not yet confirmed |
| R526-5 | 489 | pre-2026-07 (undated); issue #526 | issue-level coverage (not row-pinned): tests/unit/specification/translation_round_trip.rs | not yet confirmed |
| R526-6 | 490 | pre-2026-07 (undated); issue #526 | issue-level coverage (not row-pinned): tests/unit/specification/translation_round_trip.rs | not yet confirmed |
| R890-1 | 501 | pre-2026-07 (undated); issue #890 | issue-level coverage (not row-pinned): tests/unit/issue_890.rs | not yet confirmed |
| R890-2 | 502 | pre-2026-07 (undated); issue #890 | issue-level coverage (not row-pinned): tests/unit/issue_890.rs | not yet confirmed |
| R890-3 | 503 | pre-2026-07 (undated); issue #890 | issue-level coverage (not row-pinned): tests/unit/issue_890.rs | not yet confirmed |
| R890-4 | 504 | pre-2026-07 (undated); issue #890 | issue-level coverage (not row-pinned): tests/unit/issue_890.rs | not yet confirmed |
| R890-5 | 505 | pre-2026-07 (undated); issue #890 | tests/e2e/tests/issue-890.spec.js | not yet confirmed |
| R890-6 | 506 | pre-2026-07 (undated); issue #890 | issue-level coverage (not row-pinned): tests/unit/issue_890.rs | not yet confirmed |
| R890-7 | 507 | pre-2026-07 (undated); issue #890 | issue-level coverage (not row-pinned): tests/unit/issue_890.rs | not yet confirmed |
| R498-1 | 519 | pre-2026-07 (undated); issue #498 | issue-level coverage (not row-pinned): tests/unit/issue_498_google_trends_learning.rs | not yet confirmed |
| R498-2 | 520 | pre-2026-07 (undated); issue #498 | issue-level coverage (not row-pinned): tests/unit/issue_498_google_trends_learning.rs | not yet confirmed |
| R498-3 | 521 | pre-2026-07 (undated); issue #498 | issue-level coverage (not row-pinned): tests/unit/issue_498_google_trends_learning.rs | not yet confirmed |
| R498-4 | 522 | pre-2026-07 (undated); issue #498 | issue-level coverage (not row-pinned): tests/unit/issue_498_google_trends_learning.rs | not yet confirmed |
| R498-5 | 523 | pre-2026-07 (undated); issue #498 | issue-level coverage (not row-pinned): tests/unit/issue_498_google_trends_learning.rs | not yet confirmed |
| R498-6 | 524 | pre-2026-07 (undated); issue #498 | issue-level coverage (not row-pinned): tests/unit/issue_498_google_trends_learning.rs | not yet confirmed |
| R498-7 | 525 | pre-2026-07 (undated); issue #498 | tests/unit/issue_498_google_trends_catalog.rs | not yet confirmed |
| R498-8 | 526 | pre-2026-07 (undated); issue #498 | issue-level coverage (not row-pinned): tests/unit/issue_498_google_trends_learning.rs | not yet confirmed |
| R498-9 | 527 | pre-2026-07 (undated); issue #498 | issue-level coverage (not row-pinned): tests/unit/issue_498_google_trends_learning.rs | not yet confirmed |
| R498-10 | 528 | pre-2026-07 (undated); issue #498 | issue-level coverage (not row-pinned): tests/unit/issue_498_google_trends_learning.rs | not yet confirmed |
| R527-1 | 539 | pre-2026-07 (undated); issue #527 | issue-level coverage (not row-pinned): tests/unit/docs_requirements_issue_527.rs | not yet confirmed |
| R527-2 | 540 | pre-2026-07 (undated); issue #527 | issue-level coverage (not row-pinned): tests/unit/docs_requirements_issue_527.rs | not yet confirmed |
| R527-3 | 541 | pre-2026-07 (undated); issue #527 | issue-level coverage (not row-pinned): tests/unit/docs_requirements_issue_527.rs | not yet confirmed |
| R527-4 | 542 | pre-2026-07 (undated); issue #527 | issue-level coverage (not row-pinned): tests/unit/docs_requirements_issue_527.rs | not yet confirmed |
| R527-5 | 543 | pre-2026-07 (undated); issue #527 | issue-level coverage (not row-pinned): tests/unit/docs_requirements_issue_527.rs | not yet confirmed |
| R527-6 | 544 | pre-2026-07 (undated); issue #527 | issue-level coverage (not row-pinned): tests/unit/docs_requirements_issue_527.rs | not yet confirmed |
| R527-7 | 545 | pre-2026-07 (undated); issue #527 | issue-level coverage (not row-pinned): tests/unit/docs_requirements_issue_527.rs | not yet confirmed |
| R527-8 | 546 | pre-2026-07 (undated); issue #527 | issue-level coverage (not row-pinned): tests/unit/docs_requirements_issue_527.rs | not yet confirmed |
| R527-9 | 547 | pre-2026-07 (undated); issue #527 | tests/unit/issue_527_question_catalog.rs | not yet confirmed |
| R527-10 | 548 | pre-2026-07 (undated); issue #527 | issue-level coverage (not row-pinned): tests/unit/docs_requirements_issue_527.rs | not yet confirmed |
| R527-11 | 549 | pre-2026-07 (undated); issue #527 | issue-level coverage (not row-pinned): tests/unit/docs_requirements_issue_527.rs | not yet confirmed |
| R527-12 | 550 | pre-2026-07 (undated); issue #527 | tests/unit/issue_527_question_catalog.rs | not yet confirmed |
| R216 | 562 | pre-2026-07 (undated); issue #187 | none recorded | not yet confirmed |
| R217 | 563 | pre-2026-07 (undated); issue #187 | none recorded | not yet confirmed |
| R218 | 564 | pre-2026-07 (undated); issue #187 | tests/unit/specification/reasoning_paths.rs; tests/e2e/tests/multilingual.spec.js | not yet confirmed |
| R219 | 565 | pre-2026-07 (undated); issue #187 | tests/e2e/scripts/check-multilingual-intent-coverage.mjs | not yet confirmed |
| R220 | 577 | pre-2026-07 (undated); issue #195 | tests/unit/docker_runtime.rs | not yet confirmed |
| R221 | 578 | pre-2026-07 (undated); issue #195 | issue-level coverage (not row-pinned): tests/unit/specification/agent_isolation.rs | not yet confirmed |
| R222 | 579 | pre-2026-07 (undated); issue #195 | issue-level coverage (not row-pinned): tests/unit/specification/agent_isolation.rs | not yet confirmed |
| R223 | 580 | pre-2026-07 (undated); issue #195 | issue-level coverage (not row-pinned): tests/unit/specification/agent_isolation.rs | not yet confirmed |
| R224 | 581 | pre-2026-07 (undated); issue #195 | tests/unit/docs_requirements.rs | not yet confirmed |
| R225 | 582 | pre-2026-07 (undated); issue #195 | issue-level coverage (not row-pinned): tests/unit/specification/agent_isolation.rs | not yet confirmed |
| R226 | 593 | pre-2026-07 (undated); issue #196 | none recorded | not yet confirmed |
| R227 | 594 | pre-2026-07 (undated); issue #196 | none recorded | not yet confirmed |
| R228 | 595 | pre-2026-07 (undated); issue #196 | none recorded | not yet confirmed |
| R229 | 596 | pre-2026-07 (undated); issue #196 | none recorded | not yet confirmed |
| R230 | 597 | pre-2026-07 (undated); issue #196 | none recorded | not yet confirmed |
| R231 | 603 | pre-2026-07 (undated); issue #278 | none recorded | not yet confirmed |
| R232 | 604 | pre-2026-07 (undated); issue #278 | none recorded | not yet confirmed |
| R233 | 605 | pre-2026-07 (undated); issue #278 | none recorded | not yet confirmed |
| R234 | 606 | pre-2026-07 (undated); issue #278 | none recorded | not yet confirmed |
| R235 | 607 | pre-2026-07 (undated); issue #278 | none recorded | not yet confirmed |
| R236 | 608 | pre-2026-07 (undated); issue #278 | none recorded | not yet confirmed |
| R237 | 619 | pre-2026-07 (undated); issue #279 | none recorded | not yet confirmed |
| R238 | 620 | pre-2026-07 (undated); issue #279 | none recorded | not yet confirmed |
| R239 | 621 | pre-2026-07 (undated); issue #279 | none recorded | not yet confirmed |
| R240 | 622 | pre-2026-07 (undated); issue #279 | none recorded | not yet confirmed |
| R241 | 623 | pre-2026-07 (undated); issue #279 | none recorded | not yet confirmed |
| R242 | 624 | pre-2026-07 (undated); issue #279 | none recorded | not yet confirmed |
| R243 | 634 | pre-2026-07 (undated); issue #283 | issue-level coverage (not row-pinned): tests/unit/specification/arbitrary_skill_compilation.rs | not yet confirmed |
| R244 | 635 | pre-2026-07 (undated); issue #283 | issue-level coverage (not row-pinned): tests/unit/specification/arbitrary_skill_compilation.rs | not yet confirmed |
| R245 | 636 | pre-2026-07 (undated); issue #283 | issue-level coverage (not row-pinned): tests/unit/specification/arbitrary_skill_compilation.rs | not yet confirmed |
| R246 | 647 | pre-2026-07 (undated); issue #327 | issue-level coverage (not row-pinned): tests/unit/specification/synthesis.rs:39 | not yet confirmed |
| R247 | 648 | pre-2026-07 (undated); issue #327 | tests/e2e/tests/issue-327.spec.js | not yet confirmed |
| R248 | 649 | pre-2026-07 (undated); issue #327 | issue-level coverage (not row-pinned): tests/unit/specification/synthesis.rs:39 | not yet confirmed |
| R249 | 650 | pre-2026-07 (undated); issue #327 | issue-level coverage (not row-pinned): tests/unit/specification/synthesis.rs:39 | not yet confirmed |
| R250 | 687 | pre-2026-07 (undated); issue #244 | none recorded | not yet confirmed |
| R251 | 688 | pre-2026-07 (undated); issue #244 | none recorded | not yet confirmed |
| R252 | 689 | pre-2026-07 (undated); issue #244 | none recorded | not yet confirmed |
| R253 | 690 | pre-2026-07 (undated); issue #244 | none recorded | not yet confirmed |
| R254 | 691 | pre-2026-07 (undated); issue #244 | none recorded | not yet confirmed |
| R255 | 692 | pre-2026-07 (undated); issue #244 | none recorded | not yet confirmed |
| R256 | 710 | pre-2026-07 (undated); issue #349 | tests/integration/issue_349_reverse_sort.rs::issue_349_reverse_sort_follow_up_must_not_be_unknown | not yet confirmed |
| R257 | 711 | pre-2026-07 (undated); issue #349 | tests/unit/specification/code_generation_coreference.rs | not yet confirmed |
| R258 | 712 | pre-2026-07 (undated); issue #349 | none recorded | not yet confirmed |
| R259 | 713 | pre-2026-07 (undated); issue #349 | tests/unit/specification/code_generation_program_modifiers.rs | not yet confirmed |
| R260 | 714 | pre-2026-07 (undated); issue #349 | tests/integration/issue_349_reverse_sort.rs::issue_349_diagnostic_mode_emits_full_turn_5_reasoning_chain; tests/e2e/tests/issue-360.spec.js | not yet confirmed |
| R261 | 715 | pre-2026-07 (undated); issue #349 | none recorded | not yet confirmed |
| R262 | 716 | pre-2026-07 (undated); issue #349 | tests/unit/specification/coding_modification_benchmarks.rs::issue_362_multilingual_multi_turn_coding_modification_ratchet | not yet confirmed |
| R263 | 717 | pre-2026-07 (undated); issue #349 | tests/e2e/tests/issue-363.spec.js | not yet confirmed |
| R264 | 718 | pre-2026-07 (undated); issue #349 | tests/unit/specification/self_improvement.rs | not yet confirmed |
| R265 | 719 | pre-2026-07 (undated); issue #349 | none recorded | not yet confirmed |
| R266 | 732 | pre-2026-07 (undated); issue #398 | issue-level coverage (not row-pinned): tests/unit/reference_closure.rs | not yet confirmed |
| R267 | 733 | pre-2026-07 (undated); issue #398 | issue-level coverage (not row-pinned): tests/unit/reference_closure.rs | not yet confirmed |
| R268 | 734 | pre-2026-07 (undated); issue #398 | issue-level coverage (not row-pinned): tests/unit/reference_closure.rs | not yet confirmed |
| R269 | 735 | pre-2026-07 (undated); issue #398 | issue-level coverage (not row-pinned): tests/unit/reference_closure.rs | not yet confirmed |
| R270 | 736 | pre-2026-07 (undated); issue #398 | issue-level coverage (not row-pinned): tests/unit/reference_closure.rs | not yet confirmed |
| R271 | 737 | pre-2026-07 (undated); issue #398 | issue-level coverage (not row-pinned): tests/unit/reference_closure.rs | not yet confirmed |
| R272 | 738 | pre-2026-07 (undated); issue #398 | issue-level coverage (not row-pinned): tests/unit/reference_closure.rs | not yet confirmed |
| R273 | 739 | pre-2026-07 (undated); issue #398 | issue-level coverage (not row-pinned): tests/unit/reference_closure.rs | not yet confirmed |
| R274 | 740 | pre-2026-07 (undated); issue #398 | issue-level coverage (not row-pinned): tests/unit/reference_closure.rs | not yet confirmed |
| R275 | 741 | pre-2026-07 (undated); issue #398 | tests/source/seed/embedded.rs | not yet confirmed |
| R276 | 742 | pre-2026-07 (undated); issue #398 | issue-level coverage (not row-pinned): tests/unit/reference_closure.rs | not yet confirmed |
| R277 | 743 | pre-2026-07 (undated); issue #398 | issue-level coverage (not row-pinned): tests/unit/reference_closure.rs | not yet confirmed |
| R278 | 761 | PR #399 (issue #398) | tests/unit/data_files.rs | not yet confirmed |
| R279 | 762 | PR #399 (issue #398) | tests/unit/overrides.rs | not yet confirmed |
| R280 | 763 | PR #399 (issue #398) | issue-level coverage (not row-pinned): tests/unit/reference_closure.rs | not yet confirmed |
| R281 | 764 | PR #399 (issue #398) | issue-level coverage (not row-pinned): tests/unit/reference_closure.rs | not yet confirmed |
| R282 | 765 | PR #399 (issue #398) | issue-level coverage (not row-pinned): tests/unit/reference_closure.rs | not yet confirmed |
| R283 | 766 | PR #399 (issue #398) | issue-level coverage (not row-pinned): tests/unit/reference_closure.rs | not yet confirmed |
| R284 | 779 | pre-2026-07 (undated); issue #398 | tests/unit/total_closure.rs | not yet confirmed |
| R285 | 780 | pre-2026-07 (undated); issue #398 | issue-level coverage (not row-pinned): tests/unit/reference_closure.rs | not yet confirmed |
| R286 | 781 | pre-2026-07 (undated); issue #398 | issue-level coverage (not row-pinned): tests/unit/reference_closure.rs | not yet confirmed |
| R287 | 782 | pre-2026-07 (undated); issue #398 | issue-level coverage (not row-pinned): tests/unit/reference_closure.rs | not yet confirmed |
| R288 | 783 | pre-2026-07 (undated); issue #398 | issue-level coverage (not row-pinned): tests/unit/reference_closure.rs | not yet confirmed |
| R289 | 795 | pre-2026-07 (undated); issue #412 | tests/integration/issue_412_oracle_languages.rs; source_tests/solver_handler_oracle | not yet confirmed |
| R290 | 796 | pre-2026-07 (undated); issue #412 | issue-level coverage (not row-pinned): tests/integration/issue_412_oracle_languages.rs | not yet confirmed |
| R291 | 797 | pre-2026-07 (undated); issue #412 | issue-level coverage (not row-pinned): tests/integration/issue_412_oracle_languages.rs | not yet confirmed |
| R292 | 798 | pre-2026-07 (undated); issue #412 | issue-level coverage (not row-pinned): tests/integration/issue_412_oracle_languages.rs | not yet confirmed |
| R293 | 811 | PR #416 (issue #408) | tests/unit/specification/text_manipulation.rs | not yet confirmed |
| R294 | 812 | PR #416 (issue #408) | none recorded | not yet confirmed |
| R295 | 813 | PR #416 (issue #408) | tests/unit/specification/text_manipulation_benchmarks.rs::issue_408_text_code_edit_profile_passes_local_ratchet | not yet confirmed |
| R296 | 814 | PR #416 (issue #408) | tests/unit/docs_requirements.rs::issue_408_text_edit_benchmark_scope_documents_are_traceable | not yet confirmed |
| R297 | 815 | PR #416 (issue #408) | none recorded | not yet confirmed |
| R298 | 829 | PR #452 (issue #451) | issue-level coverage (not row-pinned): tests/unit/docs_requirements_issue_451.rs | not yet confirmed |
| R299 | 830 | PR #452 (issue #451) | issue-level coverage (not row-pinned): tests/unit/docs_requirements_issue_451.rs | not yet confirmed |
| R300 | 831 | PR #452 (issue #451) | issue-level coverage (not row-pinned): tests/unit/docs_requirements_issue_451.rs | not yet confirmed |
| R301 | 832 | PR #452 (issue #451) | issue-level coverage (not row-pinned): tests/unit/docs_requirements_issue_451.rs | not yet confirmed |
| R302 | 833 | PR #452 (issue #451) | issue-level coverage (not row-pinned): tests/unit/docs_requirements_issue_451.rs | not yet confirmed |
| R303 | 834 | PR #452 (issue #451) | issue-level coverage (not row-pinned): tests/unit/docs_requirements_issue_451.rs | not yet confirmed |
| R304 | 835 | PR #452 (issue #451) | issue-level coverage (not row-pinned): tests/unit/docs_requirements_issue_451.rs | not yet confirmed |
| R305 | 836 | PR #452 (issue #451) | tests/source/source_tests/proof_engine/decision/{sat,boolean}/tests.rs | not yet confirmed |
| R306 | 859 | PR #469 (issue #468) | issue-level coverage (not row-pinned): tests/integration/issue_716_agentic_execution.rs | not yet confirmed |
| R307 | 860 | PR #469 (issue #468) | tests/unit/agentic_coding.rs | not yet confirmed |
| R308 | 861 | PR #469 (issue #468) | issue-level coverage (not row-pinned): tests/integration/issue_716_agentic_execution.rs | not yet confirmed |
| R309 | 862 | PR #469 (issue #468) | issue-level coverage (not row-pinned): tests/integration/issue_716_agentic_execution.rs | not yet confirmed |
| R310 | 863 | PR #469 (issue #468) | tests/unit/agentic_coding.rs | not yet confirmed |
| R311 | 864 | PR #469 (issue #468) | issue-level coverage (not row-pinned): tests/integration/issue_716_agentic_execution.rs | not yet confirmed |
| R312 | 865 | PR #469 (issue #468) | tests/unit/agentic_coding.rs | not yet confirmed |
| R313 | 866 | PR #469 (issue #468) | issue-level coverage (not row-pinned): tests/integration/issue_716_agentic_execution.rs | not yet confirmed |
| R314 | 867 | PR #469 (issue #468) | tests/unit/agentic_coding.rs; tests/unit/agentic_surfaces.rs | manually confirmed 2026-08-04 (audit): `formal-ai agent --help` run; offline `agent --silent --task ...` exit 0 (see audit finding: falls back to seeded fairy-tale KB rather than reflecting custom --task) |
| R315 | 868 | PR #469 (issue #468) | issue-level coverage (not row-pinned): tests/integration/issue_716_agentic_execution.rs | not yet confirmed |
| R316 | 869 | PR #469 (issue #468) | issue-level coverage (not row-pinned): tests/integration/issue_716_agentic_execution.rs | not yet confirmed |
| R317 | 870 | PR #469 (issue #468) | issue-level coverage (not row-pinned): tests/integration/issue_716_agentic_execution.rs | not yet confirmed |
| R318 | 871 | PR #469 (issue #468) | issue-level coverage (not row-pinned): tests/integration/issue_716_agentic_execution.rs | not yet confirmed |
| R319 | 872 | PR #469 (issue #468) | tests/unit/agentic_coding.rs; tests/unit/agentic_surfaces.rs | not yet confirmed |
| R320 | 888 | PR #470 (issue #438) | none recorded | not yet confirmed |
| R321 | 889 | PR #470 (issue #438) | none recorded | not yet confirmed |
| R322 | 890 | PR #470 (issue #438) | none recorded | not yet confirmed |
| R323 | 891 | PR #470 (issue #438) | none recorded | not yet confirmed |
| R324 | 892 | PR #470 (issue #438) | none recorded | not yet confirmed |
| R325 | 893 | PR #470 (issue #438) | none recorded | not yet confirmed |
| R326 | 894 | PR #470 (issue #438) | tests/unit/docker_runtime.rs::compose_file_runs_prebuilt_telegram_image_with_minimum_configuration; tests/unit/ci-cd/release_publishing.rs::release_workflow_publishes_prebuilt_ghcr_image_after_crate_is_visible_and_optional_docker_hub_mirror | not yet confirmed |
| R327 | 895 | PR #470 (issue #438) | none recorded | not yet confirmed |
| R328 | 896 | PR #470 (issue #438) | none recorded | not yet confirmed |
| R329 | 897 | PR #470 (issue #438) | tests/unit/specification/desktop_surface.rs::{desktop_service_control_starts_and_stops_prepared_containers,desktop_web_surface_exposes_one_click_service_controls}; tests/unit/docker_runtime.rs::compose_file_offers_optional_openai_compatible_server_profile | not yet confirmed |
| R330 | 918 | PR #560 (issue #559) | tests/unit/specification/meta_frame.rs; tests/unit/docs_requirements_issue_559.rs::issue_559_problem_frame_is_traceable | not yet confirmed |
| R331 | 919 | PR #560 (issue #559) | tests/unit/specification/method_registry.rs; tests/unit/specification/reasoning_paths.rs::selected_specialized_handler_is_recorded_as_a_meta_method | not yet confirmed |
| R332 | 920 | PR #560 (issue #559) | tests/unit/specification/meta_frame.rs; tests/unit/specification/reasoning_loop.rs::handler_families_publish_loop_events_as_recursion_leaves | not yet confirmed |
| R333 | 921 | PR #560 (issue #559) | tests/unit/specification/meta_frame.rs; tests/unit/docs_requirements_issue_559.rs::issue_559_need_ledger_is_traceable | not yet confirmed |
| R334 | 922 | PR #560 (issue #559) | tests/unit/specification/solution_evidence.rs; tests/unit/docs_requirements_issue_559.rs::issue_559_solution_evidence_is_traceable | not yet confirmed |
| R335 | 923 | PR #560 (issue #559) | tests/unit/specification/recursive_core_recipe.rs; tests/unit/docs_requirements_issue_559.rs::issue_559_recursive_core_recipe_is_traceable | not yet confirmed |
| R336 | 924 | PR #560 (issue #559) | tests/unit/specification/route_method_alias.rs; tests/unit/docs_requirements_issue_559.rs::issue_559_route_method_alias_is_traceable | not yet confirmed |
| R337 | 925 | PR #560 (issue #559) | tests/unit/specification/meta_reasoning.rs; tests/unit/docs_requirements_issue_559.rs::issue_559_work_unit_reasoning_is_traceable | not yet confirmed |
| R338 | 926 | PR #560 (issue #559) | tests/unit/specification/meta_construction.rs; tests/unit/docs_requirements_issue_559.rs::issue_559_upward_construction_is_traceable | not yet confirmed |
| R339 | 927 | PR #560 (issue #559) | tests/unit/specification/selection.rs; tests/unit/docs_requirements_issue_559.rs::issue_559_selection_trace_is_traceable | not yet confirmed |
| R340 | 928 | PR #560 (issue #559) | tests/unit/specification/meta_self_improvement.rs; tests/unit/docs_requirements_issue_559.rs::issue_559_meta_self_improvement_is_traceable | not yet confirmed |
| R341 | 929 | PR #560 (issue #559) | tests/unit/specification/cue_lexicon.rs; tests/unit/docs_requirements_issue_559.rs::issue_559_cue_lexicon_is_traceable | not yet confirmed |
| R342 | 930 | PR #560 (issue #559) | tests/unit/specification/skill_ledger.rs; tests/unit/docs_requirements_issue_559.rs::issue_559_skill_ledger_is_traceable | not yet confirmed |
| R343 | 931 | PR #560 (issue #559) | tests/unit/specification/recipe_interpreter.rs; tests/unit/docs_requirements_issue_559.rs::issue_559_recipe_interpreter_is_traceable | not yet confirmed |
| R344 | 932 | PR #560 (issue #559) | tests/unit/issue_699_handler_migration.rs | not yet confirmed |
| R345 | 956 | PR #564 (issue #563) | tests/unit/specification/summarization_pipeline.rs::repository_file_summary_recurses_into_markdown_embedded_grammars | not yet confirmed |
| R346 | 957 | PR #564 (issue #563) | none recorded | not yet confirmed |
| R347 | 958 | PR #564 (issue #563) | none recorded | not yet confirmed |
| R348 | 959 | PR #564 (issue #563) | none recorded | not yet confirmed |
| R349 | 960 | PR #564 (issue #563) | none recorded | not yet confirmed |
| R350 | 961 | PR #564 (issue #563) | none recorded | not yet confirmed |
| R351 | 962 | PR #564 (issue #563) | tests/source/source_tests/summarization/mod/tests.rs::formalize_repository_file_rust_records_meta_language_and_symbols; tests/unit/docs_requirements_issue_563.rs::issue_563_repository_file_summarization_documents_are_traceable | not yet confirmed |
| R352 | 963 | PR #564 (issue #563) | none recorded | not yet confirmed |
| R353 | 964 | PR #564 (issue #563) | none recorded | not yet confirmed |
| R354 | 965 | PR #564 (issue #563) | none recorded | not yet confirmed |
| R355 | 966 | PR #564 (issue #563) | tests/unit/specification/summarization_pipeline.rs::summarize_repository_resource_subsumes_file_summarization | not yet confirmed |
| R356 | 967 | PR #564 (issue #563) | none recorded | not yet confirmed |
| R357 | 968 | PR #564 (issue #563) | none recorded | not yet confirmed |
| R358 | 969 | PR #564 (issue #563) | none recorded | not yet confirmed |
| R359 | 970 | PR #564 (issue #563) | tests/source/source_tests/summarization/mod/tests.rs::{summarize_repository_resource_topic_directory_is_identity_only,summarize_repository_resource_full_directory_recurses_into_nested_folder} | not yet confirmed |
| R360 | 986 | PR #583 (issue #492) | none recorded | not yet confirmed |
| R361 | 987 | PR #583 (issue #492) | none recorded | not yet confirmed |
| R362 | 988 | PR #583 (issue #492) | none recorded | not yet confirmed |
| R363 | 989 | PR #583 (issue #492) | none recorded | not yet confirmed |
| R364 | 990 | PR #583 (issue #492) | none recorded | not yet confirmed |
| R365 | 991 | PR #583 (issue #492) | none recorded | not yet confirmed |
| R366 | 992 | PR #583 (issue #492) | none recorded | not yet confirmed |
| R367 | 993 | PR #583 (issue #492) | none recorded | not yet confirmed |
| R368 | 994 | PR #583 (issue #492) | none recorded | not yet confirmed |
| R369 | 995 | PR #583 (issue #492) | none recorded | not yet confirmed |
| R499-1 | 1012 | PR #641 (issue #499) | issue-level coverage (not row-pinned): tests/unit/issue_499_learn_from_source.rs | not yet confirmed |
| R499-2 | 1013 | PR #641 (issue #499) | tests/unit/issue_499_learn_from_source.rs | not yet confirmed |
| R499-3 | 1014 | PR #641 (issue #499) | issue-level coverage (not row-pinned): tests/unit/issue_499_learn_from_source.rs | not yet confirmed |
| R499-4 | 1015 | PR #641 (issue #499) | issue-level coverage (not row-pinned): tests/unit/issue_499_learn_from_source.rs | not yet confirmed |
| R499-5 | 1016 | PR #641 (issue #499) | issue-level coverage (not row-pinned): tests/unit/issue_499_learn_from_source.rs | not yet confirmed |
| R499-6 | 1017 | PR #641 (issue #499) | issue-level coverage (not row-pinned): tests/unit/issue_499_learn_from_source.rs | not yet confirmed |
| R499-7 | 1018 | PR #641 (issue #499) | issue-level coverage (not row-pinned): tests/unit/issue_499_learn_from_source.rs | not yet confirmed |
| R499-8 | 1019 | PR #641 (issue #499) | tests/unit/issue_499_learn_from_source.rs; tests/unit/docs_requirements_issue_499.rs | not yet confirmed |
| R370 | 1042 | PR #601 (issue #538) | tests/unit/issue_538.rs::tomato_surfaces_pin_their_grammatical_number | not yet confirmed |
| R371 | 1043 | PR #601 (issue #538) | tests/unit/issue_538.rs::tomato_surfaces_expose_part_of_speech_from_data | not yet confirmed |
| R372 | 1044 | PR #601 (issue #538) | tests/unit/issue_538.rs::every_tomato_surface_denotes_the_tomato_meaning | not yet confirmed |
| R373 | 1045 | PR #601 (issue #538) | tests/unit/issue_538.rs::tomato_singular_and_plural_are_distinct_forms_in_each_language | not yet confirmed |
| R374 | 1046 | PR #601 (issue #538) | tests/unit/semantic_grounding.rs | not yet confirmed |
| R375 | 1047 | PR #601 (issue #538) | none recorded | not yet confirmed |
| R376 | 1048 | PR #601 (issue #538) | tests/unit/issue_538.rs::grammatical_number_meanings_are_grounded_and_multilingual | not yet confirmed |
| R377 | 1049 | PR #601 (issue #538) | none recorded | not yet confirmed |
| R378 | 1050 | PR #601 (issue #538) | none recorded | not yet confirmed |
| R379 | 1051 | PR #601 (issue #538) | none recorded | not yet confirmed |
| R380 | 1052 | PR #601 (issue #538) | none recorded | not yet confirmed |
| R381 | 1053 | PR #601 (issue #538) | none recorded | not yet confirmed |
| R382 | 1054 | PR #601 (issue #538) | none recorded | not yet confirmed |
| R383 | 1055 | not delivered — untracked | none — untracked | not yet confirmed |
| R384 | 1056 | PR #601 (issue #538) | none recorded | not yet confirmed |
| R385 | 1057 | PR #601 (issue #538) | tests/unit/issue_538_agentic.rs | not yet confirmed |
| R386 | 1058 | PR #601 (issue #538) | none recorded | not yet confirmed |
| R387 | 1073 | PR #637 (issue #558) | none recorded | not yet confirmed |
| R388 | 1074 | PR #637 (issue #558) | none recorded | not yet confirmed |
| R389 | 1075 | PR #637 (issue #558) | none recorded | not yet confirmed |
| R390 | 1076 | PR #637 (issue #558) | none recorded | not yet confirmed |
| R391 | 1077 | PR #637 (issue #558) | none recorded | not yet confirmed |
| R392 | 1078 | PR #637 (issue #558) | tests/unit/docs_requirements_issue_558.rs; tests/unit/mod.rs | not yet confirmed |
| R393 | 1079 | PR #637 (issue #558) | tests/unit/issue_558_self_healing.rs | not yet confirmed |
| R394 | 1080 | PR #637 (issue #558) | tests/unit/issue_558_self_healing.rs | not yet confirmed |
| R395 | 1081 | PR #637 (issue #558) | tests/unit/issue_558_self_healing.rs; tests/integration/issue_558_self_healing.rs | not yet confirmed |
| R396 | 1095 | PR #642 (issue #531) | none recorded | not yet confirmed |
| R397 | 1096 | PR #642 (issue #531) | none recorded | not yet confirmed |
| R398 | 1097 | PR #642 (issue #531) | none recorded | not yet confirmed |
| R399 | 1098 | PR #642 (issue #531) | none recorded | not yet confirmed |
| R400 | 1099 | PR #642 (issue #531) | none recorded | not yet confirmed |
| R401 | 1100 | PR #642 (issue #531) | none recorded | not yet confirmed |
| R402 | 1101 | PR #642 (issue #531) | tests/unit/sequences_{store,symbols,converter,compression}.rs | not yet confirmed |
| R403 | 1102 | PR #642 (issue #531) | tests/unit/sequences_{patterns_1d,grid_2d,inference}.rs | not yet confirmed |
| R404 | 1103 | PR #642 (issue #531) | tests/unit/issue_531_concepts_probe.rs | not yet confirmed |
| R405 | 1104 | PR #642 (issue #531) | tests/unit/issue_531_pattern_inference.rs | not yet confirmed |
| R406 | 1105 | PR #642 (issue #531) | none recorded | not yet confirmed |
| R407 | 1106 | PR #642 (issue #531) | tests/unit/docs_requirements_issue_531.rs; tests/unit/mod.rs | manually confirmed 2026-08-04 (audit): `npm --prefix desktop run smoke` passed (desktop/scripts/smoke.mjs) |
| R531-17 | 1107 | PR #642 (issue #531) | none recorded | not yet confirmed |
| R531-18 | 1108 | PR #642 (issue #531) | tests/unit/issue_531_algorithm_discovery.rs | not yet confirmed |
| R531-19 | 1109 | PR #642 (issue #531) | none recorded | not yet confirmed |
| R531-20 | 1110 | PR #642 (issue #531) | none recorded | not yet confirmed |
| R531-21 | 1111 | PR #642 (issue #531) | none recorded | not yet confirmed |
| R531-22 | 1112 | PR #642 (issue #531) | none recorded | not yet confirmed |
| R531-23 | 1113 | PR #642 (issue #531) | none recorded | not yet confirmed |
| R531-24 | 1114 | PR #642 (issue #531) | none recorded | not yet confirmed |
| R531-25 | 1115 | PR #642 (issue #531) | none recorded | not yet confirmed |
| R537 | 1138 | PR #645 (issue #540) | tests/unit/docs_requirements_issue_540.rs | not yet confirmed |
| R538 | 1139 | PR #645 (issue #540) | none recorded | not yet confirmed |
| R539 | 1140 | PR #645 (issue #540) | none recorded | not yet confirmed |
| R540 | 1141 | PR #645 (issue #540) | none recorded | not yet confirmed |
| R541 | 1142 | PR #645 (issue #540) | none recorded | not yet confirmed |
| R542 | 1143 | PR #645 (issue #540) | none recorded | not yet confirmed |
| R543 | 1144 | PR #645 (issue #540) | none recorded | not yet confirmed |
| R544 | 1145 | PR #645 (issue #540) | none recorded | not yet confirmed |
| R545 | 1146 | PR #645 (issue #540) | none recorded | not yet confirmed |
| R546 | 1147 | PR #645 (issue #540) | none recorded | not yet confirmed |
| R547 | 1148 | PR #645 (issue #540) | none recorded | not yet confirmed |
| R548 | 1149 | PR #645 (issue #540) | tests/unit/memory_maintenance.rs; tests/unit/docs_requirements_issue_540.rs | manually confirmed 2026-08-04 (audit): `npm --prefix desktop run smoke` passed (desktop/scripts/smoke.mjs) |
| R408 | 1150 | PR #645 (issue #540) | none recorded | not yet confirmed |
| R409 | 1151 | PR #645 (issue #540) | none recorded | not yet confirmed |
| R410 | 1152 | PR #645 (issue #540) | none recorded | not yet confirmed |
| R411 | 1153 | PR #645 (issue #540) | none recorded | not yet confirmed |
| R412 | 1154 | PR #645 (issue #540) | tests/unit/specification/dreaming_meta_algorithm.rs | not yet confirmed |
| R413 | 1155 | PR #645 (issue #540) | none recorded | not yet confirmed |
| R414 | 1156 | PR #645 (issue #540) | none recorded | not yet confirmed |
| R415 | 1157 | PR #645 (issue #540) | none recorded | not yet confirmed |
| R416 | 1158 | PR #645 (issue #540) | none recorded | not yet confirmed |
| R417 | 1159 | PR #645 (issue #540) | none recorded | not yet confirmed |
| R418 | 1160 | PR #645 (issue #540) | none recorded | not yet confirmed |
| R419 | 1161 | PR #645 (issue #540) | none recorded | not yet confirmed |
| R420 | 1162 | PR #645 (issue #540) | none recorded | not yet confirmed |
| R421 | 1163 | PR #645 (issue #540) | none recorded | not yet confirmed |
| R422 | 1164 | PR #645 (issue #540) | tests/unit/issue_540_agent_cli.rs | not yet confirmed |
| R423 | 1165 | PR #645 (issue #540) | tests/unit/memory_learning.rs | not yet confirmed |
| R424 | 1166 | PR #645 (issue #540) | none recorded | not yet confirmed |
| R425 | 1167 | PR #645 (issue #540) | none recorded | not yet confirmed |
| R426 | 1168 | PR #645 (issue #540) | none recorded | not yet confirmed |
| R427 | 1169 | PR #645 (issue #540) | tests/unit/dreaming_runtime.rs | not yet confirmed |
| R428 | 1194 | PR #675 (issue #649) | tests/unit/docs_requirements_issue_649.rs | not yet confirmed |
| R429 | 1195 | PR #675 (issue #649) | issue-level coverage (not row-pinned): tests/unit/issue_702_world_model_dialog.rs:152 | not yet confirmed |
| R430 | 1196 | PR #675 (issue #649) | issue-level coverage (not row-pinned): tests/unit/issue_702_world_model_dialog.rs:152 | not yet confirmed |
| R431 | 1197 | PR #675 (issue #649) | issue-level coverage (not row-pinned): tests/unit/issue_702_world_model_dialog.rs:152 | not yet confirmed |
| R432 | 1198 | PR #675 (issue #649) | issue-level coverage (not row-pinned): tests/unit/issue_702_world_model_dialog.rs:152 | not yet confirmed |
| R433 | 1199 | PR #675 (issue #649) | issue-level coverage (not row-pinned): tests/unit/issue_702_world_model_dialog.rs:152 | not yet confirmed |
| R434 | 1200 | PR #675 (issue #649) | tests/unit/docs_requirements_issue_649.rs; tests/unit/mod.rs | not yet confirmed |
| R435 | 1213 | PR #639 (issue #482) | issue-level coverage (not row-pinned): tests/unit/specification/nemotron_training_samples.rs | not yet confirmed |
| R436 | 1214 | PR #639 (issue #482) | issue-level coverage (not row-pinned): tests/unit/specification/nemotron_training_samples.rs | not yet confirmed |
| R437 | 1215 | PR #639 (issue #482) | issue-level coverage (not row-pinned): tests/unit/specification/nemotron_training_samples.rs | not yet confirmed |
| R438 | 1216 | PR #639 (issue #482) | issue-level coverage (not row-pinned): tests/unit/specification/nemotron_training_samples.rs | not yet confirmed |
| R439 | 1217 | PR #639 (issue #482) | issue-level coverage (not row-pinned): tests/unit/specification/nemotron_training_samples.rs | not yet confirmed |
| R440 | 1218 | PR #639 (issue #482) | tests/unit/specification/nemotron_training_samples.rs | not yet confirmed |
| R441 | 1219 | PR #639 (issue #482) | issue-level coverage (not row-pinned): tests/unit/specification/nemotron_training_samples.rs | not yet confirmed |
| R442 | 1220 | PR #639 (issue #482) | issue-level coverage (not row-pinned): tests/unit/specification/nemotron_training_samples.rs | not yet confirmed |
| R443 | 1221 | PR #639 (issue #482) | issue-level coverage (not row-pinned): tests/unit/specification/nemotron_training_samples.rs | not yet confirmed |
| R444 | 1222 | PR #639 (issue #482) | tests/unit/docs_requirements_issue_482.rs; tests/unit/mod.rs | not yet confirmed |
| R445 | 1249 | PR #689 (issue #686) | tests/unit/docs_requirements_issue_686.rs | not yet confirmed |
| R446 | 1250 | PR #689 (issue #686) | issue-level coverage (not row-pinned): tests/unit/issue_686_associative_persistence.rs | not yet confirmed |
| R447 | 1251 | PR #689 (issue #686) | issue-level coverage (not row-pinned): tests/unit/issue_686_associative_persistence.rs | not yet confirmed |
| R448 | 1252 | PR #689 (issue #686) | issue-level coverage (not row-pinned): tests/unit/issue_686_associative_persistence.rs | not yet confirmed |
| R449 | 1253 | PR #689 (issue #686) | issue-level coverage (not row-pinned): tests/unit/issue_686_associative_persistence.rs | not yet confirmed |
| R450 | 1254 | PR #689 (issue #686) | tests/unit/issue_686_associative_persistence.rs | not yet confirmed |
| R451 | 1255 | PR #689 (issue #686) | issue-level coverage (not row-pinned): tests/unit/issue_686_associative_persistence.rs | not yet confirmed |
| R452 | 1256 | PR #689 (issue #686) | tests/unit/docs_requirements_issue_686.rs; tests/unit/mod.rs | not yet confirmed |
| R453 | 1257 | PR #689 (issue #686) | issue-level coverage (not row-pinned): tests/unit/issue_686_associative_persistence.rs | not yet confirmed |
| R454 | 1258 | PR #689 (issue #686) | issue-level coverage (not row-pinned): tests/unit/issue_686_associative_persistence.rs | not yet confirmed |
| R455 | 1259 | PR #689 (issue #686) | issue-level coverage (not row-pinned): tests/unit/issue_686_associative_persistence.rs | not yet confirmed |
| R456 | 1260 | PR #689 (issue #686) | issue-level coverage (not row-pinned): tests/unit/issue_686_associative_persistence.rs | not yet confirmed |
| R457 | 1261 | PR #689 (issue #686) | issue-level coverage (not row-pinned): tests/unit/issue_686_associative_persistence.rs | not yet confirmed |
| R458 | 1262 | PR #689 (issue #686) | issue-level coverage (not row-pinned): tests/unit/issue_686_associative_persistence.rs | not yet confirmed |
| R459 | 1276 | PR #690 (issue #656) | issue-level coverage (not row-pinned): tests/integration/issue_656_improve.rs | not yet confirmed |
| R460 | 1277 | PR #690 (issue #656) | issue-level coverage (not row-pinned): tests/integration/issue_656_improve.rs | not yet confirmed |
| R461 | 1278 | PR #690 (issue #656) | issue-level coverage (not row-pinned): tests/integration/issue_656_improve.rs | not yet confirmed |
| R462 | 1279 | PR #690 (issue #656) | issue-level coverage (not row-pinned): tests/integration/issue_656_improve.rs | not yet confirmed |
| R463 | 1280 | PR #690 (issue #656) | tests/integration/issue_656_improve.rs | not yet confirmed |
| R464 | 1281 | PR #690 (issue #656) | issue-level coverage (not row-pinned): tests/integration/issue_656_improve.rs | not yet confirmed |
| R465 | 1282 | PR #690 (issue #656) | tests/unit/docs_requirements_issue_656.rs | not yet confirmed |
| R466 | 1283 | PR #690 (issue #656) | issue-level coverage (not row-pinned): tests/integration/issue_656_improve.rs | not yet confirmed |
| R467 | 1284 | PR #690 (issue #656) | issue-level coverage (not row-pinned): tests/integration/issue_656_improve.rs | not yet confirmed |
| R468 | 1285 | PR #690 (issue #656) | issue-level coverage (not row-pinned): tests/integration/issue_656_improve.rs | not yet confirmed |
| R469 | 1286 | PR #690 (issue #656) | issue-level coverage (not row-pinned): tests/integration/issue_656_improve.rs | not yet confirmed |
| R470 | 1287 | PR #690 (issue #656) | issue-level coverage (not row-pinned): tests/integration/issue_656_improve.rs | not yet confirmed |
| R471 | 1288 | PR #690 (issue #656) | issue-level coverage (not row-pinned): tests/integration/issue_656_improve.rs | not yet confirmed |
| R472 | 1289 | PR #690 (issue #656) | issue-level coverage (not row-pinned): tests/integration/issue_656_improve.rs | not yet confirmed |
| R473 | 1302 | PR #735 (issue #657) | issue-level coverage (not row-pinned): tests/unit/issue_657_self_hosting_learning.rs | not yet confirmed |
| R474 | 1303 | PR #735 (issue #657) | issue-level coverage (not row-pinned): tests/unit/issue_657_self_hosting_learning.rs | not yet confirmed |
| R475 | 1304 | PR #735 (issue #657) | issue-level coverage (not row-pinned): tests/unit/issue_657_self_hosting_learning.rs | not yet confirmed |
| R476 | 1305 | PR #735 (issue #657) | issue-level coverage (not row-pinned): tests/unit/issue_657_self_hosting_learning.rs | not yet confirmed |
| R477 | 1306 | PR #735 (issue #657) | issue-level coverage (not row-pinned): tests/unit/issue_657_self_hosting_learning.rs | not yet confirmed |
| R478 | 1307 | PR #735 (issue #657) | issue-level coverage (not row-pinned): tests/unit/issue_657_self_hosting_learning.rs | not yet confirmed |
| R479 | 1308 | PR #735 (issue #657) | issue-level coverage (not row-pinned): tests/unit/issue_657_self_hosting_learning.rs | not yet confirmed |
| R549 | 1309 | PR #735 (issue #657) | issue-level coverage (not row-pinned): tests/unit/issue_657_self_hosting_learning.rs | not yet confirmed |
| R480 | 1322 | PR #807 (issue #673) | issue-level coverage (not row-pinned): tests/unit/issue_673_self_ast_census.rs | not yet confirmed |
| R481 | 1323 | PR #807 (issue #673) | issue-level coverage (not row-pinned): tests/unit/issue_673_self_ast_census.rs | not yet confirmed |
| R482 | 1324 | PR #807 (issue #673) | issue-level coverage (not row-pinned): tests/unit/issue_673_self_ast_census.rs | not yet confirmed |
| R483 | 1325 | PR #807 (issue #673) | issue-level coverage (not row-pinned): tests/unit/issue_673_self_ast_census.rs | not yet confirmed |
| R701-1 | 1339 | PR #817 (issue #701) | issue-level coverage (not row-pinned): tests/unit/issue_701_learning_adoption.rs | not yet confirmed |
| R701-2 | 1340 | PR #817 (issue #701) | issue-level coverage (not row-pinned): tests/unit/issue_701_learning_adoption.rs | not yet confirmed |
| R701-3 | 1341 | PR #817 (issue #701) | issue-level coverage (not row-pinned): tests/unit/issue_701_learning_adoption.rs | not yet confirmed |
| R701-4 | 1342 | PR #817 (issue #701) | tests/unit/issue_701_dreaming_amendment_class.rs | not yet confirmed |
| R701-5 | 1343 | PR #817 (issue #701) | issue-level coverage (not row-pinned): tests/unit/issue_701_learning_adoption.rs | not yet confirmed |
| R701-6 | 1344 | PR #817 (issue #701) | issue-level coverage (not row-pinned): tests/unit/issue_701_learning_adoption.rs | not yet confirmed |
| R550 | 1358 | PR #815 (issue #674) | none recorded | not yet confirmed |
| R551 | 1359 | PR #815 (issue #674) | none recorded | not yet confirmed |
| R552 | 1360 | PR #815 (issue #674) | none recorded | not yet confirmed |
| R553 | 1361 | PR #815 (issue #674) | none recorded | not yet confirmed |
| R554 | 1362 | PR #815 (issue #674) | none recorded | not yet confirmed |
| R555 | 1363 | PR #815 (issue #674) | none recorded | not yet confirmed |
| R556 | 1364 | PR #815 (issue #674) | none recorded | not yet confirmed |
| R557 | 1365 | PR #815 (issue #674) | none recorded | not yet confirmed |
| R558 | 1366 | PR #815 (issue #674) | none recorded | not yet confirmed |
| R528 | 1380 | PR #816 (issue #698) | none recorded | not yet confirmed |
| R529 | 1381 | PR #816 (issue #698) | none recorded | not yet confirmed |
| R530 | 1382 | PR #816 (issue #698) | none recorded | not yet confirmed |
| R531 | 1383 | PR #816 (issue #698) | none recorded | not yet confirmed |
| R532 | 1384 | PR #816 (issue #698) | none recorded | not yet confirmed |
| R533 | 1385 | PR #816 (issue #698) | none recorded | not yet confirmed |
| R534 | 1386 | PR #816 (issue #698) | none recorded | not yet confirmed |
| R535 | 1387 | PR #816 (issue #698) | none recorded | not yet confirmed |
| R702-1 | 1410 | PR #675 (issue #702) | none recorded | not yet confirmed |
| R702-2 | 1411 | PR #675 (issue #702) | none recorded | not yet confirmed |
| R702-3 | 1412 | PR #675 (issue #702) | none recorded | not yet confirmed |
| R702-4 | 1413 | PR #675 (issue #702) | none recorded | not yet confirmed |
| R702-5 | 1414 | PR #675 (issue #702) | none recorded | not yet confirmed |
| R702-6 | 1415 | PR #675 (issue #702) | none recorded | not yet confirmed |
| R702-7 | 1416 | PR #675 (issue #702) | none recorded | not yet confirmed |
| R702-8 | 1417 | PR #675 (issue #702) | none recorded | not yet confirmed |
| R702-9 | 1418 | PR #675 (issue #702) | none recorded | not yet confirmed |
| R702-10 | 1419 | PR #675 (issue #702) | tests/unit/docs_requirements_issue_702.rs | not yet confirmed |
| R702-11 | 1420 | PR #675 (issue #702) | none recorded | not yet confirmed |
| R702-12 | 1421 | PR #675 (issue #702) | none recorded | not yet confirmed |
| R702-13 | 1422 | PR #675 (issue #702) | none recorded | not yet confirmed |
| R702-14 | 1423 | PR #675 (issue #702) | none recorded | not yet confirmed |
| R702-15 | 1424 | PR #675 (issue #702) | none recorded | not yet confirmed |
| R702-16 | 1425 | PR #675 (issue #702) | none recorded | not yet confirmed |
| R703-1 | 1442 | PR #876 (issue #703) | tests/integration/issue_703_orchestration.rs | not yet confirmed |
| R703-2 | 1443 | PR #876 (issue #703) | none recorded | not yet confirmed |
| R703-3 | 1444 | PR #876 (issue #703) | none recorded | not yet confirmed |
| R703-4 | 1445 | PR #876 (issue #703) | none recorded | not yet confirmed |
| R703-5 | 1446 | PR #876 (issue #703) | none recorded | not yet confirmed |
| R703-6 | 1447 | PR #876 (issue #703) | none recorded | not yet confirmed |
| R703-7 | 1448 | PR #876 (issue #703) | none recorded | not yet confirmed |
| R703-8 | 1449 | PR #876 (issue #703) | none recorded | not yet confirmed |
| R703-9 | 1450 | PR #876 (issue #703) | none recorded | not yet confirmed |
| R703-10 | 1451 | PR #876 (issue #703) | none recorded | not yet confirmed |
| R703-11 | 1452 | PR #876 (issue #703) | none recorded | not yet confirmed |
| R703-12 | 1453 | PR #876 (issue #703) | none recorded | not yet confirmed |
| R484 | 1469 | PR #837 (issue #834) | issue-level coverage (not row-pinned): tests/unit/docs_requirements_issue_834.rs | not yet confirmed |
| R485 | 1470 | PR #837 (issue #834) | issue-level coverage (not row-pinned): tests/unit/docs_requirements_issue_834.rs | not yet confirmed |
| R486 | 1471 | PR #837 (issue #834) | issue-level coverage (not row-pinned): tests/unit/docs_requirements_issue_834.rs | not yet confirmed |
| R487 | 1472 | PR #837 (issue #834) | issue-level coverage (not row-pinned): tests/unit/docs_requirements_issue_834.rs | not yet confirmed |
| R488 | 1473 | PR #837 (issue #834) | issue-level coverage (not row-pinned): tests/unit/docs_requirements_issue_834.rs | not yet confirmed |
| R489 | 1474 | PR #837 (issue #834) | issue-level coverage (not row-pinned): tests/unit/docs_requirements_issue_834.rs | not yet confirmed |
| R490 | 1475 | PR #837 (issue #834) | issue-level coverage (not row-pinned): tests/unit/docs_requirements_issue_834.rs | not yet confirmed |
| R491 | 1476 | PR #837 (issue #834) | issue-level coverage (not row-pinned): tests/unit/docs_requirements_issue_834.rs | not yet confirmed |
| R492 | 1477 | PR #837 (issue #834) | issue-level coverage (not row-pinned): tests/unit/docs_requirements_issue_834.rs | not yet confirmed |
| R493 | 1478 | PR #837 (issue #834) | tests/unit/docs_requirements_issue_834.rs | not yet confirmed |
| R494 | 1493 | pre-2026-07 (undated); issue #839 | issue-level coverage (not row-pinned): tests/integration/issue_839_context_export.rs | not yet confirmed |
| R495 | 1494 | pre-2026-07 (undated); issue #839 | issue-level coverage (not row-pinned): tests/integration/issue_839_context_export.rs | not yet confirmed |
| R496 | 1495 | pre-2026-07 (undated); issue #839 | issue-level coverage (not row-pinned): tests/integration/issue_839_context_export.rs | not yet confirmed |
| R497 | 1496 | pre-2026-07 (undated); issue #839 | tests/integration/issue_839_report_parity.rs | not yet confirmed |
| R498 | 1497 | pre-2026-07 (undated); issue #839 | issue-level coverage (not row-pinned): tests/integration/issue_839_context_export.rs | not yet confirmed |
| R499 | 1498 | pre-2026-07 (undated); issue #839 | issue-level coverage (not row-pinned): tests/integration/issue_839_context_export.rs | not yet confirmed |
| R500 | 1499 | pre-2026-07 (undated); issue #839 | issue-level coverage (not row-pinned): tests/integration/issue_839_context_export.rs | not yet confirmed |
| R501 | 1514 | PR #855 (issue #844) | issue-level coverage (not row-pinned): tests/unit/issue_844_statement_merge.rs | not yet confirmed |
| R502 | 1515 | PR #855 (issue #844) | issue-level coverage (not row-pinned): tests/unit/issue_844_statement_merge.rs | not yet confirmed |
| R503 | 1516 | PR #855 (issue #844) | issue-level coverage (not row-pinned): tests/unit/issue_844_statement_merge.rs | not yet confirmed |
| R504 | 1517 | PR #855 (issue #844) | issue-level coverage (not row-pinned): tests/unit/issue_844_statement_merge.rs | not yet confirmed |
| R505 | 1518 | PR #855 (issue #844) | issue-level coverage (not row-pinned): tests/unit/issue_844_statement_merge.rs | not yet confirmed |
| R506 | 1519 | PR #855 (issue #844) | issue-level coverage (not row-pinned): tests/unit/issue_844_statement_merge.rs | not yet confirmed |
| R507 | 1520 | PR #855 (issue #844) | issue-level coverage (not row-pinned): tests/unit/issue_844_statement_merge.rs | not yet confirmed |
| R508 | 1521 | PR #855 (issue #844) | issue-level coverage (not row-pinned): tests/unit/issue_844_statement_merge.rs | not yet confirmed |
| R509 | 1522 | PR #855 (issue #844) | issue-level coverage (not row-pinned): tests/unit/issue_844_statement_merge.rs | not yet confirmed |
| R510 | 1523 | PR #855 (issue #844) | tests/unit/docs_requirements_issue_844.rs | not yet confirmed |
| R847-1 | 1538 | PR #857 (issue #847) | issue-level coverage (not row-pinned): tests/unit/issue_847_task_decomposition.rs | not yet confirmed |
| R847-2 | 1539 | PR #857 (issue #847) | issue-level coverage (not row-pinned): tests/unit/issue_847_task_decomposition.rs | not yet confirmed |
| R847-3 | 1540 | PR #857 (issue #847) | issue-level coverage (not row-pinned): tests/unit/issue_847_task_decomposition.rs | not yet confirmed |
| R847-4 | 1541 | PR #857 (issue #847) | issue-level coverage (not row-pinned): tests/unit/issue_847_task_decomposition.rs | not yet confirmed |
| R847-5 | 1542 | PR #857 (issue #847) | issue-level coverage (not row-pinned): tests/unit/issue_847_task_decomposition.rs | not yet confirmed |
| R847-6 | 1543 | PR #857 (issue #847) | issue-level coverage (not row-pinned): tests/unit/issue_847_task_decomposition.rs | not yet confirmed |
| R847-7 | 1544 | PR #857 (issue #847) | issue-level coverage (not row-pinned): tests/unit/issue_847_task_decomposition.rs | not yet confirmed |
| R847-8 | 1545 | PR #857 (issue #847) | tests/unit/specification/task_decomposition.rs; tests/unit/issue_847_task_decomposition.rs | not yet confirmed |
| R848-1 | 1560 | PR #897 (issue #848) | none recorded | not yet confirmed |
| R848-2 | 1561 | PR #897 (issue #848) | none recorded | not yet confirmed |
| R848-3 | 1562 | PR #897 (issue #848) | none recorded | not yet confirmed |
| R848-4 | 1563 | PR #897 (issue #848) | none recorded | not yet confirmed |
| R848-5 | 1564 | PR #897 (issue #848) | none recorded | not yet confirmed |
| R848-6 | 1565 | PR #897 (issue #848) | none recorded | not yet confirmed |
| R848-7 | 1566 | PR #897 (issue #848) | none recorded | not yet confirmed |
| R848-8 | 1567 | PR #897 (issue #848) | none recorded | not yet confirmed |
| R848-9 | 1568 | PR #897 (issue #848) | none recorded | not yet confirmed |
| R848-10 | 1569 | PR #897 (issue #848) | none recorded | not yet confirmed |
| R706-1 | 1582 | PR #880 (issue #706) | issue-level coverage (not row-pinned): tests/unit/issue_706_any_language.rs | not yet confirmed |
| R706-2 | 1583 | PR #880 (issue #706) | issue-level coverage (not row-pinned): tests/unit/issue_706_any_language.rs | not yet confirmed |
| R706-3 | 1584 | PR #880 (issue #706) | issue-level coverage (not row-pinned): tests/unit/issue_706_any_language.rs | not yet confirmed |
| R706-4 | 1585 | PR #880 (issue #706) | issue-level coverage (not row-pinned): tests/unit/issue_706_any_language.rs | not yet confirmed |
| R706-5 | 1586 | PR #880 (issue #706) | tests/e2e/scripts/check-language-{test-coverage,change-parity}.mjs | not yet confirmed |
| R706-6 | 1587 | PR #880 (issue #706) | issue-level coverage (not row-pinned): tests/unit/issue_706_any_language.rs | not yet confirmed |
| R706-7 | 1588 | PR #880 (issue #706) | issue-level coverage (not row-pinned): tests/unit/issue_706_any_language.rs | not yet confirmed |
| R706-8 | 1589 | PR #880 (issue #706) | issue-level coverage (not row-pinned): tests/unit/issue_706_any_language.rs | not yet confirmed |
| R706-9 | 1590 | PR #880 (issue #706) | issue-level coverage (not row-pinned): tests/unit/issue_706_any_language.rs | not yet confirmed |
| R858-1 | 1602 | PR #899 (issue #858) | issue-level coverage (not row-pinned): tests/unit/issue_858.rs | not yet confirmed |
| R858-2 | 1603 | PR #899 (issue #858) | issue-level coverage (not row-pinned): tests/unit/issue_858.rs | not yet confirmed |
| R858-3 | 1604 | PR #899 (issue #858) | issue-level coverage (not row-pinned): tests/unit/issue_858.rs | not yet confirmed |
| R858-4 | 1605 | PR #899 (issue #858) | issue-level coverage (not row-pinned): tests/unit/issue_858.rs | not yet confirmed |
| R858-5 | 1606 | PR #899 (issue #858) | issue-level coverage (not row-pinned): tests/unit/issue_858.rs | not yet confirmed |
| R858-6 | 1607 | PR #899 (issue #858) | issue-level coverage (not row-pinned): tests/unit/issue_858.rs | not yet confirmed |
| R708-1 | 1620 | PR #883 (issue #708) | issue-level coverage (not row-pinned): tests/integration/memory_query.rs | not yet confirmed |
| R708-2 | 1621 | PR #883 (issue #708) | issue-level coverage (not row-pinned): tests/integration/memory_query.rs | not yet confirmed |
| R708-3 | 1622 | PR #883 (issue #708) | issue-level coverage (not row-pinned): tests/integration/memory_query.rs | not yet confirmed |
| R708-4 | 1623 | PR #883 (issue #708) | issue-level coverage (not row-pinned): tests/integration/memory_query.rs | not yet confirmed |
| R708-5 | 1624 | PR #883 (issue #708) | issue-level coverage (not row-pinned): tests/integration/memory_query.rs | not yet confirmed |
| R708-6 | 1625 | PR #883 (issue #708) | issue-level coverage (not row-pinned): tests/integration/memory_query.rs | not yet confirmed |
| R708-7 | 1626 | PR #883 (issue #708) | issue-level coverage (not row-pinned): tests/integration/memory_query.rs | not yet confirmed |
| R708-8 | 1627 | PR #883 (issue #708) | tests/e2e/tests/issue-708.spec.js | not yet confirmed |
| R708-9 | 1628 | PR #883 (issue #708) | issue-level coverage (not row-pinned): tests/integration/memory_query.rs | not yet confirmed |
| R709-1 | 1639 | pre-2026-07 (undated); issue #709 | none recorded | not yet confirmed |
| R709-2 | 1640 | pre-2026-07 (undated); issue #709 | none recorded | not yet confirmed |
| R709-3 | 1641 | pre-2026-07 (undated); issue #709 | none recorded | not yet confirmed |
| R709-4 | 1642 | pre-2026-07 (undated); issue #709 | none recorded | not yet confirmed |
| R709-5 | 1643 | pre-2026-07 (undated); issue #709 | none recorded | not yet confirmed |
| R835-1 | 1657 | PR #900 (issue #835) | issue-level coverage (not row-pinned): tests/unit/issue_835_file_legality.rs | not yet confirmed |
| R835-2 | 1658 | PR #900 (issue #835) | issue-level coverage (not row-pinned): tests/unit/issue_835_file_legality.rs | not yet confirmed |
| R835-3 | 1659 | PR #900 (issue #835) | issue-level coverage (not row-pinned): tests/unit/issue_835_file_legality.rs | not yet confirmed |
| R835-4 | 1660 | PR #900 (issue #835) | issue-level coverage (not row-pinned): tests/unit/issue_835_file_legality.rs | not yet confirmed |
| R835-5 | 1661 | PR #900 (issue #835) | issue-level coverage (not row-pinned): tests/unit/issue_835_file_legality.rs | not yet confirmed |
| R835-6 | 1662 | PR #900 (issue #835) | issue-level coverage (not row-pinned): tests/unit/issue_835_file_legality.rs | not yet confirmed |
| R835-7 | 1663 | PR #900 (issue #835) | issue-level coverage (not row-pinned): tests/unit/issue_835_file_legality.rs | not yet confirmed |
| R835-8 | 1664 | PR #900 (issue #835) | issue-level coverage (not row-pinned): tests/unit/issue_835_file_legality.rs | not yet confirmed |
| R835-9 | 1665 | PR #900 (issue #835) | issue-level coverage (not row-pinned): tests/unit/issue_835_file_legality.rs | not yet confirmed |
| R864-1 | 1677 | PR #910 (issue #864) | issue-level coverage (not row-pinned): tests/unit/issue_864.rs | not yet confirmed |
| R864-2 | 1678 | PR #910 (issue #864) | issue-level coverage (not row-pinned): tests/unit/issue_864.rs | not yet confirmed |
| R864-3 | 1679 | PR #910 (issue #864) | issue-level coverage (not row-pinned): tests/unit/issue_864.rs | not yet confirmed |
| R864-4 | 1680 | PR #910 (issue #864) | issue-level coverage (not row-pinned): tests/unit/issue_864.rs | not yet confirmed |
| R864-5 | 1681 | PR #910 (issue #864) | issue-level coverage (not row-pinned): tests/unit/issue_864.rs | not yet confirmed |
| R914-1 | 1697 | pre-2026-07 (undated); issue #914 | issue-level coverage (not row-pinned): tests/unit/docs_requirements_issue_914.rs | not yet confirmed |
| R914-2 | 1698 | pre-2026-07 (undated); issue #914 | issue-level coverage (not row-pinned): tests/unit/docs_requirements_issue_914.rs | not yet confirmed |
| R914-3 | 1699 | pre-2026-07 (undated); issue #914 | issue-level coverage (not row-pinned): tests/unit/docs_requirements_issue_914.rs | not yet confirmed |
| R914-4 | 1700 | pre-2026-07 (undated); issue #914 | issue-level coverage (not row-pinned): tests/unit/docs_requirements_issue_914.rs | not yet confirmed |
| R914-5 | 1701 | not delivered — untracked | none — untracked | not yet confirmed |
| R914-6 | 1702 | not delivered — untracked | none — untracked | not yet confirmed |
| R914-7 | 1703 | pre-2026-07 (undated); issue #914 | issue-level coverage (not row-pinned): tests/unit/docs_requirements_issue_914.rs | not yet confirmed |
| R914-8 | 1704 | not delivered — tracked in #873 | none — tracked in #873 | not yet confirmed |
| R914-9 | 1705 | not delivered — tracked in #848 | none — tracked in #848 | not yet confirmed |
| R914-10 | 1706 | not delivered — tracked in #527 | none — tracked in #527 | not yet confirmed |
| R914-11 | 1707 | not delivered — untracked | none — untracked | not yet confirmed |
| R914-12 | 1708 | pre-2026-07 (undated); issue #914 | issue-level coverage (not row-pinned): tests/unit/docs_requirements_issue_914.rs | not yet confirmed |
| R914-13 | 1709 | pre-2026-07 (undated); issue #914 | issue-level coverage (not row-pinned): tests/unit/docs_requirements_issue_914.rs | not yet confirmed |
| R914-14 | 1710 | pre-2026-07 (undated); issue #914 | issue-level coverage (not row-pinned): tests/unit/docs_requirements_issue_914.rs | not yet confirmed |
| R914-15 | 1711 | pre-2026-07 (undated); issue #914 | issue-level coverage (not row-pinned): tests/unit/docs_requirements_issue_914.rs | not yet confirmed |
| R891-1 | 1727 | PR #968 (issue #891) | tests/unit/specification/equation_corpus.rs; tests/unit/docs_requirements_issue_891.rs | manually probed 2026-08-04 with `cargo run --example issue_891_equation_probe`; raw engine output kept in docs/case-studies/issue-891/raw-data/production-solver-probe.tsv |
| R891-2 | 1728 | PR #968 (issue #891) | tests/unit/specification/equation_corpus.rs; tests/unit/docs_requirements_issue_891.rs | manually probed 2026-08-04 with `cargo run --example issue_891_equation_probe`; raw engine output kept in docs/case-studies/issue-891/raw-data/production-solver-probe.tsv |
| R891-3 | 1729 | PR #968 (issue #891) | tests/unit/specification/equation_corpus.rs; tests/unit/docs_requirements_issue_891.rs | manually probed 2026-08-04 with `cargo run --example issue_891_equation_probe`; raw engine output kept in docs/case-studies/issue-891/raw-data/production-solver-probe.tsv |
| R891-4 | 1730 | PR #968 (issue #891) | tests/unit/specification/equation_corpus.rs; tests/unit/docs_requirements_issue_891.rs | manually probed 2026-08-04 with `cargo run --example issue_891_equation_probe`; raw engine output kept in docs/case-studies/issue-891/raw-data/production-solver-probe.tsv |
| R891-5 | 1731 | PR #968 (issue #891) | tests/unit/specification/equation_corpus.rs; tests/unit/docs_requirements_issue_891.rs | manually probed 2026-08-04 with `cargo run --example issue_891_equation_probe`; raw engine output kept in docs/case-studies/issue-891/raw-data/production-solver-probe.tsv |
| R891-6 | 1732 | PR #968 (issue #891) | tests/unit/specification/equation_corpus.rs; tests/unit/docs_requirements_issue_891.rs | manually probed 2026-08-04 with `cargo run --example issue_891_equation_probe`; raw engine output kept in docs/case-studies/issue-891/raw-data/production-solver-probe.tsv |
| R909-1 | 1743 | delivered 2026-08-04; issue #909 | tests/integration/with_formal_ai_headless_global.rs:45 | manually confirmed 2026-08-04: `formal-ai with --global gemini` into a throwaway HOME wrote `.gemini/settings.json` with `security.auth.selectedType`, `--undo` removed it |
| R909-2 | 1744 | delivered 2026-08-04; issue #909 | tests/integration/with_formal_ai_headless_global.rs:147 | manually confirmed 2026-08-04: `formal-ai with --global qwen` wrote `OPENAI_API_KEY`, `OPENAI_BASE_URL`, and `OPENAI_MODEL` into `~/.profile` |
| R909-3 | 1745 | delivered 2026-08-04; issue #909 | tests/integration/with_formal_ai_headless_global.rs:171 | manually confirmed 2026-08-04: `experiments/issue-909-headless-config-gaps.sh` reported every headless requirement present |
| R909-4 | 1746 | delivered 2026-08-04; issue #909 | tests/unit/docs_requirements_issue_909.rs:13 | not yet confirmed |
| R909-5 | 1747 | delivered 2026-08-04; issue #909 | issue-level coverage (not row-pinned): tests/unit/docs_requirements_issue_909.rs | manually confirmed 2026-08-04: script run against the debug binary, exit 0 |
| R909-6 | 1748 | delivered 2026-08-04; issue #909 | tests/integration/with_formal_ai_headless_global.rs:308 | manually confirmed 2026-08-04: `--global --all --verify` sweep into a throwaway HOME satisfied every registry-declared requirement |
| R909-7 | 1749 | delivered 2026-08-06; issue #909 review | tests/unit/total_closure.rs:25 | manually confirmed 2026-08-06: `experiments/issue-909-seed-shard-conflict-blast-radius.sh` dirtied exactly 1 of 16 shards at each of four sort positions, against 11 of 11 before the fix |
| R893-1 | 1773 | PR #970 (issue #893) | tests/unit/specification/issue_893_summarization_validation.rs; tests/unit/docs_requirements_issue_893.rs | measured 2026-08-05 with `cargo run --release --example issue_893_measure`; raw run kept in docs/case-studies/issue-893/raw-data/ |
| R893-2 | 1774 | PR #970 (issue #893) | tests/unit/specification/issue_893_summarization_validation.rs; tests/unit/docs_requirements_issue_893.rs | measured 2026-08-05 with `cargo run --release --example issue_893_measure`; raw run kept in docs/case-studies/issue-893/raw-data/ |
| R893-3 | 1775 | PR #970 (issue #893) | tests/unit/specification/issue_893_summarization_validation.rs; tests/unit/docs_requirements_issue_893.rs | committed baseline data/summarization/quality-baseline.lino, re-measured by `formal-ai summarization ratchet` |
| R893-4 | 1776 | PR #970 (issue #893) | tests/unit/specification/issue_893_summarization_validation.rs; tests/unit/docs_requirements_issue_893.rs | measured 2026-08-05; embedded-grammar blocks counted against an independent CommonMark fence scanner |
| R893-5 | 1777 | PR #970 (issue #893) | tests/unit/specification/issue_893_summarization_validation.rs; tests/unit/docs_requirements_issue_893.rs | the four compression failures and the `<version>` grounding defect found by the 600-file sweep are recorded in docs/case-studies/issue-893/README.md rather than tuned away |
| R536 | 1805 | doctrine adopted 2026-08-04 | none yet — enforcement tracked in #934/#951/#952/#953 | n/a |
| R894-1 | 1820 | PR #971 (issue #894) | tests/unit/docs_requirements_issue_894.rs | revalidated 2026-08-05 against the four template default branches; commands and verbatim output kept in docs/case-studies/issue-894/raw-data/revalidation-greps.txt and revalidation-greps-2.txt |
| R894-2 | 1821 | PR #971 (issue #894) | tests/unit/docs_requirements_issue_894.rs | eight issues filed upstream 2026-08-05; bodies and API snapshot kept in docs/case-studies/issue-894/raw-data/ |
| R894-3 | 1822 | PR #971 (issue #894) | tests/unit/docs_requirements_issue_894.rs | ledger rendered and links opened 2026-08-05 in docs/case-studies/issue-479/template-comparison/REPORT.md |
| R894-4 | 1823 | PR #971 (issue #894) | tests/unit/docs_requirements_issue_894.rs | falsified 2026-08-05 by deleting a filing URL from the ledger and observing the test fail |
| R980-1 | 1836 | PR #981 (issue #980) | tests/unit/ci-cd/issue_980.rs | manually confirmed 2026-08-08 by downloading all seven referenced workflow logs and matching each run timestamp and SHA; findings preserved in dev/log/issues/980/pulls/981/ |
| R980-2 | 1837 | PR #981 (issue #980) | tests/unit/ci-cd/issue_980.rs; tests/e2e/tests/issue-282.spec.js; tests/e2e/tests/issue-541-permissions-cold-start.spec.js | manually confirmed 2026-08-08: opener parity passed 12/12 repeated cases and permission replay passed 9/9 repeated cases |
| R980-3 | 1838 | PR #981 (issue #980) | tests/unit/ci-cd/issue_980.rs | manually confirmed 2026-08-08 against complete tracked trees at rust c867f78, js 7b70923, and python 98d6dca; snapshots and control indexes preserved in the evidence bundle |
| R980-4 | 1839 | PR #981 (issue #980) | tests/unit/ci-cd/issue_980.rs | falsified 2026-08-08 by running the regression gates before the fixes; the formatting and isolation guards failed, then all three passed after the fixes |
| R973-1 | 1854 | PR #974 (issue #973) | tests/issue_973_solve_flags.rs::the_live_self_coding_entry_point_attaches_logs_and_runs_verbose; tests/issue_973_solve_flags.rs::every_published_solve_invocation_carries_both_evidence_flags | falsified 2026-08-05 by removing `--attach-logs` from examples/self-coding/run.sh and observing both tests fail |
| R973-2 | 1855 | PR #974 (issue #973) | tests/issue_973_solve_flags.rs::contributing_explains_why_both_flags_are_load_bearing | falsified 2026-08-05 by removing `--verbose` from the CONTRIBUTING.md command and observing the scan fail at CONTRIBUTING.md:115 |
| R973-3 | 1856 | PR #974 (issue #973) | tests/issue_973_solve_flags.rs::every_published_solve_invocation_carries_both_evidence_flags; tests/issue_973_solve_flags.rs::the_case_study_records_the_unrecoverable_failure_and_the_fix | not yet confirmed beyond the two falsification runs above |
| R1021-1 | 2270 | PR #1027 (issue #1021), 2026-08-19 | tests/unit/issue_1021_closed_circle.rs::closed_circle_session_replays | not yet confirmed |
| R1021-2 | 2271 | PR #1027 (issue #1021), 2026-08-19 | tests/unit/issue_1021_behaviour_range.rs (whole module -- every case is a held-out paraphrase or an unseen word order) | falsified-then-confirmed 2026-08-19 for the listing rule: Spanish was registered in data/seed/languages.lino but had no listing vocabulary, and supplying it in data/seed/shell-intents.lino made four held-out Spanish word orders route with no Rust change -- generalization by data, recorded as finding 11; falsified again 2026-08-20 while covering R1021-31 -- `contains_token` tested word boundaries with `is_ascii_alphanumeric`, so the `c` of the Spanish `codigo` matched the alias of the language C and every Spanish request mentioning code was answered as a C program (docs/case-studies/issue-1021/logs/spanish-code-boundary-before.log), fixed by making the boundary a property of letters and pinned by tests/unit/issue_1021_behaviour_range.rs::a_one_letter_alias_does_not_match_inside_an_accented_word -- recorded as finding 20 |
| R1021-3 | 2272 | PR #1027 (issue #1021), 2026-08-19 | tests/unit/issue_1021_behaviour_range.rs::a_bare_command_is_the_request | not yet confirmed |
| R1021-4 | 2273 | PR #1027 (issue #1021), 2026-08-19 | tests/unit/issue_1021_behaviour_range.rs::a_command_naming_noun_is_not_an_argument; ::a_command_naming_noun_is_stripped_for_every_command | not yet confirmed |
| R1021-5 | 2274 | PR #1027 (issue #1021), 2026-08-19 | tests/unit/issue_1021_behaviour_range.rs::a_prose_listing_request_routes_to_ls_in_any_word_order; ::listing_parts_alone_do_not_make_a_listing_request | measured 2026-08-19 with `cargo run --example issue_1021_spanish_probe`; output kept in docs/case-studies/issue-1021/logs/spanish-listing-routing-after.log; four held-out Spanish word orders route to `ls` from seed data alone, and `lista los procesos en ejecucion` still does not, so the parts still have to combine -- see finding 11 |
| R1021-6 | 2275 | PR #1027 (issue #1021), 2026-08-20 | tests/unit/issue_1021_behaviour_range.rs::a_named_exercise_is_answered_as_a_program; ::a_named_exercise_is_not_a_file_operation; ::the_stdin_answer_prints_the_input_it_was_verified_against; ::a_task_that_reads_no_input_keeps_its_plain_run_command | measured 2026-08-20 with `cargo run --example issue_1021_copy_stdin_harness`, which writes each answer's program to a scratch workspace, runs the check and run commands it printed, and compares the output against the fixture piped in; output kept in docs/case-studies/issue-1021/logs/copy-stdin-harness.log -- 10 of the 13 languages passed end to end and 3 were skipped for a toolchain absent from this machine (tsc, dotnet, scalac), 0 failed; routing recorded in docs/case-studies/issue-1021/logs/named-exercise-routing-after.log |
| R1021-7 | 2276 | PR #1027 (issue #1021), 2026-08-20 | tests/unit/issue_1021_behaviour_range.rs::a_named_exercise_is_not_a_file_operation; ::a_named_exercise_is_answered_as_a_program | measured 2026-08-20 with `cargo run --example issue_1021_named_exercise_probe`; output kept in docs/case-studies/issue-1021/logs/named-exercise-routing-after.log -- `Execute https://rosettacode.org/wiki/Copy_stdin_to_stdout in Rust` is answered with the verified Rust program rather than reaching web search |
| R1021-8 | 2277 | PR #1027 (issue #1021), 2026-08-20 | tests/unit/issue_1021_behaviour_range.rs::a_framework_named_coding_request_is_answered_in_that_framework; ::php_is_answered_from_the_catalog_like_every_catalogued_language; tests/source/source_tests/coding/catalog/mod/framework_targets.rs (whole module) | measured 2026-08-20 with `bash experiments/issue-1021-laravel/run.sh` (the composed Artisan command run inside a real `composer create-project laravel/laravel` application: Laravel Framework 13.26.1 on PHP 8.3.31) and with `node experiments/issue-1021-laravel/worker_check.mjs`, which loads the 26 browser worker shards and the seed lexicon and confirms the mirror resolves the reported prompt in all four reported natural languages to `laravel` while `write me some PHP code` still resolves to `php`; the same harness's last two of seventeen assertions record what the mirror does not carry -- eleven tasks against the engine's twelve, without `copy_stdin_to_stdout` (finding 21) |
| R1021-9 | 2278 | PR #1027 (issue #1021), 2026-08-19 | tests/unit/issue_1021_behaviour_range.rs::a_move_between_absolute_paths_is_performed; ::a_traversing_move_is_not_performed | not yet confirmed |
| R1021-10 | 2279 | PR #1027 (issue #1021), 2026-08-19 | tests/unit/issue_1021_write_path.rs::filing_an_issue_is_refused_in_both_states | not yet confirmed |
| R1021-11 | 2280 | PR #1027 (issue #1021), 2026-08-20 | tests/unit/issue_1021_write_path.rs::the_ladder_has_both_rungs_and_an_opt_in_to_climb_the_first; ::a_command_the_operator_named_is_not_the_ladders_business; `experiments/issue_916_write_effect_ladder/test_ladder.py` (`SandboxResetTests`, `ExpectedCommandTests`, `MutatingLadderDatasetTests` -- 30 judge tests, no server needed) | measured 2026-08-20 against the real release binary: `experiments/issue_916_write_effect_ladder/run_write_effect_ladder.sh` reports 16/16 rungs green including `824.L1`-`824.L5`, and the same run against the committed baseline reports `baseline 11/11 -> now 16/16`, so the ratchet moved up rather than sideways; log kept in docs/case-studies/issue-1021/logs/write-effect-ladder-after.log |
| R1021-12 | 2281 | PR #1027 (issue #1021), 2026-08-20 | tests/unit/issue_1021_recoverable_memory.rs::a_version_that_does_not_compile_leaves_the_previous_one_in_place; ::the_compile_failure_is_a_real_compiler_diagnostic; ::a_rollback_removes_a_file_the_candidate_added; ::a_failed_version_falls_back_to_the_last_adopted_one_not_to_the_first; ::a_candidate_that_edits_a_baseline_test_is_rolled_back_before_it_is_scored | not yet confirmed |
| R1021-13 | 2282 | PR #1027 (issue #1021), 2026-08-20 | tests/unit/issue_1021_bounded_autonomy.rs::a_loop_that_never_resolves_stops_at_the_limit_and_asks; ::the_question_repeats_until_the_operator_answers_it; ::granting_more_time_resumes_the_run_from_where_it_stopped; ::the_default_limit_is_the_hour_the_issue_names; ::full_trust_does_not_arrive_with_the_full_autonomous_mode | not yet confirmed |
| R1021-14 | 2283 | PR #1027 (issue #1021), 2026-08-19 | none -- not delivered; tracked in #924 | n/a -- not delivered; data/meta/self-hosting-ledger.lino still reads 0.00% self-authored |
| R1021-15 | 2284 | PR #1027 (issue #1021), 2026-08-19 | tests/unit/issue_1021_contribution_artifacts.rs::a_composed_fragment_is_one_the_changelog_gate_accepts; ::every_seeded_bump_and_category_composes_its_own_heading | not yet confirmed |
| R1021-16 | 2285 | PR #1027 (issue #1021), 2026-08-19 | tests/unit/issue_1021_contribution_artifacts.rs::a_composed_body_closes_its_issue_by_the_gates_own_rules; tests/unit/issue_1021_closed_circle.rs::the_artifacts_satisfy_the_gates_that_read_them | not yet confirmed |
| R1021-17 | 2286 | PR #1027 (issue #1021), 2026-08-19 | tests/unit/issue_1021_contribution_artifacts.rs::the_generator_composes_prose_without_containing_any | not yet confirmed |
| R1021-18 | 2287 | PR #1027 (issue #1021), 2026-08-19 | tests/unit/issue_1021_behaviour_range.rs (whole module) | not yet confirmed |
| R1021-19 | 2288 | PR #1027 (issue #1021), 2026-08-19 | tests/unit/issue_1021_closed_circle.rs::the_committed_process_artifacts_are_generator_output | not yet confirmed |
| R1021-20 | 2289 | PR #1027 (issue #1021), 2026-08-19 | tests/unit/issue_1021_write_path.rs (whole module -- both states driven from one process) | not yet confirmed |
| R1021-21 | 2290 | PR #1027 (issue #1021), 2026-08-19 | tests/unit/issue_1021_closed_circle.rs::closed_circle_session_replays | not yet confirmed |
| R1021-22 | 2291 | PR #1027 (issue #1021), 2026-08-19 | none -- not achieved; the run itself is the remaining gap | n/a -- not achieved |
| R1021-23 | 2292 | PR #1027 (issue #1021), 2026-08-19 | tests/unit/issue_918.rs::minimal_core_ledger_covers_every_recursive_handler_source; ::coding_path_has_complete_metadata_and_every_other_gap_is_data | not yet confirmed |
| R1021-24 | 2293 | PR #1027 (issue #1021), 2026-08-19 | tests/unit/docs_requirements_issue_1021.rs | not yet confirmed |
| R1021-25 | 2294 | PR #1027 (issue #1021), 2026-08-19 | none -- the probes are examples, run by hand | run 2026-08-19; output preserved under docs/case-studies/issue-1021/logs/ |
| R1021-26 | 2295 | PR #1027 (issue #1021), 2026-08-19 | tests/unit/ci-cd/issue_1021.rs::the_codex_pin_names_the_upstream_defect_and_the_bisect_that_would_lift_it | measured 2026-08-19: `python3 experiments/issue_1021_codex_tui_version/codex_trust_dialog_probe.py 0.148.0 enter-now` leaves a bare `codex` on its trust dialog after 20 s while 0.147.0 clears it; filed as openai/codex#39487 and pinned in .github/workflows/release.yml -- see finding 12 |
| R1021-27 | 2296 | PR #1027 (issue #1021), 2026-08-19 | tests/unit/docs_requirements_issue_1021.rs | not yet confirmed |
| R1021-28 | 2297 | PR #1027 (issue #1021), 2026-08-19 | tests/unit/ci-cd/issue_1021.rs::a_stalled_mirror_is_killed_at_its_own_deadline_and_the_next_attempt_succeeds; ::every_budgeted_retry_in_a_workflow_fits_the_budget_it_runs_under | measured 2026-08-19 against a stand-in `apt-get` that stalls, refuses and recovers; the stall observed in CI is preserved in docs/case-studies/issue-1021/logs/xvfb-install-budget-terminated.log -- see finding 14 |
| R1021-29 | 2298 | PR #1027 (issue #1021), 2026-08-19 | tests/integration/issue_703_orchestration_followup.rs::timeout_terminates_descendant_processes | measured 2026-08-19: five consecutive local runs pass in ~2.1s each; mutation-verified by spawning the descendant with `process_group(0)`, which the test reports as `still running 5s after the agent timed out`; the CI failure it answers is preserved in docs/case-studies/issue-1021/logs/descendant-timeout-macos-slice8.log -- see finding 15 |
| R1021-30 | 2299 | PR #1027 (issue #1021), 2026-08-19 | tests/unit/ci-cd/issue_1021.rs::the_deadline_exits_124_and_kills_the_whole_stalled_tree; ::a_command_that_beats_its_deadline_keeps_its_own_status; ::no_committed_script_reaches_for_a_timeout_binary_macos_does_not_have; ::the_deadline_never_expires_before_the_time_it_was_given | measured 2026-08-19: the eleven `ci_cd::issue_1021` tests pass locally in 3.8s; mutation-verified three times -- signalling only the root of the tree leaves the stalled child alive, restoring `timeout "$attempt_seconds"` fails the guard at `scripts/apt-install-with-retry.sh:90`, and reading elapsed time from bash's `SECONDS` alone fails the accuracy test with `a 3s deadline expired after 2.480484781s`. Measured accuracy (`experiments/issue-1021-deadline-precision/measure.sh`): 3.5s on a 3s deadline, 10.8s on a 10s one, never early. Confirmed on macOS 2026-08-19: all eleven tests pass on the macOS core slices of run 32294252392, including slices 15/16 and 16/16, the two that reported `timeout: command not found` in run 32282461075 (`docs/case-studies/issue-1021/logs/macos-deadline-tests-green.log`); the lower-bound test measured 3.835s against its 3s deadline there. See findings 16, 17 and 18 |
| R1021-31 | 2300 | PR #1027 (issue #1021), 2026-08-20 | tests/unit/issue_1021_behaviour_range.rs::a_coding_request_naming_no_language_is_a_coding_request; ::the_languageless_coding_request_is_answered_in_its_own_language; ::asking_for_code_is_a_coding_request_whatever_the_asking_verb; ::an_asking_verb_alone_is_not_a_coding_request | measured 2026-08-20 with `cargo run --example issue_1021_languageless_probe`; output kept in docs/case-studies/issue-1021/logs/languageless-request-after.log -- the reported bare request is answered with the question about the language rather than with search results, and each of the four subjects-beyond-the-artefact prompts still routes elsewhere; measured again 2026-08-20 with `cargo run --example issue_1021_languageless_followup`, which asks the same bare request and then answers the question it comes back with -- output kept in docs/case-studies/issue-1021/logs/languageless-followup.log, showing the follow-up turn is answered from the catalog rather than asked again (finding 6) |
| R1021-32 | 2301 | PR #1027 (issue #1021), 2026-08-20 | tests/unit/issue_1021_verified_move.rs::a_move_expands_into_preconditions_preparation_action_and_postconditions; ::a_copy_declares_the_same_shape_with_a_source_that_survives; ::a_destination_in_the_working_directory_prepares_the_working_directory; ::a_command_with_no_declared_effect_is_not_a_recipe; ::a_requested_move_runs_its_checks_around_the_action; ::a_move_onto_an_occupied_destination_stops_before_it_acts; ::a_move_of_a_missing_source_stops_on_the_first_check; tests/integration/issue_749_shell_routing.rs::whole_shell_task_matrix_routes_without_web_search | measured 2026-08-20 end to end against a real filesystem by ladder rungs `824.L1`-`824.L5`, all green; falsified 2026-08-20 before the helpers were updated -- driving the recipe made `tests/unit/issue_749_shell_routing.rs` assert `test -e a.txt` where it expected `cp a.txt b.txt`, which is the observable difference between issuing a command and carrying it out, and both unit matrices and the HTTP matrix now assert the whole recipe rather than its first step |
| R1021-33 | 2302 | PR #1027 (issue #1021), 2026-08-21 | tests/unit/ci-cd/issue_1021.rs::the_crate_is_on_edition_2024_and_the_judge_compiles_the_same_edition; ::nothing_in_the_tree_reaches_for_a_nightly_toolchain; tests/unit/ci-cd/issue_1014.rs::unix_agent_runner_uses_command_streams_exact_argv_api (the `=0.16.0` pin the refresh had to leave alone); and the whole suite, which is what a dependency refresh is actually tested by | measured 2026-08-21: `cargo build --all-features`, `cargo clippy --all-targets --all-features` and `cargo test --all-features` are green on the refreshed tree, as are the docs.rs profile gate (`check_docs_rs_dependency_profile`), the JavaScript manifest gate (`check_javascript_dependencies`) and `cargo audit`; falsified 2026-08-21 before the code moved -- `sha2` 0.11 produced ten `LowerHex is not satisfied` errors in nine files, which is what distinguishes a dependency that was upgraded from one whose number was changed (finding 28); the stable-only guard was mutation-verified by pointing one workflow's toolchain action at a non-stable channel and watching it fail. Re-measured 2026-08-21 against the registries rather than against the diff, with `python3 experiments/issue-1021-dependency-freshness/check.py`: 32 crates and 30 npm specs, 0 behind newest stable, 5 floating `@link-assistant/` specs skipped by rule. That run is what caught the one miss -- the `browser-commander` override had been taken to 0.16.0 when 0.16.1 had been the newest stable since six hours after it, now corrected and bundle-measured to the same 11,827,516 bytes (finding 31) -- and the tool exists because two hand checks each gave a wrong answer, one by reading a renamed crate's manifest key and one by reading npm's `latest` tag instead of the version list. Manual confirmation of the refreshed tree on a clean runner: not yet confirmed -- CI on this branch is the first unattended run of it |
| R1021-34 | 2303 | PR #1027 (issue #1021), 2026-08-21 | scripts/check-web-archive.test.mjs -- four tests naming the report shapes the single-heading lookup got wrong: a timeout with no errors section, failures spread across several failing sections, an unrecognised category, and a report with nothing but healthy sections; tests/unit/ci-cd/issue_1017.rs::link_report_parser_is_unit_tested_before_it_is_trusted keeps the workflow running them ahead of lychee, though note it only checks that the file exists and is non-empty, which is why it could not have caught this | falsified 2026-08-21 by running the four new tests against the previous parser: all four fail and the three pre-existing ones pass, which is what shows the gap was the report shape rather than the assertions. Reproduced from the real artefact with `node experiments/issue-1021-link-checker-false-positive/reproduce.mjs`, replaying the report captured verbatim from run 32454084765: 17 URLs reported broken and 16 of them links lychee had classified as healthy redirects before the fix, 1 and 0 after. Measured 2026-08-21: the two links CI timed out on both answer 200 from here in three consecutive requests each (0.67/0.62/0.51s for rowanzellers.com/hellaswag/, 4.34/0.76/0.82s for the Anthropic CLI-usage page), and run 32455788384 checked the same 1285 links with 0 timeouts and passed, so the verdict turned on the timeout and not on the links. Manual confirmation on a runner: not yet confirmed -- the failing shape needs a run in which some link happens to time out, which cannot be summoned on demand; what is confirmed is that the parser now returns the timeout alone from that exact report |
| R1021-35 | 2304 | PR #1027 (issue #1021), 2026-08-21 | tests/integration/issue_1021_client_preflight.rs::the_anthropic_hello_probe_is_answered_under_the_base_path_a_client_is_given; ::every_published_base_path_answers_a_reachability_probe; ::the_hello_probe_does_not_answer_for_paths_it_does_not_own; and the `E2E (t3code)` and `E2E (claude)` legs of `.github/workflows/agentic-cli-matrix.yml`, which are the gates that reported both failures | falsified 2026-08-21 against the unpatched `src/server.rs`: the hello-probe test fails with ``assertion `left == right` failed: HEAD /api/hello  left: 404  right: 200`` while the other two pass before and after, which is what makes them regression guards for behaviour that already held rather than tests written to match the fix. The upstream endpoint was measured rather than inferred -- `https://api.anthropic.com/api/hello` answers `200` and `{"message": "hello"}` to `GET` and `200` with a 0-byte body to `HEAD`, unauthenticated -- and the mechanism was read verbatim out of the shipped `@anthropic-ai/claude-code-linux-x64` binary, where `preconnectFired` and the `/api/hello` warm-up occur 4 and 1 times in 2.1.238 and 0 and 0 times in 2.1.215. The `t3code` half was reproduced locally under Node 22.23.2: 0.0.28 lists `start serve auth project connect`, 0.0.33 adds `pair` and `service`, and both new subcommands' help text was read to confirm neither opens a prompt path before the contract was re-recorded. Manual confirmation on a runner: not yet confirmed -- the matrix run on the commit carrying this fix is the first unattended check of it |
| R1021-36 | 2305 | PR #1027 (issue #1021), 2026-08-21 | tests/unit/ci-cd/issue_1021.rs::a_budget_that_expires_reports_the_compiler_cache_counters; ::a_budget_warning_reports_the_counters_without_touching_the_result; ::a_budget_that_wraps_no_compiler_reports_no_counters | falsified 2026-08-21 against the previous `scripts/run-with-budget-warning.sh`: the two positive tests fail there and the negative one passes on both sides, as a guard for already-correct silence should. The finding behind it was measured rather than assumed -- `experiments/issue_1021_compile_rate_compare.py` matched 480 crates against themselves across the red and green job logs of the same shard on the same lockfile and found the red run 2.4x-2.6x slower at every decile, uniformly; the re-run of the identical commit then passed in 620s of the 1200s budget against the green run's 838s, so one piece of work was observed at 620s, 838s and terminated-at-1200s. The dead `macOS-cargo-*` restore was ruled out by measurement too, at 19s and 20s of download against a 1200s budget. Manual confirmation on a runner: partly confirmed -- the silence is, the counters are not. Job 96736754559 ran the same shard at 603s of its 1200s budget, below the warning threshold, and `grep -c '[budget]'` over its log returns 0, so a healthy step stays quiet on a real runner. The counters themselves have still only been printed by the stand-in sccache the tests drive, because no CI step has blown its budget since |
| R1021-37 | 2306 | PR #1027 (issue #1021), 2026-08-21 | tests/unit/ci-cd/codeql_sink_heuristics.rs::no_function_parameter_is_named_after_a_hard_coded_cryptographic_sink; ::no_logging_macro_is_handed_a_name_that_reads_as_account_information; ::the_scan_skips_the_same_directories_the_codeql_config_ignores; and the nine tests that pin the reader itself, because a guard that silently parses nothing passes for the wrong reason; ::a_parameter_named_salt_is_found_and_the_seed_that_replaced_it_is_not; ::a_salt_in_a_comment_or_a_string_declares_nothing; ::a_multi_line_signature_reports_the_line_of_each_parameter; ::self_and_nested_types_do_not_produce_spurious_names; ::patterns_are_stripped_down_to_the_binding_name; ::an_inline_capture_is_read_out_of_the_format_string; ::placeholders_that_capture_nothing_are_not_mistaken_for_bindings; ::digests_and_hashes_are_not_read_as_account_information; ::a_substring_that_is_not_a_word_is_not_account_information | falsified 2026-08-21 with `bash experiments/issue-1021-codeql-name-heuristics/falsify.sh`, which reverts the renames and runs the guard against the tree that produced the alerts: it names `src/translation/selection.rs:324` and `:352`, `tests/source/translation/selection.rs:297` and `:325`, and `src/cli_improve.rs:84`, which are the exact two mechanisms behind the 99 alerts, and 12 of 12 pass once the renames are back. The heuristics were read at the source rather than inferred from the alert text -- `HeuristicSinks` in the upstream `HardcodedCryptographicValueExtensions.qll` and `HeuristicNames::nameIndicatesSensitiveData` in `SensitiveDataHeuristics.qll` -- and the guard deliberately anchors account names on word segments where upstream matches substrings, because the upstream form would flag `accounted_for` in `examples/issue_559_meta_core.rs`, which CodeQL itself does not report; `a_substring_that_is_not_a_word_is_not_account_information` pins the deviation. Measured 2026-08-21: the branch's 101 open alerts are identical to `main`'s on rule, severity and path, so nothing here was introduced by this pull request -- the check attributed them because 1299 files change (`docs/case-studies/issue-1021/logs/codeql-name-heuristic-alerts.log`). Manual confirmation on a runner: confirmed. The CodeQL analysis of 6149a639f, the commit carrying the rename, is green on both legs -- `CodeQL (rust)` job 96784436271 and `CodeQL (actions)` job 96784436191 -- and the aggregate `CodeQL` check that had reported *99 new alerts including 98 critical severity security vulnerabilities* now reports *No new alerts in code changed by this pull request* (run 96784677379). Open alerts on the branch went 101 to 2 with 0 critical, while `main` still carries all 101, which is what shows the count moved because the code changed rather than because the query did; the 2 that remain are the `rust/cleartext-logging` alerts on the real Agent CLI session ids in `tests/unit/docs_requirements_issue_917.rs` and `_918.rs`, expected and left open by design. Nothing was dismissed and no alert was suppressed (`docs/case-studies/issue-1021/logs/codeql-name-heuristic-alerts.log`). The same run found what the rename had left behind: four `data/meta/self-ast/` census documents keyed by content id went stale, failing the `Check self-AST census freshness` step and `issue_673_self_ast_census::committed_census_documents_match_what_the_sources_render`, and `cargo run --example regenerate_self_ast_census` rewrote exactly those four |
| R1073-1 | 2326 | PR #1074 (issue #1073), 2026-09-04 | tests/unit/issue_1073_reasoning_standard.rs::depth_floor_enumerates_every_gate_even_for_a_trivial_episode; ::the_depth_floor_holds_for_the_smallest_request_the_pipeline_can_formalize; tests/unit/specification/reasoning_standard_meta_algorithm.rs::the_meta_core_runs_the_audit_with_no_mode_in_front_of_it; tests/unit/specification/meta_construction.rs::both_directions_are_the_default_depth_floor; tests/unit/specification/selection.rs::record_is_the_default_and_off_emits_no_artifact; tests/unit/specification/skill_ledger.rs::accumulate_is_the_default_and_off_records_nothing | measured 2026-09-04 with `cargo run --example dump_reasoning_standard_audit` (docs/case-studies/issue-1073/logs/reasoning-standard-audit.log): the trivial request `"hi"` -- which triggers no world claim, no source, no conclusion and no action -- still has all seven declared gates enumerated, six reporting `not_triggered` with the trigger that was false and `instruction_formalization` reporting `violated` with the two sources it is missing named, and closes at `not_confirmed_not_refuted` with its blockers named |
| R1073-2 | 2327 | PR #1074 (issue #1073), 2026-09-04 | tests/unit/issue_1073_reasoning_standard.rs::gathered_instructions_are_compiled_into_checkable_steps | measured 2026-09-04 (docs/case-studies/issue-1073/logs/reasoning-standard-audit.log): the trivial request's `instruction_formalization` gate fires on its task class alone and reports `violated` with `courtesy:no_instructions_gathered` and `instruction_sources:0:required:2`, while the reference dialog, which gathered them, reports `satisfied` |
| R1073-3 | 2328 | PR #1074 (issue #1073), 2026-09-04 | tests/unit/issue_1073_reasoning_standard.rs::primary_documentation_is_required_by_default | measured 2026-09-04 (docs/case-studies/issue-1073/logs/reasoning-standard-audit.log): the reference dialog's `documentation_default` gate reports `satisfied`; nothing in the episode had to ask for documentation to be consulted |
| R1073-4 | 2329 | PR #1074 (issue #1073), 2026-09-04 | tests/unit/issue_1073_reasoning_standard.rs::source_trust_is_derived_from_the_primacy_chain | measured 2026-09-04: all thirteen sources in `data/seed/sources-registry.lino` derive the tier they previously asserted, and the four that asserted nothing (`wikidata`, `wiktionary`, `wordnet`, `wikipedia`) now derive `independent_corroboration` through `DerivationReason::NamedUpstreamChain` rather than through the old silent `_` fallback |
| R1073-5 | 2330 | PR #1074 (issue #1073), 2026-09-04 | tests/unit/issue_1073_reasoning_standard.rs::conclusions_need_varied_refutations_before_they_may_be_leaned_toward | measured 2026-09-04 (docs/case-studies/issue-1073/logs/reasoning-standard-audit.log): the reference dialog's `refutation_variety` gate reports `satisfied`, and the trivial request, which reached no conclusion, is blocked at `impulse_08ba5f07b55ec3da:no_conclusion_recorded` instead of leaning toward one |
| R1073-6 | 2331 | PR #1074 (issue #1073), 2026-09-04 | tests/unit/issue_1073_reasoning_standard.rs::the_standard_is_a_formal_procedure_that_replays_without_a_model; tests/unit/specification/reasoning_standard_meta_algorithm.rs (five grounding tests over data/meta/reasoning-standard-recipe.lino) | measured 2026-09-04 (docs/case-studies/issue-1073/logs/reasoning-standard-audit.log): both ledgers are produced by `standard()` reading `data/meta/reasoning-standard.lino` and `audit()` evaluating it, with no model in the loop and no network call |
| R1073-7 | 2332 | PR #1074 (issue #1073), 2026-09-04 | tests/unit/issue_1073_reasoning_standard.rs::the_reference_dialog_passes_and_each_adopted_behaviour_is_load_bearing | measured 2026-09-04 with `cargo run --example dump_reasoning_standard_audit` (docs/case-studies/issue-1073/logs/reasoning-standard-audit.log): the reference episode clears all seven gates with verdict `confirmed` |
