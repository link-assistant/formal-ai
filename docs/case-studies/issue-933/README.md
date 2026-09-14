# Issue 933 Case Study

Issue [#933](https://github.com/link-assistant/formal-ai/issues/933) asks for
the CI enforcement that issue
[#123](https://github.com/link-assistant/formal-ai/issues/123) requested and
PR [#124](https://github.com/link-assistant/formal-ai/pull/124) never delivered:
every conversational test case must hold **at least five wording variations in
each of en, ru, hi and zh**, and a check must fail the build when one does not.

## 1. Timeline

- **2026-05-19 08:30** — issue #123 (*"Unknown prompt: Купи слона"*) is filed.
- **2026-05-19 08:34** — konard comments
  ([#123 comment](https://github.com/link-assistant/formal-ai/issues/123#issuecomment-4485896648)):
  *"All features should be supported in all 4 languages with multiple
  variations in wording, we should not stop until each test will have at least
  5 variations per 4 languages, also I think we should have CI/CD checks to
  enforce that for all the test cases."*
- **2026-05-19 09:50** — PR #124 (*"Fix reported browser prompt examples"*)
  merges. It fixes the reported prompts. No counting check ships with it.
- **2026-08-04** — issue #933 separates the missing enforcement into its own
  task.
- **2026-08-14** — PR [#1010](https://github.com/link-assistant/formal-ai/pull/1010)
  delivers it. The issue body, verbatim, is in
  [`raw-data/issue-933.md`](raw-data/issue-933.md).

## 2. Requirement matrix

| ID | Requirement (issue #933) | Delivered as |
| --- | --- | --- |
| R933-1 | Define a machine-checkable convention for "wording variation". | A manifest, `data/benchmarks/conversational-variations-suite.lino`, listing the cases, the languages, the floor and the per-language partition files; one record per prompt in `data/benchmarks/conversational-variations/<language>.lino`. The byte-preserved, Agent-authored contract is `data/meta/conversational-variation-floor-contract.lino`: NFKC then lowercase, discard punctuation/symbol/separator/whitespace categories, and preserve letters, numbers and combining marks. |
| R933-2 | A CI script in the style of `check-language-parity` that walks the corpus and fails below five in any of en/ru/hi/zh. | `tests/e2e/scripts/check-conversational-variation-floor.mjs`, run as `npm run --prefix tests/e2e check:variation-floor`. |
| R933-3 | Backfill variations until the check passes. | 228 prompts across 10 cases × 4 languages, every group at or above five; the router phrasings they needed were added to `data/seed/intent-routing.lino` and `data/seed/meanings-intent.lino`. |
| R933-4 | Wire the check into `release.yml`. | `data/meta/ci-gates/check-conversational-variation-floor.lino`, stage `web`. Since issue #991 the workflow no longer holds a step list: `.github/workflows/release.yml` runs `rust-script scripts/run-ci-gates.rs --stage web`, which loads this shard. |
| R933-5 | Automated: the CI script itself, plus a unit test on its counting logic using fixture data engineered to trip the floor. | `tests/web/conversational-variation-floor.test.mjs` — 11 cases feeding `auditVariationFloor` fixtures at 4/5, at 0, at five re-punctuated copies of one phrase, and a record that shows no answer. |
| R933-6 | Manual: reduce one test case to 4 variations in one language and confirm local failure. | [`raw-data/floor-check-manual-failure.txt`](raw-data/floor-check-manual-failure.txt) and [`raw-data/rust-corpus-manual-failure.txt`](raw-data/rust-corpus-manual-failure.txt). |
| R933-7 | Multilingual: confirm coverage counts print per language. | The count table prints on success as well as failure — [`raw-data/floor-check-pass.txt`](raw-data/floor-check-pass.txt). |
| R933-8 | Verbose output listing exactly which test cases are under the floor. | One `- case <name> has <n> <language> variation(s); the floor is 5` line per shortfall, plus a remediation line naming the file to edit. |
| R933-9 | Standing clauses: `docs/case-studies/issue-933/`, single PR. | This document; PR #1010. |
| R933-10 | Dedup: confirm no overlap with `check:language-test-coverage` and note it here. | Section 6. |
| R933-11 | Beyond the issue: every recorded variation shows the exact answer it produces (R234-2), so the corpus is documentation and not just a count. | `expected_answer` on 207 of the 228 records, asserted verbatim by `tests/unit/conversational_variations.rs`; the 21 capability records pin the opening line of the multi-paragraph listing with `expected_answer_contains`. The gate rejects a record that shows neither. |
| R933-12 | Beyond the issue: the recorded answers must be complete in every language — writing them down showed they were not. | The question-necessity parity fix in `src/question_necessity.rs` and `data/seed/question-necessity.lino`, with `tests/unit/issue_933_answer_parity.rs`. Section 5. |
| R933-13 | Review follow-up: execute part of this work with Formal AI through the real Agent CLI, decomposing only after failure and learning from the same run. | Five captured Agent-CLI sessions show the compound attempt fail, three smaller tasks pass, and the parent pass on retry. The same five sessions feed `learning.lino`; its four contract proposals remain `awaiting_human_review`. Section 10. |
| R933-14 | The Node and Rust counters must implement the declared normalization identically for compatibility characters and combining marks. | Both now apply NFKC, then lowercase, and discard Unicode P/S/Z categories and whitespace. Regression examples pin `Ａ == A`, `１ == 1`, `ϒ == υ`, and Hindi `क != का`; the Greek pair also makes the operation order observable. Section 11. |

## 3. Root cause: nothing counted anything

The conversational corpus was `tests/unit/multilingual_variations.rs` — hand-written
prompt arrays with a comment stating the intended count (*"Farewells — 8-9
variants per language"*). Comments are not checks, and a hand-written array has
no notion of how many distinct wordings it holds, so the corpus drifted below
the floor without anything noticing. Measured with the same normalization the
new check applies, **8 of its 24 case × language groups sat below five**
([`raw-data/legacy-counts-before.txt`](raw-data/legacy-counts-before.txt)):

```text
  case                  en  ru  hi  zh
  assistant_free_time    6   4   3   3
  calculation           16   8  10  11
  farewell               5   2   5   5
  greeting               6   4   5   6
  identity              11   7   6   6
  wellbeing              5   3   3   3
```

`farewell`/`ru` is the clearest case. The array holds five Russian entries —
`"пока"`, `"пока!"`, `"Пока."`, `"до свидания"`, `"досвидания"` — which look
like five variations and are two: one word re-punctuated three times, and one
phrase re-spaced twice. Counting array elements would have called that group
full. This is why the floor is defined over *normalized* prompts, and why the
unit test *"re-punctuated copies of one wording do not add up to five"* exists.

## 4. What was built

**The convention (R933-1).** A benchmark corpus, following the issue #819
precedent (`data/benchmarks/local-path-discovery-suite.lino` plus per-language
partitions plus a corpus-driven Rust runner). The manifest declares the floor
itself, so the number is data:

```text
conversational_variation_suite_issue_933
  record_type "conversational_variation_suite"
  minimum_variations_per_language "5"
  languages "en|ru|hi|zh"
  case "greeting"
  ...
```

Each record is a prompt with the intent, the evidence link and the answer it
must produce:

```text
conversational_variation_case_greeting_hi_01
  record_type "conversational_variation_case"
  id "greeting_hi_01"
  case "greeting"
  language "hi"
  source "self_authored_multilingual_variation"
  prompt "नमस्ते"
  expected_intent "greeting"
  expected_evidence "response:greeting"
  expected_answer "नमस्ते, मैं आपकी कैसे मदद कर सकता हूँ?"
```

`expected_answer` is what makes the corpus documentation rather than a table of
counts: a reader sees the exact text each of the 228 wordings produces, which is
the rule `scripts/check-tests-as-docs.rs` enforces for behavioural tests
(R234-2). The multi-paragraph capability listing is recorded by its opening line
with `expected_answer_contains` instead of being inlined 21 times, and the
arithmetic records add the computed value the same way. A record carrying
neither field is rejected by the floor gate, so the next case cannot be added
without showing what it answers.

**The gate has two halves, and both are needed.**

1. `tests/e2e/scripts/check-conversational-variation-floor.mjs` counts. It also
   rejects the ways a group could look full without being full: a duplicate id,
   a record whose case or language the manifest does not list, a partition file
   holding another language's records, a prompt that is empty once punctuation
   is stripped, and a prompt that only re-punctuates one already counted.
2. `tests/unit/conversational_variations.rs` answers every recorded prompt with
   `FormalAiEngine` and asserts the routed intent, the evidence link and — for
   arithmetic — the answer text. Counting alone would let a case reach five on
   wordings the router rejects; this is what stops that.

Both halves apply the same normalization rule, so they can never disagree about
how many wordings a case holds.

**The backfill (R933-3).** Every candidate prompt was first run through the real
engine (`examples/issue_933_variation_probe.rs`, driven from
`experiments/issue_933_variation_floor/`) and only the ones that already routed
correctly were recorded; the rest became router work in
`data/seed/intent-routing.lino` and `data/seed/meanings-intent.lino`, after
which they were re-probed. Final state, printed by the gate on every run:

```text
Conversational wording variations per case (floor: 5 per language)
  case                  en  ru  hi  zh
  greeting               9   8   6   7
  wellbeing              5   6   6   6
  farewell               8   7   5   5
  courtesy_response      6   5   5   5
  identity               6   6   6   6
  capabilities           6   5   5   5
  test_status            7   6   5   5
  assistant_free_time    5   5   5   5
  assistant_name         5   5   5   5
  calculation            6   5   5   5

Conversational variation floor OK: 10 cases x 4 languages, 228 verified prompts, every group at or above 5.
```

## 5. Recording the answers exposed a language-parity defect

Writing the answers down made a failure visible that counting never could.
Sixteen records came back with `expected_answer ""` — every Chinese `谢谢`
wording, every Chinese `你好吗` wording and every Hindi `धन्यवाद` wording — and
the Russian and Hindi wellbeing answers came back with their closing sentence
missing. **The English answers were untouched.** That is precisely the failure
issue #123 was about: *"All features should be supported in all 4 languages"*.

The seed was healthy (`cargo run --example issue_933_response_probe` prints the
right text for all four languages of all nine conversational intents) and the
routing was right (the probe recorded the correct intent and evidence link for
all 228 prompts, before and after). The answer was destroyed after it was
looked up, by the issue #920 question-necessity pass in
`src/question_necessity.rs`, which removes the byte range of any question it
refuses. Two seed-blind defects there combined:

1. **A sentence boundary needed whitespace after the stop, and the Devanagari
   danda was not a stop at all.** Chinese runs its sentences together
   (`很高兴听到。接下来您想做什么?`) and Hindi closes them with `।`
   (`यह सुनकर अच्छा लगा। अब आप क्या करना चाहेंगे?`), so the pass could not find
   where the question began and treated the *whole answer* as the question.
2. **The requirement cues were English-only for the seed's own follow-ups.**
   `data/seed/question-necessity.lino` had cues for `"what would you like"` but
   not for `"接下来您想做什么"`, `"с чего начнём"` or `"अब आप क्या करना चाहेंगे"`,
   so those follow-ups fell through to `default_class "factual"`, were refused
   as factual unknowns, and were cut out. Russian `courtesy_response` survived
   only by accident — `"хотите"` happened to already be a cue.

Defect 2 selected the answer for removal; defect 1 decided how much of it went.
Together they deleted everything.

A third, smaller variant of defect 1: a question the answer *quotes as an
example* was read as a question the answer *asks*. `closing_quote` knew the
Latin and Cyrillic quotation marks but not the corner brackets Chinese quotes
with, nor the parentheses every language uses for an inline example, so
`- **概念查找**：解释术语，例如「什么是维基百科？」` lost its bullet.

**The fix**, in `src/question_necessity.rs` and `data/seed/question-necessity.lino`:
`।`/`॥` join the sentence terminators; an ideograph after a stop opens the next
sentence the way a space does in a spaced script; `「」`, `『』`, `()` and `（）`
shield a quoted example; and nine Russian, Hindi and Chinese cues classify the
seed's own follow-up questions as requirements, next to the English ones that
were already there.

**Measured effect.** `cargo run --example issue_933_question_stripping_probe`
replays the pass over every answer in `data/seed/multilingual-responses.lino`
and prints each value it rewrites:

| Engine state | Seed values rewritten |
| --- | --- |
| Before the fix | 40 |
| Sentence boundaries + cues fixed | 20 |
| Quoted examples also fixed | 15 |

All 10 conversational intents of the corpus, plus the `clarification` fallback,
are now byte-identical in all four languages
([`raw-data/question-stripping-after.txt`](raw-data/question-stripping-after.txt)
versus [`raw-data/question-stripping-before.txt`](raw-data/question-stripping-before.txt)).

**What is still rewritten, and why it was left alone.** The remaining 15 are
language-*symmetric* — the same value is rewritten in every language, so they
are not parity failures — and none of them belongs to a corpus case:
`unknown_reasoning_question` (en/ru/hi/zh/unknown) is issue #920's intended
factual-unknown handoff, and `agentic_report_target_question`,
`agentic_report_contents_question` and `agent_suggestion` are agentic surfaces
outside this issue's scope. Changing them is a behavioural change to the
question protocol that would need its own before/after evidence, so they are
recorded here rather than quietly altered.

**The regression test** is `tests/unit/issue_933_answer_parity.rs`: seed answers
survive the pass in all four languages, the seed's own follow-ups classify as
requirements in all four, a refused question keeps the statement in front of it,
a quoted example is not a question, and `thanks`/`спасибо`/`धन्यवाद`/`谢谢` plus
the four wellbeing prompts answer with their whole seed text end to end. All
five of those fail on the unfixed engine
([`raw-data/answer-parity-before.txt`](raw-data/answer-parity-before.txt)) and
pass on the fixed one
([`raw-data/answer-parity-after.txt`](raw-data/answer-parity-after.txt)).

A sixth test covers Spanish, which the floor does not. `es` joined the language
registry as `status partial` after issue #123 was written, so it has no corpus
records — but it has two conversational seed responses, and `¡Hola! ¿Cómo puedo
ayudarte?` ends in a question, which is the shape that emptied the Chinese
answers. It survives the pass unchanged, and the test pins that so the next
change to the pass cannot quietly take Spanish down the road Chinese went.
`cargo run --example issue_933_empty_answer_probe -- hola` is how it was
checked: Spanish small talk does not reach the `greeting` intent at all, it
reaches the language-gap fallback, which is what `status partial` means.

## 6. Dedup: how this differs from the checks that already existed

The issue asks for confirmation that this does not overlap
`check:language-test-coverage`. It does not, and neither do the other two
language gates:

| Check | Question it answers | Why it cannot catch a four-variation case |
| --- | --- | --- |
| `check:language-test-coverage` | "Does this pull request's *diff* add test lines mentioning every registered language?" | Diff-aware: it only fires when a PR touches language-facing files, and it is satisfied by one line per language anywhere in the added tests. A case stuck at four wordings in Hindi trips nothing, forever, as long as nobody edits it. |
| `check:language-change-parity` | "Was a change to one language's seed data mirrored into the others?" | Parity of *edits*, not a count of what exists. Four wordings mirrored across four languages is perfect parity and still below the floor. |
| `check:intent-coverage` | "Does every routed intent have a localized response in every advertised language?" | Counts intents against responses, one per intent per language. It is indifferent to how many prompts reach an intent. |
| **`check:variation-floor` (new)** | "Does every conversational test case hold at least five distinct wordings in every advertised language?" | — |

The new check is absolute and always-on: it reads the committed corpus, not the
diff, so a group that drops below five fails the build on every subsequent run
until it is refilled.

## 7. Verification

| Evidence | What it shows |
| --- | --- |
| [`raw-data/floor-check-pass.txt`](raw-data/floor-check-pass.txt) | The gate passing, with per-language counts printed (R933-7). |
| [`raw-data/floor-check-manual-failure.txt`](raw-data/floor-check-manual-failure.txt) | One Hindi `assistant_name` record removed → `- case assistant_name has 4 hi variation(s); the floor is 5`, exit 1 (R933-6, R933-8). |
| [`raw-data/rust-corpus-manual-failure.txt`](raw-data/rust-corpus-manual-failure.txt) | The same deletion failing the Rust half: `case assistant_name holds 4 hi wording(s); the floor is 5`. |
| [`raw-data/floor-unit-tests.txt`](raw-data/floor-unit-tests.txt) | 11/11 counting-logic unit tests, including the fixtures engineered to trip the floor and the record that shows no answer (R933-5, R933-11). |
| [`raw-data/legacy-counts-before.txt`](raw-data/legacy-counts-before.txt) | The pre-fix corpus with 8 of 24 groups below the floor. |
| [`raw-data/answer-parity-before.txt`](raw-data/answer-parity-before.txt) | 0 passed, 5 failed on the unfixed engine — `धन्यवाद` answers `""`, `wellbeing/ru` loses its follow-up (R933-12). |
| [`raw-data/answer-parity-after.txt`](raw-data/answer-parity-after.txt) | 5 passed on the fixed engine. |
| [`raw-data/question-stripping-before.txt`](raw-data/question-stripping-before.txt) | The 40 seed values the question pass rewrote before the fix. |
| [`raw-data/question-stripping-after.txt`](raw-data/question-stripping-after.txt) | The 15 that remain, none of them a corpus case, all language-symmetric. |
| [`self-hosting-authorship/dispatch-report.json`](self-hosting-authorship/dispatch-report.json) | Five real Agent-CLI attempts: whole failure, three passing leaves, passing parent retry (R933-13). |
| [`self-hosting-authorship/learning.lino`](self-hosting-authorship/learning.lino) | The same five sessions observed by proposal-only learning; four proposals await human review (R933-13). |

Reproduce with:

```bash
npm run --prefix tests/e2e check:variation-floor
node --test tests/web/conversational-variation-floor.test.mjs
cargo test --test unit conversational_variation
cargo test --test unit issue_933_answer_parity
cargo test --test unit issue_933_self_authoring
cargo run --example issue_933_question_stripping_probe   # which answers the pass rewrites
cargo run --example issue_933_response_probe             # what the seed holds
python3 experiments/issue_933_variation_floor/legacy-counts.py   # the "before" table
experiments/issue_933_self_authoring/run.sh               # real Formal AI -> Agent CLI run
```

The corpus itself is regenerated, not hand-edited: every prompt is answered by
the real engine (`cargo run --example issue_933_variation_probe`, output in
`experiments/issue_933_variation_floor/probe-04.tsv`) and
`python3 experiments/issue_933_variation_floor/build-corpus.py` writes the
records from that run. `probe-03.tsv` is the same run against the unfixed
engine, kept because it is the evidence of the sixteen empty answers.

## 8. Defect found while backfilling, and not fixed here

`"how is it going?"` does not reach the `wellbeing` intent. It is intercepted
earlier by `try_how_it_works` (`src/solver_handler_how.rs`), which runs before
intent routing and matches the `mechanism_inquiry` prefix surface `"how is …"`
declared in `data/seed/meanings-how.lino`; the prompt is read as a mechanism
question about the subject "it going". The contracted `"how's it going?"` does
not match that prefix, routes to `wellbeing` correctly, and is the wording the
corpus records.

Changing handler precedence is a behavioral change to the router that this
issue does not ask for and that would need its own before/after evidence, so
the prompt was left out of the corpus rather than the precedence being quietly
reordered. Recorded here so the next person does not rediscover it as a
mystery.

## 9. Relationship to `tests/unit/multilingual_variations.rs`

The legacy test is kept as-is. It still asserts what it always asserted, and it
is not the corpus the floor is measured over — measuring the floor over
hand-written Rust arrays is exactly what left it unenforced for three months.
The new corpus is additive: it is data, it is counted, and it is executed.

## 10. One execution and learning lifecycle through Formal AI

The review follow-up asked for more than a static corpus: Formal AI had to do
real work through the Agent CLI, make a whole-task attempt first, split only
after observed failure, keep splitting until the pieces were solvable, and
learn from that same execution. The reproducible recipe is
[`experiments/issue_933_self_authoring/run.sh`](../../../experiments/issue_933_self_authoring/run.sh).
It launches the optimized `formal-ai` server, invokes the installed `agent`
executable through `formal-ai agent dispatch --incremental --cli agent`, and
runs a byte-exact external verifier after every attempt.

The captured trace in
[`self-hosting-authorship/dispatch-report.json`](self-hosting-authorship/dispatch-report.json)
is ordered evidence rather than a claimed result:

| Step | Task | Result |
| --- | --- | --- |
| 0 | Compound task | Failed exact verification; this failure triggers decomposition. |
| 1 | Coordination leaf | Passed. |
| 2 | Author the variation-floor contract | Passed; copied byte-for-byte to `data/meta/conversational-variation-floor-contract.lino`. |
| 3 | Author the learning-observation record | Passed. |
| 4 | Retry the compound parent over the composed workspace | Passed. |

Every step has a replayable JSON record under
[`self-hosting-authorship/sessions/`](self-hosting-authorship/sessions/),
including its native `ses_...` identifier and resume command. The two authored
deliverables are independently checkable leaves. The complete work breakdown
in [`decomposition.lino`](self-hosting-authorship/decomposition.lino) assigns
two of six smallest deliverable leaves to Formal AI through Agent CLI: 33%,
above the contributing floor.

Task execution and learning are now the same general lifecycle, not two case-
study-only commands. `src/orchestration/incremental.rs` converts every recorded
step/session pair into a client-contract observation and invokes the existing
learner before returning the dispatch report. This run therefore produces
[`learning.lino`](self-hosting-authorship/learning.lino) with
`observation_count "5"`. It detects four possible Agent integration contract
updates, but each says `decision "awaiting_human_review"`: execution can propose
what it learned and preserve the evidence, never approve its own extension.
`tests/integration/issue_991_incremental_dispatch.rs` pins that behavior for
every future incremental run; `tests/unit/issue_933_self_authoring.rs` pins this
real run, its split/climb trace, native sessions, byte-identical authored file,
learning gate and 33% decomposition.

## 11. Cross-runtime normalization defect found by the contract

The original documentation said the Node gate and Rust runner applied the same
normalization, but a compatibility-character probe disproved it. JavaScript
lowercased and applied NFKC before discarding Unicode punctuation, symbol and
separator categories. Rust only lowercased and retained `is_alphanumeric()`.
Consequently fullwidth `Ａ` and ASCII `A` counted as one wording in Node but two
in Rust; worse, Rust discarded the combining vowel mark in `का`, collapsing it
onto `क`, while Node preserved the distinction.

The Rust implementation now uses `unicode-normalization` for NFKC and
`unicode-general-category` for the same P/S/Z category families as the Node
regular expression. Letters, numbers and combining marks remain. Both suites
pin the same four examples: `Ａ == A`, `１ == 1`, `ϒ == υ`, and `क != का`.
The Greek pair only folds when NFKC runs before lowercase, so it pins the
declared operation order as well as cross-runtime parity. This makes the
Agent-authored normalization contract executable in both runtimes rather than
merely descriptive.
