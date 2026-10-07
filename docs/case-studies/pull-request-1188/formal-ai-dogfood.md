# Dogfooding the JavaScript Formal AI on agentic coding tasks (PR #1188)

The JavaScript root of Formal AI runs from source (`node js/server/main.mjs
--agent-mode`), with no Rust build, so it is the fast loop for making Formal
AI do agentic coding itself. This ledger records every task it was given,
what it did, and the fix when it failed. The owner's direction: "the smallest
possible tasks you encounter on the way must be fully supported by it".

## How the tasks were run

- **Client-level (the real thing).** `node js/server/main.mjs --port 18931
  --agent-mode` with `FORMAL_AI_MEMORY_PATH` in a throwaway directory, and the
  link-assistant Agent CLI 0.26 (`agent run --prompt … --model
  formal-ai/formal-ai`) with an `opencode.json` provider block pointing at
  `http://127.0.0.1:18931/v1`. Each task runs in a new git sandbox, so the
  resulting `git diff` is the outcome.
- **In-process (the inner loop).** `node experiments/js_dogfood/drive.mjs
  --dir <sandbox> "<prompt>"` runs the same planner the server runs
  (`planChatStep`) and executes each tool call on the sandbox with the Agent
  CLI's result shapes. A run takes about a second instead of a CLI session.
  Every fix is reproduced here first and then confirmed through the CLI.
- **Regression tests.** `rust/tests/web/pull-request-1188-dogfood.test.mjs`
  replays each fixed task with the Agent CLI's tool names and result formats
  over an in-memory workspace, and asserts the exact file content and the
  exact answer.

## Ladder

| # | Task (prompt) | Before | After |
| --- | --- | --- | --- |
| T1 | `Add a function multiply(a, b) to math.mjs that returns a times b, add a test for it to math.test.mjs, and run node --test to confirm it passes.` | **Fail, destructive**: `math.mjs` was overwritten with the text `a function multiply(a, b)`; a plan file was written under `.formal-ai/`; the answer said the change was complete. | Open (see "Next") |
| T2 | `Create a file a.txt containing hello` | Pass: `a.txt` = `hello`. | — |
| T3 | `Read the file math.mjs and tell me its first line.` | Pass: answered with the first line. | — |
| T4 | `Append the line 'third line' to notes.txt.` | **Fail, destructive**: `notes.txt` was overwritten with `the line 'third line'`. | **Pass**: read → edit → `sha256sum` check; one line added; answer `Appended \`third line\` to the end of \`notes.txt\` and observed the result.` |
| T5 | `In greet.py rename the variable nmae to name.` | Pass: both occurrences renamed. | — |
| T6 | `Fix the typo 'smal' in README.md.` | **Fail**: read the file, echoed it, changed nothing. | Open |
| T7 | `Replace 'smal' with 'small' in README.md.` (README also says `A small tool.`) | **Fail**: edit `oldString: "smal"` — the Agent CLI refuses it ("Found multiple matches"); an edit tool that takes the first match turns `small` into `smalll`. Without the second line it passed, answering only "The command completed successfully without output." | **Pass**: read → edit of the one changed line → `sha256sum`; answer `Replaced \`smal\` with \`small\` in \`README.md\` and observed the result.` |
| T8 | `Insert 'middle' after the line 'first line' in notes.txt.` | **Fail**: no plan at all (single quotes were not literals). With double quotes it worked but answered "The command completed successfully without output." | **Pass**: read → edit → `sha256sum`; answer `Inserted \`middle\` after \`first line\` in \`notes.txt\` and observed the result.` |
| T9 | `Delete the line 'second line' from notes.txt.` | **Fail**: read the file, changed nothing. | **Pass**: read → edit (`second line\n` → empty) → `sha256sum`; answer `Removed \`second line\` from \`notes.txt\` and observed the result.` |
| T10 | `Run ls and summarize what is in this directory.` | Pass (lists the output; no prose summary). | — |
| T11 | `How many lines are in notes.txt?` | Pass (`wc -l`). | — |
| T12 | `Delete the file notes.txt.` / `Rename the file notes.txt to todo.txt.` | Pass (`rm`, `mv`). | — |
| T13 | `Run python3 greet.py.` | **Fail**: ran `python3 greet.py.` ("can't open file 'greet.py.'"); `` Run `ls -la`. `` also kept the backticks. | **Pass**: runs `python3 greet.py`. |
| T14 | `Create a directory named src.` | **Fail**: listed the directory instead. | **Pass**: `test ! -e src` → `mkdir src` → `test -d src`; answer `Completed the action \`mkdir src\` and verified it with \`test -d src\`.` |
| T15 | `Commit all changes with the message 'initial notes'.` | **Fail**: web search for the sentence. (`Commit the changes.` committed as `chore: commit pending changes` and then failed on `git push` with no remote, reported as completed.) | **Pass**: one commit `initial notes`; push only when `git remote` names one. |
| T16 | `Show me git status.` | **Fail**: web search (the seeded cue is `show git status`). | **Pass**: runs `git status` (also `Show me the git log.`, `Покажи мне статус git`). |
| T17 | `Change the value of "debug" to true in config.json.` | **Fail**: edit with `oldString: the value of "debug"`. | **Pass**: read → edit of the `"debug"` line → `sha256sum`; answer `Set \`debug\` to \`true\` in \`config.json\` and observed the result.` Also `Set the value of name to prod in config.json.` (keeps the quotes) and YAML `debug: …`. |
| T18 | `Create hello.py that prints Hello, World! and run it.` | **Fail**: answered with a program in chat (named `main.py`), wrote nothing, ran nothing. | Open |
| T19 | `Write a Python function add(a, b) that returns their sum in add.py and run it with 2 and 3.` | **Fail**: general-change `literal_file` plan, `add.py` = a phrase of the request. | Open |

## Root causes and fixes

### T4 — an append was planned as a whole-file write

**Root cause.** No planner arm knew that a quoted payload can be placed at one
end of an existing file. `Append the line '…' to notes.txt.` therefore reached
the general-change fallback (`composeGeneralChangePlan`,
`js/agentic/general_planner.mjs`), which reads any "verb + file + content"
request as `literal_file` mode: write the content as the *whole* file. The
content it extracted was the clause after the verb (`the line 'third line'`),
not the quoted payload. Same mechanism as T1.

**Fix (general, seeded).**

- `data/seed/meanings-repository-workflow.lino` gains two position meanings,
  `file_edit_position_end` and `file_edit_position_start`, next to the existing
  `file_edit_position_after` / `_before`, each with en/ru/hi/zh/es surfaces
  (`append`, `to the end of`, `в конец`, `追加`, `al final de`; `prepend`,
  `at the top of`, `в начало`, `开头`, `al principio de`, …).
  `data/seed/roles.lino` is regenerated by `scripts/generate-role-registry.py`.
- `js/agentic/workspace_change.mjs` (the arm that already owns grounded
  renames and replacements) gains an end insertion: the position from those
  meanings, the payload from the request's one quoted segment that is not the
  path, the target from the one workspace path. It reads the file, computes the
  new content (a file without a final newline keeps none; a missing file is
  created, as `>>` would), edits the smallest unique run of trailing (or
  leading) lines, checks the result with `sha256sum`, and answers from the
  seeded responses keyed by the same two meanings
  (`data/seed/multilingual-responses-agentic-tools.lino`, five languages).
- Because the arm runs before the general-change fallback, the fallback no
  longer sees append requests.

**Test.** `rust/tests/web/pull-request-1188-dogfood.test.mjs`, "an append
keeps the file it appends to": the exact T4 prompt and tool sequence
(`read`, `edit`, `bash`), the exact file content and answer, no plan file;
plus prepend, no-final-newline, missing-file and Russian variants.

**Rust twin.** `rust/src/agentic_coding/workspace_change.rs` still lacks the
end insertion, so the native engine still overwrites on T4. The seed data is
shared; the code is a port of `groundedEndInsertion`, `insertedAtEnd`,
`compactEndEdit` and `planEndInsertionStep`.

### T7 — a replacement that contains its pattern was handed to a bare edit

**Root cause.** `groundedRewrite` (`js/agentic/workspace_change.mjs`) refused
any substring replacement whose new text contains the old one, because the
substring Markov rewrite would not terminate on `smal` → `small`. The request
then fell to the intent-router edit, which sends the bare word as `oldString`
— ambiguous whenever the word also sits inside a longer one.

**Fix.** A replacement of one word by another word containing it is
word-scoped (the same scope a rename uses; the stated intent stays
"replaced"). When the bare pattern is not unique in the file, the edit carries
the smallest run of whole changed lines that is unique (`changedLinesEdit`);
the result is checked by digest as before.

### T8 — single-quoted literals were invisible to the positional insert

**Root cause.** `quotedLiterals` in `js/agentic/positional_edit.mjs` (and
`quoted_literals` in `rust/src/agentic_coding/positional_edit.rs`) had its own
scanner for `"` and `` ` `` only, while every other literal reader uses
`quotedSegmentSpans`, which also reads `'…'` (with an apostrophe guard), «»,
“”, 「」. The insert also ended in the generic tool-result answer, because only
the intent-router edit handled it.

**Fix.** Both roots read literals through the shared `quoted_segment_spans`
(Rust twin implemented). The JS workspace-change arm now owns positional
inserts: the anchor must occur exactly once (otherwise the honest
verification-failure answer, nothing written), the edit is checked by digest,
and the answer comes from seeded responses keyed by the existing
`file_edit_position_after` / `_before` meanings (five languages).

**Tests.** "a replacement edits only what it names" in
`rust/tests/web/pull-request-1188-dogfood.test.mjs`: the T7 file, the T8
prompt, and a twice-occurring anchor that must not be edited. The test's edit
tool refuses an ambiguous `oldString` exactly as the Agent CLI does.

**Rust twin.** The quoting fix is in both roots. The word-scoped replacement,
the line-scoped edit and the positional ownership (with its seeded answer)
are JS-only so far: `rust/src/agentic_coding/workspace_change.rs`.

### T9 — no arm removed quoted text from a file

**Root cause.** Nothing in the seed bound "delete/remove" to *text in a file*:
the only removal surfaces belonged to `cancel` (undo a program change) and to
reserved-word lists. With no edit pair and no write content, the request fell
through to the file-read arm, which showed the file.

**Fix.** A seeded action meaning `coding_text_remove` (role
`coding_text_remove_action`; en/ru/hi/zh/es surfaces; `drop`/`erase` were left
out because they are not grounded tokens yet) and a removal in the
workspace-change arm: every line that is exactly the quoted payload goes,
otherwise its single occurrence inside a line does; anything else is the
honest verification-failure answer with nothing written. The end insertion and
the removal now share one planner, `planComputedChangeStep` (read → compute
bytes → smallest unique edit, else whole-file write → digest → seeded answer).
`Delete the file notes.txt.` still plans `rm` — a quoted path is never a
payload.

**Tests.** "a removal takes out what it quotes": the T9 prompt, an in-line
removal, the file-deletion non-regression, and text found nowhere.

**Whole JS suite.** `node --test --test-concurrency=1 rust/tests/web/`:
945 pass, 0 fail after T7–T9.

**Rust twin.** JS-only (`workspace_change.rs`), together with T4 and T7.

### T13 — the sentence's full stop was part of the command

**Root cause.** `prefixedShellCommand` (`js/agentic/shell_command.mjs`,
`prefixed_shell_command` in Rust) took everything after the passthrough
prefix (`run`, `execute`, …) verbatim, so the sentence's final `.` and any
backticks around the command became shell text.

**Fix (both roots).** `trimCommandSentenceEnd` peels the outer quotes and the
last token's sentence dot to a fixpoint, reusing `trimTrailingSentenceDot` —
the path-token rule that keeps `.` and `..` intact — so `cd ..` and `ls .`
are unchanged.

### T14 — "directory" routed to a listing

**Root cause.** The named-capability router matched the word "directory" to
`list_dir`. It already declined a read-many route for a request naming a
*mutating* shell intent, but (a) the guard covered read-many only and (b) the
`mkdir` intent declared no effect, so it did not count as mutating — effects
could only be written for two-operand intents (`cp`, `mv`).

**Fix.** The guard covers every observing capability (`read_many`,
`list_dir`, `glob`, `grep`) in both roots. Effect expansion accepts one
operand as `{path}` in both roots (`expand` / `expand_with`), and
`data/seed/shell-intents.lino` declares the `mkdir` effect (`test ! -e
{path}` before, `test -d {path}` after), so the request runs as a verified
recipe. `rust/tests/unit/issue_680_intent_routing.rs` now expects the
recipe's first step (`test ! -e build`) for "Create a directory called
build".

### T15 — commits: unrecognised phrasing, ignored message, unconditional push

**Root cause.** The `git_commit_request` surfaces had no "commit all"; the
commit subject was always generated; and the plain-commit templates
(`data/meta/work-item-steps.lino`) pushed to `origin` unconditionally, so a
local repository's commit was reported through a push failure.

**Fix.** New surfaces ("commit all", "commit everything", "commit my
changes"); a seeded concept `message` (role `git_commit_message_lead`, five
languages) whose first following quoted segment becomes the subject
(`statedSubject` / `stated_subject`, both roots); the two plain-commit
templates push only `if test -n "$(git remote)"`. The work-item recipe commit
is unchanged.

**Tests.** "shell requests run what they name": T13's command and answer,
T14's recipe and answer, T15's exact command.

**Whole JS suite.** 1020 pass, 0 fail.

### T16 — a filler word inside a cue phrase broke the cue

**Root cause.** Shell-intent cues match as literal substrings of a requesting
sentence, so `show me git status` never contained the seeded cue `show git
status`, and the request fell to web search.

**Fix (both roots).** A seeded `cue_fillers` list in
`data/seed/shell-intents.lino` (en/ru/hi/zh/es: `me`, `the`, `please`, `мне`,
`मुझे`, …) and `sentenceCarriesCue` / `sentence_carries_cue`: a cue matches
the sentence as written or once both drop the fillers. The literal match is
tried first, so every existing match is unchanged.

### T17 — "the value of KEY" was read as literal text

**Root cause.** The edit composer split the request into old `the value of
"debug"` and new `true`; nothing knew that "the value of K" names the
assignment of K in a configuration file.

**Fix.** A seeded concept `setting` (role `config_value_lead`: "value of",
"setting", "значение", "मान", "值", "valor de") and a third computed change
in the workspace-change arm (`groundedSetting`): the key is the old clause's
quoted segment (or its last identifier), the value the new clause; the one
line assigning the key (`"k": v`, `k: v`, `k = v`) gets the value, in the
file's own quoting (bare for booleans, null and numbers; JSON strings
quoted). "set" / "установи" join the seeded edit action cues. Answers come
from seeded responses keyed by `setting`.

**Tests.** "a filler word inside a cue phrase keeps the cue" and "a setting
changes on the line that assigns it".

**Suite note.** A full `rust/tests/web/` run at this point showed 60
failures, all `handler_rules:unknown_condition`, plus a missing
`js/agentic/crate/summarization_identifier.mjs` import: both from another
agent's uncommitted edits in the same tree (`data/seed/handler-rules.lino`,
`js/agentic/crate/summarization.mjs`), not from these changes. The agentic
suites (`agentic-*`, the dogfood file) are green.

**Rust twin.** T16 is in both roots; T17 is JS-only (`workspace_change.rs`).
