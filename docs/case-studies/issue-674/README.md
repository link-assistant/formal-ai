# Issue 674: compiling arbitrary natural-language programs

Issue [#674](https://github.com/link-assistant/formal-ai/issues/674) (E55) closes
the last `ARCHITECTURE.md` §16 question carried from the E20 batch: procedures a
user states in ordinary prose fell outside the trigger/response subset
`src/skill_compiler.rs` recognizes, so `docs/USER-JOURNEYS.md` F2 — "Wei compiles
a natural-language skill" — could only be scaffolded.

## What compiles now

The reference procedure from the journey, phrased as running prose that matches
no existing compiler template:

> When I paste a link, fetch its title, translate it to Russian, save both, and
> reply with the translation.

`compile_procedure` in `src/skill_procedure.rs` reads it as one trigger plus four
ordered steps:

```text
1. skill_procedure_fetch(skill_procedure_object_title) — "fetch its title" [21..36]
2. skill_procedure_translate(language_russian) — "translate it to Russian" [38..61]
3. skill_procedure_store(skill_procedure_object_both) — "save both" [63..72]
4. skill_procedure_reply(skill_procedure_object_translation) — "reply with the translation" [78..104]
```

Every span is a byte range into the original request, which is what makes the
compiled skill answerable to *"why did you do that?"*: the rationale quotes the
words each step came from rather than paraphrasing them.

## Three design decisions

**The vocabulary is data, not match arms.** Step verbs, step objects, clause
connectives, and trigger leads all live in
`data/seed/meanings-skill-procedure.lino` under the `skill_procedure_*` roles
(the operation-vocabulary precedent from E33). The meaning slug *is* the
canonical step kind — `skill_procedure_fetch` is both the seed entry and the kind
a `ProcedureHost` dispatches on — so adding a capability is a seed edit plus a
host arm, never a new branch in the parser.

**The compiled program is language-independent.** `links_notation` and
`link_records` project canonical slugs only; no source text reaches the export.
The English, Russian, Hindi, and Chinese phrasings of the reference procedure all
content-address to the same id and the same skill links, while each keeps its own
citations. That is the round-trip guard the issue asked for.

**A gap compiles nothing.** When a clause has no vocabulary entry the compiler
returns `ProcedureCompileError::UncompilableStep` naming the clause, its span, and
`no compiled capability for "…"`. `src/solver_handlers/procedure_rules.rs`
appends a `skill_gap` event and replies with that gap in the user's language. No
partial program is emitted, because a half-compiled procedure would run steps the
user did not agree to stop at.

## Why the compiler cannot hijack ordinary prompts

Two guards must both hold before a prompt is treated as a program: it must open
with a seeded trigger lead, and at least two of its clauses must resolve to step
verbs. A gap is reported only after the prompt has already proved itself a
procedure by that test, so an ordinary sentence containing an unknown verb falls
through to the normal pipeline rather than being refused.

## Verification

`cargo test arbitrary_skill_compilation` covers the acceptance criteria:

- `arbitrary_four_step_procedure_compiles_executes_and_restates_its_steps` —
  compiles, runs every step through a host threading each output into the next,
  and checks each citation against the original bytes;
- `same_procedure_in_every_supported_language_compiles_to_the_same_skill_links` —
  byte-identical links and one shared id across en/ru/hi/zh;
- `uncompilable_step_reports_a_named_gap_and_compiles_nothing_partially` and
  `solver_answers_an_uncompilable_step_with_the_gap_and_a_skill_gap_event` — the
  honest gap and its event;
- `solver_compiles_a_freely_phrased_procedure_and_can_restate_it_later` — the
  end-to-end solver path, including the later "why did you do that?" turn.

`examples/issue_674_procedure_compiler.rs` runs the same four phrasings and the
gap case as a demonstration:

```bash
cargo run --example issue_674_procedure_compiler
```
