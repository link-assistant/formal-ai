# Issue 499 Solution Plan

## Learnable-Source Registry

Declare the sources the engine may be taught to learn from as data, not code.
`data/seed/learning-sources.lino` records, per `source`:

- a `capability` slug naming which learning loop ingests it
  (`google_trends_learning`);
- a `host` and the native-language `keyword`s that name the source in every
  supported language; and

plus a shared, language-agnostic `directive` block of cues that mark a "learn
from this source" request (`learn from`, `узнаешь`, `यहाँ से सीख`, `在这里了解`, …).
Adding a new learnable source is therefore a seed edit, never a code change.

## Directive Recognition

`LearningSources::match_directive` is the single source of truth shared by both
entry points. A lowercased prompt matches only when it carries **both** a
directive cue **and** a reference to a declared source (its host or a keyword).
This keeps recognition data-driven and multilingual: the cues fold Cyrillic,
Devanagari, and Han text through `to_lowercase`, so the same registry serves
English, Russian, Hindi, and Chinese directives.

## Chat Entry Point

`try_learn_from_source` matches the directive, resolves the source's
`capability`, renders the auto-learning summary for that capability, and answers
with `intent "learn_from_source"` — localized to the prompt's language. It
declines (returns `None`) for a capability with no wired learning loop, so it
never invents an answer.

## Agentic Recipe

The same directive drives the Agent CLI. `is_google_trends_learning_task` now
recognizes the seed-driven learn-from-source directive (for the
`google_trends_learning` capability) in addition to the operator-worded
learning-frontier task, so the planner walks the existing recipe:

```text
write_file(google-trends-learning.lino) -> run_command(verify) -> final
```

`run_agentic_task(<reported prompt>)` therefore writes the committed learning
report, and the session is pinned byte-for-byte and driven live in CI through the
real external Agent CLI.

## Auto-Learning Loop

Both entry points feed `trending_learning_report`, which splits the 80 Google
Trends catalog prompts into the 20 the engine already routes and the 60-prompt
**learning frontier**, then hands that frontier to the human-gated issue-#558
self-improvement learner. Trending searches are open-domain questions, not
program-plan modifiers, so the learner adopts nothing: the value is the auditable
frontier and the proof the gap flows into the gated loop.

## Regression Tests

Cover the directive at every boundary:

- the exact reported prompt routes to `learn_from_source` with high confidence;
- every supported language routes with different wording, localized;
- routing needs both a cue and a declared source;
- the same directive drives the Agent CLI recipe to a write;
- a fresh Agent CLI run matches the pinned session; and
- the case-study documentation stays traceable.

## Follow-Ups

New learnable sources (each with its own `capability` and learning loop) are seed
edits that reuse `match_directive` and the same human-gated loop. A scheduled
refresh of the Google Trends snapshot and optional adapters for the official
Trends API alpha remain future operational work, not blockers for this directive.
