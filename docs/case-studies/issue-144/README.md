# Issue 144 Case Study: Chat-Editable Behavior Rules & Self-Sufficient Unknown Answer

## Summary

Issue [#144](https://github.com/link-assistant/formal-ai/issues/144) was filed
in Russian: a user asked `Какая у тебя модель личности?` ("What is your
personality model?") and got a short, dead-end unknown-intent fallback. The
user comment turned the bug into an eight-point requirements list:

1. Surface a *bigger* unknown answer with **variations** like
   `I don't know how to answer that yet.` | `I didn't understand you.`,
   chosen deterministically per prompt.
2. Let the user query existing behavior rules through dialog — list them and
   read each one's detail.
3. Let the user update rules through dialog so the system can answer the same
   prompt in a different way without code changes.
4. Make the unknown answer a **self-sufficient instruction** — it must
   explain exactly how to add facts, axioms, or rules.
5. Support read and write actions through messages only.
6. Ship more detailed `README.md` and a user-friendly documentation surface.
7. Offer a **Report issue** path (links in messages or a top-bar button) so
   the user can ask developers to add a missing rule.
8. Show self-awareness by listing the facts the system knows about itself.

## Solution overview

This case study documents PR [#179](https://github.com/link-assistant/formal-ai/pull/179),
which delivers all eight items in a single branch (`issue-144-c99ce65d1915`).

### 1. Unknown answer with variations

`src/engine.rs` ships per-language opener pools
(`UNKNOWN_OPENERS_EN/RU/HI/ZH`, five variants each). `select_unknown_opener`
hashes the trimmed prompt with the same FNV-1a routine the rest of the engine
uses, so a given prompt always picks the same opener but different prompts
draw different ones. The first entry of every pool matches the opener already
embedded in the seed text, so the variation logic is a strict superset of the
seed answer.

The browser worker (`src/web/formal_ai_worker.js`) and the React fallback
(`src/web/app.js`) mirror the same opener pools and hash, so the worker, the
file:// React fallback, and the Rust solver always pick the same opener for
the same prompt.

### 2. List & read behavior rules via dialog

`src/solver_handlers/behavior_rules.rs` exposes:

- `List behavior rules` (and Russian variants like `Список правил поведения`,
  `Покажи правила поведения`, Hindi `व्यवहार के नियम सूचीबद्ध करें`, and
  Chinese `列出行为规则`) — returns a Links-Notation table grouped by
  **topic** (Greetings, Farewells, Identity, Capabilities, Hello-world
  programs, Unknown fallback). Each entry renders as a copy-pasteable
  ``When `trigger` then `response`.`` statement, so the same surface that
  *lists* the behavior is the same surface that *updates* it. A trailing
  `Dialog-local rules` section enumerates any runtime rules taught earlier
  in the same conversation.
- `Show behavior rule <id-or-slug>` (also `Read rule <slug>`,
  `describe behavior rule <slug>`, and `Покажи правило <slug>`) — returns the
  full Links-Notation body of one rule, including its topic, trigger,
  matched intent, current response, source, and the canonical
  `when_then` statement.

### 3. Update behavior via dialog

Two natural-language teaching forms are supported, both mirroring the same
``When X then Y`` / ``When X do Y`` grammar used by the listing, so reading
and writing rules share one surface. All forms accept the prompt and answer
in backticks:

- English: ``When `your prompt` then `your answer`.``
- English alt: ``When `your prompt` do `your answer`.``
- English (legacy): ``When I say `your prompt`, answer `your answer`.``
- English (legacy alt): ``If I ask `prompt`, reply `answer`.``
- Russian: ``Когда `ваш запрос` тогда `ваш ответ`.``
- Russian alt: ``Когда `ваш запрос` делай `ваш ответ`.``
- Russian (legacy): ``Когда я скажу `ваш запрос`, ответь `ваш ответ`.``
- Russian (legacy alt): ``Если я спрошу `prompt`, ответь `answer`.``
- Hindi: ``जब `your prompt` तब `your answer`.``
- Hindi alt: ``जब `your prompt` तो `your answer`.``
- Chinese: ``当 `your prompt` 时 `your answer`.``
- Chinese alt: ``当 `your prompt` 则 `your answer`.``

The new rule is appended to the conversation history as a
`behavior_rule_update` event, then evaluated by the next call to
`solve_with_history` so the very next matching prompt returns the user's
answer. Multiple rules can coexist; the most recent one wins.

Trigger and response must each appear inside backticks. Free-form text
that contains the words `when` and `then` without backticks is ignored to
keep ordinary dialog from accidentally rewriting behavior.

### 4. Self-sufficient teaching answer

The seed text in `data/seed/multilingual-responses.lino` for
`response_unknown_*` now ends with explicit instructions: how to list rules,
how to read one, the exact teaching grammar, and a pointer to **Export
memory** / **Report issue** for durability.

### 5. Read & write through messages only

All eight requirements work through plain chat messages — no UI button is
required. The HTTP API (`POST /v1/chat/completions`) accepts the same
phrasings, so curl/CLI users get the same behavior.

### 6. README & docs

`README.md` gained a "Teaching behavior in chat" section with copy-pasteable
commands. This case study (`docs/case-studies/issue-144/README.md`) is the
deep-dive companion.

### 7. Report-issue link

The web app's top bar already surfaces a **Report issue** button (issue
[#129](https://github.com/link-assistant/formal-ai/pull/130)). The unknown
answer copy now explicitly tells the user to use it — and the capabilities
answer mentions it in all four languages.

### 8. Self-facts listing

`List all facts you know about yourself` (English) and `Какие факты ты знаешь
о себе?` (Russian) both route to a `self_facts` intent that prints the seed's
self-aware records — current model id, implementation strategy, supported
languages, available tools, etc.

## Acceptance tests

`tests/unit/specification/chat_surface.rs` pins the new behavior with 59
tests covering: English/Russian/Hindi/Chinese rule listing and self-facts
queries, the full multilingual ``When X then Y`` / ``When X do Y``
teaching grammar — English ``When … then …``/``When … do …``, Russian
``Когда … тогда …``/``Когда … делай …``, Hindi ``जब … तब …``/``जब …
तो …``, and Chinese ``当 … 时 …``/``当 … 则 …`` — alongside the legacy
English (`When I say`/`If I ask`) and Russian (`Когда я скажу`/`Если я
спрошу`) phrasings; topic grouping in the catalog listing
(Greetings/Farewells/Identity/Capabilities/Hello-world programs/Unknown
fallback) and `when_then` rendering of each row; per-rule detail with
topic and `when_then` lines; false-positive prevention for ordinary
prose that uses `when` and `then` without backticks; multiple
rule-prefix forms (`Show behavior rule`, `Read rule`,
`describe behavior rule`), most-recent-rule-wins, capabilities
advertising the new commands in all four languages, opener determinism
for the same prompt, opener variation for distinct prompts, seed-opener
strict-superset invariant, dialog-local rule listing, self-fact identity
content, and Report-issue/Export-memory copy in unknown answers.

Run them with:

```sh
cargo test --test unit chat_surface
```

## Files touched

- `src/unknown_opener.rs` — opener pools, `select_unknown_opener`,
  `unknown_answer_variation_for`, `language_aware_unknown_answer`
  (extracted from `engine.rs` to stay under the 1000-line file-size limit).
- `src/engine.rs` — routes unknown intents through
  `unknown_opener::language_aware_unknown_answer`.
- `src/lib.rs` — re-export `unknown_answer_variation_for`.
- `src/solver_handlers/behavior_rules.rs` — list/show/teach handlers.
- `src/solver_handlers/user_intent.rs` — capabilities answer mentions the new
  chat commands in en/ru/hi/zh.
- `src/web/formal_ai_worker.js`, `src/web/app.js` — JS mirrors of the opener
  variation logic and behavior-rule handlers.
- `data/seed/multilingual-responses.lino` — expanded unknown-answer seed.
- `README.md` — new "Teaching behavior in chat" section.
- `tests/unit/specification/chat_surface.rs` — acceptance tests.

## Open follow-ups

None blocking for this issue. English, Russian, Hindi, and Chinese are
now all first-class for both the read-by-dialog (`List behavior rules`,
`Show behavior rule`) and write-by-dialog (`When X then Y` / `When X do
Y`) surfaces.
