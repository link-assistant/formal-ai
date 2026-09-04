# Reading the operand out of prose that names several files (#1069)

The agentic planner has to pick, out of a request written in prose, the one file
the request is asking it to change. Two rules it used to apply were wrong, and
both were caught by the first real Agent CLI run of a change-shaped ladder leaf
rather than by review. Each script here measures the population the rule has to
serve, against data this repository already holds.

## 1. A dotted run of digits is a number, not a file — `survey.py`

The run opened with

    read {"filePath": "/tmp/tmp.QBPcFTv2tg/1.1.2.2.1"}
    Error: File not found

`1.1.2.2.1` is the ladder node's own id. `is_workspace_path` accepted it: stem
`1.1.2.2`, extension `1`, every character allowed. Having observed nothing, the
planner then built an `edit` whose `oldString` was a paragraph of its own prompt,
and finally wrote the resulting error text into the tracked source file.

    tracked files: 15782
    distinct extensions: 67
    extensions that are all digits: 0

    dotted tokens the predicate accepts in committed Markdown: 36109 (8243 distinct)
      name a tracked file                22828 (63.22%)  4929 distinct
      end in an all-digit extension       4611 (12.77%)  1455 distinct
      neither                             8670 (24.01%)  1859 distinct

    most frequent all-digit-extension tokens accepted as paths:
         181  127.0.0.1
          68  Apache-2.0
          55  0.1.0-beta.1
          31  js_0.8.0
          27  v1.22.0

    all-digit-extension tokens that are in fact tracked files: 0 []

No tracked file in the repository has an all-digit extension, and every one of
the 1 455 distinct spellings that shape accepts is an IP address, a licence
identifier, a version or a section number. Rejecting the shape costs nothing and
removes an eighth of the predicate's false positives.

## 2. Which of several named paths is the operand — `order-survey.py`

The same prompt names three files: the source to edit, an effects record and a
proof note. The rule in force read the first *undelimited* path and picked the
proof note, because the file to edit was the one the prompt had bothered to mark
up. Preferring markup instead only moves the failure — nothing stops a request
from marking up an incidental file and leaving its target bare.

`order-survey.py` ranks candidate rules against labelled data the repository did
not write for the purpose: its own commit messages, each paired with the files
that commit actually touched. A commit message is a change request written after
the fact, by the same people whose requests the planner has to read.

    labelled commits (>= 2 named paths, >= 1 of them changed): 418

    rule                                                        correct
      earliest named path                                 286/418   68.42%
      earliest that is workspace-relative (contains '/')  337/398   84.67%
      latest named path                                   266/418   63.64%
      longest named path                                  371/418   88.76%

    messages that open with a bare basename rather than a path: 122 (29.19%)
      e.g. `release.yml` named before `.github/workflows/release.yml`

    restricted to messages naming >= 2 workspace-relative paths: 292
      the first such path is one the commit changed: 240/292 (82.19%)
      earliest changed path by position: {0: 240, 1: 37, 2: 6, 3: 7, 4: 1, 5: 1}

The adopted rule is the second one — prefer a token carrying a `/`, and among
those take the earliest. A `/` is how a request says *where* a file is rather
than merely mentioning it, and a request states what it is acting on before it
says what to do afterwards.

### Why not the rule that scored highest

`longest named path` scores 88.76%, four points better, and is still the wrong
rule. Length is a proxy for *has a directory prefix*, which is the real signal
the shape test reads directly; as a rule in its own right it says nothing a
reader could act on. It also fails the actual task: on the leaf prompt above the
longest of the three paths is `agent-ladder-effects/node-1.1.2.2.1.lino` (40
characters) rather than `src/engine_responses.rs` (22), so it picks the effects
record and the leaf changes no source at all.

That is the limit of this corpus, and it is worth stating rather than hiding. A
commit touches many files, so "is this path among them" is a weaker question
than "is this path the operand", and a message may name a path for context. The
survey is used to rank rules against each other over hundreds of real messages,
not as an accuracy figure for the planner. The exact obligation is pinned by a
test instead: `issue_1069_every_ladder_leaf_reaches_a_real_change` drives all 32
leaf prompts — the whole prompt the ladder really sends, not just the task
sentence — through `plan_chat_step` against the repository's real bytes, and
requires each to reach the file its contract names.

## 3. A delivery destination and a file being edited — `cue-order-survey.py`

With rules 1 and 2 in place the same leaf ran again (`after-fix/agent-stream.jsonl`).
It read `src/engine_responses.rs`, wrote it back with the change, ran `cat` on it
and answered

    Created or updated and observed `src/engine_responses.rs` through the workspace tools.

The edit is right. The node still failed the ladder's verifier with
`missing_proof`, because the same prompt asks for three things and the run
produced one: the tracked change, a structured effect at
`agent-ladder-effects/node-1.1.2.2.1.lino`, and a proof note at
`.agent-ladder/node-1.1.2.2.1-proof.md`. The change route matched the whole
request and answered `Final` for all of it.

The route that already knows how to peel "do this, and leave the answer in FILE"
into a delivery plus a residual — `evidence_record` — sat below the change
routes and never saw the request. Moving it above them is half the fix; the
other half is that its delivery test has to stop reading *"Edit the tracked file
`src/engine_responses.rs`: add "Good morning" to the GREETING_EXAMPLES list"* as
a delivery. That sentence carries the write cue `add` applied to a cued path, so
without a second rule the move makes the planner write its own status line over
the source it was told to edit.

The seed already draws the distinction the fix needs: two families of action
cue, one composing content *into* a destination, one changing the content *of* a
file that has some. `cue-order-survey.py` asks whether mere mention separates
them, over every request-shaped sentence this repository records that names a
file and carries a cue.

    write-action cues: 15 -> ['add', 'append', 'create', 'emit', 'generate', 'leave',
                             'make', 'place', 'produce', 'put', 'record', 'save',
                             'set', 'store', 'write']
    edit-action cues:  12 -> ['change', 'correct', 'edit', 'modify', 'patch',
                             'refactor', 'rename', 'replace', 'rewrite',
                             'substitute', 'swap', 'update']

    recorded request sentences naming a file and carrying a cue: 1118
      lead with a write cue: 961 (85.96%)
      lead with an edit cue: 157 (14.04%)
      mention BOTH families: 30 (2.68%)

Mention does not separate them. Thirty sentences carry both, and the ladder's
own *delivery* sentence is one of them:

    Then create `agent-ladder-effects/node-1.1.2.2.1.lino` … followed by at
    least four words that state the **change** you made

A "mentions an edit cue ⇒ not a delivery" rule discards the record the node
exists to produce. What separates them is position — the same adjacency
principle `cued_write_target` already uses to bind a cue to a path. The cue a
sentence *leads with* governs it; a later cue belongs to a clause the leading one
already took as its object. On the four file-naming sentences of the real node
prompt this reads all four correctly, including the two that carry both
families: `Edit the tracked file …` and `Apply the change … modified …` are
operands, `Then create …node-1.1.2.2.1.lino …` and `Leave supporting evidence in
.agent-ladder/…-proof.md.` are destinations.

The obligation is pinned by
`issue_1069_every_ladder_node_satisfies_every_obligation_it_was_given`, which
replays each of the 32 leaf prompts to `Final` against the leaf's real committed
bytes and requires every file the prompt named to be there at the end — not just
the first effect, which is the blind spot that let a green suite coexist with
`1.1.2.2.1 FAIL missing_proof`.
