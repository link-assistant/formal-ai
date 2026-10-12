# Issue #1174 Case Study: Text Transforms, Computed Instead of Described

Issue [#1174](https://github.com/link-assistant/formal-ai/issues/1174) (E139,
part of the #1183 umbrella). Verified on `main` at `d209aac64`; fixed on the
`qa-reasoning-coding-bulk-fixes` branch.

## What a user saw

Five kinds of everyday text requests, one shared failure shape — the engine
had no handler for any of them, so each prompt fell through every route to
the unknown-reasoning fallback of #1173, which describes a search plan
instead of transforming anything:

1. **"Summarize this paragraph: The Halley research station, opened in 1956,
   is used to study the Antarctic ice shelf. It provides year-round
   measurements of ozone and sea temperature. …"** — no free-text
   summarization existed; the paragraph was described, not shortened.
2. **"Rewrite this formally: can u send me the report asap thx"** — no
   register rewriting existed; the informal tokens survived verbatim.
3. **"Correct the grammar: She don't like apples and he have two cat."** —
   no grammar correction existed; the agreement errors went unfixed.
4. **"Write a commit message: correct an off-by-one error in the pagination
   helper"** and **"Write an email to my team: I am taking a day off on
   Friday because I am tired."** — no genre writing existed; no Conventional
   Commits line or business email was composed.
5. **"Translate to Russian: The weather is nice today, let's go for a
   walk."** — the translation pipeline only claimed quoted, backticked or
   structurally bracketed surfaces; a bare sentence after a command colon
   produced the could-not-identify refusal.

All five are computations over data the request itself carries. None of
them needs a network or a guess — which is exactly why a symbolic engine
should answer them, and why a search-shaped fallback answer is the wrong
shape entirely.

## Root cause (verified on `main` d209aac64)

No handler claimed any of these families: the dispatch chain had no
text-transform entries, `rust/src/solver_handlers/` had no
summarization-request or text-rewrite module, the translation handler
extracted only quoted/bracketed surfaces, and the seed files held no
register, agreement or genre vocabulary. Every prompt reached #1173's
fallback paragraph.

## The change

| File | Change |
| --- | --- |
| `rust/src/solver_handlers/summarization_request.rs` | free-text summarization: role-gated recognition, `free_text_payload` extraction, shared formalize → summarize → deformalize pipeline under a ~30% bound |
| `rust/src/solver_handlers/text_rewrite.rs` | the three transforms: register rewriting (word pairs + phrase rules, casing kept), grammar correction (subject-verb agreement + numeral-noun pluralization over exceptions and uncountables), genre writing (Conventional Commits and email from the styleguide's slot grammar), plus the shared `free_text_payload` command-head/payload extractor |
| `rust/src/solver_handlers/mod.rs` | `try_translation` gains the free-sentence branch (no quotable surface, source ≠ target) and `translate_free_sentence`: word-by-word translation with every token's disposition logged |
| `rust/src/translation/pipeline.rs` | `translate_sentence`: sentence-level translation over the existing pipeline — seeded stop-word drop, per-word resolution, capitalized lead, rendered in the target's word order |
| `data/seed/register-lexicon.lino` | informal→formal word pairs (en, ru) and the politeness phrase rule |
| `data/seed/agreement-rules.lino` | subject-verb rules, the numeral-noun rule, irregular plurals, uncountable nouns |
| `data/seed/writing-genre-styleguides.lino` | the two genre records with their templates, slots, prepositions, parts and frames |
| `data/seed/meanings-summarization.lino` | the summarization-action role surfaces (five languages), the probe sentence's compositional lemmas, and the translation stop-word lists |
| `data/seed/multilingual-responses-text-transform.lino` | the six response intents in en/ru/hi/zh/es |
| `data/seed/meanings-text-transform.lino` | five-field definitions grounding every new intent key and identifier |
| `data/meta/seed-registry.lino` | the six files registered |
| `rust/tests/unit/issue_1174_text_transform.rs` | handler-level probes pinning each intent and computed body, including the honest boundaries |

## Honesty rules

- A summarization request with no text, or a single sentence that cannot be
  shortened without being echoed, is **declined** — the seeded-topic and
  conversation-summary handlers keep the prompt.
- An already-formal text and an already-grammatical text pass through with
  an honest **nothing-changed** note, never a fabricated edit.
- A genre request missing a required slot names the **missing slots**
  ("missing: verb, effect") instead of inventing them, at reduced confidence.
- A translation word with no resolution is **named** ("Words without a
  seeded translation, kept as written: …"), and a sentence with nothing
  translatable reports the honest gap.
- Every substitution, correction, dropped function word, resolved word and
  unknown word is written to the event log, so the trace explains every
  token of every answer.

## Verification

`rust/tests/unit/issue_1174_text_transform.rs` pins all five families at
handler level (the dispatch wiring is owned by the #1175 routing work in
the same pull request): the summarization selection and its dropped
statement, the register rewrite with no informal token surviving, the
grammar fixes with each rule named, the commit line, the email's five
parts, the missing-slot refusal, and the sentence translation with all six
content words resolved from seed (hermetic — no network). The seed files
measure net-zero unresolved tokens under `scripts/audit-total-closure.py`,
and the literal-predicate count stays at its 536 ceiling.
