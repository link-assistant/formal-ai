# Issue #1172 Case Study: The Alias That Answered for the Wrong Country

Issue [#1172](https://github.com/link-assistant/formal-ai/issues/1172) (E137,
part of the #1183 umbrella). Verified on `main` at `d209aac64`; fixed on the
`qa-reasoning-coding-bulk-fixes` branch.

## What a user saw

```
$ formal-ai chat --prompt "What is the capital of Australia?"
The capital of the United States is Washington, D.C.
```

Australia has no seeded fact. The prompt was answered from the United States
record anyway, with full confidence (`0.9`, intent `fact_lookup`), a Wikidata
anchor for the USA (`Q30`/`Q61`), and the Washington, D.C. source URL — every
piece of evidence a correct answer would carry, attached to the wrong
country. A wrong answer is worse than no answer.

## Root cause

One substring test, applied in three places:

1. **The matcher** — `FactRecord::matches_normalized`
   (`rust/src/seed/facts.rs`) required "at least one subject alias **and** at
   least one question keyword appear as **substrings** of `normalized`". The
   normalized prompt `what is the capital of australia` contains the substring
   `us` (a**us**tralia) *and* the keyword `capital`, so `fact_capital_usa`
   matched. `try_fact_lookup` (`benchmark_prompts.rs`) takes the first
   matching record, and no Australia fact exists to compete — the USA record
   wins by being the only false positive.
2. **The planner gate** — `fact_store_resolves` (same file) runs the same
   matcher to tell the open-web planner (issue #989) "the engine already
   answers this", so the wrong cached fact also *suppressed* the live/web
   fallback that would have answered honestly.
3. **The browser worker** — `tryFactLookup`
   (`js/worker/formal_ai_worker_05.js`) selected facts with
   `containsAny(normalized, fact.subjectAliases)` — the same raw-substring
   test — so the worker false-hit its seed cache before the cache-miss path
   (`resolveFactQueryViaWikidata`, worker 10) could run. Two sibling sites in
   the same file shared the defect: the `tryCoreferenceFactLookup` keyword
   prefilter and `matchingAntecedentFactAlias`'s `previous.includes(alias)`.

The keyword side was equally unsound in principle — "capital" would match
inside "capitalism" — it just had no measured victim yet.

A prior requirement shard already claimed this class delivered:
`docs/requirements/issue-0127-structured-fact-query-reasoning-requirements.md`
rows R173/R174/R177 stood as "Implemented" while the Australia probe
contradicted them. This case study's correction of those rows is requirement
R11 of the issue.

## The change

| File | Change |
| --- | --- |
| `rust/src/seed/facts.rs` | `matches_normalized` now gates every alias and keyword through the new associated `pub fn FactRecord::contains_word_sequence(normalized, phrase)`: the phrase is tokenized with the very `engine::normalize_prompt` the caller applied to the prompt (so an alias like `japan's` compares as the token run `japan s`, exactly as the normalized prompt spells it) and must appear as a consecutive token run; phrases containing CJK — scripts written without inter-word spaces — keep substring matching. This mirrors the `surface_present` contract of `seed::meanings` (issue #386) for the fact store's own matching path. An associated function so it ships with the re-exported `FactRecord` type (the seed module keeps `facts` private). |
| `rust/src/solver_handlers/benchmark_prompts.rs` | `detect_subject_alias` uses the same comparison so the `fact_query:subject` evidence reports an alias that actually appears in the prompt as a whole word. `fact_store_resolves` and `try_fact_lookup` inherit the fix through `matches_normalized`; `try_coreference_request` inherits it through its rewrite path into `try_fact_lookup`. |
| `js/worker/formal_ai_worker_05.js` | `tryFactLookup` matches aliases and keywords through the worker's existing `surfacePresent` contract (worker 13: CJK substring, other scripts whole-token) applied to each surface via `normalizePrompt` — the same composition worker 17 already uses. The `tryCoreferenceFactLookup` keyword prefilter and `matchingAntecedentFactAlias` get the same boundary treatment. The shard sits at a falling line ceiling (`data/meta/worker-line-budget/formal_ai_worker_05.lino`, 1468), so the fix reuses `surfacePresent` instead of adding a helper: the file ends at 1467 lines, one under its ceiling (net −1). |
| `docs/requirements/issue-0127-structured-fact-query-reasoning-requirements.md` | Rows R173, R174, R177 now record the substring defect and its word-boundary correction, citing issue #1172, instead of standing as contradicted (R11). |
| `rust/tests/unit/issue_1172_factual_qa_subject_match.rs` | New acceptance suite (below). |

## Before and after

Prompt: `What is the capital of Australia?`

- **Before**: intent `fact_lookup`, confidence `0.9`, answer "The capital of
  the United States is Washington, D.C.", USA Wikidata anchors, and
  `fact_store_resolves` telling the planner the store answers it — so no
  fallback could run.
- **After**: no seeded fact matches (`usa_fact_no_longer_answers_australia_prompts`
  pins that *no* record in `facts()` matches), the prompt falls through the
  fact store to a non-`fact_lookup` answer, and `fact_store_resolves`
  returns `false` so the planner's open-web gate can release the prompt. The
  browser worker's cache-miss path (`resolveFactQueryViaWikidata`) is
  likewise reachable again for unseeded subjects.

Seeded subjects are untouched: "What is the capital of Japan?" still answers
Tokyo from the seed, in every supported language — including the CJK
substring path ("美国的首都是什么？" still matches the alias 美国) and the
Devanagari token path ("जापान की राजधानी क्या है?"). The possessive alias
`japan's`, dead under the old matcher (the normalized prompt spells it
`japan s`, which never contained the raw string `japan's`), matches again
through the phrase-side normalization.

## Why CJK keeps substring matching

Chinese and Japanese write without inter-word spaces: the normalized prompt
`美国的首都是什么` is a single token, so a whitespace-token matcher could
never isolate the alias 美国 inside it. Splitting CJK text into words is a
segmentation problem the engine does not solve, so CJK-containing surfaces
keep substring matching — the same rule `surface_present`
(`rust/src/seed/meanings.rs`) and the worker's `surfacePresent` (worker 13)
already established for meaning surfaces (issue #386). The defect being fixed
is a *Latin-script* substring coincidence ("us" in "australia"); no CJK alias
in `facts.lino` is a substring of an unrelated word.

## Tests

`rust/tests/unit/issue_1172_factual_qa_subject_match.rs`, hermetic (no
network; the engine-level cases run offline):

1. `contains_word_sequence_requires_whole_word_boundaries` — the helper's
   unit semantics: "us" not inside "australia"; "usa" as a whole token;
   "united states" as a consecutive run (and *not* as a scrambled
   "states united"); "capital" not inside "capitalism"; empty phrase/text
   never match; the CJK substring path.
2. `usa_fact_no_longer_answers_australia_prompts` — the USA record does not
   match the Australia prompt, and no record in the store does.
3. `usa_fact_still_matches_its_own_prompts` / `japan_fact_still_matches_multilingual_prompts`
   — the seeded capitals still match in en/ru/hi/zh, including the
   possessive "Japan's" and the CJK alias.
4. `keyword_inside_a_longer_word_does_not_route_to_a_fact` — the keyword side
   of the boundary rule.
5. `engine_answers_seeded_capitals_from_their_own_facts` / `engine_no_longer_answers_australia_with_washington`
   — full `FormalAiEngine` dispatch: seeded capitals answer from their own
   facts; the Australia prompt is not `fact_lookup` and never says
   "Washington".

Command: `RUSTUP_TOOLCHAIN=1.98.1 cargo test --test unit
issue_1172_factual_qa_subject_match` (add `mod
issue_1172_factual_qa_subject_match;` to `rust/tests/unit/mod.rs`; that file
is owned by the pull request's coordination lane, not this change).

## Honest boundaries

- The engine still does not *answer* "capital of Australia" — Canberra needs
  the live Wikidata path (R2/R3), which depends on #1163 (E128) and is not
  delivered here. What is delivered is the removal of the wrong answer and
  the unblocking of every fallback that could answer honestly.
- Definitions, comparisons, document Q&A, and explanations (R5–R8) are
  separate requirements: R5 shares its root cause with #1173 (that lane
  deletes the canned paragraph), R7's routing precondition is #1175's.
- The issue's proposed probe-suite layout (`data/benchmarks/factual-qa/*.lino`
  + `issue_1172_factual_qa_probes.rs`) asserts live-path answers and is
  therefore R2/R3-scoped; the delivered suite pins the matcher and dispatch
  contract instead.
- The parity case for `data/parity/cross-runtime-synthesis.json` is owned by
  the worker-parity lane of this pull request.
- No TypeScript twin exists: `ts/seed_loader.ts` loads seed records and
  performs no fact matching.
- `rust/tests/source/` (the stale checked-in snapshot of `src`) still carries
  the old matcher; that tree has drifted from `rust/src/` since the
  three-roots layout and has no sync gate — bringing it back in sync is its
  own task.
- Observed, not fixed (same defect class, different data store):
  `matchingCoreferenceAntecedent` in worker 05 and the Rust antecedent picker
  in `seed/coreference.rs` still match antecedent aliases by raw substring;
  no measured victim exists and the fix belongs with the coreference store's
  own matching path.
