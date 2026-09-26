# Plan 08 — thirty held-out verifiable tasks, five languages

**Source of the prompts.** `data/benchmarks/verifiable-task-paraphrases.lino`,
the corpus wave T authored from plan 08's *Held-out cases* section. Six families
× five languages. Every string was checked absent from `data/seed/**` and `src/**`
before that corpus was committed.

**Binary.** `formal-ai 0.350.0`, built from `dc9b0574607a26f3e1c8bdb8ce93c0c7f786f197`.
**Captured.** 2026-09-16. Agent CLI 0.26.0 against a local `serve --agent-mode`
on port 8911, and `formal-ai chat` for the library-level half.

Transcripts: `arithmetic_narrative/`, `counted_category/`, `instructed_edit/`,
`named_unknown/`, `unit_conversion/`, `honest_gap/`, one directory per language.

## Outcome

| family | expected | en | ru | hi | zh | es | class |
| --- | --- | --- | --- | --- | --- | --- | --- |
| `arithmetic_narrative` | `3` | search | search | search | search | unsupported-language | `wrong_route` |
| `counted_category` | `5` | **`5`** | search | search | search | unsupported-language | `solved_in_one_language_of_five` |
| `instructed_edit` | shorter sentence | shell-command misroute | clarifying question | search | search | search | `wrong_route` |
| `named_unknown` | `12` | shell-command misroute (chat) / scraped `y=29` (CLI) | search | search | search | search | `wrong_answer_scraped` |
| `unit_conversion` | `1250` | search | search | search | search | unsupported-language | `wrong_route` |
| `honest_gap` | no number | search | search | search | search | unsupported-language | `right_negative_wrong_reason` |

**Totals.** 30 runs. **1 solved** (`counted_category/en`). 22 of 30 answered with
the web-search capability description. 0 derivations executed.

## The one that worked

`I have two oboes, a stethoscope, three scalpels, a harp and a thermometer. How
many medical instruments do I have?` → `5`. Exactly that, nothing else.

This is the hardest of the five numeric families, because passing it requires
category membership the seed does not carry: `OBJECT_CATEGORIES` has no "medical
instrument" row, so *stethoscope*, *scalpel* and *thermometer* had to be
recognised as members and *oboe* and *harp* excluded, with multiplicity attached
to the right entity. It works. It works in one language of five.

## The three that went looking for someone else's answer

`arithmetic_narrative`, `unit_conversion` and `named_unknown` are decidable from
the prompt with school arithmetic. All three were sent to `websearch` instead.
The transcripts show the system searching for **the word problem itself** and
opening homework sites:

- `brainly.com/question/10972757` (403, run abandoned)
- `gauthmath.com/solution/…/A-jug-holds-two-litres-of-cola-If-you-pour-out-500ml…` — a *different* problem
- `mathway.com`, `soluciones.lat/7y=84`, `symbolab.com/popular-algebra/algebra-4599`

This is the retrieval analogue of the rule wave F is bound by. Fetching a page
that contains an answer is not deriving the answer, in the same way that a
read-back plan is not executed semantics.

## The worst one

`Find y: 7 * y = 84`. The answer is 12.

Through the Agent CLI the system searched, fetched three algebra pages, and
emitted the **raw scraped text** of one of them into the answer channel. That
text contains:

```
7y−84=2y+61 … Solution y=29 … The answer to 7y-84=2y+61 is y=29
```

`y=29` solves a different equation that happened to be on the same page. It
reached the answer channel with no derivation, no check and no attribution of
what question it answered. A number that is wrong for the question asked, taken
from a page found by searching for the question, is a worse failure than an
honest refusal — and worse than a wrong arithmetic result, because it carries the
authority of a source.

Through `chat` the same prompt takes a different wrong turn: it is read as a
shell command (`It looks like you want to run a terminal command: Find y: 7 * y = 84`).

## The honest gap that is right for the wrong reason

`How many litres of rainwater did the roof of the old granary collect during last
night's storm?` has no answer any trusted source supplies, and plan 08 requires
the localized skill gap plus a research trail and **no number**. No number
appeared — but no skill gap was named either, and no research trail. The reply is
the same capability description the other families received. The required
negative was produced by never getting as far as the question.

## What the system would have needed to discover

Nothing external. Every one of these tasks is decidable from the prompt. What is
missing is the recognition step plan 08 L2 describes: that a prompt of this shape
is a *verifiable task* whose answer must be derived and executed, not retrieved.
`counted_category/en` proves the recognition exists; four languages and five
families do not reach it.

## Tests this produced

`tests/unit/issue_1138_self_use_verifiable_task.rs`, all red:

- `an_arithmetic_narrative_is_derived_in_every_language` — 5/5 short
- `a_unit_conversion_is_derived_in_every_language` — 5/5 short
- `a_named_unknown_is_solved_rather_than_scraped` — 5/5 short
- `a_counted_category_is_answered_in_every_language` — 4/5 short
- `no_verifiable_task_is_answered_with_the_search_capability_description` — 22/30
