# `isogram` — plan 01 held-out coding-path prompts, five languages

**Source of the prompts.** `docs/case-studies/issue-1138/plans/01-live-concept-lookup.md`,
§ *Tests first / Held-out five-language cases*, verbatim. The word `isogram`
occurs in no file under `src/` or `data/seed/` (verified by grep before the run),
so nothing about it can be answered from memory. Neither prompt explains what the
word means: the definition has to come from a source or the case fails.

**Binary.** `formal-ai 0.350.0`
**Commit.** `dc9b0574607a26f3e1c8bdb8ce93c0c7f786f197`
**Captured.** 2026-09-16, `@link-assistant/agent` 0.26.0, local server on port 8911,
`FORMAL_AI_AGENT_MODE=1`, private empty memory, `FORMAL_AI_DREAMING=0`.

## Outcome

| lang | Agent CLI | `formal-ai chat` | class |
| --- | --- | --- | --- |
| en | refusal, localized | identical refusal | `honest_refusal_without_a_trail` |
| ru | refusal, localized | identical refusal | `honest_refusal_without_a_trail` |
| hi | refusal, partly localized | identical refusal | `honest_refusal_without_a_trail` |
| zh | refusal, localized | identical refusal | `honest_refusal_without_a_trail` |
| es | refusal, localized | identical refusal | `honest_refusal_without_a_trail` |

One chat-completion round trip per language; the CLI was never asked to run a
tool. Every run exited 0 in about two seconds.

## What it actually said (en, verbatim)

```
I cannot write this program: no synthesis route reaches task "is_isogram" in language "python".

I decomposed the request and tried every synthesis route I have, in order — catalog,
blueprint_recipes, coding_oracle, seed_idiom_composer — and none of them derives it.

Nothing was guessed: I do not return a program I cannot derive, and I do not recite the
templates I happen to hold. Teach me the missing idiom for `python`, or restate the task
in steps I can already compile.

Research trail: phrases=Write a Python function is_isogram(word) that returns True when the
word is an isogram.;parts=;failed_examples=;attempts=
```

## What is right about this

The refusal is real. The system does **not** invent an `is_isogram` body, does
not recite a nearby template, and says plainly which routes it tried. It is
localized into all five languages by the same mechanism, with no per-language
branch visible in the output. Compared with a hosted model that would have
produced a plausible-looking function from memory, this is the behaviour the
doctrine asks for.

## What the run shows the system still lacks

1. **The research trail names nothing.** `parts=`, `failed_examples=` and
   `attempts=` are all empty in all five languages. The sentence "I tried every
   synthesis route I have" is therefore unfalsifiable from the reply: a reader
   cannot tell an exhausted search from a search that never ran. Plan 01 requires
   *a lookup that finds nothing reports every consulted source and no gloss*.
2. **The held-out word is never looked up.** The refusal is about a missing
   *synthesis route for `is_isogram`*, not about a missing *meaning of `isogram`*.
   No source registry is consulted, no `Need` for the unresolved word appears in
   the trail, and the loop never asks what the word means. The concept lookup
   plan 01 L5–L14 describes does not run on this route at all.
3. **Hindi is macaronic.** The `hi` reply keeps `program`, `synthesis route`,
   `decompose`, `templates`, `idiom`, `compile`, `request`, `steps` and `task`
   untranslated inside Hindi sentence frames. The seed carries a Hindi row, so
   this is a partial translation rather than a missing one.

## What the system would have needed to discover

The gloss of `isogram` from a dictionary or encyclopaedia source — *a word in
which no letter repeats* — and then the composition of that predicate from parts
it already holds (iterate the characters, collect into a set, compare lengths).
Both halves exist in the plan set: plan 01's source registry supplies the first,
plan 02's composer the second. Neither was reachable from this prompt.

## Tests this produced

`tests/unit/issue_1138_self_use_concept_lookup.rs`:

- `an_honest_refusal_names_every_source_it_consulted` — **red**, 5/5 languages.
