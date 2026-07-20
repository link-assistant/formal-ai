# Issue #674 (E55) — compile arbitrary natural-language programs

- Session: `issue-674-claude-20260720`
- Agent: formal-ai (Claude Opus 4.8) via `/solve`
- Issue: <https://github.com/link-assistant/formal-ai/issues/674>
- Pull request: <https://github.com/link-assistant/formal-ai/pull/815>

Every claim below is either a quoted command output or a file path in this
repository. Where something is a judgement call rather than an observation, it
says so.

## 1. What the issue asked for

`ARCHITECTURE.md` §16 carried an open question from the E20 batch: arbitrary
natural-language programming beyond the supported subset of
`src/skill_compiler.rs`. `docs/USER-JOURNEYS.md` F2 described the journey — a
user states a multi-step procedure in plain language and the system compiles it
into a typed, executable skill — as "potential future / partially scaffolded".

The issue fixed five constraints: decompose into ordered sub-requirements and
map them onto a typed step vocabulary; fail honestly with a named gap plus a
`skill_gap` event where no vocabulary entry exists; grow the vocabulary as seed
data rather than Rust match arms; compile identically in en/ru/hi/zh; keep the
compiled steps inspectable with their source sentence spans.

## 2. Why the existing compiler could not be extended in place

`src/skill_compiler.rs` recognises two *shapes*: quoted trigger/response prose
and the labelled `Skill`/`Step`/`Expected test` form. Both are template matches
against a fixed grammar. A freely phrased sentence has no template to match, so
extending that module would have meant adding a third grammar rather than a
different mechanism. `src/skill_procedure.rs` is that different mechanism: it
splits on clause boundaries and classifies each clause against seeded meanings,
which is the E33 operation-vocabulary precedent applied to procedures.

The handler lives in its own module (`src/solver_handlers/procedure_rules.rs`)
because `src/solver_handlers/behavior_rules.rs` was already 946 lines against
the 900-line warn threshold of `scripts/check-file-size.rs`.

## 3. The two guards that keep ordinary prompts out

`compile_procedure` claims a prompt only when both hold:

1. a seeded trigger lead (`ROLE_SKILL_PROCEDURE_TRIGGER_LEAD`) occurs, and
2. at least `MINIMUM_STEPS` = 2 of the clauses after it classify as step verbs.

Both are necessary. Without (1) any imperative sentence would be a program;
without (2) "when I get home, remind me" — one clause — would be. The gap error
is raised only *after* both guards pass, so an unrecognised sentence beginning
"when I …" is reported as `NotAProcedure` (other handlers may claim it), not as
a missing capability.

Ordering matters too: `try_compiled_procedure` runs after
`compile_natural_language_skill` declines, so the typed compiler keeps
precedence and neither compiler shadows the other.

## 4. Why the identity is language-independent

`CompiledProcedure::canonical_program` is built from meaning slugs only — step
kind, argument objects, target language — never from surface words. The package
id is `stable_id("compiled_procedure", &canonical_program)`. That is the whole
mechanism behind the round-trip guard: the English, Russian, Hindi and Chinese
phrasings produce byte-identical canonical programs and therefore the id
`compiled_procedure_adf1f712fee0d724` in all four. Only
`source_description` and the per-step `source_span` remember the surface
wording, which is exactly what "why did you do that?" needs to quote.

## 5. Observed output

`cargo run --example issue_674_procedure_compiler`, English input *"When I
paste a link, fetch its title, translate it to Russian, save both, and reply
with the translation."*:

```
1. skill_procedure_fetch(skill_procedure_object_title) — "fetch its title" [21..36]
2. skill_procedure_translate(language_russian) — "translate it to Russian" [38..61]
3. skill_procedure_store(skill_procedure_object_both) — "save both" [63..72]
4. skill_procedure_reply(skill_procedure_object_translation) — "reply with the translation" [78..104]
```

With `print it on my printer` substituted for the translate clause:

```
Err(UncompilableStep { step: "print it on my printer", span: (38, 60),
    gap: "no compiled capability for \"print it on my printer\"" })
```

Nothing is compiled in that case — the error is returned instead of a partial
program, and `procedure_rules` appends the `skill_gap` event.

## 6. Recovering the procedure a turn later

`src/solver_handlers/meta_explanation.rs` does not store the compiled
procedure. It re-compiles the most recent `prior_turn:user` event that compiles,
which is the same history mechanism `collect_runtime_rules` in
`behavior_rules.rs` already uses. No new state, and the citation survives across
turns — pinned by
`solver_compiles_a_freely_phrased_procedure_and_can_restate_it_later`.

## 7. Verification performed

| Command | Result |
| --- | --- |
| `cargo test arbitrary_skill_compilation` | 5 passed |
| `cargo test --all-features --test unit --test source` | unit + source suites pass |
| `cargo clippy --all-targets --all-features -- -D warnings` | clean |
| `cargo fmt --check` | clean |
| `RUSTDOCFLAGS="-D warnings" cargo doc --no-deps --all-features` | clean |
| `python3 scripts/audit-total-closure.py` | `UNRESOLVED distinct: 0` |
| `python3 scripts/build-views.py --check` | `560 entities, ok` |
| `cargo run --example regenerate_self_ast_census` | in sync |
| `bash scripts/sync-seed.sh` | no drift |

## 8. Two follow-ups the seed edit forced

- The nine new bare English surfaces (`address`, `body`, `both`, `content`,
  `headline`, `retrieve`, `save`, `shorten`, `store`) failed
  `total_closure::seed_has_total_reference_closure`. They were grounded against
  Open English WordNet with `python3 scripts/ground-wordnet.py`, the remedy the
  gate itself names.
- Grounding new lemmas then made
  `total_closure::multi_source_view_is_present_and_consistent` report view-set
  drift, so `python3 scripts/build-views.py` was re-run and the nine new view
  entities committed.

Both were caught locally by running the full unit suite before pushing, not by
CI.

## 9. What is *not* done

`docs/USER-JOURNEYS.md` F2 keeps a future half: walking a stored `.lino` skill
step by step, and compiling a procedure into a Rust/JS handler, remain
unimplemented. The status line says so rather than claiming the journey is
complete.
