# Issue #1166 — The Request Formalized into Obligations

Issue [#1166](https://github.com/link-assistant/formal-ai/issues/1166) (E131):
routing and the executor decided what a request demands by matching surface
phrases, and the literal to print was collected once per mentioning clause.

## The doubled-output root cause (#1156)

The Kotlin Hello World issue body names the literal twice:

```
- The program must print exactly: "Hello, World!"
- Expected Output:
  "Hello, World!"
```

`explicit_stdout` in `rust/src/coding/program_contract.rs` collected every
`print_stdout`-meaning clause that carries a quoted span and joined them with
`\n` — two mentions, one join, and the program printed `Hello, World!` twice.
The executor had no notion that the two clauses *corefer*.

## The fix

`rust/src/intent_formalization/obligations.rs` formalizes the request first:
every enumerated clause becomes a ledger `ObligationNode` (`obligation_ledger`
supplies the node, the span, and the `Underivable` expectation — no new
ledger type), classified into `ObligationKind`s. Output literals are anchored
to their quoted value, and `coreference_pass` merges every mention of the
*same* value — across clauses, regardless of which clause they appear in —
into exactly one node. `unique_output_literal` therefore returns the value
once:

- three mentions in the canonical fixture → one output-literal node
  (`two_identical_output_clauses_produce_one_node`);
- two distinct literals → two nodes and *no* unique literal
  (`two_output_clauses_with_distinct_literals_produce_two_nodes`);
- a filename clause never merges with an output literal
  (`coreference_does_not_merge_filename_with_output_literal`).

## What the graph now catches that phrase matching dropped

The canonical body's "meaningful name like `test-hello-world.yml`", "add
clear comments / best practices", and "CI badge" clauses formalize to
`FileNaming`, `CodeStyle`, and `CiBadge` nodes — requirements the executor
previously never saw, which is how the workflow file came out as `run.yml`.

## Routing by meaning

`rust/src/solver_terminal.rs` now consults
`request_carries_work_obligations` in its leading-shell-token path: a quoted
output literal plus an authoring clause is a sentence about building
something — including in Russian, Hindi, Chinese, and Spanish, whose marker
words the English-only argument check of #1175 does not know — while
`git status` and `echo "Hello, World!"` remain commands.

## Multilingual and paraphrase parity

`rust/tests/fixtures/issue-1166/hello-world-kotlin-{en,ru,hi,zh,es}.txt` and
`-paraphrase-en.txt` carry the same seven-clause structure with per-language
enumeration cues (`First/Также/उसके बाद/接着/Después`) the ledger already
splits on; the parity tests assert the same node counts, the same obligation
kinds, and the same anchored literal. Classification reads the seed lexicon's
own multilingual lexemes first (`github actions 工作流`, `workflow de github
actions`, …); the fallback tables for style, naming, and badge vocabulary are
documented in-module as the interim until a seed meaning owns each one.

## Independent acceptance (2026-11-16)

The obligation graph tells the executor what to build; it cannot also judge
whether the result is right, because a misunderstanding in the reader reaches
the program and any test derived from the same reading. The second half of
#1166 therefore adds an acceptance checker that reads only the task text and
the candidate's observed files and output:

- `rust/src/agentic_coding/task_condition.rs` (JS `js/agentic/task_condition.mjs`)
  reads output literals verbatim and the supported subset of conditional output
  statements into a truth table;
- `rust/src/agentic_coding/task_acceptance.rs` (JS `js/agentic/task_acceptance.mjs`)
  gives every clause a verdict (`discharged`, `contradicted`, `unresolved`,
  `inapplicable`) with a justification and the clause's byte span, and
  `specify_task` gives the task a canonical `meaning` and an exact
  `task_identity`;
- `command_reroute::recipe_step` appends the actionable verdicts to a finished
  recipe's answer, ending with "The task is not complete…" while a required
  clause is contradicted or unresolved.

### Root causes the checker covers

| Criterion | Before | Now |
| --- | --- | --- |
| (a) repeated vs explicit count | every mention appended a line; `twice` was ignored | mentions merge; `explicit_count` repeats the line |
| (b) comparisons, and/or, negation | conditional outputs were bound as unconditional literals | conditions evaluate to a truth table; the outputs stay unbound in the obligations |
| (c) mandatory clauses | a "style" clause was considered satisfied | comments, run instructions and workflow clauses are checked; judgment clauses stay unresolved |
| (d) workflow path and triggers | `.github/workflows/*.yml` was always written | `ci/` vs `.github/workflows`, `push`/`pull_request` and the named file are compared with the issue |
| (e) exact values | literals passed through the reader's quoting | literals are read verbatim; an escaped quote is a gap |
| (f) paraphrases and gaps | phrase matches decided | one meaning per truth table; unsupported grammar is `unread`/`output_unread` |
| (g) shared misunderstanding | the generated test agreed with the program | the checker executes the program itself and compares against the task text |

### Supported subset

```
statement := effect MARKER condition [SEP OTHERWISE effect]
           | MARKER condition SEP effect [SEP OTHERWISE effect]
condition := atom (CONNECTIVE atom)*     one connective kind per condition
atom      := operand [NEGATION] relation operand | operand symbol operand
operand   := identifier | integer
```

Markers, relations, connectives and negation are seed roles. Checked clause
kinds: output, program file, comments, run instructions, and the workflow checks
(path, triggers, runs the program, verifies the output, runtime, observed
status, file name, badge).

### Known limits

- Hindi conditional statements (verb-final) are outside the subset and stay a gap.
- Workflow branch filters are not read.
- Condition inputs are the identifiers in code-point order, passed as
  command-line arguments, and the table enumerates the values 0..2.
- Comments are recognised by the language's line-comment prefix only; run
  instructions must be in the program's leading comment block.
- A workflow's status needs an observed conclusion; without one the clause is
  `unresolved (workflow_status_unobserved)`.
- Clauses the reader cannot read (`unread`, `judgment_required`) do not add a
  report to the answer on their own, but stay visible in `check_task`.

Evidence: `rust/tests/fixtures/issue-1166/acceptance.json`, read by
`rust/tests/unit/issue_1166_task_acceptance.rs` and
`rust/tests/web/issue-1166-task-acceptance.test.mjs`;
`experiments/issue-1166/build-acceptance-fixture.mjs` regenerates it.

## Evidence

`cargo test --test unit issue_1166_obligation_routing` (fully offline).
`cargo test --test unit issue_1166_task_acceptance` and
`node --test rust/tests/web/issue-1166-task-acceptance.test.mjs` (needs `python3`).
