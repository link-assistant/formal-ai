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
| T1 | `Add a function multiply(a, b) to math.mjs that returns a times b, add a test for it to math.test.mjs, and run node --test to confirm it passes.` | **Fail, destructive**: `math.mjs` was overwritten with the text `a function multiply(a, b)`; a plan file was written under `.formal-ai/`; the answer said the change was complete. | **No longer destructive**: reads both files and reports them; the function and test are still not authored (open). |
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
| T18 | `Create hello.py that prints Hello, World! and run it.` | **Fail**: answered with a program in chat (named `main.py`, printing `Hello, world!`), wrote nothing, ran nothing. | Open (unquoted output, see below) |
| T18q | `Create hello.py that prints "Hello, World!" and run it.` | **Fail**: same chat answer. | **Pass in-process**: writes `hello.py` + `tests/verify-output.sh`, runs `python3 -m py_compile hello.py` and the output check (`Hello, World!`), reports. Through the Agent CLI the files are written and the compile step runs, then **the CLI crashes** (see "Client defect"). **After the bytecode-free check: passes end-to-end through the CLI** (rc=0, no `__pycache__`). |
| T19 | `Write a Python function add(a, b) that returns their sum in add.py and run it with 2 and 3.` | **Fail**: general-change `literal_file` plan, `add.py` = `2 and 3.` | **No longer destructive**: answers with `def add(a, b): return a + b`; writing it to `add.py` and running it with the stated arguments is open. |
| T20 | `Write a Python function add(a, b) that returns their sum.` (solver) | **Fail, wrong code**: `def add(a, b): return sum(a)`; `multiply(a, b) … a times b` gave `math.prod(b)`. | **Pass**: `return a + b` / `return a * b` (browser); native pinned at the IR. `add3(a, b, c)` still stops at `a + b` (open). |

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

**Rust twin.** Ported (see "Rust twin of the workspace-change fixes").

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

**Rust twin.** The quoting fix is in both roots; the rest is ported (see
"Rust twin of the workspace-change fixes").

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

**Rust twin.** Ported (see "Rust twin of the workspace-change fixes").

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

**Rust twin.** Both in both roots (T17 via "Rust twin of the workspace-change
fixes").

### T18 — a named source file did not name its language

**Root cause.** The program-contract route (`programContractAnswer`, which
builds a write → compile → run → compare-output recipe) recognised the
language only from words ("Python"), so `hello.py` named none; the request
fell to the shared solver, which answered in chat with the rediscovered
documentation example (`main.py`, `Hello, world!`) and no execution recipe.

**Fix (both roots).** `namedSourceLanguage` / `named_source_language` in
`js/agentic/crate/coding_program_contract.mjs` and
`rust/src/coding/program_contract.rs`: a line that asks for printed output
(`print_stdout`), carries a creation verb (`coding_request_verb`: write,
create, implement, …) and names exactly one source file whose extension is
the `save_as` extension of exactly one non-framework catalogued language
takes that language. The catalog is the source, so every catalogued language
(`.rs`, `.go`, `.kt`, `.R`, …) is covered. Without the creation verb
(`Change greet.py so it prints "Hi"`) nothing is claimed — that is an edit.

**Still open: the unquoted output.** `prints Hello, World! and run it`
names its output without quotes, and the clause splitter keeps the whole
sentence as one clause. Reading "the words after *prints* up to *and*" as a
literal would also read `prints the sum of a and b` as the literal `the sum
of a`, so the output boundary needs a grounded rule, not a guess.

**Client defect found (link-assistant Agent CLI 0.26).** Any binary file
appearing in the workspace during a session crashes the CLI with
`UnhandledRejection` — its session-diff summary validates `additions: NaN`.
Reproduced with `Run python3 -m py_compile greet.py` alone (rc=1), while
`Run python3 greet.py` passes (rc=0). Formal AI's Python recipe triggers it
because the catalog's check command `python3 -m py_compile` writes
`__pycache__/*.pyc`; a check that leaves no byte code (for example under
`PYTHONPYCACHEPREFIX`) would avoid it, but the check commands are the
documentation-sourced catalog rows of the #1165 work, so the change belongs
there.

**Tests.** "a named source file says which language to write": the T18q
plan's first call writes `hello.py` with the exact source; `notes.txt` and
`Change greet.py so it prints "Hi".` claim nothing.

### Rust twin of the workspace-change fixes (T4, T7, T8, T9, T17)

`rust/src/agentic_coding/workspace_change.rs` now mirrors the JS arm:
`GroundedRewrite` carries `renaming`, `unique` and an optional stated
sentence; `rewritten_source`, `changed_lines_edit` and
`grounded_positional_insert` match their JS twins. The computed changes live
in a new sibling module, `rust/src/agentic_coding/workspace_computed_change.rs`
(end insertion, removal, setting, and `plan_computed_change_step`), so
`workspace_change.rs` stays well under the 1000-line limit. The code is
rustfmt-clean and was not compiled locally (no cargo on this workstation);
`changed_lines_edit` scans bytes only for `\n`, so it never slices inside a
multi-byte character.

### T1 — a description of code was written as the file's bytes

**Root cause.** The general-change fallback (`composeGeneralChangePlan`)
reads "verb + object + to FILE" as a literal write: `Add a function
multiply(a, b) to math.mjs` gave the content `a function multiply(a, b)`,
written over the whole file.

**Fix (both roots).** `describesCodeToAuthor` / `describes_code_to_author`:
when the request has no seeded content lead (`containing`, `with exactly this
content:` …) and the content is not inside a quoted segment, content that
names a code construct (seeded `coding_request_object`: function, method,
program, example, …) is a description of code to author, and the literal
plan declines. `Create a file a.txt containing hello` and `Write 'function
f() {}' to f.js` still write.

**Still open.** Authoring the function itself: composing `multiply` from
"returns a times b", adding it to an existing ES module, adding a matching
`node:test` case, and running `node --test`. That is the next rung.

**Tests.** "a description of code is never written as a file's bytes".
The worker's mirror of the plan mode (`obligationPlanMode`,
`js/worker/formal_ai_worker_obligation_plans.js`) applies the same rule to
the first bound content, so `rust/tests/web/issue-1166-worker-obligation-gaps.test.mjs`
keeps the worker and the root in agreement (`Add a function to src/lib.rs,
then run the tests, …` now composes no literal plan in either); that module's
line budget was re-baselined 486 → 498 with the reason.

## Final suite state

`node --test --test-concurrency=1 rust/tests/web/`: 1200 pass, 2 fail before
the worker mirror above; after it, the 1166 case passes. The remaining
failure, `issue-1180-rust-ast-census.test.mjs` ("only 78 current full-AST
census documents", needs ≥ 80), is the expected transient of editing Rust
sources: the census documents under `data/meta/self-ast/` are generated by
CI's census sync and are not edited by hand; this branch's and the
concurrent agents' Rust edits both count toward it.

## Where the ladder stands

**Handled end-to-end through the Agent CLI** (each confirmed against
`node js/server/main.mjs --agent-mode`): create a file with content (T2),
read a line (T3), append/prepend a line (T4), rename an identifier (T5),
replace a word inside a file that also contains a longer word (T7), insert a
line after/before an anchor (T8), delete a line (T9), list / count lines /
delete / rename files (T10–T12), run a script (T13), make a directory as a
verified recipe (T14), commit with a stated message (T15), `git status` in
natural phrasing (T16), set a JSON/YAML setting (T17). T18q (write `hello.py`
that prints a quoted string, compile, run, compare) passes in-process; through
the CLI it writes and compiles, then the CLI itself crashes on the byte-code
file (client defect above).

**Smallest task it still fails** (T20): `Write a Python function add(a, b)
that returns their sum.` The browser program-IR composer
(`browserComposeProgramIr`, `js/worker/formal_ai_worker_program_ir.js`)
discovers the `reduce_sum` part and binds its list argument to the first
untyped parameter, giving `return sum(a)`; with two parameters and "their",
the reduced collection is the parameter tuple (`sum((a, b))`, or `a + b`). The
native composer has the same search and a cross-runtime parity fixture
(`data/parity/cross-runtime-synthesis.json`), so the fix has to land in both
roots together and be compiled — it was not attempted without a compiler.
Above it on the ladder: T19 (the same function written to `add.py` and run),
T1 (author `multiply` in an existing ES module plus a `node:test` case),
T18 with an unquoted output, and T6 (fix a typo whose correction the request
does not state — needs a grounded word source; the local Wiktionary/WordNet
caches hold only 254/713 lemmas and no `small`).

### The Python check no longer writes bytecode into the workspace

The catalog's Python check was `python3 -m py_compile main.py`, which writes
`__pycache__/main.cpython-*.pyc` next to the source. A binary file in the
workspace is what crashes the Agent CLI 0.26.0 (and drops the session summary
in 0.26.11; upstream draft
`docs/case-studies/pull-request-1188/upstream-issue-drafts/06-agent.md`).
`PYTHONDONTWRITEBYTECODE` does not help (py_compile writes the cache file
explicitly), and a `python3 -c "…compile(…)…"` check needs nested quotes the
seed strings cannot carry cleanly. The check is now
`python3 -X pycache_prefix=/tmp/formal-ai-pycache -m py_compile main.py`: the
same full compile (a syntax error and `return` outside a function both still
exit 1), no shell syntax, and the byte code lands under the prefix instead of
the workspace. Changed in all three copies of the catalog
(`rust/src/coding/catalog/languages.rs`, `data/meta/agentic-coding-catalog.lino`,
`js/worker/formal_ai_worker_12.js`) and in every catalog-derived pin
(`issue_716`, `issue_908`, `issue_1165_documentation_route`, `formal_ai.rs`,
`specification/{prompt_variations,selection}.rs`, the catalog source test, the
issue-1168 workflow fixture, the 1165 web test, the dogfood test). Transcripts
that record what a client ran (issue #908/#916 envelopes) keep the historical
command: the envelope's `Command:` field is never compared.

### T20 — a reduction over two scalar parameters read only one of them

Three causes, all general:

1. **The browser never loaded the composition fragments.** The seed registry
   listed `coding-composition-fragments.lino` as *unregistered*, with a reason
   claiming the web worker fed it "through its own seed list" — no such list
   existed, so `browserFragmentCatalog` silently got an empty file and the
   browser search ran without every binary fragment the native search has
   (`integer_add`, `integer_multiply`, …). It is now a `web true` seed
   (`data/meta/seed-registry.lino`, `js/seed-files.js` regenerated by
   `scripts/generate-seed-registry.rs --write`).
2. **No fragment realized a reduction of two values.** `integer_add` /
   `integer_multiply` now declare `supports reduce_sum` /
   `supports reduce_product` (`data/seed/coding-composition-fragments.lino`,
   read by both runtimes). The existing rankers — native orders by
   parameters referenced before action cost, the browser by parameters left
   unused — then prefer `a + b` over `sum(a)`; a single sequence parameter
   still gets `sum(items)` (cheaper, equally complete).
3. **The browser's "parameters left unused" rank tested substrings.**
   `source.includes("a")` is true for `__import__('math').prod(b)`, so the
   product read "both" parameters. It now counts whole identifiers
   (`browserIdentifierCount`), as native `collect_parameter_names` counts
   parameter nodes — a parity fix. Line budget 369 → 372 with the reason.

**Tests.** JS: exact answers for `add` and `multiply`
(`pull-request-1188-dogfood.test.mjs`). Native:
`rust/tests/unit/coding_discovery/composition_search.rs` pins the first
candidate's IR (`integer_add(a, b)`, `integer_multiply(a, b)`), its lowered
`return a + b` / `return a * b`, and that `total(items)` stays `reduce_sum`
(uncompiled here; CI compiles). `data/parity/cross-runtime-synthesis.json` has
no two-parameter reduction case, so it needed no change.

**Still open.** Three parameters (`add3(a, b, c)`): the browser's bounded
pool is crowded by placeholder chains (`left + right + left …`, high
"coherence" because they reuse the fragment's own slot names, never closed),
so `a + b + c` (depth 3) is evicted before ranking.

### T19 — the prose around the content asked for code

**Root cause.** `run it with 2 and 3` contains the content lead `with`, so the
literal plan took `2 and 3.` as `add.py`'s bytes; the content itself names no
code construct, so the T1 rule did not fire.

**Fix (both roots + worker mirror).** `asksToAuthorCode(proseAround(request,
content))`: the request with the content and its file-path tokens removed
names a code construct (`coding_request_object`) and an authoring verb
(`coding_request_verb`) — then the content is never the file's bytes, lead or
no lead. Quoted content stays exempt, and a path such as
`learned-program-rules.lino` is not prose (it is dropped before the check).
Worker plan-reader budget re-baselined to 505 with the reason.

### Issue #1028 ladder: 15 of 32 leaves `missing_proof` (run 37663996930)

**Reported diagnosis (not confirmed).** "The Rust planner stops after the
write while the JS planner observes and finishes." The JS planner does finish
in-process, and so does the Rust one — `rust/tests/unit/issue_1138_agent_cli_ladder_recall.rs`
already pins that exact read → write `""` → observe shape with continuation
pings.

**Actual root cause (from the run artifact).** The third request after the
write is the Agent CLI's *compaction* call (summarizer system prompt, no
tools). Its log says why: `contextLimit 60000`, `currentTokens 66482`
(input 39,691 + output 26,791 — the whole-file `write` counts as output),
`safeLimit 38856`, `overflow: true`. The CLI ships a built-in `formal-ai`
model with a 60,000-token context, and Formal AI counts one token per
character, while the server advertises an effectively unlimited window
(`/v1/models` `context_window_tokens` ≈ 1.5e10) that the ladder's config never
passed. After compaction the session holds only the summary envelope and
"Continue if you have next steps"; Rust recovers the task, re-reads, rewrites
the whole file, crosses the threshold again — nine times, no proof. The JS
server through the real CLI hits the same compaction (and then handles the
continuation worse; see the open gap below).

**Fixes.**
- `experiments/issue_1028_agent_cli_ladder/run.sh` reads the served
  `context_window_tokens` / `max_output_tokens` from the node's own server and
  passes them as the model's `limit` (fallback: the old config). Confirmed
  through the real CLI against the JS server: L01 with the limit runs read →
  write → `cat` → final, no `overflow: true`; without it, it compacts.
- Convergence after a compaction anyway (both roots, `structured_edit`):
  members the re-read file already lists are not written again; the run
  observes the file and answers "already lists …". The idempotence pins in
  `issue_1069_structural_edit.rs` and `agentic-write.test.mjs` change with it
  (an already-present value is observed, not rewritten).
- All 15 failing leaves' full prompts, driven through the JS planner
  in-process on the real target files, now change the file with its marker and
  write both the proof and the effect file.

**Tests.** Rust `after_a_compaction_the_already_edited_list_is_not_rewritten`
(post-compaction transcript: summary envelope + ping → read → never a write
of the tracked file); JS twin in `pull-request-1188-dogfood.test.mjs`
(read → `cat` → "`rust/src/web_search_core.rs` already lists "wikiquote";
nothing needed to change."). `data/meta/ladder-ratchet.lino` is untouched; the
CI run measures it.

**Open.**
- `formal-ai with agent` writes the same limit-less model block (no render
  placeholder carries the served window), so ordinary CLI users hit the same
  compaction on large files.
- The browser worker answers the compaction request with conversation
  statistics (`## Conversation summary`, `formal_ai_worker_05.js`), not the
  native `Conversation summary: … / User turns:` envelope that
  `compactedAgentTask` reads, so after a compaction the JS planner loses the
  task (it web-searched "What did we do so far?").
- Upstream: the CLI could take a provider's limits from its `/v1/models`
  instead of a built-in 60,000.
