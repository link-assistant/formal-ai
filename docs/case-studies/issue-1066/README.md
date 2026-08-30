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
  wording.** Every one of the twenty-seven defects below is reachable by a request
  that has nothing to do with the ladder, and each has a test written in
  different words from the prompt that exposed it (CONTRIBUTING rule 4).
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
  and ten seeded replies — the recursion reaches that state in exactly two ways,
  and each way is written once per registered language — say it in the caller's
  language.
- **Falsify every guard before claiming it.**
  `experiments/issue_1066_ladder_offline/falsify-node-capabilities.sh` switches
  each fix off — one early return, in the one function that decides — and asserts
  the matching test goes red, then restores the file and asserts the set goes
  green. A guard that has never been observed failing is a claim, not evidence.

## The twenty-seven capability gaps

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
| 17 | A sentence *specifying* a document to compose was transcribed as the document's bytes | `note_composition::composed_document_specification_span` |
| 18 | A label calling the work atomic was answered instead of the work | `task_structure::plan_task_structure_step` |
| 19 | An answer composed from the request alone overruled a tool result already in hand | `task_structure::nothing_has_been_observed_yet` |
| 20 | A semicolon inside a literal payload ended the payload, because prose was read at the scope shell routing reads at | `shell_command_policy::prose_sentences` |
| 21 | A decomposition that could not enumerate answered a Spanish-speaking client in English | `data/seed/multilingual-responses-decomposition.lino` |
| 22 | The most code-shaped word *anywhere* in the prompt was the search subject, including in a note that only places the worker | `stated_request::request_blocks`, `shell_command` |
| 23 | A permission to use the web, granted in the framing, disqualified the workspace from answering | `stated_request::request_blocks`, `workspace_inspection::asks_about_the_workspace` |
| 24 | Every former of an open-web query read past the blank line, so the note that places the worker was sent to a search engine | `web_research::web_research_query_for`, `intent_router::plan_web_search_step`, `web_research::unresolved_web_research_query_for` |
| 25 | A tool result already in hand was overwritten by a research round that reported it had returned nothing | `web_research::plan_web_research_step` |
| 26 | A line a search *quoted* was read as the search's own diagnosis, so a `grep` that matched fifty lines reported itself as failed | `tool_result::own_words` |
| 27 | The same reading, one renderer along: a search that announces its match count and then quotes each hit under a heading was read as failed | `tool_result::citation_offset` |

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

Gaps 17 and 18 are the last leaf, L32, and they are the clearest example in this
issue of a mechanical check passing over nothing. The node asks: "Produce a final
evidence note containing the selected tree level, node outcomes, test results,
and session id." Its proof file contained, in full, "the selected tree level,
node outcomes, test results, and session id." — the request's own words, written
back as the answer. `judge-proof.py` accepted it: the file existed, it was
non-empty, and it mentioned the node.

Two independent defects had to be fixed before that node produced a note, and the
second was invisible until the first was gone. "Containing" is a content-lead
marker, so the literal-write parser took the words after it for the bytes; that
reading is right for "Create `a.txt` containing 42 is the answer", where the
marker really does introduce the payload, and wrong here, where the sentence
around the marker names a composition action, a document kind and four parts. The
sentence, not the marker, says which reading applies, and the recogniser that
decides is `note_composition`'s own — the route that would have composed the
document is asked whether this is a document to compose.

With the transcription gone the proof changed to "Yes — this task is atomic: no
split of it yields two sub-tasks that can be checked independently." True, and
not what was asked. The ladder labels each leaf "Atomic task L32: …", and that
label alone carries both signals the task-structure route reads — the atomicity
predicate and the task noun — so the route answered the question the *heading*
posed and the sentence after the colon was never reached. Naming something to
produce states work to do, not a task to classify, so the route stands aside for
a request that specifies a document. It stands aside on the same recogniser gap
17 uses, which is why the thirty interior nodes are untouched: they ask for
"independently checkable evidence", and "evidence" is not a document kind the
seed knows.

Gap 19 is the one that accounts for the other thirty-one leaves, and it is a
route-ordering defect rather than a reading defect. Every other agentic route
plans a tool call and stands aside once that call has been made. The
task-structure route plans no call at all, so its answer is composed from the
request alone and is therefore the *same answer on every turn* — nothing it
reads ever changes. Sitting thirty lines above the route that reports a tool
result, it did not merely repeat itself: it answered over work the planner had
already done. `Atomic task L01: Inspect the existing task-decomposition data
model and identify where a node stores its children.` is planned, correctly, as
a repository search on the first turn; the second turn exists only to report
what came back, and the task-structure route claimed it and reported a
four-step decomposition of the leaf's own instructions instead. Measured with
`FORMAL_AI_TRACE_REQUESTS=1`, turn 0 is `grep` and turn 1 is a `write` of the
template; with the route suppressed, turn 1 is a `write` of the grep output.
The rule is that an answer that needed no evidence may not overrule evidence
gathered for the same request, and the route now asks the question its
neighbour is asked at the call site — has anything been observed yet?

Gap 11 is the one worth reading twice, because the first fix for it was wrong in
a way the suite caught immediately. Bounding a marker-led payload by its sentence
is right for "Draft a handover memo containing … . Leave the memo in
`handover/2026-q3.md`", and wrong for "Create file `rules.lino` containing" with
the payload on the following lines — a newline ends a sentence, so the sentence
bound recovered nothing and four `issue_656_promotion` tests went red. The rule
that holds for both is stated in terms of the marker's own line: a marker that
says nothing more on its line introduces a *block*, and a block runs to the file
clause as it always did.

Gap 20 is gap 11 read one level down, and the real Agent-CLI end-to-end leg is
what found it. Bounding a payload by its sentence is right; the question gap 11
left open is *whose* sentence. `end_of_statement` reused the splitter that shell
routing uses, and that splitter ends a sentence on a semicolon on purpose,
because `build; deploy` names two commands to judge one at a time. Prose does not
read it that way. Asked to write out issue #918's minimal-core invariant — "… or
a host surface; domain knowledge and policy belong in data." — the planner
produced a file ending at *host surface;*, with the clause saying where domain
knowledge goes dropped, and `issue_918_agent_cli.sh` went red in CI on a file
whose first half was correct. The fix is not a special case: the two readings
disagree about one character and nothing else, so one `split_sentences` takes the
terminator set as an argument and `prose_sentences` is the reading prose gets.
The test that pins it is a retention policy — "Logs are kept for ninety days;
backups are kept for a year." — which mentions neither #918 nor the ladder.

Gap 21 is not a routing defect at all, and it is the one that most resembles the
hollowness this issue is named for. `check_language_change_parity` reported that
the decomposition response family had records for `en`, `ru`, `hi` and `zh` and
none for `es`. `localized_response` falls back to the seed's `language unknown`
record and then to English, so the gap did not fail loudly: a Spanish speaker
asking why nothing could be enumerated was told something true, in words they had
not asked in. An honest refusal delivered in the wrong language is hollow in a
second way. All thirteen decomposition intents were translated rather than the
two the gate strictly required, because the same silent fallback was covering the
other eleven, and the test pins the exact sentence per language rather than
asserting that some text came back — asserting non-emptiness is what let the
fallback hide.

What gap 21 fixes is the *reply* side: a client whose response language is `es`
is now answered in Spanish by every decomposition intent. Recognising a
decomposition *asked* in Spanish is a wider hole and is not closed here —
`data/seed/meanings-decomposition.lino` carries `en`, `ru`, `hi` and `zh`
lexemes and no `es` one, and so do most of the seed's meaning files. That is the
repository's state before this work rather than something this pull request
changed, and saying it is fixed because the parity gate is green would be the
same green-checkmark-over-nothing.

Gaps 22 and 23 are the ones only the real ladder could show, and they are the
reason this case study has a second half. Every node prompt the ladder sends is
two blocks:

```text
Atomic task L03: Inspect the existing atomicity check and record the observable
completion contract for leaves.

This is recursive binary-tree node 1.1.1.2.1 at depth 5. Solve only this node's
task in this fresh temporary repository. [...] Use web research when it
materially improves factual accuracy. Do not claim success without evidence.
```

The second block is a note that places the worker. It states no work. Two routes
read both blocks as one string, and the note won both times.

The subject went first. `shell_command::shaped_code_search_token` picks the most
code-shaped token in the prompt, scoring `.` and `_` at four, an interior capital
at three and `-` at two, and breaking ties on length. Nothing in the task above
is code-shaped; `binary-tree` is. In the pre-fix 32-leaf run kept under
`docs/case-studies/issue-1066/ladder-before-fix/`, the real Agent CLI planned
twenty-nine `grep` calls across the thirty-two leaves, and **twenty of them
searched for `binary_tree`** — see `grep-patterns.tsv`, which is regenerated from
the committed `agent-stream.jsonl` files by the command in that directory's
README. `task_decomposition`, which eight of the tasks are explicitly about,
accounts for two. The tie-break is visible in the two exceptions: `sixteen-node`
and `thirty-two-node` beat `binary-tree` because they are longer, not because
they are better.

The destination went next. `workspace_inspection::asks_about_the_workspace`
admits a request only if it names an inspection action and does *not* name an
external source. "Use web research when it materially improves factual accuracy"
names one. Granted in a separate paragraph it is a permission to reach for a
tool, not a statement that the answer is on the internet — but read as part of
the request's own words it disqualified the repository the node had just been
handed, for all sixty-three nodes. Eight of the thirty-two proofs record the
result in `verdicts.tsv`: `hollow_reported_failure`, a whole sentence saying the
lookup returned no content, which proves exactly as much as an empty file.

The fix is one rule, and it is the rule this repository already applies one layer
up: `task_decomposition::stated_task::asking_blocks` narrows a prompt to the
block that asks before it decomposes anything (gap 15). `stated_request::request_blocks`
states it for the agentic routes — a block that only places the worker is not
work, so nothing is read out of it — and each route asks it separately, because
each route asks which block carries *its* act. A prompt with a single block is
returned whole and byte-identical, so the change is visible only where a second
block exists.

Judging changed with it. A proof that reports an empty tool result is now hollow:
`returned no content` joins the failure markers in `judge-proof.py`. This is not
the same as a search that ran and matched nothing — "no matches" is an
observation about the workspace and stays a valid proof.

Gaps 24 and 25 are what the first green real run left behind, and they are the
reason a mechanical PASS is reported here alongside a judge verdict. With gaps 22
and 23 fixed, all thirty-two leaves exited zero and wrote a proof whose first line
was its own node path, and five of them said this:

```text
node_path=1.2.1.2.1

Research completed for a two node decomposition at depth one this is recursive
binary tree node 1 2 1 2 1 at depth 5 solve only this node s task in this fresh
temporary repository its completion criterion is observable evidence exists use
web research when it materially improves factual accuracy do not claim success
without evidence, but the tool returned no content.
```

Two separate defects are visible in that one sentence. The query ran past the
blank line again — gap 22 fixed the *search* subject and gap 23 fixed the
*destination*, but three further functions formed an open-web query from the
whole prompt: `web_research::web_research_query_for`, which reads the stated
research subject; `intent_router::plan_web_search_step`, which reads an explicit
search request; and `web_research::unresolved_web_research_query_for`, the
planner's last route, which searches for whatever nothing else understood. Each
now asks `stated_request::request_blocks` the same question the other routes
already ask, which is why gap 24 is one row and three call sites: it is one rule
that had three copies of the old reading.

The second defect is the clause after the comma. The node *had* searched its
workspace and *had* been handed what it said; `grep` output was sitting in the
transcript. `plan_web_research_step` looks at the last completed call to decide
whether to research further, and its "some other call completed" arm ended in
`final_answer`, which composes a report about the research round. There was no
research round. The arm was speaking for work it had not done, and speaking over
work the agent had already done. It now plans a deeper round or stands aside, and
standing aside is what lets `tool_result::latest_turn_answer` report the search
that actually ran. Nothing about the ladder is in that rule: the test that pins
it uses a Dvorak-keyboard question with a completed `grep` in front of it.

Gaps 26 and 27 are the same rule read twice, and they are the last thing the two
runs found. Sixty-two of the sixty-three offline nodes were judged `ok`; node
2.2.1.1.2 was not, and its proof opens like this:

```text
node_path=2.2.1.1.2

The command failed: ./scripts/opencode-conversation-to-lino.py:6:Formal AI's
context CLI owns the shared JSON-to-Links-Notation conversion.
./scripts/install.sh:10:#   telegram  the Telegram bot (alias for `cli`: the bot
ships inside the CLI)
[...]
./scripts/install.sh:260:    log "the 'code' CLI was not found on PATH."
```

The search succeeded. It matched fifty lines, and the node wrote every one of
them into its proof under a sentence saying the command had failed. The reason is
on the last line quoted above: `tool_result::looks_like_error` asks the failure
lexicon about the first 512 characters of a status-less result, and one of the
files `grep` matched is an installer that prints *not found* when a program is
missing. Those words are `install.sh`'s. The lexicon cannot tell whose they are.

Naming the tool would fix this node and nothing else — the same reading is wrong
for `codesearch`, for a `bash` step running `grep`, and for any harness that
hands back quoted text. What separates a quotation from a diagnosis is not
vocabulary but that **a quotation says where it came from**. So the lexicon is
asked only about the result's *own words*: `own_words` cuts the text where it
stops speaking and starts naming what it quotes, and `looks_like_error` reads
that framing instead of the whole body.

Gap 27 is what the real Agent CLI added to that, and it is why the cut is stated
in terms of citations rather than of a line's first characters. The offline
harness renders `grep` the way `grep` does, one `<path>:<line>:<text>` per line.
The real CLI does not:

```text
Found 100 matches
/tmp/tmp.V4WehYZFRA/CHANGELOG.md:
  Line 65: - Failure-driven splitting: a failed task can be shrunk [...]
```

Not one line of that starts with a path and a number, so a rule about first
characters passes it straight through to the lexicon, which finds *failed* in the
changelog entry the search matched and reports the search as the failure. Both
renderings agree on the thing that matters: somewhere on the line, a number
stands as its own word before a colon, and everything after it belongs to
somebody else. `citation_offset` looks for that anywhere in the line, and the cut
is made at the start of the citing *word*, not of its line — a step that does
fail often says so and then points at the place, so `Error: cannot read
src/lib.rs:12: No such file` keeps `Error: cannot read` and is still read as the
failure it is.

Two lines have to cite before a body counts as a quotation
(`CITED_LINES_THAT_MAKE_A_QUOTATION`). One is not enough: `HTTP/1.1 404: Not
Found` cites a place by the letter of the rule — `404` stands as its own token
before a colon — and it is a diagnosis, not a quotation. A body of quotations
comes in a list. A harness announcing its own refusal cites no place at all,
which is why `grep: /etc/shadow: Permission denied` is still read as a failure —
and that half is pinned by the same tests, so the fix cannot be widened into
"never report a failure".

The judge had the same defect, and finding it is what makes the offline numbers
above worth reading. `judge-proof.py` scored node 2.2.1.1.2's *fixed* proof
`hollow_reported_failure`, because one of the lines the search now legitimately
quotes is `except Exception as error:` and the judge matched its markers against
the whole file. It had been catching the pre-fix proof by accident, on the quoted
text rather than on the sentence above it. The judge now mirrors `own_words`, and
it carries the renderer's own failure sentence — "The command failed" and its
Russian, Hindi and Chinese equivalents — as a marker, which it never had. The
correction is strictly conservative: re-judged with it, every run recorded in
`experiments/issue_1066_ladder_offline/README.md` returns the same counts it did
before, and all eight hollow verdicts in `ladder-before-fix/` stay hollow.

## What is still hollow, and why the judge does not see it

Thirty-two leaves pass and twenty-seven are judged `ok`. Fourteen of those
twenty-seven contain the same sentence, byte for byte:

```text
This task cannot be split into two sub-tasks that can be checked independently,
and no observable completion criterion is known for it, so there is nothing to
enumerate: it is an irreducible single need.
```

That is a true statement about the request. It is not an answer to it. L03 asked
the agent to "Inspect the existing atomicity check and record the observable
completion contract for leaves"; L24 asked it to "Verify the ladder order for all
mode is 32, 16, 8, 4, 2, then the root". Both were told their task was atomic.
`judge-proof.py` accepts all fourteen, and it is right to by its own contract: it
never receives the task, and it judges the *shape* of a proof, not its wording. A
judge keyed to these particular sentences would be a phrase list, which is the
thing this issue exists to remove.

The route is `evidence_record::plan_evidence_record_step`. It splits the delivery
obligation off, re-plans the residual through `plan_chat_step`, and when every
agentic route declines the residual it falls back to `symbolic_answer` — the
symbolic engine answering the request directly. `FormalAiEngine.answer` reads
"Atomic task L03: ..." as a decomposition question and returns the verdict above.
Gap 18 taught the *agentic* task-structure route to stand aside for a label like
that; the symbolic fallback does not consult that route, so it answers anyway.

Every agentic route declines because of the subject rule in
`shell_command::workspace_inspection_query_for_task`: a request is only searched
for when it names a code-shaped subject — an underscore, a dot, an interior
capital, a hyphen. "the existing atomicity check" is prose, so nothing is
searched for, and the request falls through to a verdict. This is an open
capability gap, stated here rather than papered over. Three candidate fixes were
measured and all three were refuted.

**Fix A — grep for the longest content word the lexicon does not know.** Measured
against this repository with `git grep -lie`, the words such a rule selects are
not subjects at all: `repository` matches 3033 files, `workflow` 2470,
`existing` 2097, `structure` 1632, `committed` 1413, `distinct` 1067, `including`
1109, `explicitly` 1035, `selected` 931. Even the most specific candidates —
`composite` 291, `recursion` 237, `atomicity` 113 — return more files than any
proof could report. Grepping a prose word is noise, not an observation.

**Fix B — grep for the noun phrase instead of the word.** This one measures well:
`atomicity check` matches 3 files, `Links Notation rendering` 1, `recursive
execution adapter` 1, `dotted binary path` 1, `internal node` 2, `ladder order` 2.
It is refuted by what it would cost elsewhere. `workspace_inspection::asks_about_the_workspace`
already admits "Verify the current exchange rate between the euro and the yen" —
*verify* is a seed-declared inspection action and no external source is named —
and the only thing keeping that request out of `grep` and on the open web is the
code-shape subject rule. Relaxing the subject to any noun phrase sends the
exchange rate to the repository, which the existing guard
`a_question_the_workspace_cannot_answer_still_reaches_the_open_web` forbids, and
forbids correctly.

**Fix C — let `task_structure` stand aside whenever the request asks about the
workspace.** Written, run, and reverted the same session. It never fired where it
mattered — these fourteen nodes reach the verdict through `evidence_record`, not
through `task_structure` — and where it did fire it made things worse: "Atomic
task L16: Verify every tested internal node has exactly two children and never
three or more" went from a wrong answer to no plan at all, which is a failed
write and a hollow proof rather than a misleading one.

What all three have in common is that separating "the existing atomicity check"
from "the current exchange rate" needs the planner to know which nouns name
artefacts, and there is no such distinction in the seed. Adding one — a list of
kinds like *check*, *adapter*, *rendering*, *contract* — would pass these
fourteen prompts and fail rule 4 of CONTRIBUTING, which asks for generality
proved with different words each time. The gap stays open and stays written down.

## Where each acceptance item stands

Issue #1066 lists six. Four are done here, and two cannot be reached from a
branch — they need a merge, a release, and publish credentials. Naming them as
done would be exactly the green-checkmark-over-nothing this issue exists to end.

| # | Acceptance item | Status |
| --- | --- | --- |
| 1 | `experiments/issue_924_self_authoring/run.sh` passes on the server, evidence committed | **Done** |
| 2 | `issue-1028-agent-ladder.yml` completes at `depth=5` — the 32-leaf level, real Agent CLI | **Done** |
| 3 | A merged pull request in the open cycle with every introduced commit validly attributed | **Not reachable here** |
| 4 | `Auto Release` succeeds; the new version is on crates.io and GHCR `:latest` | **Blocked by 3** |
| 5 | A Hive Mind `solve --model formal-ai` run reports the new version and produces a non-empty pull request | **Blocked by 4** |
| 6 | No deferral budget, grace period, or bypass anywhere in the release path | **Done** |

**Item 1.** `run.sh` aborted on this server with `Encoding::InvalidByteSequenceError`
while the report it was reading recorded `"solved": true`: Ruby's `File.read`
decodes with the locale's default external encoding, and a bare container's
locale is `POSIX`. The run had succeeded and only the assertion harness had
failed. Both incremental harnesses now read the report as `Encoding::UTF_8`, the
harness passes end to end here, and its evidence is committed under
`docs/case-studies/issue-924/incremental-self-authorship/`: the dispatch report,
`formal-ai.log`, and five transcripts — four real Agent CLI sessions
(`ses_fb3751286ffeboekWxOdOhBgAs`, `ses_fb3750160ffel61GySqv2RK388`,
`ses_fb374f1c8ffe7yKcWfmhT9M26f`, `ses_fb374e16cffeh4E677qM46nCCO`) and the
composed verifier.

**Item 2.** `depth=5` selects the 32-leaf level, and every leaf is a real Agent
CLI session against a real `formal-ai serve`. The witness is workflow run
[33280775741](https://github.com/link-assistant/formal-ai/actions/runs/33280775741),
which reports `conclusion: success`; its log is kept at
`ci-logs/issue-1028-agent-ladder-33280775741.log`. Committed alongside it is a
local re-run of the same 32 leaves under
`docs/case-studies/issue-1028/agent-tree-run/`, with every node's proof file as
the Agent CLI left it.

The item is done as the issue states it, and the proofs are reported as they
are. Re-judging that run with `judge-proof.py` gives 32 of 32 mechanically
passing, 27 of 32 judged `ok`, and 5 hollow — the five gap-24/25 proofs quoted
above, which the fixes in this pull request turn into reported search results.
Of the 27, fourteen are the same decomposition verdict; that is an open
capability gap and it is written up in full under
"What is still hollow, and why the judge does not see it". Before gaps 22 and 23
the same run judged 24 `ok` and 8 hollow.

**Item 3 is the one to read carefully.** It asks for a *merged* pull request in
the open release cycle in which **every** introduced non-merge commit carries
`Formal-AI-Session`, `Formal-AI-Evidence`, and `Formal-AI-Pull-Request`. This
pull request cannot be that one, and no amount of work inside it can make it
that one: its commits were authored here, and CONTRIBUTING is explicit that
"Do not add these trailers to a human-authored or manually corrected commit."
Adding them would be the fake result the issue forbids, and the metric checks
for it anyway — "One attributed commit cannot make a mixed manually authored
pull request count as end-to-end self-development."

What this pull request can do for item 3, and does, is make the capability real:
the capability gaps below are the reason a Formal AI-authored node came back with
an empty heading instead of a decomposition. Item 1's committed sessions are the
loop running end to end on this machine.

**Item 4 is blocked by item 3, and the block is measured, not assumed:**

```console
$ rust-script scripts/check-self-development-release.rs
Output: should_release=false
Self-development release preflight failed: release cycle v0.345.0..HEAD has no
merged Formal AI-authored pull request; an end-to-end Formal AI-authored pull
request requires valid session evidence and the same canonical PR trailer on
every introduced non-merge commit
$ echo $?
1
```

There is a second gate behind that one, and saying "item 4 follows from item 3"
without checking would be its own unverified claim. So
`experiments/issue_1066_qualifying_pr/dry-run.sh` builds the missing pull
request in a throwaway clone — an attributed commit whose session id is read out
of item 1's committed evidence bundle, merged with a matching merge subject —
and runs the gates against it:

```console
$ experiments/issue_1066_qualifying_pr/dry-run.sh
== the cycle, measured ==
0.46% (1829/399988 changed lines; 8/306 commits)

== the release preflight, with a qualifying pull request in the cycle ==
Output: should_release=false
Self-development release preflight failed: self-hosting target would fall from
12.77% to 0.94% for v0.345.0..HEAD; merge additional reviewed Formal AI-authored
work before cutting the release
exit=1
```

The refusal moves, and it does not go away. The ledger's target is 1277 basis
points; this cycle is 46, and a qualifying pull request merged today projects
94. So item 4 needs more than item 3 — it needs enough merged Formal
AI-authored work to hold the ratchet — and that is the ratchet doing its job.
Item 6 is what keeps both refusals honest: before this work, the same cycle
reported **success** for the first seven days. `main` staying red on
`Auto Release` is the state of the repository, honestly reported. Do not resolve
it by relaxing a limit.

**Item 5** needs a version that exists. It reads the release Hive Mind would
pull, so it cannot run before item 4 cuts one.

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

# the 32-leaf ladder itself, real Agent CLI per node, ~3.5 min a node
TREE_DEPTH=5 experiments/issue_1028_agent_cli_ladder/run.sh

# judge any proof the way this case study judges them
python3 experiments/issue_1066_ladder_offline/judge-proof.py \
  docs/case-studies/issue-1028/agent-tree-run/1.1.1.2.1/proof.md 1.1.1.2.1
```
