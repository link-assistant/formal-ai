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
| R933-1 | Define a machine-checkable convention for "wording variation". | A manifest, `data/benchmarks/conversational-variations-suite.lino`, listing the cases, the languages, the floor and the per-language partition files; one record per prompt in `data/benchmarks/conversational-variations/<language>.lino`. Two prompts count as one wording unless they differ in more than case, punctuation, symbols or spacing. |
| R933-2 | A CI script in the style of `check-language-parity` that walks the corpus and fails below five in any of en/ru/hi/zh. | `tests/e2e/scripts/check-conversational-variation-floor.mjs`, run as `npm run --prefix tests/e2e check:variation-floor`. |
| R933-3 | Backfill variations until the check passes. | 228 prompts across 10 cases × 4 languages, every group at or above five; the router phrasings they needed were added to `data/seed/intent-routing.lino` and `data/seed/meanings-intent.lino`. |
| R933-4 | Wire the check into `release.yml`. | `data/meta/ci-gates/check-conversational-variation-floor.lino`, stage `web`. Since issue #991 the workflow no longer holds a step list: `.github/workflows/release.yml` runs `rust-script scripts/run-ci-gates.rs --stage web`, which loads this shard. |
| R933-5 | Automated: the CI script itself, plus a unit test on its counting logic using fixture data engineered to trip the floor. | `tests/web/conversational-variation-floor.test.mjs` — 10 cases feeding `auditVariationFloor` fixtures at 4/5, at 0, and at five re-punctuated copies of one phrase. |
| R933-6 | Manual: reduce one test case to 4 variations in one language and confirm local failure. | [`raw-data/floor-check-manual-failure.txt`](raw-data/floor-check-manual-failure.txt) and [`raw-data/rust-corpus-manual-failure.txt`](raw-data/rust-corpus-manual-failure.txt). |
| R933-7 | Multilingual: confirm coverage counts print per language. | The count table prints on success as well as failure — [`raw-data/floor-check-pass.txt`](raw-data/floor-check-pass.txt). |
| R933-8 | Verbose output listing exactly which test cases are under the floor. | One `- case <name> has <n> <language> variation(s); the floor is 5` line per shortfall, plus a remediation line naming the file to edit. |
| R933-9 | Standing clauses: `docs/case-studies/issue-933/`, single PR. | This document; PR #1010. |
| R933-10 | Dedup: confirm no overlap with `check:language-test-coverage` and note it here. | Section 5. |

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

Each record is a prompt with the intent and evidence link it must produce:

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
```

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

## 5. Dedup: how this differs from the checks that already existed

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

## 6. Verification

| Evidence | What it shows |
| --- | --- |
| [`raw-data/floor-check-pass.txt`](raw-data/floor-check-pass.txt) | The gate passing, with per-language counts printed (R933-7). |
| [`raw-data/floor-check-manual-failure.txt`](raw-data/floor-check-manual-failure.txt) | One Hindi `assistant_name` record removed → `- case assistant_name has 4 hi variation(s); the floor is 5`, exit 1 (R933-6, R933-8). |
| [`raw-data/rust-corpus-manual-failure.txt`](raw-data/rust-corpus-manual-failure.txt) | The same deletion failing the Rust half: `case assistant_name holds 4 hi wording(s); the floor is 5`. |
| [`raw-data/floor-unit-tests.txt`](raw-data/floor-unit-tests.txt) | 10/10 counting-logic unit tests, including the fixtures engineered to trip the floor (R933-5). |
| [`raw-data/legacy-counts-before.txt`](raw-data/legacy-counts-before.txt) | The pre-fix corpus with 8 of 24 groups below the floor. |

Reproduce with:

```bash
npm run --prefix tests/e2e check:variation-floor
node --test tests/web/conversational-variation-floor.test.mjs
cargo test --test unit conversational_variation
python3 experiments/issue_933_variation_floor/legacy-counts.py   # the "before" table
```

## 7. Defect found while backfilling, and not fixed here

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

## 8. Relationship to `tests/unit/multilingual_variations.rs`

The legacy test is kept as-is. It still asserts what it always asserted, and it
is not the corpus the floor is measured over — measuring the floor over
hand-written Rust arrays is exactly what left it unenforced for three months.
The new corpus is additive: it is data, it is counted, and it is executed.
