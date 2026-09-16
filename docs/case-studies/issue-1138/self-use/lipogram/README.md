# `lipogram` — plan 01 held-out universal-loop prompts, five languages

**Source of the prompts.** `docs/case-studies/issue-1138/plans/01-live-concept-lookup.md`,
§ *Tests first / Held-out five-language cases*, verbatim. The word `lipogram`
occurs in no file under `src/` or `data/seed/`. The question is decidable from
the sentence alone once the word's meaning is known — *"quick brown fox"* has no
`e`, so the answer is yes — which makes this the cleanest possible test of
whether the loop can fetch a meaning and then use it.

**Binary.** `formal-ai 0.350.0`
**Commit.** `dc9b0574607a26f3e1c8bdb8ce93c0c7f786f197`
**Captured.** 2026-09-16, `@link-assistant/agent` 0.26.0, local server on port 8911.

## Outcome

| lang | Agent CLI | `formal-ai chat` | class |
| --- | --- | --- | --- |
| en | searched, fetched twice, then a degenerate summary scaffold | web-search capability description | `silent_unknown` / `wrong_answer` |
| ru | searched, fetched three pages, then the same scaffold | capability description, localized | `silent_unknown` / `wrong_answer` |
| hi | searched, fetched three pages, dumped raw page text incl. `AbortError` | capability description, **in English** | `wrong_answer` |
| zh | searched, fetched twice, then the same scaffold | capability description, **in English** | `silent_unknown` / `wrong_answer` |
| es | searched, fetched twice, narration **in English**, then the scaffold | `I detected an unsupported language and am falling back to English` | `wrong_answer` |

Not one of the ten runs produced a yes or a no, a definition of `lipogram`, or an
honest refusal.

## What actually happened, through the Agent CLI

The routing is right and the retrieval is real: the system emitted a `websearch`
tool call, read the results, and chose plausible URLs to fetch —
`en.wikipedia.org/wiki/Lipogram`,
`ru.wikipedia.org/wiki/Липограмма`,
`es.wikipedia.org/wiki/El_veloz_zorro_marrón_salta_el_perro_perezoso`. Several
rounds of genuine search and fetch happened against the live web.

Then the loop stopped without using any of it. The final assistant turn in `en`,
`ru`, `zh` and `es` is:

```
Let me search the web for Is the sentence "quick brown fox" a lipogram in e.

Title: What did we do so

User turns:
  1.
  2.
```

followed by `Continue if you have next steps` → `Nothing is in progress to
continue. Tell me the task and I will start it.` That is a **summary scaffold
leaking into the answer channel** with both its slots empty: no answer, no
refusal, and a claim that nothing is in progress immediately after three fetches.

The `hi` run took the other bad branch: it pasted the fetched page text straight
into the answer, including the string `AbortError: The operation was aborted.`
from a fetch that failed, with no gloss and no verdict.

## What `formal-ai chat` says without the CLI in the loop

Worse, and more diagnostic. Four of the five languages answer with a
**description of the search machinery instead of a search**:

```
Web search requested for `Is the sentence "quick brown fox" a lipogram`.

In the browser demo formal-ai defaults to the DuckDuckGo Instant Answer endpoint …
reciprocal rank fusion (`score(d) = Σ 1 / (60 + rank_i(d))`) …

Provider: duckduckgo (default)
Providers considered: duckduckgo, internet-archive, wikipedia, wikidata, wiktionary, wikinews
Combined ranking: reciprocal rank fusion (k = 60)
```

This is the retrieval analogue of the rule wave F is bound by: *do not count a
read-back plan as executed semantics*. Reciting how search would work is not
search. Note also that `hi` and `zh` receive this text **in English** while `ru`
receives it localized, so the localization is attached to the canned description
rather than to the route.

The Spanish run does not even get that far — see below.

## The Spanish defect, and its mechanical cause

`¿La frase "quick brown fox" es un lipograma en e?` is answered with the
`language unknown` row:

```
I detected an unsupported language and am falling back to English. …
```

Spanish is one of the five registered languages of every held-out corpus in this
plan set, and the coding route (`isogram/es`) answers Spanish correctly, so this
is route-specific rather than a missing locale. The cause is mechanical and
countable: of the **94** response intents in
`data/seed/multilingual-responses.lino`, **93** carry an `en` row and **no `es`
row at all**. Exactly one intent is seeded in Spanish. Any Spanish prompt that
reaches one of the other 93 degrades to the unknown-language fallback.

## What the system would have needed to discover

One sentence from one source — *a lipogram is a text that avoids a given
letter* — bound as a licensed, attributed sense, and then applied to the operand
the prompt already supplies. Every input was in hand: the right pages were
fetched and are in the transcript. The missing step is between fetching a page
and turning it into a sense the loop can reason with.

## Tests this produced

`tests/unit/issue_1138_self_use_concept_lookup.rs`:

- `an_unresolved_word_is_looked_up_rather_than_answered_with_the_provider_description` — **red**, en/ru/hi/zh
- `a_spanish_prompt_is_not_reported_as_an_unsupported_language` — **red**
- `every_seeded_response_intent_serves_all_five_languages` — **red**, 93 of 94 intents short
