# Issue 674: compiling and learning natural-language procedures

Issue [#674](https://github.com/link-assistant/formal-ai/issues/674) closes the
`ARCHITECTURE.md` §16 question carried from the E20 batch. The old skill
compiler understood quoted trigger/response rules and labelled `Skill`/`Step`
forms, but not a procedure stated as ordinary prose. `docs/USER-JOURNEYS.md` F2
therefore stopped before a stored program could be inspected, replayed, or used
by the Agent CLI.

The reference impulse is deliberately not an existing template:

> When I paste a link, fetch its title, translate it to Russian, save both, and
> reply with the translation.

It becomes one trigger and four ordered typed operations:

```text
1. skill_procedure_fetch(skill_procedure_object_title) — "fetch its title" [21..36]
2. skill_procedure_translate(language_russian) — "translate it to Russian" [38..61]
3. skill_procedure_store(skill_procedure_object_both) — "save both" [63..72]
4. skill_procedure_reply(skill_procedure_object_translation) — "reply with the translation" [78..104]
```

## One decomposition path

The procedure compiler does not own a second sentence splitter.
`ordered_requirement_spans` in `src/intent_formalization.rs` is the shared
decomposition primitive used by both the solver's problem-frame construction
and procedure compilation. Separators come from
`ROLE_SKILL_PROCEDURE_CLAUSE_SEPARATOR`; punctuation, multilingual surfaces,
and original UTF-8 byte offsets are handled in one place.

After its cheap procedure-shape guards pass, `compile_procedure_with_ledger`
calls `formalize_intent`, retains the same stable impulse id, and materializes
every ordered clause as a `ProcedureRequirement`. The trigger and each
executable step point back to the requirement they realize. This makes the
chain inspectable:

```text
user impulse
  -> formalized impulse id
  -> ordered source-grounded requirements
  -> typed trigger and steps
  -> canonical program
  -> content-addressed artifact
```

Two guards prevent ordinary prompts from being hijacked. A program needs a
seeded trigger lead and at least two recognized operations. A named capability
gap is returned only after the prompt has proved that it is a procedure.

## Two views of one compiled program

`CompiledProcedure::links_notation` is the canonical semantic view. It contains
only meaning slugs, so equivalent English, Russian, Hindi, and Chinese impulses
produce byte-identical canonical links and the same content id.

`CompiledProcedure::artifact_links_notation` is the durable executable view. It
adds the source impulse, formalized impulse id, ordered requirements, source
spans, trigger, steps, and canonical program. Its parser recomputes the
canonical program, package id, step ids, ordering, and source-span integrity.
A modified or incomplete artifact is rejected.

The generic `ProcedureHost` interpreter walks a parsed artifact in order and
threads each result into the next operation. No generated Rust or JavaScript
handler is needed: hosts provide permissioned semantics for canonical operation
kinds, while the stored `.lino` remains the reviewable program. The solver,
interpreter, later *"why did you do that?"* answer, and Agent planner all consume
the same artifact. The explanation handler reads the earlier assistant artifact;
it does not recompile mutable conversation prose.

## Honest failure and human-gated learning

A clause with no typed operation aborts the entire compile with
`ProcedureCompileError::UncompilableStep`. It records the exact clause, byte
span, and `no compiled capability for "…"`. The solver emits `skill_gap` plus a
complete `procedure_learning_proposal` artifact and compiles no prefix.

That proposal is an automatic learning signal, not an automatic permission to
execute new behavior. The learning path accepts observations that pair an
unsupported surface with a successful, already-seeded paraphrase. It resolves
the paraphrases through the same vocabulary used by the compiler and infers the
canonical typed operation; callers do not provide the operation kind. All
observations must resolve to one meaning, and at least one unsupported surface
must belong to the original named gap, otherwise inference fails closed.

The resulting content-addressed candidate and its evidence are inspectable
Links notation. Promotion into the append-only
`data/meta/procedure-capability-ledger.lino` requires all of the following:

1. a canonical kind that already has typed host semantics;
2. non-empty, unique observed surfaces for en, ru, hi, and zh, each supported
   by a successful paraphrase resolving to that same kind;
3. a named regression suite with at least one pass and zero failures;
4. explicit approval by a non-empty human reviewer.

Declined review, a red suite, unknown operation kinds, duplicate proposals, and
missing language parity are rejected. Candidate identity and paraphrase evidence
survive the ledger round trip, and tampering is rejected. Approved surfaces
enter the same data-driven classifier as seed vocabulary after restart; they do
not create a new Rust parser branch. A genuinely new operation still needs an
explicit permissioned host implementation, which keeps learning from silently
inventing side effects.

## Agent CLI execution

Agent mode routes the same arbitrary procedure through
`src/agentic_coding/procedure.rs`. The external Agent CLI receives the Formal AI
server's tool calls, writes `compiled-procedure.lino`, reads it back for
verification, and runs the public
`formal-ai procedure conformance --artifact … --trigger …` command. Formal AI
verifies the returned step-by-step `procedure_run` before returning the same
artifact, execution evidence, and source-cited restatement. The conformance host
is deliberately side-effect-free, but it is the same generic interpreter path
used by the public CLI and proves that Agent executes rather than merely stores
the compiled program.

`experiments/issue-674-agent-cli/run.sh` is the reproducible driver. It compares
the Agent-authored file byte-for-byte with
`data/meta/issue-674-compiled-procedure.lino`. The retained server log, external
Agent stream, session evidence, and authored artifact live under
`docs/case-studies/issue-674/agent-cli/`.

## Seeded language, not handler prose

Step verbs, objects, trigger leads, and separators live in
`data/seed/meanings-skill-procedure.lino`. User-visible compiled, explanation,
gap, and learning-proposal text lives in
`data/seed/multilingual-responses-procedure.lino` for en/ru/hi/zh. Rust fills artifact
placeholders only. The canonical gap name stays English because it is an
identity-bearing event value; the surrounding response is localized.

## Verification

`cargo test arbitrary_skill_compilation` covers the original criteria and the
review-expanded whole-task path:

- arbitrary prose compiles and executes in order;
- all source clauses become formalized requirements with exact spans;
- en/ru/hi/zh produce identical canonical links and ids;
- an unknown operation emits a named gap and no partial program;
- the complete artifact round-trips, rejects tampering, and executes after
  parsing;
- *"why?"* succeeds from the persisted assistant artifact without the original
  user turn;
- automatic proposals remain inert until green tests and human approval, then
  infer one typed operation from successful paraphrases, survive a
  tamper-checked ledger round-trip, and generalize in all four languages;
- the public CLI and Agent run the persisted artifact through the same ordered
  interpreter and retain the complete execution record;
- the solver, interpreter, explanation, in-repo Agent planner, and external
  Agent CLI use the same artifact bytes.

For an interactive compiler demonstration:

```bash
cargo run --example issue_674_procedure_compiler
```
