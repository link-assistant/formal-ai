# Issue 962 case study — the infix operator word that never made it into the seed

**Issue:** [#962 — Hindi/Chinese word-operator arithmetic falls to the unknown handler while English/Russian succeed](https://github.com/link-assistant/formal-ai/issues/962)
**Source:** `test-report.md` "Broken things" #2 (MEDIUM, live-verified), from the [#957 full-history audit](../issue-957/README.md)
**Closing pull request:** <https://github.com/link-assistant/formal-ai/pull/976>

## The reported symptom

Four prompts, one arithmetic question, two outcomes:

| Prompt | Language | Before |
| --- | --- | --- |
| `What is 2 plus 2?` | en | `2 plus 2 = 4` |
| `Сколько будет 2 плюс 2?` | ru | `2 плюс 2 = 4` |
| `2 जोड़ 2 कितना होता है?` | hi | unknown handler |
| `2 जमा 2 कितना होता है?` | hi | unknown handler |
| `2 加 2 等于多少?` | zh | unknown handler |

Symbolic `2 + 2` worked in all four languages, which is what made the gap a
doctrine violation rather than a missing feature: README and USER-JOURNEYS both
claim "every operation is recognized equally across en | ru | hi | zh".

## Root causes

The three failing prompts did not share a single cause. Reproducing each one in
isolation (`experiments/issue_962_repro.rs`) separated them:

**RC1 — missing infix operator surfaces.** `data/seed/meanings-calculator.lino`
lexicalised the `addition` meaning as `जोड़` (hi) and `加上` (zh). `जमा` — the
everyday Hindi infix word — and `加` — the everyday Chinese infix character —
were simply absent, so `contains_word_operator` saw no operator and
`has_calculation_signal` rejected the prompt as prose. `2 加 2 是多少?` failed
for the same reason even with a recognised result-query cue.

**RC2 — a missing Hindi result-query cue.** `2 जोड़ 2 कितना है?` answered
correctly; `2 जोड़ 2 कितना होता है?` did not. The `calculation_result_query`
meaning carried `कितना है` but not the equally idiomatic `कितना होता है`, and
the cue is matched as a phrase, not as a stem — `कितना है` is not a substring of
`कितना होता है`. So even the *already-seeded* Hindi operator word failed on the
exact phrasing the issue reported. Fixing RC1 alone would have left `जोड़`
broken.

**RC3 — the gap was systematic, not lexical bad luck.** Once RC1 was understood,
every other operator showed the same shape: only the compound/standalone form
was seeded (`减去`, `乘以`, `除以`, `भाग`), never the bare infix form (`减`,
`乘`, `除`, `बटा`). The issue asked for a holistic pass rather than patching the
two reported words, and that is what the fix does.

## The fix

All changes are data, not code. `data/seed/meanings-calculator.lino` gains:

- `addition`: `जमा` (hi), `加` (zh)
- `subtraction`: `减` (zh)
- `multiplication`: `乘` (zh)
- `division`: `बटा` (hi), `除` (zh)
- `calculation_result_query`: `कितना होता है`, `कितने होते हैं` (hi)

`src/arithmetic_word_tables.rs` is the `no_std` materialisation of those
meanings for the wasm worker; it is regenerated with
`cargo run -p formal-ai --example issue_386_gen_arith_table` and mirrored into
`tests/source/`. The `arithmetic_word_tables_match_seed` test in
`src/calculation.rs` fails CI on a stale table, and it did — which is the
mechanism working as designed.

## Why the fix is data-only

The recogniser was never wrong. `contains_word_operator` already matched CJK
surfaces as substrings and non-CJK surfaces on token boundaries, across every
language the meanings lexicalise. The behaviour was missing because the
vocabulary was missing. That is the "generalization over one-off patching"
doctrine paying off in the negative direction too: when the only thing a bug
needs is a row in the seed, the recogniser is in the right shape.

## Regression coverage

- `tests/unit/issue_962_word_operator_parity.rs` pins the three reported prompts
  plus their English/Russian counterparts, the symbolic forms (so the fix cannot
  regress what already worked), and minus/times/divide in hi and zh.
- `tests/unit/multilingual_variations.rs` gains
  `arithmetic_hindi_word_variations_match` and
  `arithmetic_chinese_word_variations_match`, mirroring the existing English and
  Russian blocks operator for operator.

## Prior art surveyed

No new dependency was needed, and none would have helped. The recogniser this
issue exercises (`contains_word_operator` → `has_calculation_signal` →
`evaluate_calculation`) already delegates the *evaluation* to `link-calculator`
upstream; what failed was *recognition*, which is by design this repository's
own seed-driven layer (issue #386 moved the operator vocabulary out of a
hardcoded Rust array and into the `arithmetic_operation` meanings precisely so
that a language gap becomes a data edit). A tokenizer such as `lindera`
(already a transitive dependency) would not have helped either: the Chinese
prompts in this issue are already whitespace-separated, and the failure was a
missing lexical entry, not a segmentation error.

## Verification

`raw-data/after-fix-run.txt` is the live output of
`experiments/issue_962_repro.rs` after the fix — all eleven prompts route to
`intent=calculation`. `raw-data/github/` holds the issue, its comments, and the
closing pull request as fetched from the API.

Automated: `cargo test --test unit` and `cargo test --test source` pass in full.
Two generated artefacts had to be regenerated and their guards caught it before
CI did — `arithmetic_word_tables_match_seed` (stale `no_std` table) and
`issue_673_self_ast_census::committed_census_documents_match_what_the_sources_render`
(stale self-AST census).

## Requirements from the issue

| # | Requirement | Where delivered |
| --- | --- | --- |
| R962-1 | Add the missing Hindi infix operator words (`जोड़`, `जमा`) and the Chinese infix operator word (`加`) alongside the already-present `相加`. | `data/seed/meanings-calculator.lino` (`जोड़` was already present; `जमा` and `加` added). `相加` lives in `operation-vocabulary.lino`, a different table from the one the calculator reads — see the note below. |
| R962-2 | Audit the full operator table for other infix-vs-standalone gaps across all four languages. | Same file: `减`, `乘`, `除` (zh) and `बटा` (hi) added; en and ru were already complete. RC2 (the missing `कितना होता है` cue) was found by that audit. |
| R962-3 | Regression tests pinning the three reported prompts to `= 4`, in the style of the existing en/ru tests. | `tests/unit/issue_962_word_operator_parity.rs` (exact answers, not fragments) and the two new blocks in `tests/unit/multilingual_variations.rs`. |
| R962-4 | Manual re-run of the three failing prompts. | `raw-data/after-fix-run.txt`. |
| R962-5 | Spot-check minus/times/divide in hi and zh. | `other_word_operators_answer_in_hindi_and_chinese`. |
| R962-6 | `docs/case-studies/issue-{id}`; single PR. | This file; PR #976. |

### A note on `operation-vocabulary.lino`

The issue points at `data/seed/operation-vocabulary.lino`, where the Chinese
`相加` is recognised for the `sum` operation. That file drives the *text/list
operation* router ("sum of these numbers", "sort lines"), not the arithmetic
expression evaluator. The arithmetic operator words the reported prompts need
live in the `arithmetic_operation` meanings in
`data/seed/meanings-calculator.lino`, which is where the fix belongs — adding
`加` to `operation-vocabulary.lino` would not have made `2 加 2 等于多少?`
evaluate. Tracing each prompt to the handler that rejected it is what separated
the two tables.
