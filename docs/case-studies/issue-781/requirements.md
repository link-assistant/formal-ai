# Requirements trace

| ID | Requirement | Evidence |
|---|---|---|
| R1 | Find the best supportable answer for the A325-45 charger search. | `recommendation.md` separates verified requirements from conditional Amazon fallbacks. |
| R2 | Search and capture actual sources. | Official, marketplace, shared-dialog, and Amazon attempts are inventoried under `raw-data/`; capture failures are retained too. |
| R3 | Ingest the supplied ChatGPT share. | `chatgpt-web-capture.json`, both generated `.lino` files, unit coverage, and the CLI integration test. |
| R4 | Prepare a Google shared-dialog adapter without guessing unavailable text. | Normalized web-capture JSON is supported; `google-ai-mode-browser.json` preserves the provider diagnostic. |
| R5 | Improve Formal AI generally, not with charger-specific phrases. | The planner performs bounded multi-source capture for arbitrary languages and research subjects; product words occur only in the issue regression. |
| R6 | Use recursive/cross-source reasoning. | One search fans out to up to three independent fetches, whose evidence returns to the next planning step with URL identity intact. |
| R7 | Reproduce the bug before fixing it. | `raw-data/reproduction-before-fix.log` records the one-versus-three failing assertion. |
| R8 | Preserve provenance in Links Notation. | Each converted event carries the share URL; direct and adapter conversions are byte-identical. |
| R9 | Research relevant libraries and related work. | `investigation.md` traces Formal AI #552, web-capture #141, meta-language #168, and the installed web-capture adapter contract. |
| R10 | Avoid unsafe or unsupported purchase claims. | Amazon browser captures contain the automated-access notice; `recommendation.md` makes the fallback conditional on seller confirmation. |
| R11 | Cover the whole task with automated tests. | Issue regression, shared-dialog unit tests, real-fixture CLI integration test, and existing web-research suites. |
| R12 | Research the authentic part first, then official-compatible, then generic-compatible. | `option_network::Tier::LADDER` fixes the order; `authentic_part_is_found_before_any_substitute` asserts it holds regardless of discovery order. |
| R13 | Offer options made of two separate items, not only a single bundled purchase. | `OptionNetwork::ranked_plans` enumerates minimal satisfying *sets*; `two_separate_items_form_one_plan_when_neither_suffices_alone` covers the conversion-adapter case. |
| R14 | List every option, cheapest first, rather than one recommendation. | Ranking is by total price ascending, tier only breaking ties; `cheaper_options_are_listed_first_and_bundles_are_not_padded` asserts both the order and that non-minimal bundles are excluded. |
| R15 | Use multi-turn tool calling rather than a single search-and-fetch round. | `web_research::plan_deeper_round` searches again for the aspects no source covered; `research_deepens_toward_the_part_of_the_question_the_evidence_left_open` walks two full rounds to a final answer. |
| R16 | Terminate research rather than loop. | Deepening requires exactly one open aspect of a question with at least three, so anything else stops; the round budget bounds the rest, and no source is ever fetched twice. Covered by `research_does_not_repeat_a_search_that_refines_nothing`, `a_source_already_read_is_not_read_again`. |
| R17 | Hold the facts and options as an associative links network, not a scoring model. | Constraints and established facts are `world_model::Context` links; `OptionNetwork::links_notation` projects the whole network, sources included, and `the_target_and_current_contexts_are_ordinary_world_model_contexts` pins the substrate. |
| R18 | Generalize beyond chargers to any product under any constraints. | Nothing in `src/option_network.rs` names a product domain; `the_same_engine_sources_a_subject_that_shares_no_vocabulary_with_the_issue` drives the same engine with a darkroom enlarger lens mount. |
| R19 | Fill the network from what research actually fetches, not from hand-written fixtures. | `option_evidence::candidate_from_page` takes its search keys from the constraints’ units, so no prose is matched; `every_purchase_option_is_assembled_from_pages_and_listed_cheapest_first` builds the full option set from page text alone. |
| R20 | Read specifications regardless of the language around them. | `a_specification_is_read_the_same_whatever_language_surrounds_the_number` reads the identical values from Russian and English pages; `a_localised_unit_symbol_yields_no_reading_instead_of_a_wrong_one` pins that a localised symbol abstains rather than guesses. |
