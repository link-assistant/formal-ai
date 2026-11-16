# Issue #1562: a work item that only cites a Hello World repository received a Hello World program

Issue: <https://github.com/link-assistant/formal-ai/issues/1562> (parent #1561)

Pull request: <https://github.com/link-assistant/formal-ai/pull/1578>

## 1. Collected data

| Path | Content |
| --- | --- |
| `../../../data/benchmarks/fixtures/issue-1562/hive-mind-{4031,4034,4035}.txt` | The texts of hive-mind issues 4031, 4034 and 4035 (title, blank line, body), as `gh issue view` returned them to the sessions |
| `../../../data/benchmarks/fixtures/draft-replays/hive-mind-{4031,4034,4035}-cited-program.json` | The recorded chat request of each production session: the request after the issue fetch, which got the program back. Built by `experiments/issue-1562/draft-replay-of-log.mjs` |
| `raw-data/tool-calls-pr-{4037,4036,4039}.txt` | Every completed tool call of the three production sessions (`experiments/issue-1562/tool-calls-of-log.mjs`) |
| `raw-data/hive-mind-pr-comments.txt` | The comments the sessions posted on hive-mind PRs 4036, 4037 and 4039 |
| `raw-data/native-solver-0.355.27.txt` | The released native solver's answer to each whole text and to its obligation text |
| `raw-data/native-solver-binding-0.355.27.txt` | The released native solver's answer to each binding text (§4), and to the two work items whose output literal is a code span |

The three solve logs:

- 4031 → PR 4037: <https://github.com/link-assistant/hive-mind/blob/b02b8672bcc312ae11147c72e7994a6e59dddded/dev/log/issues/4031/pulls/4037/sessions/ses_e3ae4c9daffeMVQj8JSKbBAtRt/solve.log>
- 4034 → PR 4036: <https://github.com/link-assistant/hive-mind/blob/aa5438fb02bb7f6dbd26f0246b69025a0dc9831a/dev/log/issues/4034/pulls/4036/sessions/ses_e3ae51356ffeAeqIhwX14ZvCIJ/solve.log>
- 4035 → PR 4039: <https://github.com/link-assistant/hive-mind/blob/e2e7af7d435a9fdc37157604d956061d3ddabf60/dev/log/issues/4035/pulls/4039/sessions/ses_e3ae34faaffeIkkFRYMEMJp5kW/solve.log>

## 2. What the sessions did (step 1 of the issue)

None of the three issues asks for a program. Each is a bug report about Hive Mind. Each cites its evidence: test repositories named `konard/test-hello-world-019f…`, gist addresses, file names in code spans, tables of log lines and `js` fences.

| Issue → PR | Written | Run | Then |
| --- | --- | --- | --- |
| 4031 → 4037 | `main.rs` (`println!("Hello, world!")`) | `rustc main.rs -o main`, `./main` | Committed `99a61ab8`, edited the PR body to "Verified artifact: main.rs". `gh pr ready` failed with exit code 1 |
| 4034 → 4036 | `main.js` (`console.log("Hello, world!")`) | `node --check main.js`, `node main.js` | Committed `59760350`, edited the PR body, `gh pr ready` marked the PR ready. Summary: "Created and verified `main.js` through the agentic CLI harness." |
| 4035 → 4039 | `Main.scala` | `scalac`, then the prerequisite installer, then `scala run Main.scala` | `scala: command not found` (exit code 127). The PR stayed empty |

The tool calls are in `raw-data/tool-calls-pr-*.txt`. In each session, the first call after the issue fetch is the workspace probe for the program file (`"absent": true`).

## 3. The selecting rule

The native work-item route (`plan_work_item_execution`, `rust/src/agentic_coding/general_execution.rs`) solved the **whole** fetched text: `UniversalSolver::default().solve(objective)`.

1. `normalize_prompt` (`rust/src/engine.rs:668`) turns every non-alphanumeric byte into a space, so the address `…/konard/test-hello-world-019fb331-…/pull/2` reads as "… test hello world 019fb331 …".
2. `write_program_parameters` (`rust/src/intent_formalization/write_program_request.rs:81`) calls `crate::coding::program_task_by_alias(normalized)`. That call finds the catalog alias "hello world", which selects the task `hello_world`.
3. The language is taken from the rest of the text.
   - 4031: the prose says "The Rust log …", which gives Rust.
   - 4034 and 4035 (released 0.355.27, run on the fixture text): the `js` fences give JavaScript.
   - In production, the 4035 session also fetched the issue page with `webfetch`. Its text names the workflow step "Run Main.scala", which gave Scala.
4. The command reroute (`plan_symbolic_command_reroute_for_objective`) wrote the recipe's file, ran its commands, committed it and marked the PR ready. It answered "Created and verified".

To reproduce with the released binary:

```bash
node experiments/issue-1562/obligation-texts.mjs /tmp/obl
for f in /tmp/obl/*.txt; do
  HOME=/tmp/fa-home formal-ai chat --silent --format chat --prompt "$(cat "$f")" > "${f%.txt}.json"
done
```

| Text | Whole text (`*.full`) | Obligation text (`*.obl`) |
| --- | --- | --- |
| hive-mind 4031 | Rust Hello World, `rustc main.rs -o main`, `./main` | no program (a sequence analysis of the line numbers) |
| hive-mind 4034 | JavaScript Hello World, `node main.js` | no program (a sub-task list of the report) |
| hive-mind 4035 | JavaScript Hello World, `node main.js` | no program |
| Implement Hello World in Kotlin | `Main.kt` | `Main.kt`, identical |
| Implement Hello World in Rust | `main.rs` | `main.rs`, identical |
| Implement Hello World in Scala | `Main.scala` | `Main.scala`, identical |

The JavaScript runtime does not reproduce the write. Its work-item solver (`programContractAnswer`, `js/agentic/crate/coding_program_contract.mjs`) selects no recipe for the three texts (`experiments/issue-1562/js-solver-recipes.mjs`). The JS sessions of the three texts only read and glob the cited files, and they end "Not completed: … `UnresolvedReferences`". This is true on `main` and on this branch.

## 4. The fix

1. **Formalize first.** `obligation_text` (`rust/src/agentic_coding/work_item_citations.rs`, twin `js/agentic/work_item_citations.mjs`) keeps the work item's own words. It replaces every citation span with spaces of the same byte length, so every byte span of the obligation text is the span of the same words in the work item. The citations are:
   - fenced code blocks, block quotations and table rows;
   - inline code spans;
   - Markdown links and images, and autolinks;
   - bare addresses (`…://…`, `www.…`).

   A quoted phrase inside a sentence ("a \"Hello World\" program") is the work item naming what it asks for, and is kept.
2. **Bind every write to an obligation.** `unbind_cited_recipe` / `unbindCitedRecipe` (in the same `work_item_citations` modules) keeps a recipe only if the work item's binding text selects the same file in the same language.
   - The binding text (`binding_text` / `bindingText`) is the obligation text with one kind of code span kept: a span that is the **operand of an obligation clause**. These are the output literals that a print clause binds (`bound_output_literals` / `boundOutputLiterals`, the operands the program contract reads), as in "The program should print exactly: `` `Hello, World!` ``". The span is no obligation by itself, but the clause that binds it is one, and the binding reads the clause whole. Any other code span, such as a cited `` `konard/test-hello-world-01` ``, stays blank. Otherwise the recipe is dropped before the command reroute sees it, both on the work-item route and on the recovery of an owned recipe.
   - The evidence record gets `unboundWrite: {path, reason: "NamedByNoObligation"}` and the remaining work `unbound-write:<path>`.
   - The summary says "`main.rs` was not written: no obligation of the work item names it…" (seeded in five languages, intent `work-item-report-unbound-write`).
   - With no recipe, nothing is run, committed or marked ready, so "Created and verified" cannot be reached.
3. **Requirement rows that only cite** (a table row, a fence) get `status: "citation"` in the evidence record. The report lists only the obligations that are not yet verified (rows keep their numbers), and "all verified" counts obligations only.

**Why operands are kept (a regression found while checking #1162).** The first version solved the plain obligation text. That broke `rust/tests/web/issue-1155-read-validation.test.mjs`: 2 of its 10 tests failed on the branch and passed on `main`. Its Hello World work item gives its output only as a code span ("print exactly: `` `Hello, World!` ``"). With the span blanked, the JavaScript contract solver found no output literal and selected nothing, so the recipe `main.rs` was unbound and never written. The native solver has the same flaw for programs outside the catalog. For "Write a Python program that prints exactly: `` `Goodbye, moon` ``", the whole text gives `main.py`, but the obligation text gives "I cannot write this program". Even for Rust Hello World, the native obligation text gives a template that prints `Hello World`, not the requested `Hello, World!` (`raw-data/native-solver-binding-0.355.27.txt`).

The binding text fixes both runtimes:
- the #1155 work item and the Python one keep `main.rs` and `main.py`;
- the three reports still bind nothing.

The reports do have operands: 4031 `No progress`, 4034 `--tool codex`, 4035 `gh run view <run-id> --job <job-id> --log-failed` and `name — html_url` (`experiments/issue-1562/bound-outputs.mjs`). None of them names a program. The released native solver gives the same answers for the binding texts of the reports as for their obligation texts (a sequence analysis, a sub-task list, an event), and Hello World programs for the Kotlin, Rust and Scala controls.

Each fixture has 3 citation rows: 4031 has 39 requirement rows, 4034 has 21 and 4035 has 29 (`experiments/issue-1562/requirement-citations.mjs`).

The binding decision is traced as route `work_item_recipe_binding` (a JSON value `{path, language, bound}`). The trace is off by default and is enabled by `FORMAL_AI_TRACE_REQUESTS`.

## 5. Tests

- `rust/tests/unit/agentic-coding/issue_1562_cited_programs.rs` replays the recorded Agent CLI request through `create_chat_completion_with_solver` with each fixture text. It runs with the read, glob and `find` emulation of `issue_1558_reference_resolution.rs`, which panics on any other action. It checks that:
  - the sessions write, run and commit nothing;
  - the summary names the defect (the title) and the unbound `main.*`;
  - the record marks citations;
  - the Scala and Rust Hello World items keep their recipe;
  - a code span that is the operand of a print clause is read with its clause: the #1155 work item and a Python program keep `main.rs` and `main.py`, a cited code span stays blank, and the binding texts of the reports select no program;
  - a resumed session that already wrote and ran `main.rs` neither commits, pushes nor runs `gh pr ready`, and claims nothing;
  - the unbound-write paragraph is seeded in every registered language (en, ru, hi, zh, es).
- `rust/tests/web/issue-1562-cited-programs.test.mjs` is the JS twin. It covers the obligation text, `isCitation`, `unbindCitedRecipe`, the binding text and its operands (the #1155 and Python work items), the Kotlin, Rust and Scala controls, the evidence record and report, the seeded paragraph in every language, the three sessions and the resumed session.
- `rust/tests/web/draft-request-replay.test.mjs` replays the three production requests (family `work-item-cited-program`, step 6 of the issue).
- The #1162 Hello World tests stay green: `rust/tests/web/issue-1155-read-validation.test.mjs` passes 10 of 10 again.

## 6. Limits

- **The Rust tests of this change were not executed before merge.** Compiling the `formal-ai` library for the test build is killed by the 3 GB memory limit of the machine this was written on. This happened at `opt-level=2`, at `opt-level=0`, and at `opt-level=0` with 256 codegen units. The pull-request lane of CI does not build the Rust tests either: the `test`, `doc-tests` and coverage jobs of `CI/CD Pipeline` and `Coverage` run on `main` only. So `issue_1562_cited_programs.rs` first runs on `main` after the merge. What was checked instead:
  - `cargo check --lib --tests -j 1` passes, as does the PR lane's `clippy` over the library and binaries.
  - `experiments/issue-1562/rust-obligation-twin.sh` compiles the crate-independent half of `work_item_citations.rs` (`obligation_text`, `obligation_text_keeping`, `is_citation`) on its own. Its obligation texts and binding texts equal the JavaScript twin byte for byte on 8 work items: the three fixtures, the three #1364 Hello World work items, the #1155 work item and the Python one. The operands come from the JavaScript twin, because `bound_output_literals` needs the crate.
  - The released native solver selects no program for the three obligation and binding texts, and the same program for the controls (`raw-data/native-solver-0.355.27.txt`, `raw-data/native-solver-binding-0.355.27.txt`). That is the comparison `unbind_cited_recipe` makes.
- `rust/tests/web/work-item-boundary-route.test.mjs` fails 4 of its tests locally (`BoundaryGrammarUnavailable`) on both the base commit `31f631973` and this branch, so the cause is the environment, not this change.
- The issue carries the `no-formal-ai-draft` label. Removing it and dispatching `Hive Mind draft` is left to the maintainer.
