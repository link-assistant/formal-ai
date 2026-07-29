# Issue #674 (E55) — compile arbitrary natural-language programs

- Initial session: `issue-674-claude-20260720`
- Review replay: `ses_0559382b9ffeZg7gTXqS6kySQU`
- Agent: formal-ai through the external Agent CLI
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

The solver puts the complete artifact in its assistant response.
`src/solver_handlers/meta_explanation.rs` extracts that persisted artifact from
conversation history, parses it with integrity checks, and cites its retained
source spans. It does not recompile the earlier user prose. This is pinned by
`solver_compiles_a_freely_phrased_procedure_and_can_restate_it_later` and
`compiled_artifact_round_trips_executes_and_explains_without_recompiling_user_prose`.

## 7. Review feedback and the deeper root causes

The July 28 review asked for the vision to be implemented more deeply through
automatic learning and the same real task execution via Formal AI's Agent CLI.
Two concrete gaps remained:

1. The first learning API required its caller to supply
   `canonical_kind`. That was a human-authored mapping, not automatic inference.
2. Agent wrote and read back the program but never invoked the interpreter, so
   its success proved persistence rather than execution.

`ProcedureLearningProposal::infer_candidate` now consumes multilingual
observations pairing each missing surface with a successful, already-supported
paraphrase. It resolves those paraphrases through the seeded compiler
vocabulary, derives one typed operation, and fails closed on conflicts. The
candidate id covers the proposal, inferred lesson, and observation evidence.
`promote_candidate` still requires a green regression suite and explicit human
approval; schema 2 of the ledger preserves and revalidates the candidate
evidence after restart.

The public `formal-ai procedure conformance` command parses the persisted
artifact and runs the generic interpreter through a deterministic,
side-effect-free host. Agent now performs write, readback, conformance
execution, and result verification before its final response. The external
0.25.3 Agent replay completed four server turns and retained the exact
`procedure_run` under `docs/case-studies/issue-674/agent-cli/`.

## 8. CI failure and merge-conflict investigation

Run `30328997145` failed only in the repository file-size gate:
`.github/workflows/release.yml` had grown to 2,018 lines against the 2,000-line
hard limit. The five repeated sccache setup blocks are now a local composite
action at `.github/actions/setup-sccache/action.yml`, reducing the workflow to
1,998 lines while keeping each job's behavior. The disk-usage policy scans the
new action too.

The branch was also behind `main`. Merging `origin/main` produced conflicts
only in generated closure and self-AST artifacts; those were regenerated from
the merged source instead of being hand-combined.

## 9. Verification performed

| Command | Result |
| --- | --- |
| `cargo test --test unit arbitrary_skill_compilation` | 15 passed |
| `cargo test --lib --bins --tests --all-features --verbose` | 2,134 passed, 2 ignored |
| `cargo test --doc --verbose` | clean |
| `RUSTDOCFLAGS="-D warnings" cargo doc --no-deps --all-features` | clean |
| `cargo clippy --lib --bins --tests --all-features -- -D warnings` | clean |
| `cargo check --examples --all-features` | clean |
| `cargo fmt --check` | clean |
| `cargo build --release --bin formal-ai` | clean |
| `experiments/issue-674-agent-cli/run.sh` | four-turn external Agent replay passed |
| `rust-script scripts/check-file-size.rs` | release workflow is under the hard limit |
| `rust-script scripts/check-disk-usage-policy.rs` | workflow and local action are compliant |
| `actionlint .github/workflows/release.yml` | clean |
| `python3 scripts/close-total.py` | generated closure refreshed |
| `python3 experiments/closure_audit.py` | zero true meaning gaps |
| `cargo run --example regenerate_self_ast_census` | generated census refreshed |

Fresh GitHub Actions results are checked at finalization rather than inferred
from an older run.

## 10. Two follow-ups the seed edit forced

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

## 11. Deliberate safety boundary

Learned vocabulary can select only an operation that already has typed,
permissioned host semantics. A new side effect still needs an implementation
and review; evidence and a language model are not allowed to grant authority.
For reproducible review, the public conformance command uses a side-effect-free
host that records the exact ordered dataflow. Production hosts can provide
permissioned fetch, translate, store, and reply implementations without
changing the compiled program or learning protocol.
