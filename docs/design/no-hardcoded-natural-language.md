# No Hardcoded Natural Language: Meanings ↔ Naturalization

This project is a deterministic, symbolic agent. Its behaviour must be a
projection of *data* — the seed knowledge base — not of natural-language strings
typed into the engine. This document states the rule, the principle behind it,
and the CI gates that enforce it so the mistake is not repeated.

## The rule

Natural language is **data, never a string literal in the engine**. It applies
to both directions of every reasoning path:

- **Triggers / detection.** The phrases, run verbs, shell tokens, surfaces, and
  cues that a recogniser matches against a user prompt live in
  `data/seed/*.lino`. Code asks the lexicon for a *meaning* by role
  (`lexicon().meanings_with_role(ROLE_…)`) or loads a named vocabulary
  (e.g. `seed::terminal_command_vocabulary()`). Code never hardcodes a
  per-language phrase array or branches on a literal user phrasing.
- **Responses / output.** Every user-facing answer is a template in
  `data/seed/multilingual-responses.lino`, looked up by intent
  (`seed::response_for(intent, lang)` in Rust, `answerFor(...)` in the JS
  worker). Code fills placeholders such as `{command}`; it does not embed the
  surrounding prose.

- **Web front-end (React).** Every user-facing string rendered by
  `src/web/app.js` — panel titles, button labels, status words, onboarding copy,
  and system messages — is a catalog entry in `src/web/i18n-catalog.lino`,
  looked up at render time via `t(key, params)` (the `window.FormalAiI18n`
  engine in `src/web/i18n.js`). The render helper `h(tag, props, …children)`
  never receives a prose string literal as a child; visible text is always a
  `t(...)` call so it follows the active UI language (en/ru/zh/hi) and fills
  placeholders such as `{command}` or `{granted}/{total}`.

The only natural language that may appear in source files is **documentation**:
comments, doc-strings, and Markdown. Anything a user can see in a product
surface comes from the seed (engine surfaces) or the i18n catalog (web UI).

## The principle: meanings ↔ naturalization

A **meaning** is a slug grounded in the seed knowledge base. A meaning can be
**naturalized** into a natural-language surface (in any supported language), and
any natural-language word can be **formalized** back into a meaning. Code only
ever moves meanings around; the words live in the seed.

```
meaning  ──naturalize──▶  natural language   (rendering an answer)
meaning  ◀──formalize──   natural language   (recognising a prompt)
```

This is why a behavioural change is a seed edit plus a lookup, not a new branch
on a string. It is also why the same prompt is answered identically by the CLI,
the library, the HTTP server, the Telegram bot, and the website: they all read
the same seed (the Rust crate via `include_str!`, the browser via the
`src/web/seed/` deployment mirror or a byte-identical inline mirror).

## Enforcement (CI, not just convention)

1. **Total reference-closure gate.** `tests/unit/total_closure.rs` shells out to
   `scripts/audit-total-closure.py` and is run by `cargo test --tests`. Every
   bare value token in any `data/seed/*.lino` must resolve to one of: a defined
   meaning slug, a declared role (`data/seed/roles.lino`), a cached dictionary
   lemma (`data/cache/wiktionary|wordnet/…`), a Wikidata id with a cached
   record, or a supported language code. Vocabulary that resolves to nothing
   fails the build. To ground new tokens, run:

   ```sh
   python3 scripts/close-total.py          # idempotent; emits each unresolved
                                           # token as a first-class meaning under
                                           # data/seed/closure-generated-*.lino
   python3 scripts/audit-total-closure.py  # must report unresolved_distinct: 0
   ```

2. **Worker-mirror parity.** Where the JS worker embeds a byte-identical inline
   mirror of a seed vocabulary (the operation vocabulary, #386; the terminal
   vocabulary, #513), a `--check` guard fails the build on drift. For the
   terminal vocabulary the CI step runs:

   ```sh
   node experiments/issue-513-sync-worker-terminal.mjs --check
   ```

   Regenerate the mirror by running the same script without `--check`.

3. **Roles are declared, then generated.** A new `role` is declared as a
   `ROLE_*` constant in `src/seed/roles/*.rs`, re-exported from `src/seed.rs`,
   and the registry is regenerated with
   `python3 scripts/generate-role-registry.py` (keeps `data/seed/roles.lino` in
   lockstep; enforced by the `reference_closure` tests).

4. **Web-UI hardcoded-string guard (#511).** PR #528 reintroduced English prose
   directly inside `h(...)` render calls (permission-panel titles, button
   labels, status words). To stop that class of regression,
   `tests/e2e/scripts/check-web-hardcoded-ui-strings.mjs` parses every
   `h(tag, props, …children)` call in `src/web/app.js` and **fails the build**
   when a *child* argument is a bare prose string literal (a letter, whitespace,
   then another letter — i.e. two words a human reads). Dynamic values —
   `t(...)`, variables, ternaries, template literals — pass by construction, so
   the only way to render visible text is through the catalog. A short,
   documented `ALLOWED_LITERALS` allowlist holds a handful of pre-i18n `<dt>`
   labels coupled to other suites; **new prose must never be added there.** The
   companion `check-i18n-catalog.mjs` guard asserts every required key exists in
   all four locales (en/ru/zh/hi) and that representative interpolations render.
   Both run in `.github/workflows/release.yml` and locally with:

   ```sh
   npm --prefix tests/e2e run check:web-hardcoded-ui
   npm --prefix tests/e2e run check:i18n
   ```

   To add a new web-UI string: add the key (and its translation) under the right
   block in `src/web/i18n-catalog.lino` for **all four** locales, register the
   key in `REQUIRED_KEYS` in `check-i18n-catalog.mjs`, and render it with
   `t("<key>", params)` — never a literal.

## Worked example: the terminal-command intent (#513)

The terminal-command intent recognises prompts like "run `npm test` in the
terminal" / «выполни `npm test` в терминале» and answers with an Agent-mode
suggestion. Nothing about it is hardcoded:

- **Trigger vocabulary** — terminal/shell phrases, run verbs, Chinese run verbs,
  and leading shell tokens (`ls`, `git`, `cargo`, …) — lives in
  `data/seed/terminal-commands.lino`. Rust parses it via
  `src/seed/terminal_commands.rs` (`seed::terminal_command_vocabulary()`); the
  worker embeds an inline mirror kept in lockstep by
  `experiments/issue-513-sync-worker-terminal.mjs`.
- **Response prose** for all four languages lives in
  `data/seed/multilingual-responses.lino` under the `agent_suggestion` and
  `agent_suggestion_active` intents, each with a `{command}` placeholder.
  `src/solver_terminal.rs` (via `seed::response_for`) and the worker (via
  `answerFor`) look the template up and fill in the detected command.
- **Grounding** — every new value token (each shell token, `command-line`, the
  `agent_suggestion*` intents and their `response_*` templates) is grounded as a
  first-class meaning in `data/seed/closure-generated-*.lino`, so the
  total-closure gate stays at zero.

To add another shell token or another language, edit the seed, run
`close-total.py` and the sync script, and the tests confirm the engines stay in
parity — without a single new string literal in the code.
