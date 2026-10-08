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
| T1 | `Add a function multiply(a, b) to math.mjs that returns a times b, add a test for it to math.test.mjs, and run node --test to confirm it passes.` | **Fail, destructive**: `math.mjs` was overwritten with the text `a function multiply(a, b)`; a plan file was written under `.formal-ai/`; the answer said the change was complete. | **Pass end-to-end through the CLI** (round 4): read both → write `math.mjs` (keeps `add`, adds `export function multiply(a, b) { return a * b; }`) → write `math.test.mjs` (`{ add, multiply }`, `assert.equal(multiply(2, 3), 6)`) → `node --check math.mjs` → `node --test` (2 pass). |
| T2 | `Create a file a.txt containing hello` | Pass: `a.txt` = `hello`. | — |
| T3 | `Read the file math.mjs and tell me its first line.` | Pass: answered with the first line. | — |
| T4 | `Append the line 'third line' to notes.txt.` | **Fail, destructive**: `notes.txt` was overwritten with `the line 'third line'`. | **Pass**: read → edit → `sha256sum` check; one line added; answer `Appended \`third line\` to the end of \`notes.txt\` and observed the result.` |
| T5 | `In greet.py rename the variable nmae to name.` | Pass: both occurrences renamed. | — |
| T6 | `Fix the typo 'smal' in README.md.` | **Fail**: read the file, echoed it, changed nothing. | **Pass** (in-process): read → edit of the one line → `sha256sum`; `small`, discovered; answer `Replaced \`smal\` with \`small\` in \`README.md\` and observed the result.` |
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
| T18 | `Create hello.py that prints Hello, World! and run it.` | **Fail**: answered with a program in chat (named `main.py`, printing `Hello, world!`), wrote nothing, ran nothing. | **Pass in-process**: writes `hello.py` + `tests/verify-output.sh`, runs the bytecode-free check and the output comparison (`Hello, World!`). Also `Write a Python program hello.py that prints Hello, World! and run it.` (was `main.py`). `write a program that prints Hello, World! and run it` (no language, no file) is still a web search (open). |
| T18q | `Create hello.py that prints "Hello, World!" and run it.` | **Fail**: same chat answer. | **Pass in-process**: writes `hello.py` + `tests/verify-output.sh`, runs `python3 -m py_compile hello.py` and the output check (`Hello, World!`), reports. Through the Agent CLI the files are written and the compile step runs, then **the CLI crashes** (see "Client defect"). **After the bytecode-free check: passes end-to-end through the CLI** (rc=0, no `__pycache__`). |
| T19 | `Write a Python function add(a, b) that returns their sum in add.py and run it with 2 and 3.` | **Fail**: general-change `literal_file` plan, `add.py` = `2 and 3.` | **Pass end-to-end through the CLI**: write `add.py` → `py_compile` check → `python3 -B -c "from add import add; print(add(2, 3))"` → `5`; answer "Created and verified `add.py` …", no `__pycache__` in the workspace. |
| T20 | `Write a Python function add(a, b) that returns their sum.` (solver) | **Fail, wrong code**: `def add(a, b): return sum(a)`; `multiply(a, b) … a times b` gave `math.prod(b)`. | **Pass**: `return a + b` / `return a * b` (browser); native pinned at the IR. `add3(a, b, c)` still stops at `a + b` (open). |
| T21 | `Replace 'rust/src/solver_handlers/feature_capability.rs' with 'data/seed/feature-capabilities.lino' in check.mjs.` | **Fail**: no plan ("I didn't understand you"); with a filename-shaped literal (`'feature_capability.rs'`) the request was explained as a `sed` command instead. The edit reader took the quoted path as the file to edit, because the cue after it (`in`) reads as the target cue. | **Pass**: read → edit → `sha256sum`; a path the request leaves unquoted names the file before a quoted literal that is exactly a path (`composeEditRequest` / `compose_edit_request`); pinned by `rust/tests/web/pull-request-1188-quoted-path-payload.test.mjs` and `rust/tests/unit/pull_request_1188_quoted_path_payload.rs`. Formal AI then made both edits of the CI fix it was asked for. |
| T22 | `Insert the line <T21 row> after the line containing <the T20 cell> in ledger.md.` (this ledger's own T21 row) | **Fail**: first no plan -- the payload itself contains the word "before", so the position read both cues and gave up; then the row was spliced into the middle of the T20 row, since the anchor `\| T20 \|` was replaced as a bare substring. | **Pass**: read → edit → `sha256sum`; position cues are read only outside quoted literals (`composePositionalInsert` / `compose_positional_insert`), and a positional anchor widens to the whole line it sits on (`anchoredLines` / `anchored_lines`); pinned by `rust/tests/web/pull-request-1188-line-anchor.test.mjs` and `rust/tests/unit/pull_request_1188_line_anchor.rs`. Formal AI then wrote rows T21 and T22 of this ledger. |
| T23 | `Replace '<R382 status cell>' with '<new status: "... every handler in precedence order, ... every subcommand (also equal to the built formal-ai --help list), ...">' in docs/requirements/issue-0538-detailed-meanings-and-words.md.` (this PR's own R382 row) | **Fail**: read the file, then answered `computer_use_incomplete: verification failed for plan synthesized-computer_use_resource_orders-computer_use_list_directory` -- the single-quoted replacement named a precedence "order" and a built "list", and the computer-use route (tried before every edit arm) read them as a resource and an operation. | **Pass**: the row is replaced and observed. `instructionSurface` / `instruction_surface` drop every quote pair `textOutsideQuotedSegments` reads, not only double quotes; tests `rust/tests/web/pull-request-1188-quoted-payload-cues.test.mjs`, `rust/tests/unit/pull_request_1188_quoted_payload_cues.rs`. |
| T24 | `Set beta to 5 in cfg.toml.` (a probe on a sandbox copy, `alpha = 1` / `beta = 2` / `gamma = 3`) | **Fail, destructive**: the key was overwritten (`5 = 2`) by an edit sent without reading the file, and the answer only relayed the tool result. "Set" was an edit action and nothing else, so the unquoted key read as the text to replace. | **Pass**: read → edit → `sha256sum`, `beta = 5`; "set" (and its ru, hi, zh and es surfaces) now also names a setting in the seeded `config_value_lead` meaning, so the setting interpreter assigns the key; a key assigned nowhere is an honest verification failure with the file untouched. Pinned by `rust/tests/web/pull-request-1188-setting-verb.test.mjs` and `rust/tests/unit/pull_request_1188_setting_verb.rs`. |
| T25 | `Replace the heading '# Title' with '# Project' in README.md.` (a probe on a sandbox copy) | **Fail**: the edit was sent with the whole clause `the heading '# Title'` as its old text, which the file does not contain. | **Pass**: read → edit → `sha256sum`; a clause holding one quoted literal led only by the words that say what it is (`the heading`, `the word`) stands for the literal (`describedLiteral` / `described_literal`, shared by every edit composer). Pinned in `rust/tests/web/pull-request-1188-quoted-path-payload.test.mjs` and `rust/tests/unit/pull_request_1188_quoted_path_payload.rs`. |
| T26 | `Create a file new.txt containing 'hello'.` (a probe on a sandbox copy) | **Fail**: the file held `'hello'.` -- the sentence's closing period stayed on the payload, so the quotes were never recognised as one literal. | **Pass**: the file holds `hello`; a closing sentence mark after exactly one quoted literal belongs to the sentence (`cleanContent` / `clean_content`). Pinned in `rust/tests/web/pull-request-1188-quoted-path-payload.test.mjs` and `rust/tests/unit/pull_request_1188_quoted_path_payload.rs`. |
| T29 | `In meanings-web-navigation.lino, delete the line '        text "get …"' and the '      surface' line directly above it.` (issue #1175 p154, a sandbox copy) | **Fail, destructive**: one `write` replaced the whole 380-line file with the text `get …`, without reading it first. | **Fixed since T39**; it was open: the positional-edit planner (`js/agentic/positional_edit.mjs`, `write_request.mjs`) is another agent's claim in this batch; the move was made by hand. A request that names lines to delete must never become a whole-file write. |
| T30 | `Delete lines 266-267 from meanings-web-navigation.lino.` (a sandbox copy) | **Fail**: the file was read and nothing was edited; the answer was `The read command completed.` | **Fixed since T40**; it was open: a numeric line range is not yet a deletion target (same claimed planner). |
| T31 | `In meanings-calculator.lino, insert the two lines '      surface' and '        text find' after the line '        text solve'.` (issue #1175 p014) | **Fail**: the unknown fallback answered; no tool call was planned. | **Fixed since T52**; it was open: two single-line inserts after the same anchor, the second line first, did the edit (Formal AI made both). |
| T32 | `Replace the line '        text pay' with '        text "pay"' in meanings-statistics.lino.` (issue #1175 p350) | **Fail**: the replacement acted on the substring, so `text pays` became `text "pay"s` as well. | **Fixed since T51**; it was open: `Replace the line …` should match whole lines only; the edit was made by hand. |
| T33 | `Insert the line '…' after line N in seed-registry.lino.` (R379 proof-library registration) | **Fail**: a numeric line anchor is not an insertion target; the planner fell back without editing. | **Fixed since T41**; it was open (`js/agentic/positional_edit.mjs` is claimed by another agent): the same insert worded with a unique text anchor (`after the line '…'`) succeeded and was made by Formal AI. |
| T34 | `Insert '    web true' after the line '    bundle true' that follows '  seed coding-guidance' in seed-registry.lino.` (R379 coding-guidance registration) | **Fail**: a nested anchor ("the line X that follows Y") is not parsed; the edit was not planned. | **Fixed since T53**; it was open: the edit was reworded with a unique anchor and Formal AI made it. |
| T35 | `Delete the lines containing 'rust/src/proof_engine/presenter.rs' from hardcoded-language-allowlist.txt.` (R379 allowlist prune) | **Fail**: the path-shaped needle was read as a file to `cat`, so the planner read a missing file and deleted nothing. | **Fixed since T42**; it was open (`js/agentic/workspace_change.mjs` carries another agent's uncommitted edits): the allowlist rows were deleted by hand. |
| T36 | The third open edit shape below: a double-quoted line payload holding double quotes, `… after the line "    path '/v1/x/learn'" in r.lino.` | **Fail, unsafe**: the payload split apart, and the read fallback took `/v1/x/learn` -- a path inside the payload -- as the file, outside the workspace. The named file `r.lino.` was never a candidate, because its sentence period left it no extension. | **Pass (safe)**: the read goes to `r.lino`; a path the request leaves unquoted outranks one inside a quoted payload, and the sentence's period is peeled (`firstPath` / `first_path`). The nested-quote payload itself is still ambiguous and makes no edit; backtick-delimited lines work. Pinned in `rust/tests/web/pull-request-1188-quoted-path-payload.test.mjs` and `rust/tests/unit/pull_request_1188_quoted_path_payload.rs`. |
| T37 | `Replace '… works.' with '… works. The escape is fixed (T36): the read now goes to the named file.' in ledger.md.` (the note on this ledger's open section) | **Fail**: the line became `'… goes to the named` -- the new text was cut before "file" and kept its opening quote, because the walk back to the target clause ("in ledger.md") read the quoted word "file" as a target cue. | **Pass**: a cue word inside a quoted literal is payload for the target clause too (`composeEditRequest` / `compose_edit_request`); Formal AI then repaired the line it had damaged. Pinned in `rust/tests/web/pull-request-1188-quoted-path-payload.test.mjs` and `rust/tests/unit/pull_request_1188_quoted_path_payload.rs`. |
| T38 | The second open edit shape below: `In m.mjs, insert these lines after the line 'import b from "b";':` followed by an indented block of two import lines | **Fail**: "I didn't understand you" -- a positional insert needed exactly two quoted literals, and the block's own double quotes made more. | **Pass**: read → edit → `sha256sum`, both lines in place; a request whose first line ends in a colon is read from that line alone (cues, anchor, file), and the lines under it, their shared indentation removed, are the text inserted (`introducedBlock` / `introduced_block`). Pinned in `rust/tests/web/pull-request-1188-line-anchor.test.mjs` and `rust/tests/unit/pull_request_1188_line_anchor.rs`. |
| T39 | T29 reproduced on a sandbox copy: `In m.lino, delete the line '        text "get it"' and the '      surface' line directly above it.` | **Fail, destructive**: the table router chose `write_file` and wrote the quoted text `get it` over the whole file, without reading it. | **Pass**: read → edit → `sha256sum`, both lines gone; the quoted line and its named neighbour are one adjacent removal (seeded `line_adjacent_above` / `line_adjacent_below`; `groundedLineOperation` / `grounded_line_operation`), a routed `write_file` declines any request with a removal verb, and a computed change that would drop most of a file without the request stating that extent is refused (seeded `coding_workspace_fragment_refused`). Pinned by `rust/tests/web/pull-request-1188-line-operations.test.mjs` and `rust/tests/unit/pull_request_1188_line_operations.rs`. |
| T40 | T30 reproduced: `Delete lines 2-3 from f.txt.` (also `lines 2 to 3`, `line 4`, and `Удали строки с 2 по 3 из f.txt.`, `f.txt से पंक्तियाँ 2 से 3 हटाओ।`, `删除 f.txt 的第2到3行。`, `Elimina las líneas 2 a 3 de f.txt.`) | **Fail**: read the file and answered `The read command completed.`; nothing was edited. | **Pass**: read → edit → `sha256sum`, the numbered lines gone; a seeded `numbered_line_noun` with a number or a range (dash, or a seeded `line_range_connector`, after an optional `numbered_line_lead`; `第…行` reads the noun after the number) is a one-based range; a line past the end is an honest verification failure. Answer `Deleted line(s) 2-3 of \`f.txt\` and observed the result.` in the request's language. Same tests as T39. |
| T41 | T33 reproduced: `Insert the line 'X' after line 2 in f.txt.` (also `before line 1`, `after line 6`, `Вставь строку 'X' после строки 2 в f.txt.`) | **Fail**: "I didn't understand you"; no tool call. | **Pass**: read → edit → `sha256sum`; a numbered line is an insertion anchor beside the seeded after/before position (`numbered_line_insert_after` / `_before` sentences). Same tests as T39. |
| T42 | T35 reproduced: `Delete the lines containing 'rust/src/x.rs' from allow.txt.` | **Fail**: `cat 'rust/src/x.rs' 'allow.txt'`, nothing deleted -- two paths, so no removal arm took the request. | **Pass**: read → edit → `sha256sum`, both lines gone; a path the request leaves unquoted names the file and a quoted path is then the payload (`namedTargetAndPayloads` / `named_target_and_payloads`, shared by every computed change). Same tests as T39. |
| T43 | G4: `Move the line 'gamma = 3' to the top of cfg.toml.` (also `to the end of`, `after the line 'beta = 2'`, `Перемести строку … в начало cfg.toml.`) | **Fail**: the line was prepended and the original kept (the end-insertion arm read "to the top of"). | **Pass**: read → edit → `sha256sum`; the seeded `line_move_action` takes the line out and puts it back at the start, the end, or beside a quoted anchor, read before any rewrite or insertion arm. Same tests as T39. |
| T44 | G3: `Swap the lines 'alpha = 1' and 'beta = 2' in cfg.toml.` (also `交换 cfg.toml 中的 'alpha = 1' 和 'gamma = 3'。`) | **Fail**: read the file, changed nothing. | **Pass**: read → edit → `sha256sum`; the seeded `line_swap_action` exchanges the two whole lines, each named exactly or by a unique containing line. Same tests as T39. |
| T45 | `Delete the lines containing 'x' from f.txt.` on a ten-line file where nine lines contain `x` (a guard probe for T29) | **Fail, silent**: nine of ten lines would have been removed and reported as done. | **Pass (refused)**: `Refused to change \`f.txt\`: the computed result would drop most of the file, and the request does not ask for that.`, file untouched; `Delete lines 1-9 from f.txt.` states its extent and still runs, as does a named-function removal. Same tests as T39. |
| T50 | G2: `Change MAXIMUM_RATIO from 8 to 16 in p.rs.` (line `    const MAXIMUM_RATIO: u32 = 8;`, another `8` elsewhere) | **Fail**: the edit tool got `oldString` `MAXIMUM_RATIO from 8` and found nothing. | **Pass**: read → edit → `sha256sum`, only the declaration changes. A seeded old-value lead (`file_edit_old_lead`, `file_edit_old_lead_cue`: from, с/со, की जगह, 从, de) states the value the key holds, and the line assigning the key that value is the one changed; `assignment` reads declaration lines (`const K: T = v;`, `let K = v;`, `K := v`, a trailing `;`); a key assigned nowhere changes the stated value where it occurs once as a word. Russian too. `statedOldValue` / `stated_old_value`, `js/agentic/workspace_setting.mjs` / `rust/src/agentic_coding/workspace_setting.rs`. Pinned in `rust/tests/web/pull-request-1188-replace-semantics.test.mjs` and `rust/tests/unit/pull_request_1188_replace_semantics.rs`. |
| T51 | T32 reproduced: `Replace the line '        text pay' with '        text "pay"' in m.lino.` | **Fail**: a substring rewrite, so `text pays` became `text "pay"s`. | **Pass**: a request that names a line (the seeded `line` meaning outside quotes) replaces whole lines: lines equal to the quoted text, else lines equal to it but for indentation (kept), else its one occurrence; never text inside a longer line (`groundedLineReplacement` / `grounded_line_replacement`, `replacedLines` / `replaced_lines`). Same pins. |
| T52 | T31 reproduced: `In m.lino, insert the two lines '      surface' and '        text find' after the line '        text solve'.`, and the open shape `Insert … in m.lino, and insert … in m.lino.` (one file or two) | **Fail**: the unknown fallback answered; the two-clause request made no edit. | **Pass**: every quoted literal that is not the anchor (or its context) is a line inserted, in order, when only seeded joiners (`file_edit_joiner_cue`) and commas separate them; a repeated add action led by a joiner or a clause mark opens a new insert clause, and each clause is its own anchored edit over the file as the earlier ones left it, then each file's digest is checked (`positionalInserts`, `planInsertSequenceStep` / `plan_insert_sequence_step`). Formal AI then made the seed edits of this round with four- and five-clause inserts. Same pins. |
| T53 | T34 reproduced: `Insert '    web true' after the line '    bundle true' that follows '  seed coding-guidance' in r.lino.` | **Fail**: three literals; no plan. | **Pass**: a seeded anchor context (`file_edit_anchor_context_cue`: that follows, following, следует за, के बाद आने वाली, 之后的, que sigue a) names the line the anchor comes after; the edit carries the lines from the context through the anchor. Found while Formal AI added the `bump` surface: `after the line '        text set' that follows …` first matched `text setting`; after a context, a line that is the anchor now outranks one that only contains it. Same pins. |
| T54 | `Append these lines to meanings-file-edit.lino:` followed by indented seed lines (the seed edit of this round) | **Fail, destructive**: the block was not read as the payload; the evidence-record route searched the web for it and wrote its failure answer over the whole file. | **Pass**: lines under an append request are the text appended (`groundedEndInsertion` reads `introducedBlock`); a fenced block keeps its own indentation, so seed lines keep their structure (partly answers MIGRATE2's G16: indented blocks without a fence are still dedented, as T38 intended). Formal AI then appended this round's three seed meanings. Same pins. |
| T55 | Coordinator gap G23: `Bump the version in package.json to 1.1.0.` | **Fail**: read only. | **Pass**: `bump` (повысь, बढ़ाओ, 提升, incrementa) is a seeded edit action and setting lead, and an edit request whose file clause sits right before the new value (`… in package.json to 1.1.0`) keeps the words before the file clause as its old clause (`composeEditRequest` / `compose_edit_request`, one guarded hunk). The `version` line becomes `"1.1.0"`. Formal AI made both seed edits (a four-clause insert and a five-clause insert with an anchor context). Same pins. |
| T56 | T62 (MIGRATE2's G17) reproduced: `In l.lino, replace the line '    status pending' that follows the line '  handler b' with '    status migrated'.` | **Fail, destructive**: the general change plan wrote `status migrated` as the whole file. | **Pass**: the anchor context scopes a line replacement: the first matching line after the context is replaced, as one edit from the context's line through it (`anchorContext` / `anchor_context`, `contextLinesEdit` / `context_lines_edit`). Same pins. |
| T57 | Delegating this round's ledger rows T50-T56 as one fenced block (`In formal-ai-dogfood.md, insert these lines after the line containing '\| T45 \|':`) | **Fail**: no plan; the dialog-rule handler answered. A row held `when … then` around backticks, and the skill-description gate read the payload as a skill being taught. | **Pass**: the gate reads an edit request's own words -- the head, when its lines follow it (`ownText` / `own_text`, `js/agentic/planner.mjs` / `rust/src/agentic_coding/planner.rs`); Formal AI then inserted these rows. Same pins. |
| T60 | Calendar batch edits on sandbox copies: `Delete the function calendarOffsetWeekdayLabel from formal_ai_worker_calendar_offset.js.`, `Replace 'calendarOffsetWeekdayLabel(' with 'calendarWeekdayLabel(' in …`, four `Replace '…' with '…' in issue_699_handler_migration.rs.` count updates, three `Replace '    value N' with '    value M' in debt-ratchet.lino.` and the `worker_ceiling` drop | **Pass**: ten edits, each read → edit → `sha256sum`, byte-correct; the function went with its JSDoc. | **Pass** (no change needed). |
| T61 | `Insert the following lines after the line 'b' in f.lino:` / `Append the following lines to f.lino:` with an indented `.lino` table block (gap G16) | **Fail**: the block landed dedented by its common indent, so seed indentation (structure) was lost; the calendar tables were appended by hand. | **Fixed since T100** (G16). |
| T62 | `In l.lino, replace the line '    status pending' that follows the line '  handler b' with '    status migrated'.` (a ledger flip, gap G17) | **Fail, destructive**: the general change plan wrote `status migrated` as the whole file. The two ledger rows were flipped by hand. | **Open** (G17, the literal-write guard LEAD owns as G13). |
| T70 | DEBUG2: `Insert the line 'js/mermaid.bundle.js' after the line 'js/web-search-component.bundle.js' in gitignore.txt.` | **Fail, wrong file**: the turn routed to the agentic-recipe diagram recipe because the cue word `mermaid` sat inside the quoted payload, and wrote `agentic-recipes.md`; the ignore list was untouched. | **Pass (routing)**: a diagram cue counts only outside quoted literals (`isDiagramTask` / `is_diagram_task` over `quotedSegmentSpans` / `quoted_segment_spans`, js/agentic/diagram.mjs, rust/src/agentic_coding/diagram.rs). Pinned by rust/tests/web/agentic-recipes-a.test.mjs and rust/tests/unit/issue_538_agentic.rs. The insert itself still fails on the path-shaped literals (T71). |
| T71 | DEBUG2: `Insert the line 'foo' after the line 'js/ocr.bundle.js' in gitignore.txt.` (also `'js/x.js' after 'js/y.js'`) | **Fail**: `composeEditRequest` returns null when a quoted literal is shaped like a path, so the turn falls to a file read or the unknown answer; `'foo' after 'bar'` passes on the same file. | **Fixed since T103** (G21); it was open: the ignore line was added by hand. |
| T72 | DEBUG2: `Replace the line '  await gateTurn(ctx, answer.thinking_steps);' with these three lines in solve.mjs:` followed by the three lines | **Fail**: the line was replaced by the words `these three lines`; the block after the colon was not read as the new text. | **Fixed since T104** (G22); it was open: the same edit passed with the three lines inside the quotes: `Replace '…' with '<line 1><newline><line 2><newline><line 3>' in solve.mjs.` |
| T73 | DEBUG2 delegations that passed: the gate move in js/server/solve.mjs (`Replace '…' with '…'` and `Insert the line '…' before the line '…'`), three `Delete the line containing '…'` and one multi-line `Insert these lines after the line '…'` in rust/src/derivation.rs, two module lines in rust/src/server.rs, two CSS edits in js/styles/07-interactions.css, the vscode test-script list, both diagram regression assertions (the JavaScript one after re-quoting a request whose payload held escaped double quotes) and this ledger's rows T70-T73 | **Pass**: 18 edits, each checked by diff before copying back. | Tally for R383: Formal AI 18 edits, DEBUG2 by hand the new modules (debug-stage, debugger-client, mermaid-entry), tests, docs and the T70 fix. |
| T80 | `Add an assertion that add(2, 2) equals 4 to m.test.mjs.` (a probe on a sandbox copy) | **Fail, destructive**: the test file was replaced by the sentence "an assertion that add(2, 2) equals 4" -- the general change plan took the described addition as the file's whole new bytes. | **Pass (safe)**: when the write verb is the seeded add action and the content comes before the file, the request names an addition, never the file's whole content (`namesAnAddition` / `names_an_addition`); the file is left untouched. Authoring the assertion itself is open. Pinned by `rust/tests/web/pull-request-1188-addition-guard.test.mjs` and `rust/tests/unit/pull_request_1188_addition_guard.rs`. |
| T81 | LEAD: `Insert the line …` adding the `local-gates.mjs` row to the subagent README | **Partial**: the edit was right, but the answer echoed the inserted line, which holds backticks, inside single backticks, so its Markdown broke (G30). | Fixed: a seed placeholder the template wraps in backticks becomes a CommonMark code span with a longer fence (`codeSpan` / `code_span` in code_task); tests: pull-request-1188-subagent-folder in both roots. |
| T82 | LEAD probe: `Add the line … - run tests … at the end of the section … ## Usage … in README.md.` (payload and heading quoted) | **Fail, unsafe**: ran `bun test`; the shell intent table matched its cue `run tests` inside the quoted payload (G32). | **Safe**: a request about text inside a file (`editsInsideAFile` / `edits_inside_a_file`) reads intent cues outside its quotes only, so it runs nothing, while a quoted command still runs. Pinned by `rust/tests/web/pull-request-1188-quoted-payload-command.test.mjs` and `rust/tests/unit/pull_request_1188_quoted_payload_command.rs`. The section insert itself is G35. |
| T83 | TEACH-E probe: `Append the line … a … then removed from f.md to g2.md.` | **Fail, unsafe**: planned `mv f.md g2.md`; the shell intent cue `move` matched inside the word `removed` (G66). | **Safe**: intent cues match whole words, CJK aside (`containsWordSequence` / `FactRecord::contains_word_sequence` in `sentenceCarriesCue` / `sentence_carries_cue`). Pinned in `rust/tests/web/pull-request-1188-quoted-payload-command.test.mjs` and its Rust twin. |
| T84 | CIFIX: `set the contents of note.txt to hello` (issue #745 Rust test) | **Fail**: read `note.txt` and kept it; the setting arm took `the contents` as a key named `contents`, and the write guard saw no consent (G68). | **Pass**: the contents of a file (the seeded `file_contents_source_cue`) are never a setting key, and `set the contents of` / `pon el contenido de` / `установи содержимое` consent to replacing the file (`file_overwrite_consent`). Pinned in `rust/tests/web/pull-request-1188-cifix.test.mjs` and its Rust twin. |
| T90 | G12: `Find all usages of add.`, `Where is add used in this project?`, `Search for add in the files of this directory.`, `Grep for add.`, `Найди все использования add.` (and the hi/zh/es forms), Agent CLI tool list and `read,write,edit,bash` | **Fail**: a web search for the sentence, a grep for the whole question, `list .`, or `find -iname all-usages-of-add`. | **Pass**: a seeded content-search vocabulary (`workspace_content_search_form` slot forms and `workspace_content_search_noise`, every registered language) reads the identifier or quoted literal; the new `workspace_search` arm (after `file_analysis`, ahead of the file-name locate arm and web search) greps it -- the grep tool as `\bname\b`, else the seeded `grep -rnH{w,F}` of shell-intents.lino -- and answers with the file:line hits or a seeded not-found naming pattern and scope. A request that renames, replaces or removes the uses is not a search. `search for TODO in the code` now greps with line numbers instead of `rg --fixed-strings` (agentic-server and issue_749 expectations updated). Tests: rust/tests/web/pull-request-1188-workspace-search.test.mjs, rust/tests/unit/pull_request_1188_workspace_search.rs. |
| T91 | G11: `Run the tests in m.test.mjs.` | **Fail**: ran `bun test`, the whole-suite command the server's own directory marker chose. | **Pass**: a named test file runs with the runtime its extension needs, from the seeded `test_file_runners` group of shell-intents.lino (`node --test m.test.mjs`, `python3 -m pytest test_m.py`, ...); `js/agentic/test_file_runner.mjs` and its Rust twin. |
| T92 | G10: `Add a function sub(a, b) that returns a - b to m.mjs.` | **Fail**: read the module and stopped: the browser composer discovers no structure in an expression body, and the returned value was read to the end of the clause (`a - b to m.mjs`). | **Pass**: the returned value is the longest expression right after the verb (`stated_value`, both roots), and the JS arm falls back to the native search -- the one seeded integer operation (`integer_subtract`) meeting the specification at every contract sample pair, lowered through `javascript_ir_function`. |
| T93 | G14: `Fix the bug in m.mjs: add should return the sum.` on a correct module | **Fail**: the unknown fallback answer. | **Pass**: a bug report with a stated expectation (seeded `coding_bug_fix_request` and `coding_expectation_cue`) reads the module, computes the expectation from the statement at the contract's samples, runs the function through the contract's seeded `probe`, and answers "No defect found" (or a confirmed defect, file unchanged) from seeded templates; `js/agentic/function_expectation.mjs` and its Rust twin. |
| T94 | TEACH-C: `Delete the line 'x' from formal-ai-dogfood.md.` (removing a probe line from a sandbox copy of this ledger) | **Fail, destructive**: 245 lines were removed -- every line containing the letter x, although a line equal to `x` existed (G60). | Open (TEACH-D owns line removal): a quoted line is the whole line when one equals it; containment only when the request says "containing". |
| T95 | TEACH-C: an append whose quoted line held a single-quoted word and a file name (`Append the line '- G31 … "Delete the line '\''x'\'' from f.md." …' to g2.md.`) | **Fail, destructive intent**: the rename recipe `mv f.md g2.md` was planned (it failed only because f.md did not exist): the inner quote closed the outer one and "from f.md … to g2.md" read as a move (G61). | Open; the same append passed with the line in «…
| T96 | Backstop behind G13, G17 and T29: `Replace all uses of add with plus in m.mjs.` (a probe on a sandbox copy) | **Fail, destructive**: the literal-file general change plan wrote its target without reading it, so the misread edit replaced `m.mjs` with a sentence. | **Pass (safe)**: unless the request's first write verb is a seeded whole-file write (`file_whole_write_action`: create, write, save, new file, ... in every registered language), the plan reads the target first; an existing non-empty file is replaced only with `file_overwrite_consent` wording, otherwise a seeded refusal names the file (`general_change_existing_file_kept`); a missing target is created as before. `js/agentic/literal_write_guard.mjs` and its Rust twin; the per-shape guards stay. |
| T97 | G36: `Search for 'foo' in this project.` | **Fail**: `rg -n 'foo this'` -- the scope phrase leaked into the query. | **Pass**: the content-search forms (T90) carry the scope phrases (in this project / the repository / the workspace, in every registered language), so the arm greps `foo` in `.`. |
| T98 | G29: `List the files in src.` | **Fail**: listed the workspace root (`list .`). | **Pass**: `listedDirectory` / `listed_directory` takes the operand after a seeded place preposition (`statement_place_preposition`, every registered language), so the list call and the `ls` fallback name `src`; a prose word or a scope word (this project, the workspace) still lists `.`. |
| T99 | G26: `Which functions does src/m.mjs export?`; G27: `Summarize README.md in one sentence.` | **Fail**: G26 read the file and dumped it; G27 answered `Read 1 file(s): README.md: # Demo`. | **Pass**: G26 reads the module and names its exported functions (a seeded `module_export_marker` followed by a seeded declaration keyword), or says none are exported; G27 reads the file and hands its text to the summarization handlers, answering with their summary. |
| T100 | G16 (T61): `Insert the following lines after the line 'b' in f.lino:` and `Append these lines to f.lino:` with an indented `.lino` block | **Fail**: the block landed dedented by its shared indentation, and `the following lines` read `following` as an anchor context, so the insert got no plan. | **Pass**: `introducedBlock` / `introduced_block` now says whether the block was fenced or quoted (`verbatim`) and which indentation it shared; an unfenced block is rebased on the anchor line's indentation once the file is read, so it lands as the anchor's siblings (`rebasedBlock` / `rebased_block`, `insertedText` / `inserted_text`); appended lines keep the indentation they were given when the file already has lines indented so (`appendedBlock` / `appended_block`); with a block, a context cue cannot name a second line. An anchor also found inside other lines names the line it is alone (`loneLineOccurrences` / `lone_line_occurrences`), and every positional insert is planned by the insert sequence. Pinned by `rust/tests/web/pull-request-1188-edit-composer-gaps.test.mjs` and `rust/tests/unit/pull_request_1188_edit_composer_gaps.rs`. |
| T101 | G19: `Remove the first line of f.txt.` (also `the last line`, `вторую строку`, `तीसरी पंक्ति`, `最后一行`, `la primera línea`, `before the first line`) | **Fail**: read the file and answered with its first line. | **Pass**: read → edit → `sha256sum`; seeded `line_ordinal` (first, second, third) and `line_ordinal_last` meanings in five languages, each `defined-by` the cardinal meaning whose digits give its number; an ordinal right before a seeded line noun names that line, counted from the end for `last` once the file is read (`ordinalLines` / `ordinal_lines`); the ordinal's words are blanked before the adjacency cues are read, since `最后一行` holds `后一行`. Same pins. |
| T102 | G20: `Elimina las líneas 2 a 3 de f.txt.`, `Intercambia las líneas 'a' y 'b' en f.txt.` | **Fail**: the edit was made but answered in English: no Spanish marker, so the script detector returned the fallback language. | **Pass**: `responseLanguage` / `response_language` keeps the script detector's answer unless it is the fallback language; then the language the request's unquoted words are seeded in alone decides (at least two such words, more than the fallback's, no tie; `lexicalLanguage` / `lexical_language`, `rust/src/agentic_coding/tool_result/response_language.rs`). `renderSeededChange` / `render_seeded_change` and the outcome sentence use it. Same pins. |
| T103 | G21 (T71): `Insert the line 'foo' after the line 'js/ocr.bundle.js' in paths.txt.` | **Fail**: the quoted path was taken for the file, so one literal was left and the turn fell to a read. | **Pass**: a path the clause leaves unquoted outranks a quoted literal that is exactly a path (`namedPaths` / `named_paths` in the positional insert). Same pins. |
| T104 | G22 (T72): `Replace the line 'const X = 2;' with these three lines in f.mjs:` followed by the lines | **Fail**: the line became the words `these three lines`. | **Pass**: `composeEditRequest` / `compose_edit_request` composes the request's first line, and a new clause that only describes lines (the seeded `line` meaning, unquoted; `describesLines` / `describes_lines`) stands for the block; the line replacement takes it rebased on the replaced line's indentation. Same pins. |
| T105 | G24: `Rename the file m.py to math_utils.py.` | **Fail**: on a75bf3772 the identifier rename rewrote `the` and ended in a verification failure; before that a bare `mv` answered `The command completed successfully without output.` | **Pass**: an unquoted new name that is a workspace path is no edit of the file's bytes (`composeEditRequest` declines it), and the named-capability run route defers a mutating shell intent to the shell arm's verified recipe: `test -e m.py`, `test ! -e math_utils.py`, the `mv`, `test -e math_utils.py`, `test ! -e m.py`, answered by the seeded `mutating_action_completed` sentence in the request's language. Same pins. |
| T106 | G28: `Rename the function mul to multiply in src/m.mjs.` (also with a JSDoc and a use, a blank line between functions, a `cumulative` function) | **Not reproduced** on this tree through the in-process driver: read → edit (or the word-scoped `perl` rewrite) → `sha256sum`, and the rename is stated. | **Pinned**: the plain case is asserted in both roots. Same pins. |
| T107 | G31: `Insert an empty line before the line '## Probe' in README.md.` (also `Add a blank line after the line 'text'`) | **Fail**: read the file and echoed it. | **Pass**: one quoted anchor beside the seeded `file_edit_blank_line` is an insert of an empty line, stated by the `file_edit_blank_line` sentence (`insertIntent` / `insert_intent`). Same pins. |
| T108 | TEACH-D delegations to Formal AI: two claim lines in claims.md, the 124-line ordinal seed appended to meanings-repository-workflow.lino as an unfenced indented block (the T100 append behaviour), these rows and the gap marks | **Pass**: every edit checked by diff before it was copied back. | Tally for this round: see the TEACH-D report. |
| T120 | MIGRATE3 delegations that passed (text_manipulation batch, issue #918): the claim line; ``Run `cat text-cues.lino >> handler-rules.lino` `` (the five `text_*` cue tables); the ledger flip `In handler-migration-ledger.lino, replace the line '    status pending' that follows the line '  handler text_manipulation' with '    status migrated'.`; both debt-ratchet ceilings (`Replace 'value 23' with 'value 22' …`, `value 501` → `value 487`) and the literal-predicates note; the three core-boundary values, the text_manipulation reason and a note (`Insert the line '  note "…"' after line 49 in core-boundary-ledger.lino.`); seven requirement-row count replacements (R344, R918-2, R914-6, R1085-2); five gap lines and their renumbering. | **Pass**: 25 edits by Formal AI, each checked against the intended diff before copying back. | Pass. By hand: the JS and Rust cue readers, both test pins, the ledger row's four field lines (T122) and one debt-ratchet note (T123). |
| T121 | MIGRATE3: `Append the contents of text-cues.lino to the end of handler-rules.lino.` | **Fail**: routed to read_many and ran `cat 'text-cues.lino' 'handler-rules.lino'` -- both files shown, nothing written; no append-file-to-file cue makes `namesMutatingShellIntent` decline the observing route. | **Open** (G50). Worked around through Formal AI with the explicit command ``Run `cat a >> b` ``. |
| T122 | MIGRATE3: `Insert the contents of rows.txt after the line '    status migrated' that follows the line '  handler text_manipulation' in handler-migration-ledger.lino.` | **Fail**: read both files and answered with the contents of rows.txt; nothing inserted. Another file as the inserted text is not an edit shape the composer reads. | **Open** (G51); inserted by hand. |
| T123 | MIGRATE3: `Replace '…pinned by … .test.mjs.' with '…test.mjs. Lowered from 23 to 22 …' in debt-ratchet.lino.` and `Replace '- G32 (MIGRATE3)' with '- G51 (MIGRATE3) (was G32)' in gaps.md.` | **Fail, intermittent**: one read, one edit, then "Verification failed … the observed bytes differ from the planned workspace effect"; the identical gaps.md request passed on the next run, and composeEditRequest parses all of them correctly. | **Open** (G52); retried, or done by hand. |
| T124 | MIGRATE3 delegations that passed (document_originality_check and software_project_followup batches, issue #918): two claim lines; four seed appends as ``Run `cat x.lino >> y.lino` `` (marker table and policy, price-claim heading responses, the follow-up table, policy and seventeen sentences, and the grounding entries that keep the closure gap unchanged); both ledger flips (`replace the line '    status pending' that follows the line '  handler …'`); five debt-ratchet ceilings and a note; nine core-boundary values and reasons; fourteen stale hardcoded-language rows removed (`Delete the line containing '…'`, `Delete line 627 from a.txt.`, `Delete lines 742 to 755 from a.txt.`); fourteen requirement-row count replacements; a gap line and its renumbering. | **Pass**: 47 edits by Formal AI, each diffed against the intent before copying back. | Pass. By hand: both handlers' JS and Rust readers, the test pins, the two ledger rows' field lines (G51), the ratchet notes and the core-boundary notes. |
| T125 | MIGRATE3: ``Run `cat orig-rules.lino >> handler-rules.lino && cat orig-responses.lino >> multilingual-responses-policy.lino` `` and `Insert these lines after the line '    status migrated' that follows the line '  handler document_originality_check' in handler-migration-ledger.lino:` followed by four indented lines | **Fail**: the compound command was not taken as a passthrough command (four reads, the contents answered); the colon-introduced block inserted nothing. | **Open** (G62; compare G22, G51). Worked around with one command per run and a hand insert. |
| T130 | ROUTE2 delegations that passed (R1173-3 routing probes): seed inserts with `Insert the lines '…', '…' before the line '  format_conversion_refusal' in meanings-code-task-templates.lino.` (and the same shape for the `json_target` cue block, the `function_call_verb` role, the `function_call` meaning, two response-intent meanings, the browser summary responses and the `tryLearnFromSource` binding), `Replace 'undefined_function' with '{function}' in multilingual-responses-synthesis.lino.`, `Insert the line 'mod issue_1173_route2_probes;' after the line 'mod issue_1173_local_requests_route;' in mod.rs.`, four worker-line-budget ceilings and both routing-probe ceilings (`Replace 'rust_misroute_ceiling 15' with 'rust_misroute_ceiling 5' in budget.lino.`), the two gap entries and their renumbering. | Pass (20 edits). | Kept; each sandbox file was diffed before it was copied back. |
| T131 | ROUTE2: `Insert the lines '  response …', '    text "…\n\n{json}…"' before the line '  response response_format_conversion_refusal_en' in multilingual-responses-code-tasks.lino.` | **Fail**: every `\n` escape inside the quoted line became a real newline (`unescapeProseNewlines` runs on each literal), so the lino string broke across lines. | **Open** (G53, in TEACH-D's positional_edit.mjs); inserted by hand. |
| T132 | ROUTE2: `Append the lines '  response r_en', …, '    text "I compose … {product}. …"' to multilingual-responses-product-search.lino.` | **Fail, destructive**: the plural append plans nothing, and a double-quoted span inside one literal was read as the whole file content, so the plan was a `write` that replaced the 28-line file with one sentence (caught in the sandbox). | **Open** (G54, write_request / workspace_change, TEACH-D files); appended by hand. |
| T133 | ROUTE2: a gap line whose text itself held nested quotes (`Append the line «- G32 … Insert the lines 'a', '    text "x\ny"' …» to …gaps.md.`) | **Fail**: the nested quotation marks inside the guillemet literal were read as separate literals and the turn ended on an unrelated plan. | Rephrased without nested quotes and passed (counted in T130); recorded as part of G53. |
| T140 | DEBUG3: Replace a phrase in docs/vscode/debugger.md (one list item of the tests paragraph: config.test.mjs and becomes config.test.mjs,). | Pass: one edit, checked by diff and copied back. | Pass. |
| T141 | DEBUG3: append a claims line naming four source paths with a parenthetical about one begin_turn line placed early in the solver entry, to experiments/formal_ai_subagent/claims.md. | Fail: the capability route read the whole request, payload included, as read_many and ran cat on every path named in the payload; nothing was appended. A second try, appending the G64 gap line that quotes the same shape, failed the same way. | Open (G64, capability_router.mjs is TEACH-D's); both lines were appended by hand. |
| T150 | TEACH-E (G60): `Delete the line 'x' from f.md.` | **Fail, destructive**: every line containing the letter x went (245 lines of a ledger copy), though a line equal to `x` existed. | **Pass**: a quoted line is the whole line equal to it (indentation aside), else its one containing line, else nothing; every containing line only with the seeded `line_containment_cue` (`lines containing`, `that contain`, ru/hi/zh/es). `removedLineIndices` in `js/agentic/workspace_change.mjs`, twin `rust/src/agentic_coding/line_removal.rs`. Tests: `rust/tests/web/pull-request-1188-teach-e.test.mjs`, `rust/tests/unit/pull_request_1188_teach_e.rs`. |
| T151 | TEACH-E (G61): `Append the line '- G31 "Delete the line 'x' from f.md." removed lines' to g2.md.` | **Fail, destructive**: the inner quote closed the payload, `removed` carried the `move` cue and the move recipe ran `mv f.md g2.md` over the existing g2.md -- its `test ! -e g2.md` precondition passed because the dogfood driver's bash dropped exit codes. | **Pass**: a single quote with a space before it and a word character after it opens a nested literal, so the outer pair is the payload (`closingDelimiter`, `js/agentic/crate/normal_markov.mjs`, twin `rust/src/normal_markov.rs`); the driver reports a failing command's `Exit Code`, so a failed precondition blocks a recipe. Same tests. |
| T152 | TEACH-E (G53): `Insert the lines '  text "a\nb"' and '  row z' after the line '  row a' in f.lino.` | **Fail**: the `\n` inside one literal became a line break. | **Pass**: lines given as separate literals are already lines; an escape inside one is content (`clauseInsert`, `js/agentic/positional_edit.mjs`, twin `clause_insert`). Same tests. |
| T153 | TEACH-E (G54): `Append the lines '  row b', '  row c "q" d' to f.lino.` | **Fail**: no plan; the fallback answered with the two literals glued together. | **Pass**: several quoted literals joined only by commas and seeded joiners are the appended lines, as written (`joinedLiteralLines`, `quotedLinesAndPath`); never a whole-file write. Same tests. |
| T154 | TEACH-E (G50): `Append the contents of a.txt to the end of b.txt.` | **Fail**: read_many ran `cat a.txt b.txt` and wrote nothing. | **Pass**: the seeded `file_contents_source_cue` beside a path names a source file; it is read and the request is restated with its lines as a fenced block for the additive composers (`js/agentic/contents_source.mjs`, twin `rust/src/agentic_coding/contents_source.rs`). Same tests. |
| T155 | TEACH-E (G51): `Insert the contents of rows.txt after the line 'Y' that follows the line 'Z' in f.lino.` and `Insert these lines after the line '  Y' that follows the line 'Z' in f.lino:` with a block | **Fail**: both files were read and shown; nothing was inserted. | **Pass**: a block under an insert may name the line its anchor follows (two literals, a seeded anchor context); the contents source feeds the same arm. Unquoted anchors (`after the line Y`) still plan nothing. Same tests. |
| T156 | TEACH-E (G33): `Show the last 2 lines of n.txt.`, `the first 2 lines`, `lines 2-3`, `the last line` | **Fail**: the whole file was printed. | **Pass**: a `line_slice` read mode (`lineSlice` in `js/agentic/workspace_line_operation.mjs`: numbered lines, seeded ordinals, or a count between the seeded `line_slice_head_cue`/`line_slice_tail_cue` and a line noun) answers `Lines A-B of path` with only those lines. Same tests. |
| T157 | TEACH-E (G35): `Insert the line 'See also docs.' at the end of the section '## Usage' in README.md.` | **Fail**: read only, no edit. | **Pass**: the seeded `file_section_noun` with a quoted Markdown heading scopes an end/start insertion to the section (heading to the line before the next heading of the same or a higher level; fenced code is no heading), stated as inserted after the line it now follows (`js/agentic/markdown_section.mjs`, twin `markdown_section.rs`). Same tests. |
| T158 | TEACH-E (G37): `Rename the file 'a.txt' to 'b.txt'.` | **Fail**: read a.txt and edited the word `the` into `b.txt`. | **Pass**: a quoted new path renames the file when the old clause quotes nothing (`composeEditRequest`), and a quoted operand before the full stop is a path argument (`collectPathArguments`); the verified `mv` recipe runs. A rename inside a file (`'./a.mjs'` to `'./b.mjs'` in m.mjs) still edits it. Same tests. |
| T159 | TEACH-E (G52): a Replace inside the 7000-character line of debt-ratchet.lino | **Fail, intermittent**: "Verification failed" after one read. | **Not reproduced** on this tree (three runs on a copy of the HEAD file, and a pinned 7000-character case in both roots). The causes found for "read, then Verification failed" are fixed: an escape in the needle (T161), a nested quote (T151); the gaps.md case likely saw a live file another agent edited between copies. |
| T160 | TEACH-E (G62): ``Run `cat a.lino >> b.lino && cat c.lino >> d.lino` `` | **Fail**: read the four files and showed them. | **Pass**: a passthrough command that chains, pipes or redirects is no file read (`fileReadTaskFor`, twin `file_read_task_for`); it runs as written. Same tests. |
| T161 | TEACH-E (G65, found delegating T152's own edit): ``Replace «… .join('\n'),» with «…» in positional_edit.mjs.`` | **Fail**: the request's `\n` was unescaped into a line break, the needle was found nowhere, and the answer was "Verification failed". | **Pass**: when the file holds no unescaped needle but holds it as written, the literals are the text as written (`asWritten`, twin `as_written`). Same tests. |
| T162 | TEACH-E delegations that passed: three claim lines, the seed meaning `line_containment`, the G53 line in positional_edit.mjs, the G37 line in shell_command.mjs, these ledger rows (inserted with T154's own `Insert the contents of …`), and the FIXED marks in gaps.md. | Pass. | Pass. |
| T170 | REPO-PROTO delegations that passed (R1138-3-5, one protocol document for all three repository callers): the claim line; the nine-stage protocol document built in a sandbox copy of `repository-workspace-protocol.lino` by single-line inserts (`Insert the line '  editor "any"' after the line '  id "clone"' …`, six times), three order renumbers (`Replace '  order "6"' with '  order "8"' …`), three nine-line step blocks (`Insert the following lines before the line 'repository_step_edit' …:` with the block on the following lines) and four detail-sentence extensions; five Rust inserts in `rust/src/repository_workspace/mod.rs` (`pub mod trace;`, the `editor` field, its parse and regenerate lines); the `mod protocol_callers;` line; two spec-test replaces; the G63 gap line (a guillemet-delimited append) | **Pass**: 27 edits, each observed; the regenerated document reproduces the committed one byte for byte. | Pass, no fix needed. |
| T171 | REPO-PROTO: the R1138-3-5 row rewrite as one ``Replace `<old cell>` with `<new cell>` in req.md.`` whose payloads hold sentence breaks and several backtick spans | **Fail**: the replacement was cut at `… each with an` (before `` `editor`: `any` ``) and the answer said `The command completed successfully without output.`; minimal form in gaps.md G63. | **Open** (G63): `composeEditRequest` lives in TEACH-E's claimed `write_request.mjs` / `normal_markov.mjs`, so it is left to that owner; the row was written by hand. |
| T180 | CIFIX (G67, found driving the retargeted ladder leaf L10 through the JS agent): the L10 node prompt, add "code_formatting" to the OTHER_INTENTS list of rust/src/solver_handlers/shell_command_compose.rs, whose result= clause quotes the member again | **Fail**: the member was inserted twice. | **Pass**: memberInsertion keeps one copy of a repeated quoted value in both roots; pinned by rust/tests/web/pull-request-1188-cifix.test.mjs and rust/tests/unit/pull_request_1188_cifix.rs. |
| T181 | CIFIX delegations that passed: the repeated-member guard in structured_edit.mjs and structured_edit.rs (two Replace), the coding-guidance registry language list, the duplicate document_format meaning (Delete lines 658 to 665), the expanded tool_result_failure surface (a two-line insert), gaps G67 to G70 and these two rows. Both Replace edits were right but answered The command completed successfully without output. instead of naming the replacement. | Pass. | Pass. |
| T190 | TRANSLATE delegations that passed: the filed-issue line appended to upstream-issue-drafts/README.md, the paragraph inserted after `## Upstream drafts` in js-rust-translation.md (both payloads hold links, `#` and `;`), and this row | **Pass**, each on the first run. | No fix needed. An insertion after a Markdown heading lands with no blank line before the paragraph, which is literally what "after the line" asks for; the blank line was added by hand. |
| T210 | TEACH-F (G71, unsafe): an Insert of a ledger row whose quoted payload holds backslash-escaped quotes and the words set the contents of note.txt to hello | **Fail, unsafe**: the escaped quote split the payload, so the words inside it were read as the instruction and note.txt was read (in LEAD's run, written at the workspace root). | **Pass (safe)**: a request whose quotes do not pair is declined before any arm reads it, with a seeded answer that names the escaped or unclosed quote (quoteFault / quote_fault in normal_markov, responses request_quote_escaped and request_quote_unpaired in five languages); the same row quoted with guillemets is inserted. Pinned by rust/tests/web/pull-request-1188-teach-f.test.mjs and rust/tests/unit/pull_request_1188_teach_f.rs. |
| T211 | TEACH-F (G70, parity): contractions in a request (I'm confused, you can't perform that action) and 23 seed surfaces written contracted (i m confused, what s your name, you can t perform …) | **Fail**: the JS agentic normalizer kept i m while the Rust one expanded it to i am, so a contracted surface matched only in JS and an expanded one only in Rust. | **Pass**: the seed's contraction and expansion records decide for the expanded form; normalizePrompt in js/agentic/crate/engine.mjs expands them as rust/src/engine.rs does, and the contracted surfaces were dropped for their expanded twins (two rewritten). Pinned by rust/tests/web/pull-request-1188-teach-f.test.mjs and rust/tests/unit/pull_request_1188_teach_f.rs (a provider denial written with can't, no seed surface written contracted). |
| T212 | TEACH-F (G69): add hello to config.txt, append hello to config.txt, add hello to the end of config.txt | **Fail**: no plan in either root; the unknown answer. | **Pass**: a request that leads with the seeded add verb and quotes no text gets a seeded question naming the file and what is missing (unquotedAdditionPath / unquoted_addition_path, response file_addition_unquoted), never a write; Create a test for add in t.mjs is not read as an addition. Pinned by the teach-f tests in both roots. |
| T213 | TEACH-F (G63): a backtick-delimited Replace whose replacement holds a sentence break and later backtick spans, and one whose replacement ends in a backtick span | **Fail**: the replacement was cut at the first literal the inner spans formed, the answer was The command completed successfully without output., and a trailing span was taken as the target file. | **Pass**: a replacement quoted whole up to the file clause runs to it (wrappedInQuotePair / wrapped_in_quote_pair), a cue between two paths introduces the later one, and groundedRewrite / grounded_rewrite accept a literal quoted whole (quotesWhole / quotes_whole), so the edit is verified with sha256sum. Pinned by the teach-f tests in both roots. |
| T214 | TEACH-F (G64): an append whose quoted line names a.rs and says at the start of x | **Fail**: cat a.rs c.md ran instead of the append. | **Pass**: the end-insertion arm reads its position cues outside the quoted payload (groundedEndInsertion / grounded_end_insertion), so the line is appended and verified; the capability router is no longer reached for this shape and is unchanged. Pinned by the teach-f tests in both roots. |
| T215 | TEACH-F (G15): a general change run from the repository root | **Fail**: the step said append the composed plan but wrote the whole .formal-ai/general-change-plan.lino, replacing the committed plan event. | **Pass**: the step says write the composed plan (data/meta/agentic-messages.lino and rust/src/agentic_coding/general_planner.rs), and the dogfood driver writes .formal-ai/ plan events under the git-ignored sandboxes/plan-events/ when --dir is the repository root; the tool sequence is unchanged. Pinned by the teach-f tests (JS: the driver and the step text; Rust: the step text). |
| T216 | TEACH-F (T181): a correct Replace planned by the intent edit arm, whose new text holds the old (run(); to guard(); run();) | **Fail**: answered The command completed successfully without output. | **Pass**: an edit tool's quiet reply is stated as the replacement (latestTurnQuietSuccess / latest_turn_quiet_success, response coding_text_replaced_unchecked, which says the edit tool reported no error rather than claiming an observation). Pinned by the teach-f tests in both roots. |
| T217 | TEACH-F (G72, MIGRATE4's T241): an insert after an anchor line found twice | **Fail**: one read, no edit, then Verification failed … the observed bytes differ. | **Pass**: the answer names the anchor and how often it occurs, and asks for more of the line or its number (repeatedAnchorCount / repeated_anchor_count, response file_edit_anchor_repeated). Pinned by the teach-f tests in both roots and the updated dogfood test. |
| T218 | TEACH-F (G25, open): Create a Python test for add in test_m.py and run it. | **Fail**: no file was written and a bare python ran (missing on macOS). | **Open**: the bare run comes from namedShellCommandInSentence / named_shell_command_in_sentence, whose loose branch takes the first shell token (here the language adjective Python) once the sentence has a run verb; authoring the test needs a stated expectation or a seeded shape and is not implemented. |
| T219 | TEACH-F delegations that passed: the planner import of quoteFault, rows T210 to T219, the gap marks, the issue_680 comment and the dogfood test's G72 answer | Pass. | Pass. |
| T220 | TEACH-F (G73, ROUTE3's T231): Insert the line '    2,' before the line '];' in t.rs, where the first line also ends in ]; | **Fail**: the edit's old text was the bare anchor line, found twice by the edit tool, and the answer was Verification failed. | **Pass**: the insert's old text widens back line by line until it occurs once in the file (uniqueFrom / unique_from in insertEdit / insert_edit), so the line found once as a whole line is edited once. Pinned by the teach-f tests in both roots. |
| T221 | TEACH-F (G74, ROUTE3-fork's T232): Insert the line 'pub mod cache;' followed by an empty line before the line 'use std::collections::BTreeMap;' in r.rs. | **Fail**: the empty line was dropped and the answer reported success. | **Pass**: a seeded blank-line phrase beside the inserted lines adds the empty line on the side its words stand (clauseInsert / clause_insert), and the answer states it with the seeded file_edit_blank_line sentence (blankBeside / blank_beside). Pinned by the teach-f tests in both roots. |
| T222 | TEACH-F (G25): Create a Python test for add in test_m.py and run it. | **Fail**: no file, and a bare python taken from the adjective Python ran. | **Pass (question)**: a request to write a test into a named file that states no expected result (no quoted literal, value or seeded expectation cue) gets the seeded question test_expectation_missing (planTestExpectationQuestion / test_expectation_question, in the workspace-change arm ahead of the shell arm); nothing is written or run. Writing the test from a stated expectation is not implemented, and the loose shell-token branch of namedShellCommandInSentence is unchanged. Pinned by the teach-f tests in both roots. |
| T223 | TEACH-F (G75, MIGRATE4's T244): a silent sed edit of allowlist.txt | **Fail**: answered This folder is empty., because the result label held list anywhere in the command. | **Pass**: a label with operands is a shell command, read by its own word through the seeded result_command_words group of data/seed/shell-intents.lino (ls, tree, dir for a listing; grep, rg, find, ag for a search); a one-word label is still a tool name (isListing / isSearch, Rust twin in rust/src/agentic_coding/tool_result/result_kind.rs). Pinned by the teach-f tests in both roots. |
| T224 | TEACH-F (G78, MIGRATE4's T246): In s.txt replace 'key 3' with 'key 33' | **Fail**: declined as a whole-file write (Left s.txt unchanged), since the new text holds the old one and a second rewrite pass found it again. | **Pass**: a substring rewrite whose new text holds the old text is applied in one pass and verified against that pass (singlePass / single_pass in groundedRewrite and rewrittenSource, both roots). Pinned by the teach-f tests in both roots. |
| T225 | TEACH-F (G79, DISCOVER's T261): In rows.mjs, in the row whose slug is rust, replace a run line found in every row | **Fail, silent**: every row was rewritten and the answer reported one replacement. | **Pass (decline)**: a clause a seeded target cue leads outside the edit's own clauses is the part the edit is scoped to (editScope / edit_scope in the new edit_scope module, both roots, read from composeEditClauses / compose_edit_clauses spans); where the old text occurs more than once, nothing is changed and the seeded file_edit_scope_unapplied answer counts it and names the part; found once, the edit is made. Pinned by the teach-f tests in both roots. |
| T226 | TEACH-F (G80, DISCOVER's T262): Replace the three lines check: some(, the kotlinc line and ), with the line check: none, in k.mjs | **Fail**: Error: oldString not found in content; the list words and quote marks were taken as the old text. | **Pass**: quoted literals led by words that name lines (the seeded line meaning) and joined only by commas and seeded joiners are one block of lines (listedLines / listed_lines in literalText); the block is found once but for indentation and replaced at its indentation (replacedLines / lineBlockStarts in workspace_setting, both roots). Pinned by the teach-f tests in both roots. |
| T227 | TEACH-F (G51, unquoted anchors): Insert the contents of rows.txt after the line row a that follows the line table u in f.lino. | **Fail**: both files were read and the answer was the contents of rows.txt; nothing was inserted. | **Pass**: with a contents block and no quoted literal, the words after the position cue up to the context cue name the anchor, and the words after the context cue up to the file name the context (unquotedAnchorInsert / unquoted_anchor_insert); each is resolved to the longest file line its words end with (resolvedSpans / resolved_spans), so the rows land after the second table's row a. Pinned by the teach-f tests in both roots. |
| T228 | TEACH-F (G13, assertion authoring): Add an assertion that add(2, 2) equals 4 to m.test.mjs. | **Fail**: since T80 the file was left untouched and the answer asked for quoted text; nothing was written. | **Pass**: the seeded coding_assertion concept and assertion equality cue (new data/seed/meanings-coding-assertions.lino) name the call and its value; the line is written in the form of the file's last assertion line (assert.equal, expect(…).toBe, assert … ==, self.assertEqual: assertion_styles in data/meta/function-test-contracts.lino) after it at its indentation, and the file is run with its seeded runner (planTestAssertionStep / plan_test_assertion_step in the new test_assertion module, both roots, in the workspace_change arm). Pinned by the teach-f tests in both roots. |
| T229 | TEACH-F (G25, stated expectation): Create a Python test for add in test_m.py that add(2, 3) returns 5, and run it. | **Fail**: the round-2 question asked for the expectation even when it was stated, and a shell word anywhere in a sentence with a run verb (Python) was taken as the command. | **Pass**: a missing test file is written whole from the language's seeded new_file form around the one assertion, importing the function from the module its seeded test_name pattern names (from m import add), then run; namedShellCommandInSentence / named_shell_command_in_sentence takes a shell word outside a run verb's object only when it is named as a command (written as code, or after a seeded command noun). Pinned by the teach-f tests in both roots. |
| T230 | ROUTE3 delegations that passed (issue #1173 R1173-3 fifth pass, issue #1175 p352/p355/p223-p225/p133): the task_print_text row, the tasks line and the operand_role line put in and taken out again in hello-world-programs.lino; the print_text task row of agentic-coding-catalog.lino; the Rust ProgramTask row of catalog/tasks.rs (anchored on the row before it); the ru `печатает` surface of print_stdout; the program_task_print_text and print_text meanings, the print_text role line and both es lexemes in meanings-coding-catalog.lino; the before_promotion field and its note in handler-precedence.lino; `pub mod operand_program;` in coding/mod.rs and the route3 module line in tests/unit/mod.rs; the misroute line deletions in probes-code.lino and both ceilings of budget.lino (three runs); three worker-line-budget ceilings and their rationales (six runs); these three rows (one `Run` of sed with an r command) and the G73/G74 gap lines -- 25 edits; ROUTE3-fork: its claims line and the recurrence test module line -- 2 edits. Done by hand: operand_program.rs, recurrence/cache.rs, the browser operand reader and tryWriteProgram arm, the tests, the dispatch_order and before_promotion_handlers functions, the discover_and_compose candidate, the R1173-3 row and these rows (prose with backtick spans and pipes). | -- | **Pass** (27 of 29 runs; the two misses are T231 and T232). |
| T231 | ROUTE3: `Insert the lines … before the line '];' in tasks.rs.` -- the anchor `];` is one whole line, but line 3 of the file ends in `` `templates_extended`]; `` | **Fail**: one read, then edit with oldString `];`, which the edit tool rejects as multiple matches, then "Verification failed … the observed bytes differ from the planned workspace effect". The insert is lowered to a substring edit of the anchor alone, so a line that occurs once as a whole line still matches twice; the repeated-anchor answer of T217 (G72) does not fire because the whole-line count is 1. Reproduced on a four-line t.rs (sandboxes/ROUTE3-anchor). | **Open** (G73, owner TEACH-F: insertEdit in js/agentic/workspace_change.mjs). Worked around through Formal AI by naming the line before it: `… after the line '    },' that follows the line '        slug: "copy_stdin_to_stdout",'.` |
| T232 | ROUTE3-fork: `Insert the line 'pub mod cache;' followed by an empty line before the line 'use std::collections::BTreeMap;' in recurrence.rs.` | **Partial fail**: the line went in and the answer reported success, but the empty line was silently dropped: composeEditRequest returns the new text `pub mod cache;` plus the anchor, so the trailing "followed by an empty line" modifier is never read. | **Open** (G74, owner TEACH-F: js/agentic/write_request.mjs / positional_edit.mjs). The blank line was added by hand. |
| T240 | MIGRATE4 delegations that passed (diagnostic and coreference batch, issue #918): the claim line; ``Run `cat diag.lino >> handler-rules.lino` `` (the diagnostic_marker table) and ``Run `cat diag.js >> formal_ai_worker_native_lanes.js` `` (the browser twin); five single-line Rust replacements in meta_method_dispatch.rs (the stripped prompt, the marker line twice, the exact-marker check, the call site); the two solve() lines of formal_ai_worker_20.js (`Insert the line '…' before/after the line '…'`); the claim-evidence reader (one Replace, one Delete the line); the test module line in mod.rs; both ledger flips, the diagnostic reason line replaced and eight data/primitive/justification/held_out inserts; the pending ceiling and two ratchet notes; three worker-budget ceilings and rationales; six requirement-row replacements; the G72 gap line and these two rows -- 41 edits. Done by hand: the three-line guard and the parameter removal in try_diagnostic (a multi-line replacement has no request shape), both coreference twin bodies, the two new test files and the issue_699 count test. | -- | **Pass** (41 of 44 runs). |
| T241 | MIGRATE4: `In handler-migration-ledger.lino, insert the line '    justification "…"' after the line '    primitive "rust/src/solver_handlers/benchmark_prompts.rs"'.` -- the anchor line is shared by four ledger rows | **Fail**: one read, no edit, then "Verification failed for `handler-migration-ledger.lino`: the observed bytes differ from the planned workspace effect". `insertEdit` (js/agentic/workspace_change.mjs) returns null for a lead that is not unique and `planInsertSequenceStep` reports that null through `failed()`, so an ambiguous anchor reads as a digest mismatch. Reproduced on a five-line file holding `anchor` twice. | **Open** (G72; possibly G52's cause). Worked around through Formal AI by naming the row: `… after the line '…' that follows the line '    data "data/seed/coreference.lino"'.` |
| T242 | MIGRATE4 delegations that passed (verifiable_task review and nl_tool, issue #918): the claim line; the pattern_inference.rs disposition, core_component and reason lines (`replace the line '…' that follows the line '…'`), both outside-core ceilings and the boundary note; the handler_files ceiling and note; the verifiable_task and nl_tool ledger flips with their data, primitive, justification and held_out lines; ``Run `cat nl-responses.lino >> multilingual-responses-synthesis.lino` `` and the groundings append; the worker-modules.js line and the two solve() lines of formal_ai_worker_20.js; the nl_tools budget shard and the worker_20 ceiling and rationale; the pending and hardcoded-row ceilings with their notes; eight requirement-row replacements -- 41 edits in 16 runs. Done by hand: the nl_tool_text reader and its call sites in natural_language_tools.rs, the new worker module, the test files. | -- | **Pass** (16 of 16 runs). |
| T243 | MIGRATE4 delegations that passed (installation_conversion, issue #918): ``Run `cat inst-rules.lino >> handler-rules.lino && cat … >> …` `` (seven installation_* tables and the policy, sixteen responses, twenty-three groundings in one run); both worker-budget ceilings; four core-boundary replacements (two baselines, two ceilings) and the boundary note (`sed -i '' '54r note.lino'`); the ledger flip (`sed -i '' -e '371r row.lino' -e '371d'`); both ratchet values; the four issue_699 count edits; the G75 gap line and these rows. Done by hand: the table readers in installation_conversion.rs and formal_ai_worker_11.js/_12.js, the two tests, the reasons and notes whose text is a long quoted line. | -- | **Pass** (15 of 15 runs; every edit read back). |
| T244 | MIGRATE4: ``Run `sed -i '' '/^rust\/src\/solver_handlers\/installation_conversion.rs	/d' scripts/hardcoded-language-allowlist.txt` `` | **Partial fail**: the seven rows were deleted, but the answer was "This folder is empty.": `isListing` in js/agentic/tool_result.mjs tests the whole command label for `list`, so a file named allowlist.txt makes a silent edit an empty listing. Reproduced in sandboxes/MIGRATE4-g73 with list.txt. | **Open** (G75). The edit itself is right; read back with grep. |
| T245 | MIGRATE4 delegations that passed (translation, issue #918): ``Run `cat tr-responses.lino >> … && cat tr-ground.lino >> …` `` (six translation_* responses and their groundings); moving them to multilingual-responses-translate.lino once the synthesis file hit 1501 lines, then to multilingual-responses-text-transform.lino as the worker does not load the translate file (``Run `sed -i '' '1478,1501d' … && cat … >> …` ``); repointing the six groundings (a ranged `sed s|…|…|`); the allowlist note above the rust/src/translation/ rows (`sed '794r note.txt'`) and its debug-flag correction; three core-boundary replacements (the mod.rs baseline and both ceilings); the ledger flip and its data path; both ratchet values; the four issue_699 count edits; the G78 gap line and these rows -- 15 runs, plus four probe runs in sandboxes/MIGRATE4-g76. Done by hand: the translation_text and clarification_text readers in mod.rs, code.rs and selection.rs, renderTranslationGap in formal_ai_worker_05.js, the tests, the long reasons and notes. | -- | **Pass** (15 of 15 runs; every edit read back). |
| T246 | MIGRATE4: `In s.txt replace 'key 3' with 'key 33'` (a probe beside the ledger edits: the new text extends the old) | **Fail**: "Left `s.txt` unchanged: it already has content, and the request does not say to replace the whole file."; 'key 3' with 'key 7' is replaced. | **Open** (G78); it struck again on the T245 row edit, a replacement whose new text began with its needle, and was rephrased. Avoided in the ledger work by replacing whole distinct values. |
| T250 | UPSTREAM-TR (meta-language worktree, issues #202 and #203): `Append the line '(call siblings-to-rust (javascript cappedTwice) (rust capped_twice) (arguments (f64 1)) (result 2))' to parity/self-translation/cases.lino.` | Pass: one read, one edit anchored on the file's last line, then the sha256 check; the call is the last line of cases.lino and the JS corpus test passes with it. | Pass. |
| T251 | UPSTREAM-TR: ``Run `node --test js/tests/self-translation.test.js` and tell me whether every test passes.`` | **Fail, unsafe**: the bash call was the whole remainder with its backticks, `` `node --test js/tests/self-translation.test.js` and tell me whether every test passes ``, so /bin/sh ran the span as a command substitution and then tried to run the test output as a command; the answer said "The command completed successfully without output." | **Open** (G76). |
| T252 | UPSTREAM-TR: ``Run `node --test js/tests/self-translation.test.js`.`` | **Fail**: the right command ran, but the suite takes 80 s and drive.mjs kills a bash call after 60 s; the killed call returns its partial stdout (`TAP version 13`) with no exit code, and the answer said the command "exited with code 1". The same command run by hand: 13 pass, 0 fail. | **Open** (G77). |
| T260 | DISCOVER delegations that passed (issue #1165 R1165-6): the claim line; four replacements in languages.rs (the TypeScript check, the Java check and run, the Kotlin run) and one rewording of their comment; five line deletions each from formal_ai_worker_12.js and agentic-coding-catalog.lino (the TypeScript, Java and Kotlin command lines); the install line after the installSeedProgramTasks line of formal_ai_worker_00.js; the mod line in rust/tests/unit/mod.rs; gap entries G79 and G80; these ledger rows -- 24 edits in 26 runs (one correct decline, T263, and one duplicate insert of this row from a rerun issued before reading the first answer, removed by hand), each in a sandbox copy and then applied to the live file, which other agents edit too. Done by hand: the Kotlin check (three lines, G80) and the Rust run (a shared needle, G79), the derivation in both roots, the new worker module, the catalog resolver, the tests and the requirement rows. | -- | **Pass** (24 edits; the duplicate came from the rerun, not from Formal AI). |
| T261 | DISCOVER probe: in languages.rs, in the row whose slug is rust, replace `run_command: Cow::Borrowed("./main"),` with `run_command: Cow::Borrowed(""),` | **Fail, silent**: the C and C++ rows were rewritten too, and the answer reported one replacement. | **Open** (G79); done by hand. |
| T262 | DISCOVER probe: replace the three lines `check_command: Some(Cow::Borrowed(`, the kotlinc line and `)),` with the line `check_command: None,` in languages.rs | **Fail**: Error: oldString not found in content; the list words and quote marks were taken as the old text. | **Open** (G80); one Delete per line works. |
| T263 | DISCOVER: a gap line whose payload nested guillemet quotes inside guillemet quotes | Declined: a quote in this request opens and never closes; nothing was done. | **Pass**: the request was ambiguous, and the rephrasing with backtick spans appended G79. |
| T200 | LEAD delegation: in js-literal-ratchet.lino replace worker_ceiling 686 with worker_ceiling 645 (R1010 after MIGRATE4 and DISCOVER removed 41 worker literals) | Pass: one replace, observed. | **Pass**: done by Formal AI on the first run. |
| T201 | LEAD probe: in formal-ai-dogfood.md move the T200 row (quoted by its leading cells) so that it follows the T263 row | **Fail**: read the file and answered with its lines; no move was planned (G81). | **Open** (G81); moved by hand. |
| T202 | LEAD delegations that passed: copy check-worker-line-budget-js-twin.lino to check-changelog-fragment-js-twin.lino, then three single replaces (the gate name, the run line, the description) | Pass, though the copy's answer printed `null`. | **Pass**: four edits by Formal AI. |
| T203 | LEAD probe: in check-changelog-fragment-js-twin.lino replace «check-worker-line-budget» with «check-changelog-fragment» everywhere | **Fail**: Left the file unchanged: it already has content, and the request does not say to replace the whole file (G84). | **Open** (G84); three single replaces worked. |
| T204 | LEAD fixes of G88, G89 and G91 (both roots): a replace list takes its file from any clause; quoted texts match with newline escapes decoded; an edit naming several files is declined with the files named (`request_several_edit_targets`) | Each probe re-driven after the fix: both replacements made; the escaped list applied; the two-file edit declined with nothing changed. | **Pass**: pinned in `pull-request-1188-teach-f.test.mjs` and `pull_request_1188_replace_lists.rs`. |
| T205 | LEAD delegations: in gaps.md mark G88, G89 and G91 FIXED (three single replaces), then insert the T204 and T205 rows after the T203 row | Pass: each replace observed; the inserts planned as positional edits. | **Pass**: done by Formal AI. |
| T206 | LEAD: an adversarial-quoting property test (240 seeded replace requests, both roots) found two defects: a quoted edit word read as the instruction made the routed write overwrite the file, and a lone double quote after a letter passed as an apostrophe; both fixed, with G92 (the answer follows the request, not the quoted payload) | Each defect re-driven after the fix: the edit composed, the odd quote declined, the answer in English. | **Pass**: pinned in `pull-request-1188-adversarial-quoting.test.mjs` and `pull_request_1188_quoted_payload_safety.rs`. |
| T290 | TEACH-F (G76, UPSTREAM-TR's T251): Run the backticked command node --test x.test.js and tell me whether every test passes. | **Fail, unsafe**: the bash call was the whole remainder with its backticks, which /bin/sh read as a command substitution followed by words. | **Pass**: after a passthrough prefix a leading code span is the command and the words after it are prose, and a remainder whose backticks do not pair is no command (commandSpan / command_span in shell_command_policy, used by prefixedShellCommand / prefixed_shell_command). Pinned by the teach-f tests in both roots. |
| T291 | TEACH-F (G77, UPSTREAM-TR's T252): a bash call that outlives the driver's timeout | **Fail**: the killed call came back as partial stdout, or as Exit Code 1 from node --test's own SIGTERM handler, and the answer said the command exited with code 1. | **Pass**: drive.mjs reports a call killed at its timeout (BASH_TIMEOUT_MS, FORMAL_AI_BASH_TIMEOUT_MS) as Signal and Timeout fields with no exit code, and the shell envelope reader takes a Signal or Timeout trailer without an exit code as a failure whose error is the kill (parseShellEnvelope / parse_shell_envelope), so the answer states the kill and invents no exit code. Pinned by the teach-f tests in both roots. |
| T292 | TEACH-F (G81, LEAD's T201): In f.md move the line that starts with - alpha so that it follows the line that starts with - gamma. | **Fail**: the file was read and answered with its lines; nothing moved. | **Pass**: seeded line_move_follow and line_move_precede meanings (so that it follows, comes after; so that it precedes, comes before; five languages) state the place of a moved line when no after or before cue does, in groundedLineOperation / grounded_line_operation; a line named by its start is the one line that holds it. Pinned by the teach-f tests in both roots. |
| T293 | TEACH-F (G83, SCRIPTS-B's T306): In x-line.lino replace file with zzz. | **Fail**: Verification failed; the word line inside the file name made the request a whole-line replacement. | **Pass**: the unquoted path tokens are blanked before a request is read for the seeded line meaning (withoutPathWords / without_path_words, used by namesLine / names_line and the line-operation adjacency check), so x-line.lino and line-budget.lino are file names, not lines. Pinned by the teach-f tests in both roots. |
| T294 | TEACH-F (G82, SCRIPTS-A's T304): Copy a.lino to b.lino, then in b.lino replace x with y, p with q and m with n, each with its own replace verb. | **Fail**: planned as one literal_file write of the request's tail; nothing was copied or replaced. | **Pass**: a request whose first sentence is a seeded shell intent (copy, move) and whose later sentences edit a file it names is planned sentence by sentence over one tool history (planRequestSequenceStep / plan_request_sequence_step, first in the workspace_change arm); one sentence with several replace verbs is one computed change applying the quoted pairs in order (replaceList / replace_list, groundedReplaceList / grounded_replace_list), stated by the seeded coding_texts_replaced answer. Pinned by the teach-f tests in both roots. |
| T295 | TEACH-F: delegating ledger row T292 with its example line starts quoted in guillemets inside the guillemet-quoted row | **Fail, destructive**: wrote the untracked file f.md at the repository root and answered Completed the general change request for f.md; removed by hand, and the row was sent again without inner quotes. | **Pass** (G86, fixed in T296). |
| T296 | TEACH-F (G86, T295): insert a ledger row whose own text holds guillemet pairs after the line containing a row id in a ledger file | **Fail, destructive**: the payload was cut at its first inner pair, and a path inside it was written as a new file at the repository root. | **Pass**: guillemets pair as a stack (an inner pair is part of the payload; an outer quote whose stack never empties is declined as unpaired, the G71 path), and a literal-file write never targets a path the request names only inside quoted text (seeded general_change_target_unnamed, five languages); tests in both roots. |
| T297 | TEACH-F (G84, LEAD T202-T203): In f.txt replace a with b everywhere. | **Fail**: refused as a whole-file write (Left f unchanged: it already has content); the word everywhere was read as part of the new text. | **Pass**: the seeded file_edit_all_occurrences cue (everywhere, all occurrences, in all occurrences, throughout the file; five languages) is blanked outside the quotes before the edit clauses are composed, and every occurrence is replaced; a cue word inside the quotes stays text; tests in both roots. |
| T298 | TEACH-F (G87, REQ-AUDIT T324): the same Replace asked twice | **Fail**: the second run answered Verification failed: the observed bytes differ from the planned workspace effect, though nothing was planned or written. | **Pass**: when the old text is gone the answer is seeded, coding_text_already_replaced when the new text is there and coding_text_not_found otherwise (five languages); a removal of quoted text that is gone says the same; tests in both roots. |
| T299 | TEACH-F (G85, CIFIX2 T314): Move lines 2-3 of a.md to the end of b.md. | **Fail**: planned as mv a.md b.md, a rename of the whole file. | **Pass**: a move or copy of text inside a file is never a file move; a numbered line range moved to another file is appended (or prepended) to the destination and then removed from the source, each checked by its digest, and a range past the end of the source is answered with its line count (seeded line_range_moved_end and _start, line_number_moved_end and _start, line_range_outside_file; five languages); tests in both roots. |
| T300 | SCRIPTS-A: in experiments/formal_ai_subagent/README.md (a sandbox copy) replace the four `rust-script` requirement-pipeline commands with `node scripts/assemble-requirements.mjs --write && node scripts/generate-requirement-status.mjs --write && node scripts/render-status.mjs --write && node scripts/check-requirement-status.mjs` | Pass: one replace, observed. | **Pass**: done by Formal AI on the first run. |
| T301 | SCRIPTS-A: the same replace in experiments/formal_ai_subagent/preamble.md (a sandbox copy) | Pass: one replace, observed; only that command changed. | **Pass**: done by Formal AI on the first run. |
| T302 | SCRIPTS-A: derive `data/meta/ci-gates/check-render-status-js-twin.lino` from `check-status-render.lino` in a sandbox, one request per step: copy the file, then replace the gate name, the stage, the run line, the description and the justification | Pass: 6 of 6 steps, each one `cp` or one edit. | **Pass**: the gate file is Formal AI's. |
| T303 | SCRIPTS-A: derive the assemble-requirements, generate-requirement-status and check-requirement-status twin gates from the T302 gate, six requests each (copy, then five replaces) | Pass: 18 of 18 steps. | **Pass**: the three gate files are Formal AI's; the G82 gap line was appended by Formal AI as well. |
| T304 | SCRIPTS-A: one compound request: copy `check-status-render.lino` to `check-render-status-js-twin.lino`, then in the copy replace the gate name, the stage and the run line (three replaces in one sentence) | **Fail**: planned as `literal_file` and wrote the request tail as the whole new file; nothing was copied or replaced (G82). | **Open** (G82): the planner is in `js/agentic`, which TEACH-F owns this round; worked around by sending each step alone (T302, T303). |
| T305 | SCRIPTS-B: derive `data/meta/ci-gates/check-file-size-js-twin.lino` from a sandbox copy of `check-file-size.lino`, one request per field: replace the gate name, the description, the run line and the justification | Pass: 4 of 4 replaces, each observed. | **Pass**: the gate file is Formal AI's. |
| T306 | SCRIPTS-B: derive the debt-ratchet, hardcoded-language, minimal-core-boundary and worker-line-budget twin gates from the T305 gate (copy, then replace `check-file-size` and `check_file_size_js_twin` in each; append `--base origin/main` to the debt ratchet's run line) | Pass: 8 of 9 replaces. **Fail** on replacing `check-file-size` with `check-worker-line-budget` in `check-worker-line-budget-js-twin.lino`: "Verification failed ... the observed bytes differ from the planned workspace effect", nothing changed. | **Open** (G83; js/agentic belongs to TEACH-F): every Replace fails in a file whose name holds `-line` (`x-line.lino`, `line-budget.lino`), whatever the old and new text. The same request on a copy named `wlb-gate.lino` passed, so Formal AI finished the gate there and it was renamed back. |
| T307 | SCRIPTS-B: in a sandbox copy of rust/tests/web/pull-request-1188-subagent-folder.test.mjs replace the regex tail `+rust-script scripts\/check-file-size\.rs$/` with `+node scripts\/check-file-size\.mjs$/` (local-gates.mjs now lists the twin) | Pass: one replace, backslashes kept. | **Pass** |
| T308 | SCRIPTS-B: in experiments/formal_ai_subagent/preamble.md (in place, `--dir` at the repository root) replace the hardcoded-language command, then the span naming the worker-budget, boundary and debt-ratchet commands, with their `node scripts/<name>.mjs` forms | Pass: 2 of 2 replaces. | **Pass** |
| T309 | SCRIPTS-B: insert the routine-commands line for the five check twins after the JS-to-Rust translation line of experiments/formal_ai_subagent/README.md (in place), then these five rows after the T304 row of this ledger | Pass: both inserts landed after their anchors. | **Pass** |
| T310 | CIFIX2 (Rust CI on 6e1c539fc): `add file note.txt containing hello`, `add a new file src/lib.rs with content pub fn ready() {}`, `new file: notes.txt, contents: hello` (issue_745, issue_680, issue_712) | **Fail**: the first call was `read_file`; the T96 literal-write backstop reads the target unless the first write verb is a seeded whole-file write, so 71 of the 108 write-routing prompts of those tests (add, append, record, set ... in all five languages) read first. JS failed the same way, so the suspects named in the task (contents_source, grounded_setting, unquoted_addition_path) were not the cause. | **Pass**: the seeded `file_declared_noun` (meanings-file-write.lino) marks a request that declares the file it writes: a path right after that noun, no destination or location cue ahead of it, and a content lead after the path writes unread (`declaresFile` / `declares_file` in literal_write_guard, both roots); `add to the file note.txt containing hello` still reads first. Pinned in `rust/tests/web/pull-request-1188-cifix2.test.mjs` and `rust/tests/unit/pull_request_1188_cifix2.rs`. |
| T311 | CIFIX2: `execute grep TODO note.txt` (issue_749) | **Fail**: ran `grep -rnHw --exclude-dir=.git -- 'TODO' 'note.txt'`; the T90 workspace-search arm read an explicit command as a search request. | **Pass**: `contentSearchFor` / `content_search_for` decline a request an explicit passthrough prefix introduces, so the command runs as written. The whole-word lowering of a real search request (`grep -rnHw`, `\bRouteIntent\b`) is kept as T90 designed it; issue_749 and issue_745 now expect it, with the reason written in the tests. |
| T312 | CIFIX2: `qué hay en la carpeta actual` (issue_745) | **Fail**: `ls 'la'`; the article after the place preposition `en` was read as the listed directory (T98). | **Pass**: `listedDirectory` / `listed_directory` never take a seeded `request_function_word` (articles, pronouns, in every language) as the operand; `List the files in src.` still lists `src`. |
| T313 | CIFIX2 delegations that passed: three response translations (hi, zh, es of learn_from_source_browser_summary) inserted after an anchor line; the 20-line block appended to multilingual-responses-parity.lino; lines 671-690 deleted from multilingual-responses.lino; the trace.rs line inserted into the DECLARED list of the repository_workspace source test and its doc comment replaced; two lines inserted into plan 03's module tree; gap G85 appended; these ledger rows inserted -- 8 edits in 8 runs, each in a sandbox copy (the gap and the rows in place). Done by hand: the guard, search and listing fixes in both roots, the seed meaning, the solver split, the test fixes and the new tests. | -- | **Pass** (8 of 8). |
| T314 | CIFIX2: `Move lines 671-690 of multilingual-responses.lino to the end of multilingual-responses-parity.lino.` (a sandbox copy) | **Fail, safe**: planned `mv multilingual-responses.lino multilingual-responses-parity.lino`; the existence probe `test ! -e` stopped it and nothing changed. | **Open** (G85); an Append plus a Delete did the move. |
| T320 | REQ-AUDIT: replace one backticked file suffix in the new requirement shard `issue-1188-user-requirements.md` (sandbox copy) | **Pass**: one edit, digest checked. | No change needed. |
| T321 | REQ-AUDIT: replace a backticked span and the word after it with a longer parenthesised note in `docs/architecture/runtime-and-surfaces.md` | **Pass** | No change needed. |
| T322 | REQ-AUDIT: insert a line holding a Markdown link after a named line of `docs/requirements/README.md` | **Pass** | No change needed. |
| T323 | REQ-AUDIT: replace `arg1` with `left`, then `arg2` with `right`, in `CONTRIBUTING.md` (four lines each) | **Pass** (2 of 2) | No change needed. |
| T324 | REQ-AUDIT: replace the naming bullet of the `CONTRIBUTING.md` code standards with a 230-character bullet holding backtick spans and a semicolon | **Pass** the first time. Asked again, with the old text gone, it answered "Verification failed for `CONTRIBUTING.md`: the observed bytes differ from the planned workspace effect." although nothing was planned or written. | **Open** (G87): the answer should say the quoted text is not in the file. |
| T325 | REQ-AUDIT: three insert-the-line-after-the-line-containing requests and one replace in `CONTRIBUTING.md` | **Pass** (4 of 4) | No change needed. |
| T326 | REQ-AUDIT: insert an 11-line block, written with newline escapes, before a named heading of `VISION.md` | **Pass**: one edit. | No change needed. |
| T327 | REQ-AUDIT: from the repository root, append gap G87 to `gaps.md` and rows T320-T327 to this ledger | **Pass** (2 of 2) | No change needed. |
| T328 | REQ-AUDIT: replace a phrase holding a count with a longer phrase holding a parenthesised count in the requirement shard (sandbox copy) | **Pass** | No change needed. |
| T340 | CI-SPEED: from the repository root, append a claim line holding backticks, braces, commas and parentheses to `experiments/formal_ai_subagent/claims.md` | **Pass** (2 of 2) | No change needed. |
| T341 | CI-SPEED: replace a job-level `timeout-minutes` line, four-space indent kept, in seven workflow copies (40, 45 or 60 to 30) | **Pass** (7 of 7) | No change needed. |
| T342 | CI-SPEED: replace a `TEST_BUDGET_SECONDS` line in three workflow copies (1500 or 2400 to 1200) | **Pass** (3 of 3) | No change needed. |
| T343 | CI-SPEED: replace whole YAML comment lines holding backticks, percent signs, issue numbers and parentheses with longer lines in workflow copies | **Pass** (7 of 7) | No change needed. |
| T344 | CI-SPEED: replace a Rust tuple row and a Rust string literal holding escaped double quotes and dollar signs in test copies | **Pass** (2 of 2) | No change needed. |
| T345 | CI-SPEED: from the repository root, insert ledger rows T340-T345 one after another, each after the line containing the previous row id | **Pass** (6 of 6) | No change needed. |
| T360 | TEACH-F (G84, LEAD T203): Copy a.txt to b.txt, driven with a small step budget | **Fail**: the driver printed the answer null when the budget ran out mid-task, and a long transcript lost its answer line when the process exited. | **Pass**: drive.mjs returns a stop reason and prints why a session has no answer (the step budget ran out after N tool calls, or the planner planned no step), never a bare null, and writes its output before it exits; tested in the JS root. |
| T370 | RENAME: (R1188-U5) nine Replace and Delete requests that let the worker modules load in any order (`CLAIM_EVIDENCE` reads become `claimEvidence()`, the `LOOKUP_STEM_STOPWORDS` spread goes, three test lines follow) in sandbox copies of four files | **Pass** (9 of 9) | No change needed. |
| T371 | RENAME: replace a test line whose backtick-quoted old and new texts each hold a backtick of their own (`Replace `return plain(evaluate(worker, `CLAIM_EVIDENCE[${` with ... in t.mjs.`) | **Fail**: nothing was edited, and the answer was the fragment `claimEvidence()[${` instead of a decline naming the quotes (the G90 class) | Open: G90, the reproduction is noted under it in gaps.md; the same request with double quotes passed. |
| T372 | RENAME: insert the eleven-line `claimEvidence()` function before `function claimRouteRows() {` with one Replace whose new text holds newline escapes and an apostrophe | **Pass** | No change needed. |
| T373 | RENAME: four Replaces of comments that named `CLAIM_EVIDENCE` in three worker modules | **Pass** (4 of 4) | No change needed. |
| T374 | RENAME: replace the whole line `init();`, whose text also sits inside `await init();`, with four lines that start the worker in a microtask | **Pass**: only the whole line changed | No change needed. |
| T375 | RENAME: 43 requests generated from `data/meta/rename-map.lino`: each renamed module's `// Worker module N of 21` line replaced with its wrapped subject lines, each line-budget rationale's opening sentence replaced with the subject, one header prefix dropped | **Pass** (43 of 43) | No change needed. |
| T376 | RENAME: 21 whole-line ceiling Replaces in worker line-budget shards | **Pass** (21 of 21) | No change needed. |
| T377 | RENAME: 21 rationale appends whose double-quoted old and new texts end in a double quote of their own (`Replace "…synthesized."" with "…holds."" in f.lino.`) | **Fail** (21 of 21): no file changed; most answered `oldString not found`, two a verification mismatch and one a several-files decline, and none declined naming the quotes (the G90 class) | Open: G90; the same 21 appends with «» quotes passed (21 of 21). |
| T378 | RENAME: delete the `frozen_prefix_max 24` line and replace the worker_modules note in `merge-conflict-policy.lino` | **Pass** (2 of 2) | No change needed. |
| T379 | RENAME: replace a sentence of `runtime-and-surfaces.md` holding backticks, an ellipsis and parentheses | **Pass** | No change needed. |
| T380 | RENAME: replace a regex literal in `issue-513-sync-worker-terminal.mjs`, and a filter line and an import line in `check-prompt-specialization.mjs` | **Pass** (3 of 3) | No change needed. |
| T381 | RENAME: reword the text and the action of one contradiction record in `requirement-coverage.lino` | **Pass** (2 of 2) | No change needed. |
| T382 | RENAME: replace the R1188-U5 status cell (about 1,900 characters with backticks, parentheses and semicolons) with both texts in «» | **Pass** | No change needed. |
| T383 | RENAME: insert 25 review notes into `rename-map.lino`, each after the line holding its rename's reason, then three Replaces that add the blueprint capability table to one module's subject | **Pass** (28 of 28) | No change needed. |
| T384 | RENAME: insert a sub-entry under G90 in `gaps.md` whose «» payload holds backticks and a «» pair of its own | **Pass** | No change needed. |
| T385 | RENAME: from the repository root, insert ledger rows T370-T384 one after another, each after the line containing the previous row id, with «» payloads holding backticks, pipes of the table and «» pairs of their own | **Pass** (15 of 15) | No change needed. |
| T386 | RENAME: from the repository root, move the tag of row T370 to the front of its task cell (`RENAME (R1188-U5):` to `RENAME: (R1188-U5)`), so the tally counts the row for its subagent | **Pass** | No change needed. |
| T470 | CIFIX-LOOP (ee9fd4de8, E2E issue-153): replace the two `toLower(...)` lines of `extractCoreferenceSeeds` in `js/seed_loader.js` so a coreference context keeps its spaces (sandbox copy) | **Pass** (2 of 2) with «» quotes; the first try, with backslash-escaped quotes, was declined with the reason (the unpaired quote named), which is the designed answer | No change needed. The CI failure itself was a worker defect, fixed by this edit: the seed loader trimmed `" program "` to `program`, so "programming" in a Hindi search request after an English one resolved to the program-artifact body (pinned in `rust/tests/web/issue-0918-handler-rules-batch-2.test.mjs` and `rust/tests/unit/issue_918_handler_rules_batch_2.rs`). |
| T471 | CIFIX-LOOP (ee9fd4de8, clippy): replace a Rust line holding the char literal ``'`'`` in `shell_command_policy.rs`, the old and new texts in single quotes (sandbox copy) | **Fail**: the single quotes inside the quoted texts paired with the outer ones; Formal AI edited nothing and answered a fragment of the new text as if it were the result | Not fixed in this cycle (gap G90). The same request with double quotes: **Pass**. |
| T472 | CIFIX-LOOP (ee9fd4de8, clippy): replace the wildcard arm `plan => return Some(plan),` with `plan @ AgenticPlan::ToolCalls(_) => return Some(plan),` in `request_sequence.rs` (sandbox copy) | **Pass** | No change needed. |
| T473 | CIFIX-LOOP (aada2f87a, js tier): in `local_search.mjs`, insert a strip-list line after another and delete the old one, one request with two steps (sandbox copy) | **Fail**: Formal AI read the file and answered "The `read` command completed." without editing (gap G99) | Not fixed in this cycle. The same change as one Replace: **Pass** (T474). |
| T474 | CIFIX-LOOP (aada2f87a, js tier): one Replace and one Insert-the-line in `local_search.mjs`, one Insert-the-line in `local_search.rs`, so the local search strips interrogatives from its subject (sandbox copies) | **Pass** (3 of 3) | No change needed. The CI failure was a routing defect: the Spanish scope surfaces of round 18 made "qué hay en la carpeta actual" a search for a file named `qué`; the subject now drops `interrogative_opener` words (plus the seeded plural `cuáles`) in both roots, pinned by `pull-request-1188-cifix2.test.mjs` and `issue-1021-behaviour-range.test.mjs` with their Rust twins. |
| T500 | READABLE (R1188-U23): replace one bullet line of the plan doc `16-js-ts-rust-cycle.md` with a longer line holding bold markers, parentheses, a semicolon and an em dash (sandbox copy) | **Pass** | No change needed. |
| T501 | READABLE: append a sentence holding parentheses, a colon and a script path to `ts/README.md` (sandbox copy) | **Pass** | No change needed. |
| T502 | READABLE: insert a blank line before the line containing a phrase in `ts/README.md` (sandbox copy) | **Pass** | No change needed. |
| T503 | READABLE: break a 342-character array line of `formal_ai_worker_meta_synthesis.js` with one Replace whose new text holds two newline escapes (sandbox copy) | **Pass** | No change needed. |
| T504 | READABLE: reflow `NUMERIC_CLAIM_EVIDENCE` in `formal_ai_worker_16.js` with one request listing seven Replaces whose new texts hold newline escapes | **Fail**: declined as a whole-file write (G88). | Not fixed: G88 is recorded for TEACH-F, whose claim holds replace_list and write_request; worked around with seven single requests (T506). |
| T505 | READABLE: probe the Replace list in a one-line `m.js`: four orders and escape forms | **Fail, destructive** for `Replace A with B and replace C with D in m.js.`: the first old text was replaced with the rest of the request (G89); the `In m.js, replace ...` order passes with three items, and a list with newline escapes is declined (G88). | Not fixed: G89 is recorded for TEACH-F. |
| T506 | READABLE: the T504 reflow as seven single Replace requests, each new text holding a newline escape (sandbox copy) | **Pass** (7 of 7) | No change needed. |
| T507 | READABLE: in two worker-line-budget files, replace the ceiling line and extend the rationale's closing sentence, two Replaces in one request (sandbox copies) | **Pass** (2 of 2 files) | No change needed. |
| T508 | READABLE: replace the R1188-U23 status cell (about 1,300 characters with backticks, an arrow, brackets and a link) in a sandbox copy of the requirement shard | **Pass**, but an apostrophe written doubled inside the single-quoted new text stayed doubled; a second Replace with double-quoted texts fixed it. | No change needed: the request, not the planner, chose the doubling. |
| T509 | READABLE: from the repository root, append gaps G88 and G89 to `gaps.md`, then repair G88, whose example newline escapes the Append had decoded into line breaks | **Pass** (3 of 3) | No change needed: escapes are decoded by design, so the entry now names them in words. |
| T510 | READABLE: from the repository root, insert ledger rows T500-T510 one after another, each after the line containing the previous row id | **Pass** (11 of 11) | No change needed. |
| T420 | SPANISH: replace one translated row (`text "project"` with its Hindi word) in a sandbox copy of `multilingual-responses-installation.lino` | **Pass** | No change needed. |
| T421 | SPANISH: one Replace naming two files, `... in multilingual-responses-installation.lino and in multilingual-responses-policy.lino.` (sandbox copies) | **Fail**: only the first file was edited; the answer named that file alone and did not say the second was skipped (G91) | Not fixed in this cycle: G91 is recorded for the write_request owner. |
| T422 | SPANISH: the T421 Replace on the second file alone (sandbox copy) | **Pass** | No change needed. |
| T440 | TEXT-CAPABILITY: replace the R1188-U18 status cell (one long cell with backticks, parentheses and a semicolon) in a sandbox copy of `issue-1188-user-requirements.md`, old and new texts in guillemets | **Pass** for the edit; the answer was in Spanish (G92) | Not fixed in this cycle: G92 is recorded for the `tool_result` owner. |
| T441 | TEXT-CAPABILITY: the same Replace for the R1188-U19 status cell (arrows and backticks in the new text) | **Pass** | No change needed. |
| T442 | TEXT-CAPABILITY: the same Replace for the R1188-U21 status cell | **Pass** | No change needed. |
| T443 | TEXT-CAPABILITY: insert two Hindi `inflection-ending` lines after a third in a sandbox copy of `data/seed/text-formalization.lino` | **Pass** | No change needed; the measures rose (Hindi unknown terms 1045 to 1042). |
| T444 | TEXT-CAPABILITY: three Replaces of measured numbers inside the R1188-U18, U19 and U21 cells after the ratchets were raised | **Pass** (3 of 3) | No change needed. |
| T445 | TEXT-CAPABILITY: one more Replace of the U21 compression count | **Pass** | No change needed. |
| T446 | TEXT-CAPABILITY: from the repository root, append gap G92 to `gaps.md` | **Pass** | No change needed. |
| T447 | TEXT-CAPABILITY: from the repository root, insert ledger rows T440-T447 one after another, each after the line containing the previous row id | **Pass** (8 of 8) | No change needed. |
| T520 | REQ-ROUTE: insert three lines (the `requirement_listing` precedence row, each given as a single-quoted literal with double quotes inside) after a `note` line of a sandbox copy of `data/seed/handler-precedence.lino` | **Fail**: the session searched the web (`websearch`) instead of editing. | Not reproduced on a re-run of the same request later in the round (read, edit, digest check): the planner had changed concurrently; no fix of its own. |
| T521 | REQ-ROUTE: the same three lines as a block under `... insert these three lines after the line containing '...':` | **Fail**: the block, given at the file's record indentation (two spaces), was rebased on the anchor line's four spaces, so the row landed inside the previous record (G93). | **Fixed in T524** (G93); the row was written by hand. |
| T522 | REQ-ROUTE: insert the five-line `requirement_listing` `rule_set` row before a `handler` line of a sandbox copy of `data/seed/browser-handler-precedence.lino` | **Pass** | No change needed. |
| T523 | REQ-ROUTE: insert the five-line `requirement_listing` claim record after the summarization claim's `because` line of a sandbox copy of `data/seed/capability-routing.lino` | **Fail**: the same rebasing as T521 (G93). | **Fixed in T524**. |
| T524 | REQ-ROUTE: T523 again after the fix | **Pass**: an unfenced block keeps the indentation it was given when the file already indents lines so (indentsLinesAt / indents_lines_at, the rule appended lines followed), else it is rebased on the anchor. | G93 fixed in the positional_edit and workspace_change modules of both roots; regression cases in the edit-composer gap tests of both roots. |
| T525 | REQ-ROUTE: insert the `requirement_listing` method meaning before a meaning line of a sandbox copy of `data/seed/meanings-routing-vocabulary.lino` | **Pass** | No change needed. |
| T526 | REQ-ROUTE: insert the `requirement_listing` row before a `handler` line of a sandbox copy of `data/meta/handler-migration-ledger.lino` | **Pass** (one word of the justification fixed by hand) | No change needed. |
| T527 | REQ-ROUTE: from the repository root, append gap G93 to `gaps.md` | **Pass** | No change needed. |
| T528 | REQ-ROUTE: from the repository root, insert ledger rows T520-T528 as one block after the line containing the T447 row id | **Fail**: one row named two source files in its prose, and the several-files guard (G91) read the paths inside the block as edit targets, so the edit was declined as naming several files (G94). | **Fixed in T530** (G94). |
| T529 | REQ-ROUTE: T528 again with that row reworded without paths | **Fail, destructive**: rows that read "append gap G93 to `gaps.md`" and "... of a sandbox copy of `data/seed/capability-routing.lino`" were taken by the evidence-record route for deliveries: it inserted the rows cut at those phrases and then wrote its answer over `data/seed/capability-routing.lino` and into a new `gaps.md` at the root (G95). Both restored by hand, the rows repaired by hand. | **Fixed in T530** (G95). |
| T530 | REQ-ROUTE: T529 again in a sandbox after the fixes, with a row naming two source paths added | **Pass**: the rows land exactly, and nothing else is written. The lines under a colon-ended first line are the request's payload: the instruction ends with that line (instructionEnd / instruction_end), so the replace-list cuts and the several-files guard read only the first line (G94), and the evidence-record route masks the payload as it masks multi-word quotes (G95). | Fixed in positional_edit, replace_list and evidence_record of both roots; regression cases in the edit-composer gap tests of both roots. |
| T531 | REQ-ROUTE: from the repository root, insert rows T530 and T531 after the line containing the T529 row id, then append gaps G94 and G95 to `gaps.md` | **Pass** | No change needed. |
| T580 | TWINS: from the repository root, insert a `Rust built-in` doc line into the `rustLines` comment of `js/agentic/content.mjs` (R1188-U16) | **Pass** | No change needed. |
| T581 | TWINS: replace the `Mirrors #[derive(PartialEq)]` one-line comments of `candidatesEqual` and `proceduresEqual` with `Rust built-in` ones (two requests) | **Pass** | No change needed. |
| T582 | TWINS: replace the one-line comments of `sortedQualifiers` and `cloneStep` with ones naming their Rust built-in (two requests) | **Pass** | No change needed. |
| T583 | TWINS: insert `Rust built-in` lines into the `debugString` and `debugOption` comments (two requests) | **Pass** | No change needed. |
| T584 | TWINS: an insert and a replace in the `charIndices` and `byteSlice` comments of `formalization_segment.mjs` | **Fail, answer language**: both edits right, both answers in Spanish ("Se insertó …"), because the path's `formalization` begins with the Spanish marker `formaliza` (G97). | **Fixed in T601** (G97). |
| T585 | TWINS: an insert and two replaces in the `nodeHistoryIo`, `toMemoryEvent` and `writeCursor` comments of `history_store.mjs` | **Pass** | No change needed. |
| T586 | TWINS: replace the `matchIndices` comment of `intent_formalization_requirements.mjs` | **Fail, answer language**: the edit right, the answer in Spanish (G97). | **Fixed in T601** (G97). |
| T587 | TWINS: replace the lino-objects-codec citations of `escapeReference` and `formatIndentedOrdered` with `Rust dependency` lines (two requests) | **Pass** | No change needed. |
| T588 | TWINS: two inserts and a replace in the serde_json comments of `orchestration_json_stream.mjs` | **Pass** | No change needed. |
| T589 | TWINS: replace the comments of `canonicalSessionText`, `ioErrorFrom` and `jsonObject` (three requests, three files) | **Pass** | No change needed. |
| T590 | TWINS: three inserts of `Mirrors` lines into the `rust_ast_census.mjs` comments | **Pass** | No change needed. |
| T591 | TWINS: insert `Mirrors const` lines into the `responseFiles` and `meaningFiles` comments (two files) | **Pass** | No change needed. |
| T592 | TWINS: insert a `Mirrors fn stable_id` line into the `fastStableId` comment | **Pass** | No change needed. |
| T593 | TWINS: two inserts and a replace in the `maxByWeight`, `compareStrings` and `sameSignature` comments | **Pass** | No change needed. |
| T594 | TWINS: insert `Rust built-in` / `Rust dependency` lines into the meta-language parser comments (two files) | **Pass** | No change needed. |
| T595 | TWINS: insert a line after the `strLines` comment line that quotes backslash-r and backslash-n escapes | **Fail**: the file was read, nothing written, and the answer said verification failed; the quoted anchor's backslash-n was read as a line break, so no line matched (G96). | **Fixed in T596** (G96). |
| T596 | TWINS: T595 again after the fix | **Pass**: an anchor the file holds only as written is the written text (writtenEscapes / written_escapes). | G96 fixed in workspace_change (JavaScript) and positional_edit/unquoted_anchor (Rust); regression cases in the edit-composer gap tests of both roots. |
| T597 | TWINS: replace the `sameMode` and `pathExtension` comments (two files) | **Pass** | No change needed. |
| T598 | TWINS: insert a `Rust built-in format!` line into the `agenticMessage` comment, then `Delete lines 38-42 from js/agentic/messages.mjs.` (the unused `hasAgenticMessage` export) | **Pass** | No change needed. |
| T599 | TWINS: replace the four one-line comments of `plan.mjs` (`toolCalls`, `plannedCall`, `isToolCalls`, `isFinal`) | **Pass** | No change needed. |
| T600 | TWINS: eleven inserts and replaces citing the Rust functions extracted for R1188-U16 (conversation summary, cache file text, ranking, step id, slot kind, census rendering and histogram, owned content id, agent info value, census document) | **Pass** | No change needed. |
| T601 | TWINS: T584's insert again in a sandbox after the G97 fix | **Pass**: answered in English. | G97 fixed in the language detector of both roots; regression cases in the edit-composer gap tests of both roots. |
| T602 | TWINS: from the repository root, insert rows T580-T601 as one block after the line containing the T531 row id, then append gaps G96 and G97 to `gaps.md` (two requests) | **Pass** | No change needed. |
| T603 | TWINS: replace the R1188-U16 status cell (a long quoted text holding backticks) with the delivered status | **Pass** | No change needed. |
| T620 | GENERALIZE: from the repository root, `Replace «- G98 (LEXEMES, T542; destructive; not fixed)» with «- G98 (LEXEMES, T542; destructive; FIXED by GENERALIZE, T620)» in experiments/formal_ai_subagent/gaps.md.` | **Pass**: one edit, observed by checksum. | No change needed. |
| T621 | GENERALIZE: replace the tail of the G98 entry ("Left for the write_request owner …") with the fix note | **Pass** | No change needed. |
| T622 | GENERALIZE: insert the four-line `seed multilingual-responses-behavior-rules` block before the line `  seed multilingual-responses-capabilities` in `data/meta/seed-registry.lino` | **Pass** | No change needed. |
| T623 | GENERALIZE: insert three `example` lines after `    response_link "response:greeting"` in `data/seed/intent-routing.lino` | **Pass** | No change needed. |
| T624 | GENERALIZE: insert two `example` lines (one holding a comma) after the courtesy_response link in `intent-routing.lino` | **Pass** | No change needed. |
| T625 | GENERALIZE: insert two `example` lines (one in Russian) after the assistant_free_time link | **Pass** | No change needed. |
| T626 | GENERALIZE: insert four `example` lines (two holding an apostrophe) after the test_status link | **Pass** | No change needed. |
| T627 | GENERALIZE: insert five `example` lines in four scripts after the assistant_name link | **Pass** | No change needed. |
| T628 | GENERALIZE: insert four `example` lines after the identity link | **Pass** | No change needed. |
| T629 | GENERALIZE: insert two `example` lines after the write_program link | **Pass** | No change needed. |
| T630 | GENERALIZE: insert one `example` line after the unknown link | **Pass** | No change needed. |
| T631 | GENERALIZE: insert the three-line `seed meanings-translation-phrases` block before `  seed meanings-units` in `seed-registry.lino` | **Pass** | No change needed. |
| T632 | GENERALIZE: `Replace «  specialization_ceiling 23» with «  specialization_ceiling 0» in data/meta/prompt-specialization-ratchet.lino.` | **Pass** | No change needed. |
| T633 | GENERALIZE: replace the Chinese surface `显示规则详情` with `规则详情` in `data/seed/meanings-behavior-rules.lino` (the first was a list prompt already) | **Pass** | No change needed. |
| T634 | GENERALIZE: T542's two requests again in a sandbox after the G98 fix (`Cambia 'mundo' a 'amigo' en el archivo f.txt.`, `Change 'world' to 'friend' in the file g.txt.`) | **Pass**: `hola amigo` and `hello friend`, each answered in the request's language. | G98 fixed: composeEditClauses (js/agentic/write_request.mjs) and compose_edit_clauses (rust/src/agentic_coding/write_request.rs) run the file clause back over the seeded `request_function_word` words between target cues; pinned in the edit-composer gap tests of both roots. |
| T635 | GENERALIZE: insert rows T620-T635 as one block after the line containing the T603 row id in this file | **Pass** | No change needed. |
| T448 | TEXT-CAPABILITY: from the repository root, append the done line of this task to `claims.md` (one long line of paths with braces and slashes) | **Pass** for the edit; the answer was in Spanish again, as in T440 and T443 (G92) | Not fixed in this cycle: G92 is recorded for the `tool_result` owner. |
| T540 | LEXEMES: run the rule script in a sandbox, `node add-lexemes.mjs meanings-file-edit.lino es meanings-file-edit-es.tsv` | **Pass**: one bash call wrote the three Spanish `file_edit_*` lexemes (the `file_edit_action_cue` gap the owner named). | No change needed. |
| T541 | LEXEMES: `Reemplaza 'hola' por 'adiós' en f.txt.` and `Cambia 'mundo' a 'amigo' en f.txt.` with the new Spanish edit cues | **Pass**: both edited and answered in Spanish. | No change needed. |
| T542 | LEXEMES: `Cambia 'mundo' a 'amigo' en el archivo f.txt.` (and the English `Change 'mundo' to 'amigo' in the file f.txt.`) | **Fail** (destructive): the new text became `'amigo' en el` / `'amigo' in the`; the file clause is walked back only over target cues, so an article between two cues (`in the file`) stays in the new text (G98). | Not fixed in this cycle: `composeEditClauses` (js/agentic/write_request.mjs) and its Rust twin are claimed by other agents; recorded as G98. |
| T543 | LEXEMES: in a sandbox, `In formal-ai-dogfood.md insert the contents of row.txt after the line that starts with "| T542 "` | **Pass**: one edit placed this row after T542. | No change needed. |

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

**The unquoted output (round 4).** `prints Hello, World! and run it`
names its output without quotes, and the clause splitter keeps the whole
sentence as one clause. Reading "the words after *prints* up to *and*" as a
literal would also read `prints the sum of a and b` as the literal `the sum
of a`, so the output boundary needed a grounded rule, not a guess.

*Root cause.* `boundOutputLiterals` / `bound_output_literals` (the obligation
graph's output binding, `js/agentic/crate/intent_formalization_obligations.mjs`,
`rust/src/intent_formalization/obligations.rs`) bound quoted segments only,
so `explicitStdout` was empty, the program contract declined, and the shared
solver answered in chat with the documentation example.

*Fix (both roots).* A clause that quotes nothing is read by `unquotedOutput` /
`unquoted_output`: the words after its first `print_stdout` word, up to the
first seeded clause separator (`skill_procedure_clause_separator`: and, then,
и, फिर, …) or through the first word that ends a sentence. They are bound only
when they read as an utterance — they open with a capital (as a quotation
does mid-sentence), contain no `statement_function_word` (the, of, for, …),
and mention no `coding_structure` and no `program_task_alias` unless that
alias is itself a `social_greeting` (the hello-world program *is* its text).
Accepted: `Hello, World!`, `Привет, мир!`, `Hi` (beside a quoted `Bye`, in
request order). Rejected: `the sum of a and b`, `Fibonacci numbers up to
100`, `FizzBuzz`, `FizzBuzz for 1 to 100`, `prime numbers below 50`, `the
greeting!`.

*Second gap on the way.* `Write a Python program hello.py that prints …`
wrote `main.py`: the noun "program" before `hello.py` is no write cue, so
`typedWriteTarget` found nothing. `namedSourceFile` / `named_source_file`
now take the one relative source file with the language's extension the
request names, cued or not — the same reading `namedSourceLanguage` makes
for the language.

*Tests.* JS: "an unquoted output is bound only when it reads as an
utterance" (both T18 plans write `hello.py` with the exact source; both
directions of the binding). Rust: `rust/tests/unit/pull_request_1188_unquoted_output.rs`
(uncompiled here). Not touched: the native-only `extract_expected_stdout` in
`rust/src/coding/task_spec.rs` (the discovery task spec) still binds every
word after a print slot without this rule; it has no JS twin.

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

### T19 (continued) — writing the synthesized function where asked, and running it

Three pieces, the first JS-only (native already attaches recipes to its
verified drafts), the other two in both roots:

- **B.** The browser worker's composed function travels as
  `synthesizedProgram` (`formal_ai_worker_07.js`, copied through
  `formal_ai_worker_20.js`, carried off the wire by `symbolicFromWorker`), and
  the reroute turns it into an execution recipe with `attachExecutionRecipe`
  (`js/agentic/crate/coding_program_contract.mjs`), the twin of native
  `attach_execution_recipe`: the catalog's file and check command; a function
  is checked, not run as a program.
- **C.** `requestedRecipe` / `requested_recipe` (command reroute): the recipe
  is saved as the file the request names (`typedWriteTarget(request, ext)`),
  and its commands follow the rename. Recipes with supporting files keep
  their own path.
- **D.** The same function appends the call when the request runs the
  function with stated arguments: `statedCallArguments` reads the numbers and
  quoted literals after the request's last seeded run verb; the call comes
  from new `definition` / `call` templates in
  `data/meta/stdout-program-contracts.lino` (Python: `def {name}({parameters}):`
  and `python3 -B -c "from {module} import {name}; print({name}({arguments}))"`;
  `-B` so the import writes no byte code), and only when the argument count is
  the parameter count.

The dogfood driver now mirrors the server's fall-through (solve, then the
symbolic command reroute). Tests: JS exact transcript and answer, the
argument reader; Rust `rust/tests/unit/pull_request_1188_function_recipe.rs`
(write `add.py`, check, call, final; without arguments only the write and
check). Worker budgets 07 → 1393 and 20 → 1346 with the reason.

## Coordinator using Formal AI as a sub-agent (2026-10-08)

The coordinator handed real PR edits to the JavaScript Formal AI through `experiments/js_dogfood/drive.mjs`, one small step per prompt, reviewed each sandbox result and copied it into the repository.

- Landed through Formal AI: the case-study ladder note, 41 requirement rows pinned to their test files, the `file_edit_blank_line` seed block, the Rust twins of the three fixes below, and this section (one insert, replace or append per prompt).
- Fixed: a replacement whose new text carried a backticked path edited that path; a cue word and a full stop inside the quoted text split the literals (`unquotedPathTokens`).
- Fixed: an append request could take the whole-file general change route; a blank line is now a seeded concept, and an appended payload unescapes newlines.
- Fixed: a read whose text quotes error lines was treated as a missing file, so an append overwrote this very ledger with one newline. A file-block read is now always the file.
- Open: an unquoted payload is declined rather than appended; role cues inside a quoted payload still steer routing; a replacement whose text contains code ends with a generic answer instead of the seeded one.

### T6 — a typo whose correction the request does not state

**Fix (both roots).** A seeded concept `typo` (role `typo_fix_lead`; en
`typo`, ru `опечатку`, hi, zh `错别字`, es `errata`) and a fourth computed
change, `groundedTypoFix` / `grounded_typo_fix`: one quoted word, one path,
and the correction discovered by `correctedSpelling` /
`corrected_spelling` (`js/agentic/crate/spelling.mjs`,
`rust/src/agentic_coding/spelling.rs`): the unique most frequent word one
Damerau edit away in a vocabulary counted once (cached) from the bundled seed
files — the registry's `bundle true` set, which is what `seed::seed_files()`
embeds — in the word's capitalisation. A tie or no candidate claims nothing.
The change is the word-scoped replacement (so `small` elsewhere in the file is
untouched) and the answer is the seeded `coding_text_replaced` sentence. No
word list was authored; `misspelling` / `misspelled` were left out as surfaces
because they are not grounded tokens yet.

**Tests.** JS exact transcript and answer, plus `smal → small`,
`Projet → Project`, `teh → the`; Rust
`rust/tests/unit/pull_request_1188_typo_discovery.rs` (read, then the edit of
the one line).

### Sub-agent gap 1 — an unquoted appended line

`Append the line third to notes.txt.` was declined (after 882a7d711 it no
longer rewrote the file). Now grounded in both roots
(`linePayloadAndPath` / `line_payload_and_path`): no quoted text, exactly one
path, a seeded concept `line` (role `file_edit_line_lead`: `the line`,
`a line`, `line`, `строку`, `पंक्ति`, `一行`, `la línea`, …), and the line is the
words after the lead up to the *last* seeded destination cue before the path —
so `go to bed to notes.txt` keeps `go to bed`, and `третья в конец notes.txt`
cuts at `в`. A request without a lead (`Append third to notes.txt.`) is still
declined; the coordinator's pin in
`rust/tests/web/formal-ai-subagent-quoted-payloads.test.mjs` moved to that
prompt, and its old prompt now pins the append. Tests: JS (three languages
and the declined case) and Rust `rust/tests/unit/pull_request_1188_subagent_gaps.rs`.

### Sub-agent gap 3 — code in a replacement; the empty line's wording

Reproducing the reported generic answer exposed a worse case: a multi-line
replacement written with `\n` escapes (`replace "fn old() -> u8 {\n    1\n}"
with "…2…"`) **overwrote the whole file** with the quoted new text. The edit
composer unescapes `\n` / `\t` in its literals, but `groundedRewrite` compared
them against the raw quoted segments, decided the literals were not quoted,
and the general-change fallback took the request as a literal write. Both
roots now compare against the unescaped segments, so the request is a grounded
rewrite: read → edit of the function → digest → the seeded
`coding_text_replaced` sentence. (Single-line code literals such as
`fn keep() {}` already answered with the seeded sentence.)

An appended or prepended empty line is now stated as one, from new seeded
responses keyed by the existing `file_edit_blank_line` meaning ("Added an empty
line to `{path}` and observed the result.", five languages) instead of
"Appended `` to …". Tests: JS (exact transcript, file and answer for both) and
Rust `a_multi_line_replacement_written_with_escapes_edits_the_function`.

### Sub-agent gap 2 — cues inside a quoted payload steered routing

Reproduced worse than reported: `Append "/// Append an empty line to
notes.txt." to src/lib.rs.` read `src/lib.rs` and then **wrote a tool-result
rendering over it**, and `Append "Commit all changes" to notes.txt.` ran
`git commit`.

- The evidence-record arm (which runs before the workspace-change arm) split
  sentences at the payload's inner full stop and took the quoted `notes.txt`
  for a delivery target. It now detects sentences and delivery cues on a copy
  of the request with every multi-word quoted segment masked to the same
  length (`maskedMultiWordQuotes` / `masked_multi_word_quotes`), so spans
  still index the original; single-token quotes (backticked paths and names,
  the ladder's field lines) stay readable, and the pinned first line, field
  lines and the work before the delivery are read from the unmasked text.
- The commit arm reads its cue from the text outside quoted segments
  (`textOutsideQuotedSegments` / `text_outside_quoted_segments`).

Both roots. Tests: JS (the doc-comment append, the quoted commit) and Rust
`cues_inside_a_quoted_payload_do_not_steer_routing`.

### Client configs state the served model limits (`formal-ai with`)

The ladder harness fix (above) only covered the harness: every config
`formal-ai with` writes for an opencode-shaped client (`opencode`,
`opencode-vscode`, `opencode-desktop`, `agent` — ephemeral and `--global`)
carried no model limits, so the Agent CLI fell back to its built-in 60,000-token
`formal-ai` entry and compacted long sessions.

- `rust/src/client_integrations/server.rs` `served_model_limits` reads
  `context_window_tokens` and `max_output_tokens` from the target server's
  `/v1/models` (the same raw HTTP the wrapper already uses for `/health`).
- `render_context` fills two new placeholders, `{context_window_tokens}` and
  `{max_output_tokens}`, from it — or, when nothing listens yet, from this
  host's `ContextCapacity` and `ADVERTISED_MAX_OUTPUT_TOKENS`, which is
  exactly what a wrapper-started server here serves.
- `data/seed/client-integrations.lino` sets
  `provider.{provider_id}.models.{model}.limit.context=json:{context_window_tokens}`
  and `limit.output=json:{max_output_tokens}` for those four clients, in both
  the ephemeral and the global blocks. Codex already wrote the window through
  its model catalog. The other clients' config formats state no model window.
- JS has no config renderer (`formal-ai with` is native-only), so there is no
  JS twin.

Test: `rust/tests/integration/with_formal_ai_global.rs`
`with_formal_ai_global_states_the_served_model_limits` — `with --global agent`
writes `limit.output = 8192` and a context window above the 60,000 guess.

### The JS server answers a compaction request like native; the task survives repeated compactions

**Compaction summary.** The browser worker answers a summarize request with
conversation statistics (`## Conversation summary`), which carry no task, so
after an Agent CLI compaction the JS planner web-searched "What did we do so
far?". The worker realm cannot load the ported summarizer
(`js/agentic/crate/summarization*.mjs`), and the CLI-facing JS root is the
server, so `solveSymbolic` (`js/server/solve.mjs`) now replaces the worker's
statistics with the native envelope — `conversationSummaryEnvelope`
(`js/agentic/crate/conversation_summary.mjs`, twin of the summary body of
`try_summarize_conversation`): `Conversation summary: <summarize_dialog>`,
`Title:`, and every user turn, with the en/ru/zh headers taken from
`data/meta/agentic-messages.lino` (`conversation_summary_envelope_*`, the same
strings as the Rust `format!`). The returning-user recap (plain format) is
left as is. Replaying the run's compaction request against the JS server
reproduces the native answer line for line except one: the native body
stored by the CLI shows the second user turn ("What did we do so far?") as
empty — unexplained (the history builders are twins), most likely the
client's own post-processing of its stored summary. The planner host install
is now shared (`js/server/node-host-install.mjs`).

**Nested envelopes (both roots).** A second compaction summarizes the first
summary turn: the envelope nests, its first listed user turn is "What did we
do so far?", and its head trails residue (`… What did we do so far? Title: …
User turns:.`). `compactedAgentTask` / `compacted_agent_task` now read every
`Conversation summary:` envelope, latest first, and take the first listed user
turn that is a standing task (not a continuation cue, not a seeded
`conversation_summary_phrase`, not an envelope), else the head's standing
sentences (`standingSentences` / `standing_sentences`).

**Convergence (both roots).** A member list read that already lists every
requested member is itself the observation: the run answers "already lists …"
at once instead of planning a `cat` that, on a 25 KB file, re-crossed the
CLI's threshold before the final answer. The idempotence pins in
`agentic-write.test.mjs` and `issue_1069_structural_edit.rs` now expect that
final answer.

**Through the CLI (JS server, no limit configured so it compacts at 60,000):**
L01 runs read → write → compaction → continue → read → "`rust/src/web_search_core.rs`
already lists "wikiquote"; nothing needed to change." Before: three
compactions, then a web search for "What did we do so far".

Tests: JS envelope shape, nested envelope, residue head, compacted ladder
convergence; Rust `a_nested_compaction_envelope_still_yields_the_task`,
`a_re_summarized_head_keeps_the_task_sentences`.

### `add3(a, b, c)` — the browser pool was crowded by slot-name chains

`Write a Python function add3(a, b, c) that returns their sum.` gave
`return a + b`. Three browser-composer causes (`js/worker/formal_ai_worker_program_ir.js`):

1. Every fragment's first slot name was treated as a binder, so `{left} + {right}`
   preferred operands mentioning `left`; the slots filled with
   `left + right + left …` chains that never close. The binder preference now
   applies only to slots the idiom binds (`browserBinderSlots`: names between
   `for` and `in`, `lambda` parameters — native `fragment_binder_slots`).
2. Parameter reads now rank ahead of slot-name coherence, both when the pool is
   bounded and when a slot's eight choices are picked.
3. A slot's choices always include every unifying parameter, so covered sums
   (`a + b`, `a + c`, …) cannot crowd out `c`.

Now `a + b + c` / `a * b * c`; `add(a, b)`, `total(items)` and `count_vowels`
are unchanged, and the parity suites (1164 code examples, 1184 derivation,
1163, 1172, 1175, worker mirror) stay green. Pins: JS exact answers for
`add3`/`mul3`; native `a_reduction_over_three_scalar_parameters_reads_all_three`
(uncompiled here). Module budget 372 → 399 with the reason.

### Line deletion — a request that names a line removed only the quoted text

`Delete the line containing '| R56kfQp |' from t.md.` (coordinator, while
using Formal AI as a sub-agent) answered ``Removed `| R56kfQp |` …`` and left the
row's tail ` drop me |` behind.

**Root cause.** The computed removal (`groundedRemoval` /
`grounded_removal`) knew two readings: a line that *is* the payload goes,
otherwise the payload's one occurrence inside a line does. A request that
says *the line containing* the payload fell to the second.

**Fix (both roots).** When the request names a line outside its quotes —
the seeded `line` meaning, which now also carries `lines`, `строки`,
`पंक्तियाँ`, `的行` and `líneas` — every line that contains the payload is
removed whole (`removedLines` / `removed_lines`). Lines that were exactly the
payload keep the old sentence (``Removed `second line` …``, T9 unchanged);
lines that only contained it are stated by the new seeded `line` response
with their count (``Deleted 1 line(s) containing `| R56kfQp |` from `t.md`
and observed the result.``, also ru/hi/zh/es). Without a line named,
`Remove 'drop me' from t.md.` still removes the text inside its line.

**Second gap on the way (destructive).** `t.md से '| R56kfQp |' वाली पंक्ति
हटाओ।` ran `rm t.md`: the Hindi verb closes the sentence, `हटाओ।` was no
whole word, so the removal arm declined and the request reached a shell
translation that deleted the file. The computed-change arms now read their
seeded cues over `sentenceWords` / `sentence_words` (sentence and clause
marks followed by a space set off as spaces; `t.md` keeps its dot). The
shell translation that turned a quoted-payload removal into `rm` is still
open.

**Tests.** JS: "a request that names a line deletes whole lines" (en
singular/plural, ru, hi with `।`, zh, and the in-line removal). Rust:
`rust/tests/unit/pull_request_1188_line_removal.rs` (uncompiled here).

### T1 — a function and its test added to existing ES modules

`Add a function multiply(a, b) to math.mjs that returns a times b, add a test
for it to math.test.mjs, and run node --test to confirm it passes.` was no
longer destructive after round 1, but nothing authored the function: the
browser composer lowered Python only, the fragments had no JavaScript
surface, and no route added a function to an existing module.

**Root causes and fixes.**

1. *No JavaScript surface.* `integer_add` and `integer_multiply`
   (`data/seed/coding-composition-fragments.lino`) now carry `realization` →
   `javascript` blocks, the form native `fragment_catalog` already parsed but
   no seed used.
2. *Python-only lowering.* The browser composer
   (`js/worker/formal_ai_worker_program_ir.js`) composes over the fragments
   realized in the named language and lowers through the seeded
   `{language}_ir_function` template of `coding-discovery-runtime.lino`
   (`javascript_ir_function` is new; the Python `def` now comes from
   `python_ir_function` too). That seed was never in the browser bundle — the
   composer asked for it and got an empty text — so it is registered as a web
   seed (`data/meta/seed-registry.lino`, `js/seed-files.js`), which also gives
   the browser the runtime fragments native already had. A named language
   with a lowering is a synthesis domain (`looksLikePythonFunctionSynthesis`),
   and the answer names and fences the language it synthesized in. Native
   twin: `rust/src/coding/ir_lowering/javascript.rs`, a registered backend that
   renders only seeded `javascript` realizations and names every other node
   as a gap (`javascript_ir_gap`).
3. *No route added a function to a module.* `js/agentic/module_function.mjs`
   (a settled route after the workspace change): a seeded code construct the
   request asks to write or add, a signature, the module (the first path
   after it whose extension the seeded extension table of
   `page-formalization-rules.lino` maps to a language with a test contract),
   and — when the request names a test (`coding_test_artifact_kind`) — the
   test module. It reads both, synthesizes the function from the clause that
   states it through the shared solver, and hands an execution recipe to the
   reroute: the module with the function appended, the test module with the
   name added to its import of the module and one case appended, the
   catalog's check command and the request's own command (`node --test`,
   bounded by the seeded run verbs, shell tokens, function words and clause
   separators). The test's expected value is the **specification** evaluated:
   the words after the seeded return action (`a times b`), parameters bound to
   the contract's sample arguments (`2 times 3`), computed by the calculator
   (`6`) — never the synthesized code run against itself. The import line,
   test case, header and samples are `data/meta/function-test-contracts.lino`.
   A specification the calculator cannot evaluate declines the route.

**Tests.** JS: "a function and its test are added to existing ES modules (T1)"
(the six-call transcript and exact files and answer; the parsed request and
two declines; the browser composer's JavaScript answer); the round-1 guard now
expects `math.mjs` to keep `add` and gain only `multiply`. Rust:
`rust/tests/unit/pull_request_1188_module_function.rs` (the IR lowers to the
ES module function; a Python-only fragment is a named gap; uncompiled here).
Worker budgets: `program_ir` 399 → 425, `07` 1393 → 1394, with reasons.

**Still open.** The agentic route has no native twin yet (JavaScript first).
`their sum` is not evaluable by the calculator, so `add(a, b) that returns
their sum` with a test declines. `write a program that prints Hello, World!
and run it` (no language, no file) is still a web search: nothing chooses a
language for a request that names none.

### Safety — an edit-shaped request never becomes a file deletion (round 5)

`t.md से drop शब्द हटाओ।` ("remove the word drop from t.md") ran `rm t.md` and
deleted the file: no edit step grounds an unquoted word, and the seeded `rm`
intent's Hindi cue is the bare verb `हटाओ`, so the shell translation composed
the deletion.

**Fix (both roots).** The `rm` and `rmdir` intents of
`data/seed/shell-intents.lino` are marked `destructive true` (read into
`ShellIntent::destructive` / `intent.destructive`; no command list in code).
A destructive intent is never composed when the request edits inside a file
(`editsInsideAFile` / `edits_inside_a_file`): it quotes a payload that is not
a path, or names a line (the seeded `line` meaning) or another unit of text
(the new seeded `file_text_unit` meaning: word, phrase, occurrence, text,
слово, शब्द, 文字, palabra…) outside its quotes. The planner then declines
honestly with the seeded `file_text_unit` sentence naming the file
(`destructiveEditDecline` / `destructive_edit_decline`, ahead of the shell
cascade), in five languages. Every other caller of the shell translation
(the capability router's shell lowering, the semantic intent reader) goes
through the same guarded intent reader; the shared solver and the shell
composer answer these requests without a command.

**Held-out probes** (in-process driver and real sandbox files; the file keeps
its bytes): `Delete the file t.md word drop.`, `Удалить файл t.md слово drop`,
`t.md से drop शब्द हटाओ।`, `删除文件 t.md 里的文字 drop`. A request that
plainly deletes the file is still a deletion: `Delete the file t.md`,
`Удали файл t.md`, `t.md हटाओ`, `删除文件 t.md` plan `rm t.md`.

**Tests.** JS: "an edit-shaped request never becomes a file deletion". Rust:
`rust/tests/unit/pull_request_1188_destructive_edit.rs` (uncompiled here).

### Round 5 — one output binding, a seeded default language, named relations

**Rung 5: both roots bind the same unquoted output.** Rust's coding task
specification (`coding/task_spec.rs`, `extract_expected_stdout`) read a
prefix slot form ("prints …") to the end of the request, so "write a Python
program that prints Hello, World! and run it" expected the stdout
`Hello, World! and run it`, and "prints the sum of a and b" expected the
description itself. A prefix form now reads its span the way the obligation
graph does (T18): a quoted payload right after the form, else the words up to
the first seeded clause separator or sentence end, when they open with a
capital and do not describe a value (no `statement_function_word`, no
`coding_structure`, no `program_task_alias` other than a `social_greeting`).
The rule lives once, in `intent_formalization::unquoted_utterance` and
`describes_a_value`; `unquoted_output` and the task specification both call
it. A circumfix form ("打印 … 的") keeps its own closing marker but no longer
binds a described value. Pinned by
`the_task_specification_binds_the_same_output` in
`rust/tests/unit/pull_request_1188_unquoted_output.rs` with the same accepted
and rejected probes as the JavaScript T18 test (uncompiled here: no cargo).

**Rung 1: a program the request asks to run gets a seeded language.** "write
a program that prints Hello, World! and run it" named no language, so the
program contract declined and the planner fell through to a web search. The
seed had no default program language (the `default_language` of
agent-info.lino is the reply language). `data/meta/stdout-program-contracts.lino`
now carries `unnamed_language python` with its stated reason: the run is the
agent's own obligation, so the program must be written in something runnable,
and python is the catalog's first row, covered by every stdout, function and
test contract, and needs no build step. It applies only when the request also
asks to run the program (the seeded `software_followup_execution` role:
"run it", "запусти", "चलाओ", "运行"); a request that only asks for a program
still asks which language (issue #906). Both roots: `unnamedProgramLanguage`
/ `unnamed_program_language`. Probes (en, ru, zh) write `main.py` and check
its output; pinned in the dogfood test and
`rust/tests/unit/pull_request_1188_unnamed_language.rs`.

**Rung 3: "returns their sum".** The T1 route computed the expected value
only from operands after the return verb ("a times b"), so "returns their
sum" gave no value and the request fell to the generated-source arm, which
wrote `main.js`. The expected value now also comes from the arithmetic
relation the specification names: the `coding_fragment` records of
`data/seed/coding-composition-fragments.lino` whose idiom is
`{left} <operator> {right}` and the reductions they `supports` (`integer_add`
supports `reduce_sum`: sum, сумму, योग, 总和; `integer_multiply` supports
`reduce_product`: product, произведение, गुणनफल, 乘积), so the calculator gets
`2 + 3`. Running the same request in ru, hi and zh exposed three more gaps,
all fixed. (1) A relative clause after a comma ("…, которая возвращает их
сумму") and fullwidth or danda punctuation now bound the specification
clause. (2) The module may precede the signature (SOV): the module is now
the path the signature's own clause names. (3) The worker read a function
name only after the English "function " marker; it now also reads a
signature that a seeded `program_synthesis_subject` word names (функцию,
फ़ंक्शन, 函数), as the native `inline_function_signature` already does. Hindi
"लौटाता" joined the seeded `synthesis_action_return` surfaces. The module
path is left out of the clause the solver synthesizes from, so a Hindi
clause that opens with `math.mjs` is no longer read as a page to open.
Pinned by the T1 test "a relation the request names …" (en sum/product, ru,
hi, zh). "difference" is still open: the seed has no subtraction relation
or fragment, and adding one would enter the composition search pool.

### Round 6 — the native T1 arm, "difference", verb-final output, run verbs

**Native twin of the T1 route.** `rust/src/agentic_coding/module_function.rs`
mirrors `js/agentic/module_function.mjs` function by function, and each Rust
doc comment names its JS original. The arm is named `module_function` in
`data/seed/planner-precedence.lino`, in `PLANNER_ROUTE_ARMS` and in the JS
list, after `workspace_change` in both cascades. To keep `planner.rs` under
its 1000-line cap, the route-arm table and its seed join moved whole into
`planner/precedence.rs` (re-exported, so `checked_route_precedence` keeps its
path).

One step cannot mirror the browser: the JS arm synthesizes through the
worker's composer, which the native solver lacks. The native arm instead
takes the one seeded binary `coding_fragment` whose idiom meets the
specification at every sample pair of the contract (2 3, 4 5, 6 7). It
lowers that fragment through the language's IR lowering. A specification
that two operations meet, or none, declines.

Pinned by three new tests in `rust/tests/unit/pull_request_1188_module_function.rs`:
- the planner drive (read, read, write, write, bash, bash);
- the request reading, which equals the JS one;
- the relation probes in en, ru, hi and zh.

None of the Rust here was compiled: cargo is not available in this
environment, so CI is the first compile.

**"difference".** The seed gained:
- the `integer_subtract` fragment (`{left} - {right}`, JS realization, with
  full metadata);
- the `arithmetic_difference` coding structure (en "their difference",
  "difference of"; ru разность, разницу; hi "उनका अंतर", "का अंतर"; zh 差值,
  之差; es "su diferencia", "diferencia de").

The surfaces avoid the bare word "difference". That word already belongs to
`predicate_abs_diff_lt` ("difference is less than"), `absolute_deviation`
and the research-table criterion, so using it bare would add a structure to
those requests.

The JS synthesis suites keep their programs: 0703, 0921, 1163, 1164, 1175,
1177, 1184, 991, r1017 and worker-mirror all pass. The Rust composition
suites could not be run here.

The Hindi probe ("जोड़ो जो उनका अंतर लौटाता है") at first produced
`a - a + b`. The member-add verb जोड़ो canonicalizes to the sum operation, so
the composer saw `reduce_sum` as well as the difference. The fix is in the
clause the arm hands to the solver: it now drops the seeded
`coding_member_add_action` words. That verb says where the function goes,
not what it computes. The fragment pool was left alone.

**Verb-final output binding.** `'एक प्रोग्राम लिखो जो "Namaste" प्रिंट करे
और उसे चलाओ'` bound no output, because the print verb follows the quoted
value. The seed already records word order: `verb_final` of
`data/seed/formal-targets.lino`, read natively by `verb_final_language`. A
quoted literal in a verb-final request is now bound when the words after it,
up to the sentence break or the first seeded clause separator, evidence
`print_stdout`.
- Both roots: `followedByPrint` / `followed_by_print` in `boundOutputLiterals`
  / `bound_output_literals`.
- A value that a separator parts from the verb is not bound
  (`'"a.txt" बनाओ और "b" प्रिंट करो'` binds only `b`).
- The Hindi program request now writes `main.py` with `print("Namaste")`.

**Non-English run verbs.** запусти, चलाओ and 运行 were already seeded as
`run_verbs` / `cjk_run_verbs`. The T1 reader matched words only through the
ASCII-only `normalizeCommandWord`, so it never saw them. It now also matches
the bare word in any script and a word ending in a CJK run verb (然后运行). A
verb-final clause ("node --test चलाओ") takes the command from the shell token
up to the verb. The request's own `node --test` now runs in ru, hi and zh.

### T23 — a single-quoted replacement payload was read as a computer-use request

**Root cause.** The computer-use route (`computerUsePlanAgenticStep`, tried
before every edit arm in `planChatStepRoutes`) recognises a resource and an
operation on the request's *instruction surface*. `instructionSurface`
(`js/agentic/crate/computer_use_lexicon.mjs`, twin `instruction_surface` in
`rust/src/computer_use/lexicon.rs`) removed indented payload lines and only
double-quoted spans, so the single-quoted payloads of `Replace 'X' with 'Y'`
stayed in. A payload that happened to say "precedence order" and "the built
`formal-ai --help` list" evidenced the seeded `computer_use_resource_orders`
resource and the list-directory operation, the first step mapped onto the
Agent CLI's `read`, and the second step's `http.fetch` failed verification.

**Fix (general).** The instruction surface now drops every quote pair the
shared quote reader knows (`textOutsideQuotedSegments` /
`text_outside_quoted_segments`: single, double, backtick, guillemet and CJK
quotes, with in-word apostrophes such as `system's` left alone). A request
written in ordinary prose is unaffected; the seeded computer-use suites still
pass. Tests: `rust/tests/web/pull-request-1188-quoted-payload-cues.test.mjs`
and `rust/tests/unit/pull_request_1188_quoted_payload_cues.rs` (the edit is no
plan; an unquoted computer-use request still plans).

### T27 — a removal that names functions instead of quoting text

`Delete the functions algorithmDetectLanguage, algorithmSortingAnswer and
handleAlgorithm from formal_ai_worker_code_plans.js.` (coordinator, while
using Formal AI for the issue #918 algorithm migration) read the file and
answered with the read output; the file was unchanged. The singular
`Delete the function try_algorithm from mod.rs.` did the same.

**Root cause.** Every computed removal (`groundedRemoval` /
`grounded_removal`) needed one quoted payload. A request that names
declarations by their identifiers had no arm, so the planner fell through to a
bare read.

**Fix (both roots).** `groundedDeclarationRemoval` /
`grounded_declaration_removal` takes the seeded `coding_text_remove_action`
with the new `coding_declaration_noun` (function, functions, method, функцию,
फ़ंक्शन, 函数, función …), no quoted text and one named path. The names are
the request's identifiers that the file itself declares as functions: a
seeded `function_declaration_keyword` (`function`, `fn`, `def`, `func`,
`fun`) right before the name, then `(` or `<`. Each declaration goes whole: a
braced body to the first later line that closes at the header's indentation,
an indented body (a header ending in `:`) to its last deeper line, with the
comment, doc-comment and attribute lines directly above it; a block left
between two blank lines takes one of them, and a block that ended the file the
blank line before it. A name declared nowhere, or more than once, is left
alone; if no name is declared the change is not verifiable and the seeded
failure is stated. The answer is the seeded `coding_text_remove` sentence
naming the removed functions (``Removed `double`, `triple` from `util.js` …``,
also ru/hi/zh/es).

**Tests.** JS: `rust/tests/web/pull-request-1188-declaration-removal.test.mjs`
(one name with its JSDoc, several names with one undeclared, a Rust function
with its doc comment and attribute in Russian, an undeclared name refused).
Rust: the twin cases appended to
`rust/tests/unit/pull_request_1188_line_removal.rs` (uncompiled here). After
the fix the same requests deleted the three worker functions and the three
Rust functions of the algorithm migration on the first try.

### Open — three edit shapes found while delegating the R383 server edits

Found while Formal AI made 41 single-line edits for the R383 debug session
(`js/server/*`, `rust/src/server*`, the route manifest, the parity corpus).
Each needs the positional-edit arm (`js/agentic/positional_edit.mjs`, twin
`rust/src/agentic_coding/positional_edit.rs`). Another agent was editing
that arm at the same time, so these three were worked around and are still
open:

- Two inserts in one request, joined by `, and insert …`, went to the
  capability router's read-many fallback (`cat '/debug-session.mjs'
  '/conversations.mjs'`) and made no edit. Each insert works on its own.
- `insert these lines after the line '…':` followed by an indented
  multi-line block got no plan. Seven single-line inserts, sent in reverse
  order, worked. Fixed since (T38): the block is the text inserted.
- A double-quoted line payload that itself contains double quotes
  (`Insert the line "    body '{"token":"p"}'" after the line "    path
  '/v1/x/learn'" in r.lino.`) read the inner `/v1/x/learn` as the target
  file, a sandbox escape. A variant went to web search. The same request
  with backtick-delimited lines works. The escape is fixed (T36): the read now goes to the named file.
