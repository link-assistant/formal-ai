## Summary

Fixes #1166

Two changes, in both the native and the JavaScript roots:

1. **Typed obligations keep the issue's meaning.** The obligation binding no longer turns every mention of a literal into another output line, honours an explicit count, and leaves conditional outputs unbound.
2. **Independent acceptance.** A new checker reads only the task text and the candidate's files and observed output. It never sees the derivation that wrote the candidate, so a program and a test generated from the same misunderstanding are rejected together.

### What changed

| Area | Native | JavaScript (TS generated) |
| --- | --- | --- |
| Output count and conditional outputs in obligations | `rust/src/intent_formalization/obligations.rs` (`explicit_count`, conditional literals unbound) | `js/agentic/crate/intent_formalization_obligations.mjs` |
| Output literals and conditions from task text | `rust/src/agentic_coding/task_condition.rs` | `js/agentic/task_condition.mjs` → `ts/agentic/task_condition.mts` |
| Per-clause verdicts, canonical meaning, task identity | `rust/src/agentic_coding/task_acceptance.rs` | `js/agentic/task_acceptance.mjs` → `ts/agentic/task_acceptance.mts` |
| Planner wiring: a finished recipe's answer carries the verdicts | `rust/src/agentic_coding/command_reroute.rs` (`recipe_step`) | `js/agentic/command_reroute.mjs` |
| Vocabulary (markers, relations, workflow and comment clauses) | seed roles in `data/seed/meanings-repository-workflow.lino` | same seed |
| Report wording | `data/meta/work-item-steps.lino` templates | same templates |

Every clause gets one of four verdicts, with a specific justification and the clause's byte span:

| Verdict | Example justifications |
| --- | --- |
| `discharged` | `output_matches`, `comments_present`, `workflow_path_matches` |
| `contradicted` | `output_differs`, `comments_missing`, `workflow_trigger_missing`, `workflow_path_differs` |
| `unresolved` | `workflow_status_unobserved`, `unread`, `judgment_required` |
| `inapplicable` | clauses that ask for nothing to check |

There is no generic "style satisfied" verdict. A clause that needs judgment ("clear", "best practices") stays `unresolved`. The report is `complete` only when no required clause is contradicted, and none is unresolved for any reason other than needing judgment.

Example: a finished recipe for the #1450 body whose program has no comments ends with:

```
Independent acceptance of the task: 14 discharged, 4 contradicted, 8 unresolved, 6 inapplicable.
- contradicted (comments_missing, bytes 192-230): Add clear comments explaining the code
- unresolved (judgment_required, bytes 234-290): Ensure the code follows Kotlin best practices and idioms
- contradicted (run_instructions_missing, bytes 294-375): If applicable, include build/run instructions in a comment at the top of the file
- unresolved (workflow_status_unobserved, bytes 817-856): Show a green check mark when tests pass
…
The task is not complete: a required clause above is contradicted or unresolved.
```
(`node experiments/issue-1166/print-bare-report.mjs` prints the full report.)

### Acceptance criteria → evidence

All cases live in one shared fixture, `rust/tests/fixtures/issue-1166/acceptance.json`. Both `rust/tests/unit/issue_1166_task_acceptance.rs` and `rust/tests/web/issue-1166-task-acceptance.test.mjs` read it.

| Criterion | Fixture cases / tests |
| --- | --- |
| (a) repeated mentions → one line; `twice` → two | `once`, `restated`, `mentioned_twice`, `twice`; executed `restated_line_printed_once`, `twice_honoured` |
| (b) reversed comparisons, and/or, negation distinct | `distinctGroups`; executed `reversed_comparison_rejected`, `disjunction_for_conjunction_rejected`, `dropped_negation_rejected` (plus the `*_honoured` counterparts) |
| (c) comments, run instructions, workflow cannot be omitted | #1450 `candidateCases` (no comments, wrong output, other task, …); `a_finished_recipe_reports_only_actionable_verdicts` |
| (d) workflow file name, path, triggers, #1450's `ci/` | `under .github` and `no pull_request trigger` candidates; `workflow_triggers_are_read_in_every_yaml_form` |
| (e) exact Unicode, quotes, backslashes | `exact_backslash_unicode`, `exact_cyrillic_cjk`, `escaped_quote` (a gap, not a guess); executed `exact_backslash_unicode_kept`, `backslash_escape_misread` |
| (f) paraphrases share a meaning, unsupported grammar is a gap | `paraphraseGroups`; six `hello-world-kotlin-*` fixtures (en, paraphrase, es, ru, zh, hi) share one meaning; `unsupported_relation`; executed `unsupported_grammar_is_a_gap` |
| (g) JS/TS/native agree; a shared misunderstanding is rejected | the same fixture in both roots, TS held by `translate-es --check`; `shared_misunderstanding_of_twice` passes its own generated test and is still `contradicted (output_differs)` |
| Exact task identity | `task_identity = stable_id("task", text)`: one added newline gives a different identity |

The twelve Python candidates are actually executed, row by row over the condition's truth table, and only their stdout is observed.

### Supported subset (stated in the module headers)

```
statement := effect MARKER condition [SEP OTHERWISE effect]
           | MARKER condition SEP effect [SEP OTHERWISE effect]
condition := atom (CONNECTIVE atom)*     one connective kind per condition
atom      := operand [NEGATION] relation operand | operand symbol operand
operand   := identifier | integer
```

Checked clause kinds:
- output
- program file
- comments
- run instructions
- workflow: path, triggers, runs the program, verifies the output, runtime, observed status, file name, badge

Anything else is `unresolved (unread)`.

### Known limits

- Hindi conditional statements (verb-final) are outside the subset and stay a gap.
- Workflow branch filters are not read.
- Condition inputs are the identifiers in code-point order, passed as command-line arguments; the truth table uses the values 0..2.
- Comments are recognised by the language's line-comment prefix only.
- Run instructions must be in the leading comment block.
- A workflow's status needs an observed conclusion; without one the clause stays `unresolved`.
- Reader gaps (`unread`, `judgment_required`) do not add a report to an otherwise passing answer on their own, but stay visible in `check_task`.

### How to reproduce the original bug

Take the #1450 body (`rust/tests/fixtures/issue-1166/issue-1450-body.md`) and a candidate that:
- has no comments,
- writes `.github/workflows/*.yml` instead of `ci/`, or
- prints the restated line twice.

Before this change the run was reported done. Now each of these clauses is `contradicted`, with its span, and the answer says the task is not complete.

### Docs

- Requirement rows R1166-9 … R1166-16 in `docs/requirements/issue-1166-request-formalization.md`, the traceability rows, and the regenerated ledger and assembled register. The shard moved to the "coding and agents" part because the reasoning part reached its 1400-line limit.
- `docs/case-studies/issue-1166/README.md` (independent acceptance section)
- `changelog.d/20261116_120000_issue_1166_task_acceptance.md`
- New seed role and template names use `-` (the notation ratchet).

### Unrelated CI repair included here

`data/benchmarks/fast-pull-request-javascript-tests.json` on `main` is missing a comma after `stale-pull-request-report.test.mjs` (lost in a merge). Because of this, every fast JS tier fails while parsing the manifest. This PR restores the comma.

The shared acceptance fixture is laid out compactly (same data, checked by `experiments/issue-1166/format-acceptance-fixture.mjs`) so that it stays under the 1500-line limit for maintained files.
