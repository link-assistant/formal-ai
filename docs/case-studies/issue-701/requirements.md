# Issue 701 requirements decomposition

Every requirement and acceptance criterion in
[issue #701](https://github.com/link-assistant/formal-ai/issues/701), enumerated
verbatim in intent, with where it is implemented and what proves it. Nothing is
deferred; where a claim is narrower than the issue's wording, the narrowing is
stated rather than hidden.

## Stated requirements

| # | Requirement | Registry | Implementation | Proof |
| --- | --- | --- | --- | --- |
| 1 | Define the adoption contract: frontier item → candidate knowledge/rules → validation against generated tests → promotion proposal in the exact shape #656 consumes. | R484 | `src/learning_cycle.rs`; `formal-ai learn cycle --frontier google-trends --dry-run [--proposals]` | `the_learning_cycle_emits_promotion_proposals_in_the_issue_656_shape`, `the_cycle_is_deterministic_and_reproducible_offline` |
| 2 | Prove the capability delta: a before/after pair per adopted item, in all supported languages, in `data/meta/learning-adoption-ledger.lino`. | R485 | `src/learning_adoption_ledger.rs`, `examples/issue_701_adoption_ledger.rs` | `committed_adoption_ledger_matches_a_fresh_run`, `the_adoption_ledger_records_a_real_capability_delta_in_every_language` |
| 3 | Close the Google Trends gap concretely: drive the #498/#499 frontier until the top-10 trends prompts and paraphrase matrices answer (target 80/80), or record each failure as a named blocking gap. | R486 | `data/seed/learned-request-openers.lino` promoted through the #656 gate | `the_trends_corpus_unknown_rate_is_ratcheted_to_zero`, `the_learning_report_is_a_faithful_proposal_only_run` |
| 4 | Dreaming amendments (#540, R413) must change solving behaviour through the production path; extend the single-prompt test into a class-level suite; delete any decoration-only code path found. | R487 | `src/dreaming_application.rs` (`append_amendments` reclassifies; `matching_amendments`/`amendment_lines` unify the three paths; `matching_amendment_lines` deleted) | `tests/unit/issue_701_dreaming_amendment_class.rs` (4 tests), audit reproducer `examples/issue_701_amendment_body.rs` |
| 5 | Run the loop periodically (dreaming runtime and/or scheduled CI) in proposal mode by default, honouring the human gate. | R488 | `dreaming_runtime::write_learning_cycle_record`, `.github/workflows/learning-cycle.yml` | `every_idle_dreaming_run_leaves_a_proposal_only_learning_cycle_record` |
| 6 | Preserve every failure to adopt as a durable frontier record, never silently dropped (consistent with R425). | R489 | `data/meta/learning-frontier-google-trends.lino`; blocked classes in the cycle record | `every_failure_to_adopt_is_preserved_as_a_durable_record`, `the_frozen_frontier_record_spans_every_supported_language` |

## Acceptance criteria

| Criterion | Result |
| --- | --- |
| `formal-ai learn cycle --frontier google-trends --dry-run` produces ≥1 valid promotion proposal with tests, deterministic and reproducible offline from cached data. | 2 proposals from 6 validated candidates over 48 held-out tests; two runs in one tree are byte-identical; the cycle reads only committed data. |
| The adoption ledger shows ≥20 before/after capability pairs across ≥3 topics and 4 languages. | 60 pairs, 10 topics, 4 languages (`en`, `ru`, `hi`, `zh`). |
| A regression test proves an adopted amendment changes the answer of a *held-out paraphrase*. | `retained_amendments_change_held_out_answers_across_topics_and_languages` — 12 held-out paraphrases across 3 topics × 4 languages, both protocol surfaces, plus the byte-identical negative control. |
| The `intent: unknown` rate over the committed trends corpus drops measurably and is ratcheted. | 7500 bp → 0 bp; the ratchet is `the_trends_corpus_unknown_rate_is_ratcheted_to_zero` and `report.frontier_count() == 0`. |

## Narrowings stated honestly

- The learning cycle derives one *class* of knowledge — request-opener surfaces —
  because that is the class the recorded frontier actually consists of. The
  contract (derive → validate on held-out → propose) is not specific to that
  class, but no other class is exercised by committed data yet, and we do not
  claim coverage we cannot test.
- Requirement 4's delta is a change in how a covered task is *classified* and
  what evidence it cites, not a rewrite of the answer body by the standing rule.
  A deterministic engine cannot invent a body from a rule it has no handler for;
  claiming otherwise would be the decoration problem in a new costume. What is
  now true, and tested, is that the amendment participates in routing rather than
  only in prose.
