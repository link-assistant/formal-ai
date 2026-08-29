# Issue #1066 — making the ladder's nodes actually solvable

Issue: <https://github.com/link-assistant/formal-ai/issues/1066>

Pull request: <https://github.com/link-assistant/formal-ai/pull/1067>

## What this case study is about

Issue #1066 asks for one capability: Formal AI must be able to author a complete
pull request end-to-end through the Agent CLI. Its second acceptance item is the
measurable proxy — `issue-1028-agent-ladder.yml` completing at `depth=5`, all
sixty-three nodes of a binary decomposition tree.

The issue is explicit about how a failure there is to be treated:

> a failure at any depth is a real capability gap, not a flake, and should be
> fixed generically — never with a prompt-specific branch.

This records the gaps that were found, how each was made visible before it was
fixed, and the two acceptance items this pull request cannot reach on its own.

## The green run that proved nothing

The ladder's per-node criterion is mechanical, and deliberately so: the Agent CLI
must exit 0, and `.agent-ladder/node-<id>-proof.md` must exist, be non-empty, and
open with the exact line `node_path=<id>`. A harness is not supposed to grade
prose.

Driven against this repository, that criterion reported **63 of 63 nodes
passed**. Thirty-two of those proof files said nothing. This one is
representative:

```text
node_path=1.1.1.1.1

Sub-tasks:
```

It exits 0. It is non-empty. Its first line is exactly right. It is also not
evidence that a task was decomposed, and a run of the ladder that accepts it is
measuring the harness rather than the capability.

So the first change was to the measurement, not the code:
`experiments/issue_1066_ladder_offline/judge-proof.py` reads what is *under* the
marker line and fails a node whose proof is a heading with no list, a bare word
naming the work product, a report that the write step failed, or fewer than four
words. Every proof is retained so a reader can check the judgement instead of
trusting it.

## Decisions made in this pull request

- **Judge the proof's content, not only its marker.** The alternative — leaving
  the mechanical criterion alone and reading a sample by hand — is how thirty-two
  hollow proofs passed in the first place.
- **Fix each gap where the general rule was wrong, never at the ladder's
  wording.** Every one of the sixteen defects below is reachable by a request that
  has nothing to do with the ladder, and each has a test written in different
  words from the prompt that exposed it (CONTRIBUTING rule 4).
- **Put general lexical capability in the lexicon, not in a handler.** The
  separable-phrasal-verb reader ("break *the customer import rewrite* into
  sub-tasks") was first written inside
  `src/solver_handlers/task_decomposition.rs`, where it grew the handler by 109
  lines. It belongs beside `mentions_role` / `mentions_role_raw` in
  `src/seed/meanings.rs`, which is where it now is — issue #918's minimal-core
  boundary is the reason, and the boundary ledger records the handler shrinking
  rather than growing.
- **Make a decomposition that cannot enumerate say so.** The alternative to an
  honest refusal is the heading-with-no-list that started this, so
  `Decomposition::unenumerable_reason()` reports *why* the split did not happen
  and eight seeded replies (four reasons × the registered languages) say it in
  the caller's language.
- **Falsify every guard before claiming it.**
  `experiments/issue_1066_ladder_offline/falsify-node-capabilities.sh` switches
  each fix off — one early return, in the one function that decides — and asserts
  the matching test goes red, then restores the file and asserts the set goes
  green. A guard that has never been observed failing is a claim, not evidence.

## The sixteen capability gaps

Each row is a general defect: the request shape that exposes it never mentions
the ladder.

| # | The general rule that was wrong | Where it now lives |
| --- | --- | --- |
| 1 | A path the request asks to *write* was opened for *reading* | `write_request::is_stated_write_target`, `file_read` |
| 2 | Only the agentic router could answer a delivery obligation's residual | `evidence_record::symbolic_answer` |
| 3 | No agentic route reached task decomposition at all | `agentic_coding/task_structure.rs` |
| 4 | Sentence punctuation was peeled one layer, so `` (`Cargo.toml`). `` stopped being a path | `file_path_shape` |
| 5 | A pinned first line rejected the literal write instead of repairing it | `write_request::honouring_pinned_first_line` |
| 6 | A payload that *names* the work product ("the findings") was written as the body | `general_planner::names_deferred_work_product` |
| 7 | An answer describing only a pending web search was delivered as a finding | `SymbolicAnswer::defers_to_the_open_web` |
| 8 | An authoring sentence was read as a delivery destination | `evidence_record::parse_obligation` |
| 9 | An announced enumeration with zero entries was still an answer | `Decomposition::unenumerable_reason` |
| 10 | `enforce_questions` silently deleted a sub-task's text when it ended in `?` | `stated_task::without_sentence_end` |
| 11 | A marker-led literal payload was read across a sentence boundary | `general_planner::end_of_statement` |
| 12 | An English separable phrasal verb was unreadable by the lexicon | `Lexicon::mentions_role_separated` |
| 13 | Work coordinated into its own delivery sentence was consumed with it | `evidence_record::work_before_delivery` |
| 14 | The prompt's *last* colon was read as the one introducing the task | `stated_task::after_introducing_colon` |
| 15 | Framing addressed to the solver was decomposed as if it were work | `stated_task::asking_blocks` |
| 16 | A calculation cue claimed every word written after it | `calculation::sentence_end_from` |

Gap 13 is the one the offline harness could not have found. It came out of the
real Agent-CLI end-to-end leg added for CONTRIBUTING rule 6: the first leg — a
ladder-shaped node prompt written across two sentences — passed, and the second
leg, which asks for the same thing in one coordinated sentence ("Break the
customer import rewrite into sub-tasks **and** record what you work out in
`import-split.md`"), never wrote its file at all. The delivery parser consumed
the whole sentence, so the residual it handed on was empty and the request was
answered in the transcript. Two sentences are the tidy shape and the ladder's own
nodes use it; English does not require it, and the fix reads the work in front of
the write cue, stripping the connective that opens the delivery clause with the
seed's own `skill_procedure_separator` surfaces rather than a new list of English
words (R386).

Gap 14 is the one the offline harness found only after it went green. Every node
proof was non-empty, judged on its content, and honest — and every interior
node's proof said the node was an irreducible single need, while
`decompose_task` on the same text returned four checkable children. Two parts of
one recursion disagreed, which the handler's own module comment says can never
happen. The cause was not in the recursion: the task handed to it was
`all_children_pass`. Every ladder node ends with "Its completion criterion is:
`<criterion>`", the task was read from the prompt's last colon, and a criterion is
a single need. The ladder's wording is incidental — "Break the warehouse
restocking rewrite into sub-tasks. Deadline: the end of the quarter." fails
identically — so the fix is scoping rather than a criterion-shaped exception:
a colon introduces the task only in the sentence that asks the question, which
the handler's own recogniser identifies. The reading moved out of
`src/solver_handlers/task_decomposition.rs` into
`src/task_decomposition/stated_task.rs` beside the recursion it must agree with,
and the boundary ledger records the handler shrinking from 328 lines to 243.

Gaps 15 and 16 are the two the ladder's own PASS was hiding. The node scored
green, and its proof file listed a sub-task that was the seven framing sentences
pasted together with commas — a numbered line a reader can do nothing with, which
is the hollowness this issue is about surviving a mechanical check. A node prompt
states the task, leaves a blank line, and then addresses the solver: which node
this is, where to work, where to leave evidence, what not to claim. That second
block says how to work and how to report; it is not work of its own. So the same
scoping that fixed gap 14 applies one level up — the blocks that *ask* are the
task, and `asks` is the caller's own recogniser rather than a copy of it. Nothing
is dropped when no block asks on its own, because a task can be stated across a
blank line and losing half of it would be the worse reading.

Gap 16 was underneath gap 15 and only became visible once gap 15 was fixed: with
the framing no longer enumerated, the leaf and interior prompts stopped reaching
decomposition at all and came back "I parsed 'only this node's task in this fresh
temporary repository. …' as an arithmetic request but could not evaluate it". The
framing block says "**Solve** only this node's task", and `solve` is one of the
calculator's request cues. An embedded cue was read from where it appeared to the
end of the prompt, so it swallowed four later sentences; those carried the digits
of `node_path=1.1.1.1.1` and an `=`, which is enough to look evaluable. It then
failed to evaluate and answered anyway, at confidence 0.3 — low enough to be
plainly wrong, high enough to displace the 0.86 decomposition the first sentence
asked for. The rule is the one this issue keeps arriving at from different
directions: a request is stated in a sentence, so a cue claims its sentence and
not the rest of the document. "Solve" is ordinary English, and the test that pins
this uses a worker-assignment prompt that mentions no ladder at all.

Gap 11 is the one worth reading twice, because the first fix for it was wrong in
a way the suite caught immediately. Bounding a marker-led payload by its sentence
is right for "Draft a handover memo containing … . Leave the memo in
`handover/2026-q3.md`", and wrong for "Create file `rules.lino` containing" with
the payload on the following lines — a newline ends a sentence, so the sentence
bound recovered nothing and four `issue_656_promotion` tests went red. The rule
that holds for both is stated in terms of the marker's own line: a marker that
says nothing more on its line introduces a *block*, and a block runs to the file
clause as it always did.

## How to reproduce any of this

```bash
# what the planner does with one prompt, tool call by tool call
cargo run --example issue_1066_ladder_node_plan "<prompt>"

# what the symbolic engine alone concludes, with no route involved
cargo run --example issue_1066_symbolic_answer -- --trace "<prompt>"

# what recursive decomposition produces, and why it produced nothing
cargo run --example issue_1066_decomposition_probe "<task>"

# which language a prompt is detected as, which decides the reply's surfaces
cargo run --example issue_1066_language_probe "<prompt>"

# whether a segment is judged independently checkable, which decides a split
cargo run --example issue_1066_checkable_probe "<segment>" "<segment>"

# all sixty-three ladder nodes offline, each proof judged on its content
bash experiments/issue_1066_ladder_offline/run.sh

# every fix switched off in turn, each matching test observed going red
experiments/issue_1066_ladder_offline/falsify-node-capabilities.sh

# the real Agent CLI against a real `formal-ai serve`, both legs
experiments/agent_cli_e2e/run_issue_1066.sh
```
