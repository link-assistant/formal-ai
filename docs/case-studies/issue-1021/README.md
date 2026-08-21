# Issue #1021 — the full range, and the paperwork that lets it land

Issue: <https://github.com/link-assistant/formal-ai/issues/1021>
Pull request: <https://github.com/link-assistant/formal-ai/pull/1027>

One deliverable, one pull request: the reported behaviour range and the process
artifacts a contribution carries, each covered by a test that would fail if the
fix had been written for the reported wording alone.

## 1. Collected data

`raw-data/`, `logs/` and `closed-circle-run/` preserve everything the analysis
below is built on.

| File | Origin | Content |
| --- | --- | --- |
| `raw-data/github/issue-1021.json` | GitHub API | The umbrella issue body and metadata |
| `raw-data/github/issue-{868,867,866,865,863,862,824,723}.json` | GitHub API | The reported behaviour range, one file per sub-issue |
| `raw-data/github/issue-{943,944,946,947}.json` | GitHub API | The blocked-by capability issues (E91, E92, E94, E95) |
| `raw-data/github/issue-924.json` | GitHub API | E77, the closed issue whose requirement is carried forward here |
| `logs/php-laravel-before.log` | `cargo run --example issue_1021_php_laravel` on `main` | The PHP/Laravel answers before the change |
| `logs/php-laravel-after.log` | the same example on this branch | The same prompts after PHP was catalogued |
| `logs/php-numeric-list-generation.log` | `cargo run --example issue_1021_php_numeric_list` | Numeric tasks rendered in PHP |
| `logs/php-numeric-list-verification.log` | `php -l` and `php` over the rendered programs | The toolchain check behind the "compiles and runs" claim |
| `logs/named-exercise-routing-after.log` | `cargo run --example issue_1021_named_exercise_probe` | Where the #862 and #863 prompts land now, prose and URL alike |
| `logs/copy-stdin-harness.log` | `cargo run --example issue_1021_copy_stdin_harness` | Each answer's program written out, compiled and run with the fixture piped in |
| `logs/languageless-request-after.log` | `cargo run --example issue_1021_languageless_probe` | A coding request with nothing but the artefact named, and the four subjects that must not be one |
| `logs/languageless-followup.log` | `cargo run --example issue_1021_languageless_followup` | The same request answered from the catalog once the follow-up turn supplies the missing half |
| `logs/spanish-code-boundary-before.log` | `cargo run --example issue_1021_spanish_code_boundary` before the fix | Every Spanish request naming `código` read as a request for C |
| `logs/spanish-code-boundary-after.log` | the same example after it | The same prompts once word boundaries stopped being an ASCII question |
| `logs/bounded-recovery.log` | `cargo run --example issue_1021_bounded_recovery` | A run stopped by its own limit, a candidate version that does not compile and is rolled back, and a delegated choice |
| `experiments/issue-1021-laravel/` | `composer create-project laravel/laravel`, `php artisan` | The real application the Laravel template was verified inside, and the browser-mirror check |
| `logs/macos-timeout-not-found.log` | run 32282461075, jobs 96170638546 and 96170638704 | The macOS red that `scripts/run-with-deadline.sh` answers |
| `logs/deadline-precision-measurements.log` | `experiments/issue-1021-deadline-precision/compare-clocks.sh` | Three drafts of the deadline against the same 3s budget |
| `logs/macos-deadline-tests-green.log` | run 32294252392, all sixteen macOS core slices | The same eleven tests passing on the runner family that reported the gap |
| `logs/apt-mirror-outage-every-attempt.log` | run 32294252072, job 96206598860 | An outage the per-attempt deadline retries and still cannot rescue |
| `closed-circle-run/input.json` | authored with the change | The half of the session a replay cannot derive |
| `closed-circle-run/session.json` | `cargo test --test unit -- issue_1021_closed_circle` | The replayable capture of the whole circle |
| `closed-circle-run/pull-request-body.md` | `formal_ai::contribution_artifacts::compose` | The body this pull request uses, composed rather than typed |
| `experiments/issue-1021-php/` | `php -l`, `php` | The scratch programs the PHP templates were verified with |

## 2. Timeline

| When | What |
| --- | --- |
| 2026-06 … 2026-08 | The behaviour range is filed one prompt at a time: #723, #824, #862, #863, #865, #866, #867, #868 |
| 2026-07 | #924 (E77) asks for one real repository change per release, "landing as a normal reviewed pull request", and is closed without one landing |
| 2026-08 | The hive-mind series closes; its standing instruction — solve by generalization (hive-mind#2119) — becomes the frame for this issue |
| 2026-08-18 | PR #1019, a four-line CI fix, takes six iterations to go green: evidence that the gates, not the code, are what an unattended harness runs into |
| 2026-08-19 | This branch: the routing rules fixed where they were wrong, PHP catalogued, the contribution artifacts and the write-path ladder added, the circle captured as a replayable session |
| 2026-08-20 | The review's work list: the injected clock and the rollback proof for #946/#947, `copy stdin to stdout` end to end, Laravel as its own target, a coding request that names no language, and the mutating rungs `824.L1`-`824.L5` with sandbox reset for #944 |

## 3. Requirements

The full list, with IDs, lives in
[`docs/requirements/issue-1021-full-range-coding-and-contribution-artifacts.md`](../../requirements/issue-1021-full-range-coding-and-contribution-artifacts.md)
(assembled into `REQUIREMENTS.md` as R1021-1 … R1021-31) and each row has a
traceability entry in
[`docs/requirements-traceability.md`](../../requirements-traceability.md).

In short: seven reported prompts must be answered correctly (R1021-3 … R1021-9),
by fixing the rule rather than the prompt (R1021-2); a contribution must carry a
changelog fragment and a linked pull-request body (R1021-15, R1021-16); the
commands that publish it must sit on a ladder that refuses by default, with
`gh issue create` refused in both states (R1021-10, R1021-11); and all of it must
be test-covered, including the closed circle as a replayable session (R1021-18 …
R1021-21). Two requirements are reported **not delivered** — see §9.

## 4. Root causes

Each wrong answer had a different rule behind it, and every one of those rules
had been written for the phrasing that was in front of it.

1. **A bare `ls` was refused (#868).** `bare_shell_tokens` listed only the
   multi-word tools, so a command with no arguments did not read as a request at
   all.
2. **`Execute ls command` ran `ls command` (#866, #867).** The passthrough
   short-circuited as soon as it saw a known head token and handed the rest over
   as arguments, so the noun *naming* the command became an argument to it —
   `/bin/ls: cannot access 'command'`.
3. **`List me files here` reached web search (#865).** The detector matched an
   array of complete English phrases. *"list files"* was in it, *"list me files
   here"* was not, and no array of phrases ever converges — every new way of
   asking is a new entry, in one language.
4. **`copy stdin to stdout in Rust` ran `cp stdin stdout` (#863).** The `copy`
   cue accepted any word as an operand, so two words that merely follow the verb
   became two paths.
5. **A Rosetta Code URL became `cp _stdin_to_stdout Rust` (#862).** The same
   unanchored cue, plus a passthrough that never asked whether the head token
   was a program at all — `https://…` is a resource to act on, not a command.
6. **A filesystem move matched nothing (#824).** `is_safe_path` rejected every
   absolute and `~`-relative token *before* any policy saw it. Since every
   operand is recovered verbatim from the user's own words, that never stopped
   Formal AI from reaching outside the workspace; it only stopped the user from
   saying where.
7. **A PHP request fell to the uncatalogued-language fallback (#723).** PHP
   existed only in the coding oracle — the reviewed-snippet path for languages
   the catalog does not template — so no parametric task route could render it.
8. **A named task was answered with a hello world.** Found while covering the
   above and older than it: `data/meta/cue-lexicon.lino` hoists
   `handler:write_script` to the front of method selection on the bare tokens
   *script* and *code*, jumping `data/seed/handler-precedence.lino` (where
   `numeric_list` is line 29 and `write_script` line 43). So *"Sort the numbers
   3, 1, 2 in Python, write me the code"* returned a Python hello world — on
   `main`, with the plain verb *write*.
9. **A listing verb inside a longer word panicked.** `mentions` advanced the
   scan by one byte after a non-boundary match, landing mid-character on
   `создай`. Any Russian or Hindi prompt containing such a word aborted the
   process. Fixed in `279b7199`; it was already failing on `main`.

Behind the process half of the issue there is one cause and it is simpler: the
solver did not know what a changelog fragment or a pull-request body looks like,
and nothing decided whether a command that publishes a change may run at all.
Issue #943 records what the second gap costs — five issues filed that nobody
asked for (#784, #786, #789, #790, #791).

## 5. Research and prior art

| Need | Options considered | Decision |
| --- | --- | --- |
| Changelog fragments | `towncrier` (Python), `changesets` (JS), `cargo-release` | Not adopted: `scripts/check-changelog-fragment.rs` already fixes the shape, and adding a Python or JS toolchain to render a six-line file would be a dependency for a formatting step. The shape is mirrored into seed data instead; the frontmatter-plus-heading convention is taken from the same prior art. |
| Pull-request body templates | `gh pr create --template`, `.github/PULL_REQUEST_TEMPLATE.md` | Not sufficient alone: a template is a blank form, while the gate checks the content — a closing keyword and the issue URL. The composer fills the form; the template idea survives as the seeded section list. |
| Mutating-action policy | `sudoers`, OPA/Rego, `deno --allow-*` | Not adopted: all three are policies over a process boundary, while the decision here is over a command string Formal AI is about to run on its own behalf. Seed-declared action lists keep the policy readable next to the artifacts it governs. |
| PHP parsing/execution | `php-parser-rs`, `tree-sitter-php` | `meta-language` already ships the tree-sitter PHP grammar the CST checks use, so PHP needed a grammar record in `data/seed/program-cst-grammars.lino`, not a new dependency. |
| Grounding the new vocabulary | `wn` (Python, OEWN 2024) | Adopted — it is what `scripts/ground-wordnet.py` already drives. *need* and *want* were grounded through it so the total closure stays resolved. |

Two findings came out of the research rather than the code:

- **Links Notation has no comment syntax.** Confirmed against
  `links-notation 0.13.0` with `examples/issue_1021_lino_comment_probe.rs`: a
  bare `:` inside a `#` prose line fails to parse (`code: Eof`). Seed prose
  written for this change avoids that character in `#` lines.
- **The reported Rosetta Code address resolves to a task page**, which is what
  makes "a web address in head position is a resource, never a program" the
  right general rule rather than a special case for one URL.

## 6. Tests-first reproduction

Every fix below was written against a failing test, and each test is stated so
that it fails for the *class* of request rather than for the reported string:

- `tests/unit/issue_1021_behaviour_range.rs` reproduces all seven prompts and
  then holds out paraphrases the fix has never seen — different word orders
  (`a_prose_listing_request_routes_to_ls_in_any_word_order`), different
  languages, different commands in the same shape
  (`a_command_naming_noun_is_stripped_for_every_command`), and negative cases
  that must keep failing (`listing_parts_alone_do_not_make_a_listing_request`,
  `a_traversing_move_is_not_performed`, `an_asking_verb_alone_is_not_a_coding_request`).
- `tests/unit/issue_1021_contribution_artifacts.rs` drives the generator and
  checks its output against the gates' own rules, including
  `the_generator_composes_prose_without_containing_any` — the R379 property.
- `tests/unit/issue_1021_write_path.rs` exercises both opt-in states from one
  process, so "refused by default" and "permitted under opt-in" are the same
  build.
- `tests/unit/issue_1021_closed_circle.rs` replays the whole loop.

## 7. Implemented fix

**Routing, by generalization.** `src/agentic_coding/directory_listing.rs`
recognises a listing request from the *parts* it is composed of — a listing verb
or question word, a noun naming what is listed, and a phrase scoping it to the
current place — all declared per language in `data/seed/shell-intents.lino`, so
any word order carrying the three parts routes without a seed edit.
`data/seed/terminal-commands.lino` declares the command-naming nouns, and
`command_named_in_prose` collects the command out of a noun phrase *about* a
command, switching itself off inside quotes and shell metacharacters so
`git commit -m 'fix command parsing'` keeps its message. `reads_as_prose`
rejects a `://` head token. `names_a_path_object` splits anchored cues ("copy
the **file** a.txt") from bare ones ("copy"), and a bare cue accepts only
operands written the way paths are written — which is what makes adding a bare
*move* cue safe. `is_safe_path` now excludes `..` traversal instead of excluding
absolute and `~` paths.

**PHP.** The language joins `src/coding/catalog/` with the same eleven task
templates, grammar record and execution metadata the other catalogued languages
carry, and graduates out of the oracle exactly as Kotlin did under #921.
Cataloguing, not scaffolding, is the generalizing move: every parametric task
route gains PHP at once.

**The implementation target.** #723 does not name PHP, it names *PHP Laravel*,
and the framework is the more specific half of that request — answering in the
base language throws away the half that was hardest to satisfy. So the axis is
widened rather than a rule added: `ProgramLanguage::framework_of`
(`src/coding/catalog/types.rs`) records the language a row is a framework of,
`program_language_by_alias` consults framework rows before language rows so the
more specific target wins, and `composition_language` resolves a target through
`base_language`, keeping composition a question about the language while the
template, the file to save and the command to run stay the framework's own. A
framework earns a task template only once that task has been run inside a real
application of that framework, so `write me PHP Laravel code` answers with an
Artisan command class saved where Artisan looks for it, run by
`php artisan hello:world`.

**A task whose subject is its input.** Every task before this one produced its
output from nothing, so `ProgramTask` described one completely with `output`
alone — which is why #863 and #862 had nothing to resolve to. `ProgramTask`
gains `input`, the standard input the task is defined against, and
`ProgramSpec::run_command_line` carries that fixture into the answer's run
command (`printf 'hello\nworld\n' | ./main`) while a task that reads nothing
keeps its plain one. `copy_stdin_to_stdout` then joins `PROGRAM_TASKS` with a
template in each of the thirteen catalogued languages, so the prose request, its
paraphrases and the Rosetta Code URL that names the same exercise all reach one
catalogued task.

**A request that names code and nothing else.** `мне нужен код` names the
artefact and no language, no task and no framework. Rather than a phrase list,
`src/intent_formalization/write_program_request.rs` subtracts: a prompt is a
coding request naming nothing else when every one of its surfaces is an
authoring verb, an artefact noun, a program genus or a request function word —
the last a new role, `request_function_word`, in `src/seed/roles/intent.rs`, so
the words stay seed data in all five languages. What is left over is what the
request is really about, which is why `give me the code of this repository` and
`I need a code review` still route to search. The answer is the honest dead end
`write_program_request_unspecified`: it asks what the program should do and
which language to write it in, in the language the request was written in.

**The minimal-script route** stands aside for a task it cannot render.
`names_no_task_beyond_the_minimal_script` in `src/solver_helpers/mod.rs` declines
when the prompt names a catalogued task other than the hello world, or one
`solve_numeric_list` can answer. It is a property of the route, not a phrase.

**A move that is performed and verified, not issued.** The routing half of #824
above stops at planning a `mv`, and #944 asks for the other half. A read-only
command answers by what it prints, so issuing it is the whole job; a mutating
command answers by what the workspace *holds afterwards*, and a zero exit status
is not that — `mv a b` exits zero whether or not `b` was something the user
wanted overwritten, and exits non-zero for a parent directory the request plainly
implied. So `src/agentic_coding/mutating_action.rs` carries the command out as
the ordered recipe its intent declares: preconditions, preparation, the action,
postconditions, with each step observed before the next is planned. None of
those predicates are written in Rust. They are an `effect` block per intent in
`data/seed/shell-intents.lino`, which is the generalizing move and the reason
nothing under `src/` mentions `cp`: a copy differs from a move only in which
`after` line the seed declares. A step that exits non-zero ends the recipe where
it stopped and is reported as itself — which check, which status, and that
nothing changed — so a blocked move is neither a false completion (#916 rung
`R916-01`) nor the refusal #824 filed.

**The artifacts.** `src/contribution_artifacts.rs` composes the changelog
fragment and the pull-request body from `data/seed/contribution-artifacts.lino`,
holding no prose of its own. `src/contribution_write_path.rs` decides the
publishing commands on two rungs: refused unless `FORMAL_AI_CONTRIBUTION_WRITE=1`,
and never delegated at all (`gh issue create`, `gh pr merge`, the delete
commands). The ladder governs the write path Formal AI takes on its own behalf,
and deliberately not a command an operator names — #749 pinned `execute git push`
as explicit passthrough and #687 pinned "report this on GitHub"; refusing those
would be the over-refusal #824 reports.

**A version of itself it can take back.** `src/memory_revision.rs` gives the
self-modifying half of the loop something to fall back to: `BaselinePin` records
the digest of every specification a version is judged against, `MemoryRevision`
captures the bytes of every tracked file *before* a candidate is written, and a
candidate is adopted only on a verdict that compiled and passed a non-empty
baseline — the same "positive evidence, or no" rule `PromotionProposal` already
applies to promotions. A candidate that edited a pinned file is rolled back
before its verdict is even consulted, because a judge the defendant rewrote
decides nothing. Every attempt leaves `MemoryEvent`s, so the trail travels the
bundle path with the rest of memory.

**A loop that stops on its own.** `src/bounded_autonomy.rs` takes its clock as a
parameter — `Clock`, with `SystemClock` in production and a hand-advanced
`ManualClock` in tests — so the one-hour stuck-recovery limit is exercised in
microseconds against the arithmetic that ships, rather than being the one
constant nobody tests. `RecoveryLoop` answers `Continue` until the limit is
spent, then presents the plan it accumulated and asks; granting an extension
resumes from where it stopped rather than restarting the budget. Per-command
permission is the default and full trust is a separate opt-in, so delegating the
commands is not the same act as delegating the choices, and a choice whose two
best options weigh the same goes back to the operator instead of being decided
by tie-break.

**The circle** is captured by `examples/issue_1021_write_contribution_artifacts.rs`
(which writes the committed fragments and body) and
`tests/unit/issue_1021_closed_circle.rs` (which replays everything and fails if
either drifts).

## 8. Verification

```
cargo test --test unit -- issue_1021
cargo test --test unit
cargo test --test source
cargo test --test integration
rust-script scripts/check-minimal-core-boundary.rs
rust-script scripts/check-hardcoded-language.rs
rust-script scripts/check-changelog-fragment.rs
rust-script scripts/run-ci-gates.rs --stage rust
node tests/e2e/scripts/check-multilingual-intent-coverage.mjs
cargo run --example issue_1021_copy_stdin_harness
cargo run --example issue_1021_named_exercise_probe
cargo run --example issue_1021_languageless_probe
cargo run --example issue_1021_languageless_followup
cargo run --example issue_1021_spanish_code_boundary
cargo run --example issue_1021_bounded_recovery
cargo run --example issue_1021_php_laravel
bash experiments/issue-1021-laravel/run.sh
node experiments/issue-1021-laravel/worker_check.mjs
cd experiments/issue_916_write_effect_ladder && python3 -m unittest test_ladder.py
cargo build --release --bin formal-ai
experiments/issue_916_write_effect_ladder/run_write_effect_ladder.sh
```

The commands below the gates are the measurements the delivery claims rest
on, and each writes a log committed under `logs/`. `issue_1021_copy_stdin_harness` compiles and runs
the `copy_stdin_to_stdout` template in every catalogued language, feeding each
one the task's own fixture and comparing what comes back:
`pass=10 fail=0 skip=3` on this machine, the three skips being the toolchains it
does not have (`tsc`, `dotnet`, `scalac`) rather than failures
(`logs/copy-stdin-harness.log`). `experiments/issue-1021-laravel/run.sh` creates
a real Laravel application with `composer create-project laravel/laravel`,
installs the composed command class into it and runs `php artisan hello:world` —
Laravel Framework 13.26.1 on PHP 8.3.31, printing `Hello, world!` exactly. And
`worker_check.mjs` loads all 26 browser-worker shards into one VM context,
hydrates the lexicon from `data/seed/` the way the worker does at init, and
checks seventeen assertions about the mirror: the reported prompt resolves to
`laravel` in all four reported natural languages, `write me some PHP code` still
resolves to `php`, the uncatalogued `write me PHP Symfony code` falls back to
`php` rather than inventing a target, and the two last assertions record what the
mirror does *not* carry — eleven tasks against the engine's twelve, without
`copy_stdin_to_stdout` (finding 21).

The write-effect ladder is the measurement for #944, and it is the one that
judges the workspace rather than the answer: it boots the real release binary in
agent mode, executes the planned tool calls for real inside a per-rung directory,
and reads the files back off disk afterwards. All sixteen rungs are green,
including the five appended ones, and the same run against the baseline committed
before this work reports `baseline 11/11 -> now 16/16`, so the ratchet moved up
rather than sideways (`logs/write-effect-ladder-after.log`). Rung `824.L5` is
held out on purpose: a fix that special-cased `mv` would pass `824.L1`-`824.L4`
and fail the copy.

The deadline is checked where the gap was, not only where the tests are easy to
run. All eleven `ci_cd::issue_1021` tests passed on the macOS core slices of run
32294252392, including the two — 15/16 and 16/16 — that reported
`timeout: command not found` in run 32282461075
(`logs/macos-deadline-tests-green.log`). On that runner the lower-bound test
measured 3.835s against its 3s deadline and the two-attempt stall finished in
3.826s, so the accuracy the Linux measurements claim holds on the family that
has no `timeout(1)` to compare against.

Two ratchets moved, both in the direction their gates allow without review:

- `data/meta/core-boundary-ledger.lino`:
  `src/solver_handlers/software_project.rs` drops from 915 to 911 lines, because
  the language-label table became a `strip_prefix`, and the outside-core ceiling
  drops from 19,543 to 19,539. The routing fix is line-neutral for the ledgered
  handler: it lives in `src/solver_helpers/mod.rs`, where the predicate it
  belongs to already lived.
- `tests/unit/issue_918.rs`: the seed-metadata gap floor rises from 3,719 to
  3,793. Every added gap is a `closure-generated-*.lino` record for a token the
  new prose pulled into the total closure; no hand-written seed record lost its
  metadata.

## 9. Findings — what this branch does not deliver

Reported rather than papered over, because the issue asks that the bar not be met
by lowering it.

1. **The definition of done is not achieved (R1021-22).** This pull request was
   not opened by a Formal AI `solve` run. It carries the artifacts such a run
   needs and pins them as generator output, but the run itself is the remaining
   gap — and `data/meta/self-hosting-ledger.lino` therefore still reads
   `0.00% self-authored`, leaving R1021-14 (#924/E77) open with it.
2. **"R379-clean generated code" is delivered narrowly (R1021-17).** It holds for
   the artifact generator: `src/contribution_artifacts.rs` contains no
   natural-language literal, all wording living in seed data. It is not a proof
   about arbitrary code Formal AI generates in future.
3. **Laravel is catalogued for one task, not for twelve (R1021-8).** #723 is
   answered in Laravel now — an Artisan command class, saved where Artisan looks
   for it, run by `php artisan hello:world` — but `TEMPLATES_FRAMEWORK` holds
   exactly one row, because a framework earns a task template only once that
   task has been run inside a real application of that framework
   (`experiments/issue-1021-laravel/run.sh`). Ask Laravel for fizzbuzz and the
   answer is the honest dead end `write_program_skill_gap`, not a PHP snippet
   wearing the framework's name. The sparseness is the deliberate half; the gap
   is that eleven verified Laravel templates would each need their own run
   inside that application, and only one has had it.
4. **E94 and E95 are delivered as machinery, and the machinery has not yet
   driven a real unattended run (R1021-12, R1021-13).** The earlier draft of
   this case study called both untouched, on the reasoning that a stuck-recovery
   limit is a property of an unattended run and nothing here runs unattended.
   The review was right that this deferred the wrong half. Both are implemented
   and tested now: `src/memory_revision.rs` pins the baseline by digest,
   snapshots the tracked bytes before a candidate is written, and restores them
   when the candidate fails — the test fails a compile on purpose with a real
   `rustc` and then compares the workspace byte-for-byte, because a rollback
   that is only *reported* is not a rollback. `src/bounded_autonomy.rs` takes
   its clock as a parameter, so the pathological run that never resolves is
   exercised against the same arithmetic the default hour uses, in
   microseconds instead of an hour. What is still missing is narrower than
   "untouched" and worth naming precisely: no unattended `solve` run has yet
   been driven through `RecoveryLoop`, so the limit is confirmed against a
   hand-advanced clock rather than against a wall clock in production, and no
   version of this repository has yet been adopted or rolled back by
   `RevisionLedger` — that is the same gap as R1021-14 and R1021-22, and it
   closes with the run, not with more tests.
5. **The ladder does not govern operator-named commands**, by design — see §7.
6. **`мне нужен код` is recognised as a coding request, and is still not
   answered with code (R1021-31).** It no longer reaches web search: eleven bare
   requests across five languages reach `write_program_request_unspecified`,
   while `give me the code of this repository`, `I need a code review`,
   `I need to find a python tutorial` and `дай мне код этого репозитория` still
   route to search, and `I need information about Rust` to concept lookup
   (`logs/languageless-request-after.log`). What comes back is a question — what
   should the program do, and in which language — asked in the language the
   request was written in. That is the correct answer to a request that names
   neither, and it is still not a program: nothing here picks a default language
   or a default task, because guessing one is the specialization R1021-2
   forbids. A conversational follow-up that supplies the missing half is
   answered from the catalog; a single prompt that supplies neither cannot be.
7. **`src/coding/catalog/mod.rs` has pre-existing drift from its `tests/source`
   mirror**, which is why the mirror compiles green while differing from `src/`.
   Not introduced here, not fixed here.
8. **No upstream defect was found (R1021-26).** Every root cause traced back into
   this repository; the one third-party constraint met — Links Notation having no
   comment syntax — is documented behaviour, not a defect.
9. **#863 and #862 are delivered, and three of the thirteen languages were not
   run here (R1021-6, R1021-7).** The prose request, its held-out paraphrases in
   five languages and the Rosetta Code URL that names the same exercise all reach
   the catalogued `copy_stdin_to_stdout` task and are answered with a program
   (`logs/named-exercise-routing-after.log`):

   ```
   === Give me example of how to do copy stdin to stdout in Rust
   -- intent: write_program
   === Execute https://rosettacode.org/wiki/Copy_stdin_to_stdout in Rust
   -- intent: write_program
   ```

   The blocker named in the earlier draft of this finding was real and is what
   the fix removed rather than worked around. `ProgramTask` was
   `{ slug, label, output }` — a task's verified output being a function of its
   source alone — and every path that runs a generated program set
   `.stdin(Stdio::null())`. A program whose output *is* its input could not be
   stated under that contract, so the task gained the input it is defined
   against, `ProgramSpec::run_command_line` carries that fixture into the
   answer, and the harness pipes the same bytes. Adding the template for Rust
   alone would have been the specialization R1021-2 forbids, so all thirteen
   catalogued languages have one.

   What is *not* claimed is that all thirteen were run on this machine.
   `cargo run --example issue_1021_copy_stdin_harness` writes each program out,
   compiles it, pipes the task's fixture in and compares what comes back:
   `pass=10 fail=0 skip=3` (`logs/copy-stdin-harness.log`). The three skips are
   TypeScript, C# and Scala, whose toolchains (`tsc`, `dotnet`, `scalac`) are
   absent here — a gap in this machine rather than in the templates, and the
   honest way to say so is that they are unverified until a machine that has
   them runs the same harness.

10. **Cataloguing PHP moved two existing tests, and neither was weakened.**
    `write a hello world program in php` and the Russian follow-up of issue #461
    both used to resolve through the coding oracle — the cached Hello World
    Collection entry the fallback supplies for a language the catalog does not
    template. Catalogued, PHP takes the catalog route, so
    `issue_412_oracle_languages.rs` and `issue_461_php_followup.rs` now assert
    `write_program` where they asserted `write_program_oracle_hello_world_php`.
    That is the graduation Kotlin already made under issue #921, and the oracle's
    own module doc calls it correct: the handler fires only when the catalog
    "does not template" a language. The guarantee each test was written for is
    kept — #412 still pins Swift on the fallback, and #461 still pins that a
    follow-up naming only a language inherits the advertised Hello World task
    instead of dead-ending on unknown. What the catalog route drops is the
    "Hello World Collection" attribution line, which has no source to attribute
    once the program comes from a template this repository verified itself with
    `php -l` and executed.

11. **Spanish was registered but had no listing vocabulary, and that was found
    by a gate rather than by reading.** `check_language_test_coverage` requires
    that a change touching language-facing data carry test evidence for every
    language in `data/seed/languages.lino` — en, ru, hi, zh, **es** — and this
    branch had none for Spanish. Probing it showed why: `directory_listing` in
    `data/seed/shell-intents.lino` had `language` blocks for four languages and
    no `es` block at all, so every Spanish phrasing fell through to web search
    exactly the way the reported English *"List me files here"* did in #865. The
    fix is the seed data alone — verbs, question words, objects and scopes — and
    no Rust changed, because the detector combines parts and is never told which
    language it is reading. Four held-out Spanish word orders now route to `ls`,
    and `lista los procesos en ejecución` still does not, so the parts still have
    to combine. Measured output is in `logs/spanish-listing-routing-after.log`;
    the probe is `examples/issue_1021_spanish_probe.rs`.

    This is the shape R1021-2 asked for, arriving from an unexpected direction:
    a gate the repository already had caught a language gap that no prompt in
    issue #1021 mentions, and the gap closed with data rather than a rule.

12. **One CI job went red for a reason that was not in this branch, and saying
    so needed a bisect rather than an assertion.** `E2E Tests (agent CLI <->
    formal-ai)` failed on the pull request while passing on `main`, on a branch
    that changed nothing the Codex TUI startup path reads. The difference was
    the client: `.github/workflows/release.yml` installed `@openai/codex`
    unpinned, and `0.148.0` was published at 2026-08-18T22:30Z — after `main`'s
    last green run (11:13Z, codex 0.147.0) and before the branch's first red one
    (06:02Z the next morning, codex 0.148.0).

    From 0.148.0 on, the ENTER that answers Codex's first-run "Do you trust the
    contents of this directory?" screen is dropped if it arrives as soon as that
    screen renders. The failing artifact shows the keystroke *was* delivered
    (`interactionCount: 1`) and that `formal-ai.log` recorded only `GET /health`:
    the harness never got far enough to ask the server anything. A bare `codex`
    in a pseudo terminal under a throwaway `HOME` — no wrapper, no config —
    reproduces it, which is what rules Formal AI out as the cause. Reported as
    <https://github.com/openai/codex/issues/39487>; still present in
    `0.149.0-alpha.1`.

    The interesting part is the fix that was rejected. The harness could in
    principle answer the dialog "once the screen stops moving" —
    `command-stream` exposes a per-interaction `idleMilliseconds` for exactly
    that. Measuring first showed why it cannot work: the trust screen animates,
    repainting about every 80 ms in 0.147.0 and 0.148.0 alike, so the idle
    window never opens. Setting it made the Codex leg fail on **0.147.0 too**,
    because the ENTER was then never sent at all. Every workaround that does
    clear the dialog needs a wall-clock delay the driver cannot express.

    So the change is a pin, and the repository had already written down why:
    "Versions are pinned rather than floating so a matrix leg fails because our
    server changed, not because an upstream CLI shipped overnight"
    (`experiments/agentic_cli_matrix/clients.lock`). `release.yml` was following
    that rule only for the one package `--trust` forced someone to name.
    `tests/unit/ci-cd/issue_1021.rs` now holds it for every third-party CLI any
    workflow installs, so the next floating install fails review instead of a
    job; packages in the `@link-assistant/` scope stay unpinned on purpose,
    since an E2E leg that pinned our own client would stop reporting whether
    today's client works against today's server. `experiments/issue_1021_codex_tui_version`
    holds the bisect and the wrapper-free reproduction, and is what a future
    bump has to pass before the pin moves.

13. **Three more red jobs, and none of them was the change under review
    either.** The same CI run that proved the client pin (finding 12) failed
    for three unrelated reasons, each worth more than its fix.

    *A ratchet the fix walked into.* `.github/workflows/release.yml` sits in the
    warning band `tests/unit/ci-cd/issue_999.rs` guards at 1,510 lines, and the
    eight-line comment explaining the pin made it 1,511. Issue #1021 says the
    bar is not to be met by lowering it, so the limit stayed and the comment
    shrank to five lines — the URL, the rule, and the bisect to run before
    bumping — with the full account left where it belongs, in
    `experiments/issue_1021_codex_tui_version/README.md`. 1,508 lines.

    *Evidence that was never committed.* `docs_requirements_issue_1021` asserts
    the probe logs the analysis cites are in the repository, and it failed in CI
    while passing locally: `.gitignore`'s `logs/` rule had swallowed
    `docs/case-studies/issue-1021/logs/`. The `!docs/case-studies/**/*.log`
    re-include below it cannot help, because git never descends into an excluded
    directory — the same trap issue #1017 hit one directory over in
    `dev/log/**/ci-logs/`. `git add` reported success and committed nothing;
    only a checkout that was not this working copy could tell. The directories
    are now re-included beside the files, and the six logs are in the tree.

    *A race that a green local run cannot disprove.* The write-path test read
    the opt-in — a process-wide environment variable — outside the lock its own
    comment says "every test that touches it holds", so while a sibling test was
    inside its opted-in window the refusal under test did not happen. The
    failure looks like a bug in the ladder (`left: Ok([...git push...]), right:
    Err(OptInAbsent)`) and is a bug in the test. It reproduces locally 33 times
    in 200 rounds of the compiled binary, and 0 times once both states are
    entered through one locked helper:
    `experiments/issue_1021_opt_in_race/run.sh`, with both runs preserved under
    `logs/`. A flake that is merely re-run is a defect the suite has agreed to
    keep; measuring it is what turns "it passed this time" into a claim.

14. **A budget that reports a stall is not a budget that survives one.** The
    next run failed again in a place the branch does not touch:
    `E2E (opencode-desktop)` spent its full 300s `Install Xvfb` budget inside
    `apt-get` and was terminated (run 32272689026, job 96135410333, `logs/xvfb-install-budget-terminated.log`),
    while `opencode-vscode` and `cursor` installed the same package from the
    same commit in 52s. That budget is issue #1017's own fix: before it, the
    same hang ran to the 25-minute job cap and GitHub reported the kill as
    `cancelled`, a false negative. The budget converted the false negative into
    a true one — and then spent the whole of itself on a single attempt.

    So the attempt is bounded too, not only the step.
    `scripts/apt-install-with-retry.sh` gives each attempt its own deadline
    inside the step's, kills a stalled one while there is still room for
    another, and — the part that makes it more than a retry loop — refuses to
    start when `attempts x per-attempt + delays` exceeds the budget above it,
    because a retry that outlives its budget recreates the terminated step it
    was added to prevent. `desktop/scripts/package-macos-with-retry.sh` learned
    the same rule one runner family over. `timeout` is placed *inside* `sudo`
    so the signal reaches `apt-get` rather than an unprivileged parent; a kill
    that missed apt would leave root holding the dpkg lock and fail every
    remaining attempt with a lock error instead of the stall that caused it.
    Verified with a stand-in `apt-get` that stalls, that refuses, and that
    recovers: `tests/unit/ci-cd/issue_1021.rs`. The step also drops `-qq` for
    `-q`, because the terminated step logged nothing about which phase was
    stuck — the standing instruction to add output where a root cause is not
    visible, applied to CI.

    Three findings in a row (12, 13, 14) are the same shape: a red job whose
    cause is upstream or environmental, and whose fix is a rule the repository
    can hold rather than a re-run. The definition of done asks for a pull
    request with all checks green and no human edits; every re-run that is
    quietly repeated instead of explained is a hole in that claim.

15. **A file that has not been written yet is not proof of a dead process.**
    The same run's `CI/CD Pipeline` was red for a different reason:
    `macOS Core Tests / Run macOS core slice 8/16` failed
    `issue_703_orchestration_followup::timeout_terminates_descendant_processes`
    on `assertion failed: !workspace.path().join("descendant-survived").exists()`
    (run 32272689475, job 96137354605,
    `logs/descendant-timeout-macos-slice8.log`). Nothing in this branch touches
    `run_agent` or its fixture; the test dates from #703 and the same assertion
    failed the same way during PR #1015, which answered it by upgrading
    command-stream so the allowlisted executable, not an added `/bin/sh`, leads
    the process group.

    That answer removed one process from the window without removing the
    window, because the assertion was never about the process at all. The test
    gave the agent a 20 ms timeout, spawned a descendant that wrote
    `descendant-survived` after 150 ms, and read the file 250 ms later. An
    absent file means *either* the descendant was terminated *or* it is alive
    and has not reached its write, so the test passed for the right reason only
    while the kill kept beating a 130 ms head start. On a runner slicing
    sixteen ways it did not: the failing test took 464 ms where the same test
    takes ~280 ms here.

    The question the test wants to ask is whether a process is running, so it
    now asks the kernel. The fixture records the descendant's pid; the test
    polls `ps -o state=` for it and separates the three ways this can go wrong,
    each with its own message: never spawned (the timeout was too short for the
    fixture to reach its `spawn`), still running after 5s (the kill did not
    reach the process group), and gone but having outlived the 20s its file
    costs (the kill arrived far later than the 2s it was given). The 2s timeout
    and the 20s descendant are margins, not behaviour: they make the descendant
    certainly exist when the kill lands and certainly outlive any honest delay
    in observing it.

    Two measurements were needed to get this right, and both contradicted a
    plausible first answer. `kill -0` looked like the way to ask the kernel, and
    it is wrong: it succeeds for a process that has already terminated but
    whose exit status nobody collected, and this repository's own container runs
    PID 1 as a `node` process holding 384 such entries, so the first version of
    the fixed test reported a descendant that had been dead for ten seconds as
    alive. And the group kill itself was never broken: `ps` taken every 250 ms
    through a run shows the fixture, its `sh` and its `sleep` sharing one pgid
    and all three vanishing together at the 2s timeout. The upstream one-shot
    `kill(-pgid)` was the suspect and the evidence acquitted it, which is why no
    upstream issue was filed for this one. The mutation that does make the test
    fail is a descendant spawned with `process_group(0)` — outside the group the
    kill addresses — and the test names it exactly: `descendant … was still
    running 5s after the agent timed out, so the timeout did not reach the
    process group`.

16. **The fix for a red job was itself only tested where it runs.** The commit
    that added `scripts/apt-install-with-retry.sh` bounded each attempt with
    GNU `timeout`. Its own job — Linux — installed Xvfb on attempt 1 in 9s and
    went green, and two macOS core slices went red on the *tests* that drive
    the wrapper (run 32282461075, jobs 96170638546 and 96170638704):

        scripts/apt-install-with-retry.sh: line 91: timeout: command not found
        left: Some(127), right: Some(100)

    (`logs/macos-timeout-not-found.log`).

    macOS ships no `timeout`; coreutils installs it as `gtimeout` and neither is
    guaranteed on a hosted runner. The script only ever runs on Linux, so a
    reviewer reading it in isolation sees nothing wrong — but a test is code
    that runs everywhere the suite runs, and the suite is sliced across both
    runner families. A wrapper written to keep a transient failure from turning
    a pipeline red had turned it red on a second platform.

    Branching on whichever binary is present (`timeout` here, `gtimeout` there)
    would have made the failure go away while leaving the tested path and the
    shipped path different on the two families, which is exactly how the gap got
    in. `scripts/run-with-deadline.sh` is one deadline that runs on both: the
    command is started in its own process group, signalled whole at the
    deadline, and reported with `timeout`'s own 124 so the retry wrapper can
    still tell a stalled mirror from apt's own failure by that number. The same
    technique already bounds a whole CI step in `run-with-budget-warning.sh`,
    which is why the macOS slices could run *that* one all along.

    Two tests pin the primitive — an expired deadline kills a stall that lives
    in a *child* of the command, and a command that answers in time keeps its
    own exit status — and a third generalizes the defect instead of the
    instance: `no_committed_script_reaches_for_a_timeout_binary_macos_does_not_have`
    reads every tracked script and workflow, because the next script to reach
    for a GNU-only binary will not be this one. That guard needed a second
    version. The first asked what followed the word `timeout` — a number or a
    flag — and a quoted `timeout "$attempt_seconds"` walked straight through it:
    the mutation run reported the guard *passing* on the exact line that had
    just failed CI. It now asks where the token stands rather than what follows
    it, and the same mutation fails with
    `scripts/apt-install-with-retry.sh:90: timeout is GNU coreutils and the macOS runners do not have it`.

17. **Writing the deadline meant owning its accuracy, and the first draft
    expired early.** Replacing `timeout(1)` replaces a promise, not just a
    binary: a command given 3s must get 3s. The first draft polled once a second
    and read elapsed time from bash's `SECONDS`, and every test still passed —
    because every assertion about it was an *upper* bound. Measuring instead of
    asserting (`experiments/issue-1021-deadline-precision/measure.sh`) showed
    two defects in one primitive (`logs/deadline-precision-measurements.log`,
    five runs of each draft against the same 3s deadline):

        draft 1: 1s poll, SECONDS clock      expired after 4.26-4.41s
        draft 2: 0.1s poll, SECONDS clock    expired after 2.54-3.01s
        shipped: 0.1s poll, both bounds      expired after 3.42-3.56s

    The first is a full extra second spent in the grace loop, because a command
    signalled at the deadline is still alive at the next check. The second is
    the one that mattered: `SECONDS` is a difference of whole-second clock
    readings, so it reaches 3 as little as 2.01s after the shell starts, and a
    deadline that expires early converts work that was going to finish into a
    failure — the exact flake the retry exists to remove. Sharpening the poll is
    what exposed it; the coarse poll had been hiding it behind its own lateness.

    The elapsed time is now the larger of two lower bounds — counted poll
    intervals, exact at the short end but drifting by a fork per iteration, and
    the `SECONDS` reading minus the second it may be reading high — so it is
    never early, and never late by more than a fork's-worth or two seconds,
    whichever is smaller. Measured: 3.4-3.6s on a 3s deadline and 10.8s on a
    10s one, and early on none of the runs above.
    `the_deadline_never_expires_before_the_time_it_was_given` is the assertion
    that was missing; restoring the `SECONDS`-only reading fails it with
    `a 3s deadline expired after 2.480484781s`.

    One assertion in the suite had to be *loosened* to say something true. It
    read `Attempt 1 exited 124 after 3s of its 3s deadline`, and it failed
    against a deadline that had done its job in 3.6s. Pinning the rounding of a
    duration is not testing the duration: it now parses the reported seconds,
    requires them inside `3..=6`, and separately holds the whole two-attempt run
    against the clock — 300s of stall must finish in under 45s. Lowering a bar
    is weakening what is required; this raised what is checked while dropping a
    coincidence of formatting.

18. **The retry rescues a mirror that is hung, not one that is merely slow.**
    The first CI run to carry the portable deadline still turned
    `E2E (opencode-desktop)` red (run 32294252072, job 96206598860,
    `logs/apt-mirror-outage-every-attempt.log`). The deadline itself behaved
    exactly as written — each of the three attempts was killed at 91s of its 90s
    budget, one second late and never early, and each was retried — and the step
    failed anyway, on an upstream outage: `azure.archive.ubuntu.com` returned
    `Ign` for every index while `archive.ubuntu.com` served them.

    The timings say why, and they are not about the deadline. Inside one
    attempt: 30s for apt to time out the dead mirror, ~8s of backoff, then a
    fallback that fetched the `InRelease` files in a second and was still
    downloading package indices 53s later when the deadline killed it. Every
    attempt re-paid the same fixed 38s and had the remainder for progress that
    was never kept. Three attempts of 90s inside a 300s budget spend the budget
    on the attempts *least* likely to succeed: the last one, the only one with
    nothing to retry after it, gets no more time than the first.

    That is a real defect in `scripts/apt-install-with-retry.sh` and it is ours,
    not upstream — but it is a different defect from the one this branch fixes,
    it predates this branch (the wrapper's 3×90s shape comes from #1017), and
    the same commit was green on the same job two hours earlier
    (run 32282460920), so it is the mirror that changed and not the code. The
    honest shape of the fix is to let the deadlines grow across attempts inside
    the budget rather than repeat — a short first probe catches a hung mirror
    cheaply, and the last attempt takes what is left, because killing it
    converts a slow success into a certain failure. Filed as #1028 rather than
    folded in here, where it would arrive without the reproduction it needs: the
    stand-in `apt-get` these tests drive models a mirror that hangs, and this
    one was slow. Re-running the job passed, which is what confirms the mirror
    changed and not the code.

19. **The catalog understates four toolchains, and this machine proved it.**
    `ExecutionStatus::Unavailable` is what a catalogued language carries when
    this repository has no execution profile for it, and the rendered answer
    says so: *"not compiled or run in … is not configured in this repository
    runtime"*. Seven of the thirteen languages carry it. The copy-stdin harness
    ran them anyway, and four of the seven — C++, Java, Ruby and Kotlin —
    compiled and produced the fixture back, byte for byte
    (`logs/copy-stdin-harness.log`). Only TypeScript, C# and Scala really have
    no toolchain here.

    They were not flipped to `Verified`. That status is a claim about the
    repository's own verification harness rather than about whichever machine
    happens to run a probe, and flipping four languages would rewrite every
    answer they give — far outside the reported range, on the strength of one
    container. What is honest to say is that the label is now known to be
    conservative for four of them, and that the harness in
    `examples/issue_1021_copy_stdin_harness.rs` is what a future change should
    run on CI's own image before moving any of them.

20. **A one-letter language alias was matching inside a Spanish word, and the
    languageless request is what exposed it.** `contains_token` decided word
    boundaries by asking `is_ascii_alphanumeric` of the neighbouring character.
    `ó` is not ASCII, so the `c` of `código` — Spanish for *code* — read as an
    isolated token and matched the alias of the language C. Every Spanish
    request that mentions code was a request for a C program:

        escribe código                                -> write_script_c
        necesito código                               -> write_script_c
        dame código                                   -> write_script_c

    (`logs/spanish-code-boundary-before.log`; after the fix all three reach
    `write_program_request_unspecified`, in `…-after.log`.) A word boundary is
    a property of letters rather than of ASCII, which is what the rule now says,
    with the scripts written without word spaces kept as boundaries exactly as
    `contains_cjk` and `contains_devanagari` already had them. `escribe código
    en Python` and a bare `C` still resolve, so nothing was switched off.

    This is the second time in this branch that a defect in a language nobody
    reported was found by covering something else (finding 11 was the first).
    Both were reachable only because the fix was written for the structure of
    the request rather than for the reported wording — the argument R1021-2
    makes, arriving as evidence rather than as an assertion.

21. **The browser mirror carries the framework target and not the stdin task.**
    `src/web/worker/` is a hand-maintained mirror of the Rust solver, and this
    branch widens the two sides unequally on purpose. `formal_ai_worker_13.js`
    gained the framework-first resolution order, so the reported prompt resolves
    to `laravel` in the browser in all four reported languages, and
    `formal_ai_worker_12.js` gained the `laravel` target with its template. The
    `copy_stdin_to_stdout` task was not mirrored: `WRITE_PROGRAM_TASKS` in the
    worker holds the same eleven tasks it held before, against twelve in
    `PROGRAM_TASKS`.

    That is the sparse-coverage precedent the mirror already runs on rather than
    an oversight — the worker has never carried every task — but it is a real
    difference in what a user is answered depending on where they ask, and the
    reason it is defensible for this task in particular is that the answer's
    value is its *run command*, `printf 'hello\nworld\n' | ./main`, which is
    the one part of the answer a browser cannot run. Verified with
    two assertions added to `experiments/issue-1021-laravel/worker_check.mjs`:
    the mirror's task table has no `copy_stdin_to_stdout` key and holds eleven
    entries.

22. **Two worker ceilings moved, and both bought something measured.**
    `data/meta/worker-line-budget/` caps each mirror shard, and the caps rise
    here: `formal_ai_worker_12.js` from 1,235 to 1,251 (+16, the `laravel`
    target and its template) and `formal_ai_worker_13.js` from 1,388 to 1,396
    (+8, of which six lines are the comment explaining why framework rows are
    consulted first). A raised ceiling with a rationale nobody checked is a
    lowered bar, so the rationale is checked: `worker_check.mjs` loads all 26
    shards into one VM context, hydrates the lexicon from `data/seed/` the way
    the worker does at init, and passes seventeen assertions — the four reported
    prompts resolving to `laravel`, `write me some PHP code` still resolving to
    `php`, the uncatalogued `write me PHP Symfony code` falling back to `php`,
    and the mirrored template being the Artisan command class rather than a
    plain PHP script, and the two that record the missing stdin task.

    Getting that evidence took two corrections worth recording. The lexicon is
    not installed by any function named for installing it; it is hydrated by
    `hydrateLinoSeedText(raw)` with a `{path: text}` map, at
    `formal_ai_worker_00.js:222`. And every assertion has to be *evaluated
    inside* the context: a top-level `const` in a script run through
    `vm.runInContext` lives in the realm's global lexical environment and is not
    a property of the sandbox object, so reading `sandbox.WRITE_PROGRAM_LANGUAGES`
    from Node returns `undefined` while the same expression evaluated in the
    context returns the table. A harness that had stopped at the first reading
    would have reported the mirror broken when it was not.

23. **Two tests this branch cited as evidence had never been written, and the
    traceability gate could not tell.** `an_example_request_is_not_a_command_to_run`
    and `a_web_address_is_a_resource_not_a_program` were named as the automated
    evidence for R1021-6 and R1021-7 in `REQUIREMENTS.md`, in
    `docs/requirements/issue-1021-…md` and in `docs/requirements-traceability.md`.
    Neither function exists. `git log -S` puts the citations in commit
    `53bf9c5d1` — a commit on this branch, so this is not inherited drift but a
    defect written here on 2026-08-19 and found on 2026-08-20 while rewriting
    those two rows for the delivery they now record.

    The reason it survived a day is the more useful half. `docs_requirements_issue_1021`
    checked that every `R1021-N` has a row and that the undelivered ones are
    named; it never checked that a row is *true*. A table whose evidence cannot
    be run is precisely the failure the table exists to prevent, so the gate now
    reads the citations back: `every_test_the_traceability_rows_cite_exists`
    parses each row's `<path>::<test>` and `; ::<test>` continuations and
    requires the file to exist and to define the function, across all
    thirty-odd citations the issue's rows carry. Restoring either name fails it.

    The parser refuses to guess rather than skipping what it does not
    understand: a `::` preceded by anything other than a test path, a
    continuation marker or a prose code span such as `` `ci_cd::issue_1021` ``
    fails the test outright. A citation quietly passed over would leave exactly
    the hole that was just closed.

24. **The move rung the issue words as "move+cleanup" is delivered on the half a
    move owes, and reported on the half it does not.** Issue #944 asks for
    "multi-step move+cleanup" as its third rung. What rung `824.L3` verifies is
    the cleanup the move itself owes: after `mv notes/2026 backup/2026`, the
    postcondition `test ! -e notes/2026` must hold, so a copy masquerading as a
    move fails the rung. What it does *not* do is remove the now-empty `notes/`
    parent. Deleting a directory the user did not name is a write they did not
    request, and #824 is a report about over-refusal, not a licence to
    over-reach — a system that tidies beyond the instruction is the same class of
    defect seen from the other side. Recorded here and in the ladder's
    `README.txt` rather than quietly narrowed; if the intended reading is that
    the parent should go, it is a one-line `after` step in
    `data/seed/shell-intents.lino` and no Rust change.

25. **The response file crossed its warning band, and the band did not move.**
    Adding the two verified-action responses in five languages each took
    `data/seed/multilingual-responses-agentic.lino` to 1421 lines against a
    warning band of 1400, failing
    `ci_cd::issue_999::warning_band_files_are_small_and_split_responses_cover_the_registry`.
    The band is one of the four gates the review names as not-to-be-relaxed, and
    the file already had the answer written into the same test: issue #999 split
    the tool-outcome responses into
    `data/seed/multilingual-responses-agentic-tools.lino`, and
    `mutating_action_completed` / `mutating_action_blocked` are that same family
    — what the workspace tools observed, not what the planner said. Moving the
    forty lines there leaves the main file at 1381 and the split file at 121, and
    the two new intents join the four the test already pins as having to stay
    directly available in every registered language after the split. Nothing was
    hidden by the move: the registry declares the split file once and
    `scripts/generate-seed-registry.rs` carries it to every production surface.

26. **The compiler that judges a self-authored version was pinned to an older
    dialect than the one the system writes in.** `memory_revision::rustc_verdict`
    is the real compile step behind E94's "if compilation of next version of
    itself fails" — an actual `rustc` on an actual file. It passed
    `--edition 2021`. The moment this branch moved the crate to edition 2024, a
    candidate version whose only novelty was a let-chain would have come back
    `compiled: false` with `error: let chains are only allowed in Rust 2024 or
    later`, and the ledger would have rolled back a version that `cargo build`
    accepts. The reproduction is
    `issue_1021_recoverable_memory::the_verdict_compiles_the_edition_the_crate_is_written_in`:
    it writes a let-chain, asks for the verdict, and fails on exactly that
    mismatch and nothing else.

    The fix is not the string `2024`. A constant typed into `memory_revision.rs`
    would be a second place to remember the edition, and the day it fell behind
    `Cargo.toml` the same silent rollback returns. `build.rs` reads the
    `[package]` table and exports `FORMAL_AI_CRATE_EDITION`, so the manifest is
    the only place the edition is written down and the next edition move carries
    the verdict with it. The same pin sat in two more places that compile
    system-authored Rust — `experiments/issue_847_coding_ladder/run_coding_ladder.sh`,
    which decides whether a ladder task's generated source counts as verified,
    and `rule_synthesis::execution_commands`, which tells the user how to build
    the substitution program the server just wrote. Both would have failed
    working output for the same reason.

27. **`sh src/web/wasm-worker/build.sh` was broken by the edition move, and no
    committed artifact would have shown it.** The worker's `lib.rs` is not a
    standalone program: it `#[path]`-includes `src/language.rs`,
    `src/arithmetic.rs`, `src/web_search_core.rs` and
    `src/web_search_fusion_core.rs` from the crate itself. Once clippy's
    `collapsible_if` rewrote those four into let-chains, a build script pinned to
    `--edition=2021` stopped compiling, with ten errors and no output file. CI
    does run the script, so this would have gone red there — but the *committed*
    `src/web/formal_ai_worker.wasm` is checked only against a size budget
    (`scripts/check-wasm-worker-size.rs`), never against a fresh build, so the
    stale binary sat in the tree looking healthy the whole time. The build is at
    edition 2024 now and the worker is rebuilt from it: 291518 bytes against the
    400 KiB warning band.

    The missing check is left as a finding rather than closed with a gate. The
    obvious one — `git diff --exit-code` on the `.wasm` after the CI build, the
    way `src/web/vendor.bundle.js` is already checked ten lines below — compares
    bytes emitted by CI's `@stable` `rustc` against bytes emitted by whichever
    `rustc` the contributor had. Those differ across compiler releases for
    reasons that are nobody's mistake, so the gate would fail honest pull
    requests. Naming the gap is the accurate move; picking a comparison that
    survives a toolchain bump is separate work.

    Both `src/web/wasm-worker/src/lib.rs` and the WebAssembly programs
    `src/substitution_compiler/webassembly.rs` writes export their entry points
    through `#[no_mangle]`, which edition 2024 makes a hard error in favour of
    `#[unsafe(no_mangle)]`. That spelling has been accepted in every edition
    since Rust 1.82, so it is a rename and not a version floor.

    One nearby command was found stale and left that way:
    `experiments/issue709_wasm_heap.rs`'s doc comment names a `rustc` line that
    has not worked since `web_search_fusion_core.rs` grew a
    `crate::search_fusion_grammar` reference — the same breakage is on `main`,
    predates this branch, and needs the experiment's `#[path]` list rebuilt
    rather than an edition flag.

28. **A major bump deleted a trait, and nine call sites had each written the
    same encoding by hand.** `sha2` 0.10 returned its digest as a
    `generic_array::GenericArray`, which implements `LowerHex`; `sha2` 0.11
    returns a `hybrid_array::Array`, which does not. Nothing in this repository
    called `LowerHex` by name, so the removal surfaced as ten
    `error[E0277]: the trait bound ...: LowerHex is not satisfied` at once, in
    nine files that had independently written `format!("{:x}", Sha256::digest(..))`
    — `computer_use/executor.rs`, `orchestration/{workspace,runner,replay}.rs`,
    `memory/upgrade.rs`, `memory_revision.rs`, `agentic_coding/workspace_change.rs`,
    `file_legality.rs`, and `tests/unit/issue_848_coding_ladder.rs`. Six of them
    had wrapped it in a private `fn digest`/`fn sha256` of their own.

    Pinning back to `sha2 = "0.10"` would have made the ten errors go away and
    left the nine copies in place, so the bump was taken as the review asks — a
    major bump is a code change, not a version change. The encoding is now
    written once, as `source_fetch::hex_lower`, beside the `sha256_hex` the
    crate already exported; every one of the nine goes through it, and the seven
    `use sha2::…` lines that became dead were deleted with them.
    `src/file_legality.rs` keeps its import because it streams a file through
    `Sha256::new()` rather than hashing a slice, and it renders the result with
    the same shared helper. What the upgrade cost, in other words, is what it
    bought: one implementation of "digest bytes as text" where there were nine.

29. **Five crates in the lockfile are not on their newest release, and none of
    the five is ours to move.** "Every dependency updated" is a claim about the
    manifests, and `cargo tree -i` is what separates a stale requirement from an
    inherited one. `Cargo.lock` still carries `links-notation` 0.13.0 beside our
    0.14.0 (required by `meta-language` 0.58.2), `sha2` 0.10.9 beside our 0.11.0
    (`p256` → `rtc-dtls` → `webrtc`), `which` 7.0.3 beside our 8.0.5,
    `cc` 1.2.67 against an available 1.4.3 (`cmake` → `aws-lc-sys`), and
    `generic-array` 0.14.7 against an available 0.14.9 (`aead` → `aes-gcm` →
    `rtc-shared`). The `which` duplicate is the interesting one: it comes from
    `command-stream`, which issue #1014 pinned at `=0.16.0` deliberately and
    `tests/unit/ci-cd/issue_1014.rs` asserts — so unpinning it to collapse a
    duplicate would undo a decision another issue made on purpose.

    Recording the five is the point. A duplicate in the lockfile is normally
    read as a manifest that nobody refreshed, and here each one is a transitive
    requirement of a crate that has not yet published against the newer major.
    They will collapse when their parents move, not when we edit anything.

30. **The `agent` leg of the CLI matrix went red without a commit touching it,
    and the cause was a floating dependency two levels below a pinned one.**
    `experiments/agentic_cli_matrix/clients.lock` pins
    `@link-assistant/agent@0.25.0` exactly, so the client the leg drives is
    supposed to be a constant. It was not. Run 32307282670 (2026-08-19 22:07 UTC)
    passed; runs 32415475370, 32422971033 and 32434228261 all failed the same
    two cases — `read-file` and `interactive` — with
    `client output never contained 'ALPHA_MARKER_11111'`, on three different
    commits, none of which is near the tool path.

    `serve.log` names the defect precisely. Round one planned the tool call, and
    round two arrived carrying its result:

    ```
    [trace] agentic_outcome: planned ToolCalls([PlannedToolCall { tool: "read", … }])
    [trace] agentic_outcome: planned Final("Contents of `alpha.txt`:\n\n```text\nTool execution aborted\n```")
    ```

    The server did its job twice: it planned the read, and it answered with what
    the client handed back. What the client handed back was an abort, and the
    client's own log says why, one line before it:

    ```
    "hint": "Provider returned undefined finishReason but made tool calls",
    "message": "inferred tool-calls finish reason from pending tool calls"
    ```

    Our stream is not the one at fault, and that was checked rather than assumed
    — `curl`ing the streaming endpoint with a `tools` array returns the
    spec-correct terminator, `"delta":{},"finish_reason":"tool_calls"`, in its
    own chunk before the usage chunk and `[DONE]`.

    The first version of this finding then guessed wrong about where the
    `undefined` comes from, and said so in public: it claimed the pinned
    `@ai-sdk/openai-compatible@^1` “does not map an OpenAI `finish_reason` onto
    the field the current `ai` package reads.” It maps it correctly —
    `mapOpenAICompatibleFinishReason` turns `tool_calls` into `"tool-calls"`, and
    driving that exact provider build through a plain `streamText` executes the
    tool and answers with the marker. The guess is corrected below and on the
    upstream issue, because a wrong mechanism published with a right symptom is
    worse than no mechanism at all.

    What actually happens is a version-shim hole in `ai` itself, and
    `experiments/issue-1021-agent-cli-finish-reason/` reproduces it with no
    server and no network. `ai@6` still accepts `LanguageModelV2` providers by
    proxying them: `asLanguageModelV3` pipes their stream through
    `convertV2StreamToV3`, which rewrites the V2 plain-string `finishReason`
    into `{unified, raw}` and the flat V2 `usage` into its nested V3 shape.
    `wrapLanguageModel` defeats that proxy, because `doWrap` stamps the version
    on the object it returns while forwarding `doStream` to the wrapped model
    untouched:

    ```js
    return {
      specificationVersion: "v3",
      …
      async doStream(params) { … return wrapStream ? … : doStream(); }
    };
    ```

    Wrap a V2 model and you get an object that *claims* V3 and still emits V2
    chunks. `asLanguageModelV3` sees `"v3"` and hands it straight back
    unconverted — it does not even print its V2 compatibility warning any more.
    `@link-assistant/agent@0.25.0` wraps its model to rewrite prompts
    (`src/session/prompt.ts:1003`) and floats `"ai": "^6.0.1"` beside
    `"@ai-sdk/openai-compatible": "^1.0.32"`, whose models declare
    `specificationVersion = "v2"`, so its `finish` chunk carries a bare string.

    That was harmless until 2026-08-20T16:03Z, when `ai@6.0.260` shipped this
    line:

    ```
    - 9e15cb4: Prevent automatic tool execution when a model call ends with an
      unsafe finish reason.
    ```

    The new guard reads the V3 field —
    `isToolExecutionAllowedFinishReason(chunk.finishReason.unified)` — and
    `.unified` on a string is `undefined`, which is an unsafe finish reason, so
    from that release onwards it cancels exactly the tool call the client had
    just decided to make. Instrumenting the call site inside the installed
    `ai@6.0.261` and re-running the unmodified client says it in two lines:

    ```
    [probe] chunk.finishReason = "tool-calls" typeof string
    [probe] finishReason handed to the guard = undefined
    ```

    and the mutation closes it — forcing that one predicate to `return true`
    makes the same 0.25.0 client produce `ALPHA_MARKER_11111` with no aborts.
    The network-free reproduction reduces the whole thing to a hand-written V2
    model and one middleware that changes nothing:

    ```
    v2 model, unwrapped        : tool executions = 1
    v2 model, wrapLanguageModel: tool executions = 0
    ```

    The caret let it in: the last passing run resolved `ai` at 6.0.259, and the
    first failing run started at 2026-08-20T20:41Z, thirty-one minutes after
    `6.0.261` was published.

    Three local runs pin it down, and the third is the one that matters:

    | tree | client | `ai` | result |
    | --- | --- | --- | --- |
    | this branch | 0.26.0 | 6.0.256 | leg passes |
    | this branch | 0.25.0 | 6.0.261 | `read-file`, `interactive` fail |
    | 91e469774 — *the last commit CI passed* | 0.25.0 | 6.0.261 | `read-file`, `interactive` fail |

    The third row is the finding. The tree CI was green on fails today, on the
    same machine, with no formal-ai code in between — so nothing on this branch
    broke the leg, and re-running the old commit would not have shown that.
    `0.26.0` moves to `"@ai-sdk/openai-compatible": "^2.0.62"`, whose models
    declare `specificationVersion = "v3"` — so the version `doWrap` stamps on
    the wrapper is true, the finish reason arrives as `{unified, raw}`, and the
    guard has nothing to fire on. `clients.lock` on this branch already names
    it, and that is the fix. Note that the escape is a package major, not a
    spec major: provider `1.x` is spec V2 and provider `2.x` is spec V3, which
    is why “bump the provider” and “adopt the newer specification” are the same
    move here.

    The lesson is the one the lock file was written for and did not achieve: a
    version pinned without its dependency tree is not a pin. `bun add -g`
    resolves the client's own carets at install time, so a leg that pins the
    client exactly still installs a different program each day. Recording it
    here rather than raising the round bound is deliberate — a floating
    transitive dependency is a supply-chain fact about the matrix, and the
    honest response is to name it, not to make the assertion looser.

    Filed upstream with the reproduction, per #1021's standing clause on
    third-party defects: [link-assistant/agent#297][agent-297], corrected there
    once the probe disproved the first mechanism. Two things there that this
    branch cannot fix from the outside — `0.25.0` is broken *as published*, so a
    pin to it will keep rotting until the 0.25 line raises its
    `@ai-sdk/openai-compatible` floor; and the client's processor logs
    `inferred tool-calls finish reason from pending tool calls` on the very turn
    it then cancels for having an unsafe finish reason, so the two halves of the
    client disagree about one turn and the generic `Tool execution aborted`
    sends the reader to look at the tool.

    The deeper defect is not the agent's, though, and the honest place for it is
    `vercel/ai`: `doWrap` asserting `specificationVersion: "v3"` over a V2 model
    is what makes a correct provider, a correct server and a correct client add
    up to a dropped tool call. `wrap-v2-repro.mjs` is written to be filed as-is,
    with no formal-ai in it. Filing an issue on a third-party org's repository
    under this account is an outward-facing act that has not been asked for, so
    it is raised on the pull request for a decision rather than done unasked.

31. **A minor bump on a JavaScript override drags a native addon into a bundle
    that cannot carry one, and it is taken anyway.** `browser-commander` is
    pinned by both `desktop/package.json` and `vscode/package.json` inside an
    `overrides` block for `@link-assistant/web-capture`, and the JavaScript half
    of this refresh raised it 0.10.0 → 0.15.0. The 0.16 line adds
    `better-sqlite3@^12.11.1` and, with it, twenty-five more transitive packages
    — `prebuild-install`, `node-abi`, `bindings`, `tar-fs` and the rest of the
    prebuilt-binary toolchain. The VS Code extension does not ship
    `node_modules`: `vscode/scripts/prepare-resources.mjs` bundles
    `desktop/lib/web-tools.cjs` with esbuild and `.vscodeignore` whitelists only
    `playwright` and `playwright-core`, so a `.node` binary resolved at runtime
    through `bindings` has nowhere to live in the VSIX.

    The first draft of this finding held the override at 0.15.0 on that reasoning
    and it was wrong, which is why the measurement is written down here. Bundling
    each graph with the extension's own `bundleWebTools` succeeds every time:
    9,295,234 bytes at 0.15.0 against 11,827,516 at both 0.16.0 and 0.16.1. The
    addon backs `browser-commander/src/browser/browser-cookie-database.js`,
    reachable only from `browser-cookies.js`, and
    `@link-assistant/web-capture/src/browser.js` — the only entry point this
    repository imports — never touches cookies. So the bump is taken: both
    lockfiles resolve with exactly the intended delta and no version churn
    elsewhere, `scripts/check-javascript-dependencies.sh` audits all five
    committed locks clean, and `vscode/scripts/bundle-web-tools.test.mjs` passes.

    The version delivered is 0.16.1, not the 0.16.0 the first pass took. That
    correction came from re-reading the registry rather than the branch: `npm
    view browser-commander time` dates 0.16.0 to 2026-08-02T07:56Z and 0.16.1 to
    2026-08-02T14:39Z, so 0.16.1 was already the newest stable release when this
    refresh started and the first pass simply missed it by six and a half hours
    of publish history. It is recorded because it names the failure mode: "newest
    stable" has to be read from the registry at the moment of the bump, and a
    version that looks new is not evidence that it is newest. The two releases
    bundle to the same byte count, so nothing above changes.

    What stays a finding is the 2.5 MB the VSIX grows to carry a database engine
    nothing in this repository calls, and the fact that no gate would notice if
    the unreachable path ever became reachable — the bundle has a test that it
    *builds*, not one that it *loads*.

32. **One link timing out makes the gate name sixteen healthy links as broken,
    and the gate fails on the timeout either way.** The `Broken Link Checker`
    went red twice on this branch (runs 32242196357 and 32454084765) and once on
    `main`, each time printing `::error::Broken link detected:` over links that
    answer 200. Both branch reports say the same thing in the summary table:

    | Run | Errors | Timeouts | Redirected | Verdict |
    |-----|--------|----------|------------|---------|
    | 32242196357 | 0 | 1 | 19 | failed |
    | 32454084765 | 0 | 1 | 18 | failed |
    | 32455788384 | 0 | 0 | 19 | passed |

    Zero errors in all three. The only thing that separates the passing run from
    the failing ones is a single timeout, and it was a different link each time
    — `rowanzellers.com/hellaswag/` in one, `docs.anthropic.com/en/docs/claude-code/cli-usage`
    in the other. Both answer 200 in well under lychee's 30s budget when measured
    from here: three consecutive requests each, 0.67/0.62/0.51s and
    4.34/0.76/0.82s. The victim is whichever link the runner's network happened
    to stall on.

    Two separate defects sit behind that, and only the first is fixed here.

    The first is a parser bug in this repository. `extractBrokenUrls` in
    `scripts/check-web-archive.mjs` narrowed its deliberately permissive bullet
    matcher to the failure section by searching for one hard-coded heading,
    `## Errors per input`. lychee writes only the sections it has links for, so a
    report whose sole failure is a timeout is headed `## Timeouts per input` and
    that search finds nothing — whereupon the function fell back to parsing the
    *whole document*, harvesting every URL under `## Redirects per input` and
    posting the ones Wayback had no snapshot for as broken links. The fallback
    was not an oversight; it is there for legacy headingless output, and it is
    kept. What was wrong is which reports reached it.

    The fix inverts the selection. Every `## … per input` section is now sliced
    out by heading, and a section counts as failing unless it is one of the
    outcomes known to be healthy — redirects, exclusions, successes, suggestions.
    Selecting by exclusion rather than inclusion is the point: a category this
    parser has not heard of, from a lychee release or a renamed heading, is
    reported rather than dropped. Getting that wrong in the reporting direction
    names a healthy link, which is loud and gets fixed; getting it wrong in the
    dropping direction turns a real broken link into a green build, which is
    silent. `.github/workflows/links.yml` already asserted this property in a
    comment — "a parser regression silently turns healthy redirects into 'broken'
    links (a false positive) or drops real failures (a false negative)" — and
    already ran `node --test scripts/check-web-archive.test.mjs` ahead of lychee
    to enforce it. The enforcement missed because all three tests described a
    report that *has* an errors section, which is the one shape the old lookup
    got right. Four tests now cover the shapes it got wrong, and all four fail
    against the previous parser. The reproduction in
    `experiments/issue-1021-link-checker-false-positive/` runs the real report
    from run 32454084765, captured verbatim from the job log: 17 URLs reported
    broken before the fix, 16 of them links lychee itself had classified as
    healthy redirects; 1 after, which is the timeout the fallback exists to
    check.

    The second is a policy question, and it stays a finding rather than a change.
    `Fail if broken links were found` fires on `steps.lychee.outputs.exit_code
    != 0` and exits 1 unconditionally — it never reads what the Wayback check
    concluded. So even with the parser corrected, a single runner-side timeout
    still fails the gate over a link that is fine, and the `--accept
    '200..=204,429,500..=599'` list cannot soften it because a timeout carries no
    status code to accept. Making that non-fatal means deciding that an
    unreachable-right-now link no longer blocks a pull request, which is moving a
    limit rather than meeting it, so per the standing clause on #1021 it is
    reported here for a decision instead of being applied. Worth weighing
    together: `--max-retries 3` is already set, so these links had three attempts
    against a 30s timeout and still did not answer, and a timeout is genuinely
    weaker evidence of breakage than a 404 is.

33. **Refreshing two pinned clients turned two matrix legs red, and neither red
    was a regression in this repository — one was a contract that had moved and
    one was a probe that had not existed before.** `02f590106` lifted
    `experiments/agentic_cli_matrix/clients.lock` from `t3code 0.0.28` to
    `0.0.33` and from `claude 2.1.215` to `2.1.238`. Run 32455788406 on
    `1149a1118` then failed exactly those two legs and no others, which is what
    makes the refresh the cause rather than a coincidence: `main`'s last matrix
    run (32129820467, 2026-08-18) was green, and this branch's pre-refresh runs
    failed only on `agent`, the leg finding 30 covers and which the
    `@link-assistant/agent` 0.25.0 → 0.26.0 bump in the same refresh fixed. Per
    the standing clause that the gates decide, a bump that trips a gate is
    unfinished work, so both were root-caused rather than pinned back.

    The `t3code` failure is the tripwire working:

    ```
    !! launch: subcommands changed from 'auth connect project serve start ' to 'auth connect pair project serve service start ' — check for a prompt path
    ```

    `t3code` is a web application server, so `case_launch()` in `run_leg.sh`
    cannot hand it a prompt; instead of leaving the *reason* implicit it asserts
    the client's `--help` subcommand list verbatim, so that upstream growing a
    way to drive it non-interactively fails the leg instead of passing unnoticed.
    Two names appeared. Reproduced locally under Node 22.23.2 — the version t3's
    own `engines` field requires, and without which its `node-pty` addon makes
    `--help` print nothing at all — `0.0.28` lists `start serve auth project
    connect` and `0.0.33` adds `pair` ("Mint a pairing token for a running T3
    Code server and print it as a QR code.") and `service` ("Manage the T3 Code
    background service.", with `install`, `uninstall`, `update`, `status`).
    Neither is a prompt path: one mints a token for a server that is already
    running and one manages a background daemon. So the contract is re-recorded
    in `data/seed/client-integrations.lino` and the assertion stays exact. The
    seed file has no comment syntax, so the reasoning lives here and in the
    changelog entry rather than beside the seven `launch_subcommand` lines.

    The `claude` failure needed a disassembly to explain:

    ```
    !! greeting: proxy recorded failing exchanges: 404 /api/anthropic/api/hello
    ```

    with the trace showing the probe landing between a healthy `GET /health` and
    a conversation that worked:

    ```
    [trace] GET /health (0 byte body)
    [trace] HEAD /api/anthropic/api/hello (0 byte body)
    [trace] POST /api/anthropic/v1/messages (88170 byte body)
    ```

    `2.1.238` added a once-per-session connection warm-up. Read verbatim out of
    the shipped `@anthropic-ai/claude-code-linux-x64` binary, it is guarded by
    `providerCache.preconnectFired`, skipped whenever Bedrock, Vertex, Foundry, a
    gateway, an HTTP proxy, a unix socket or a client certificate is configured,
    and otherwise fires:

    ```js
    let t = V.ANTHROPIC_BASE_URL || al().BASE_API_URL;
    fetch(`${t.replace(/\/+$/,"")}/api/hello`, {method:"HEAD", signal: AbortSignal.timeout(1e4)}).catch(()=>{})
    ```

    That it is new was checked rather than assumed: `preconnectFired` and
    `` `${t.replace(/\/+$/,"")}/api/hello` `` both occur zero times in the
    `2.1.215` binary. The twelve `/api/hello` hits that version does contain are
    a bundled `bun init` project template and two connectivity checks
    (`tengu_preflight_check_failed`, `/doctor`), all of which build their URL
    from the hardcoded `BASE_API_URL` and so never reach a local base URL.

    The doubled `/api` in `/api/anthropic/api/hello` is a defect on neither side.
    Our wrapper writes `ANTHROPIC_BASE_URL=http://127.0.0.1:PORT/api/anthropic`;
    `/api/hello` is Anthropic's own endpoint, and
    `https://api.anthropic.com/api/hello` answers — verified by `curl`, with no
    credentials — `200` and `{"message": "hello"}` to `GET`, and `200` with an
    empty body to `HEAD`. Serving an Anthropic-compatible surface means serving
    that too, so `src/server.rs` now answers both spellings and the assertion is
    left alone. The `404` broke no session — the client discards the result, and
    the `POST` two lines later carried an 88,170-byte response — but
    `matrix_assert_proxy_ok` fails on any non-2xx exchange, which is precisely
    the situation issue #671 already fixed for the bare base path, for the same
    reason: a probe answered `404` reads in a transcript exactly like a base URL
    pointing nowhere.

    `tests/integration/issue_1021_client_preflight.rs` holds three tests, and
    they were falsified against the unpatched server rather than merely run:
    the `/api/hello` one fails with ``assertion `left == right` failed: HEAD
    /api/hello  left: 404  right: 200``, while the other two — every published
    base path answers a `HEAD` probe, and the hello route does not swallow paths
    beneath it — pass both before and after, which is what an honest regression
    guard for already-correct behaviour looks like.

34. **A step that ran out of time said how long it took and nothing about what
    made it slow.** The head that carries finding 33 went green everywhere
    except one job: `Test (macos-15-intel / specification)` was terminated at
    its full 1200s budget (run 32463873155, job 96716556814,
    `logs/macos-specification-shard-runner-rate.log`). It is not a test
    failure. The last line before the deadline is
    `rustc --crate-name formal_ai --edition=2024 src/lib.rs`, so the shard was
    still compiling its own library and no test had started; steps 8, 9, 10 and
    12 never ran.

    The same shard on the previous commit had compiled the same work in 566s
    and passed 1037 tests in 267s, finishing at 838s of the same 1200s budget
    (job 96699699539). Two runs an hour apart, one lockfile, opposite verdicts.
    The question is which of the two candidate causes it was — a slower runner,
    or a compiler cache that stopped answering — and *the logs cannot tell them
    apart*, because cargo prints ``Running `sccache rustc ...` `` on a cache hit
    exactly as it does on a miss. Narrowing it took
    `experiments/issue_1021_compile_rate_compare.py`, which matches each crate
    against itself across the two megabyte-scale job logs, since cargo schedules
    ready units across jobserver slots and the *order* differs run to run even
    when the *set* does not. The set was identical — 480 crates in both — and
    the red run was 2.4x to 2.6x slower at every decile of the green run's
    progress, uniformly, from `hashbrown` at 20% through `formal_ai` at the end.
    Uniform rules out a *partial* cache difference — some crates hit, some
    missed would be lumpy — but it does not separate a uniformly slower machine
    from a cache that answered everything in one run and nothing in the other.
    Which is the point: the measurement narrows the question and cannot close
    it, because the one number that would close it was never printed.

    Re-run on the same commit with nothing changed, the shard passed in 620s —
    52% of the budget. The commit that carries the reporter, on the same
    lockfile, then took 603s (job 96736754559). Four observations of one piece
    of work: 603s, 620s, 838s, 1200s-and-terminated.

    One thing that looked like the cause and is not, because ruling it out is
    part of the answer: both runs logged
    `Cache not found for input keys: macOS-cargo-<hash>, macOS-cargo-`. That
    restore can never hit. Every consumer of the `macOS-cargo-*` key family in
    the repository is `actions/cache/restore@v5` — the `test` matrix here, and
    `build-archive` in `macos-core-tests.yml` — while the only step that
    *writes* that key family, `lint` in `release.yml`, runs on `ubuntu-latest`
    and therefore writes `Linux-cargo-*`. Nothing populates the macOS side, so
    the restore is a guaranteed miss on every run, forever. It is also not worth
    what it looks like: the download window it would have skipped is 19s in one
    run and 20s in the other, against a 1200s budget. Naming it and measuring it
    is the finding; changing it would trade a 20s saving for an upload of the
    whole registry on a runner family that is already the slow one.

    What changed is the reporting, not the budget.
    `scripts/run-with-budget-warning.sh` is the script that prints the
    `::error` naming the blown budget, so it is where a reader is standing when
    they ask why; it now asks sccache for its counters at the two moments a
    budget is in trouble — the 70% warning and the termination — and only when
    `RUSTC_WRAPPER` names sccache, so the Xvfb install of finding 14 stays
    exactly as quiet as it was. Three tests in `tests/unit/ci-cd/issue_1021.rs`
    pin it, and the two positive ones were falsified against the previous
    script. That is the standing debug-output clause applied to CI: the next
    person to meet this red job reads the answer instead of narrowing it.

    Half of that is confirmed on a real runner and half is not, which is worth
    saying rather than rounding up. Job 96736754559 finished at 603s, half the
    budget and below the 70% threshold, and `grep -c '[budget]'` over its log
    returns `0` — a healthy step is exactly as quiet as before. The other half,
    counters printed by a step that is genuinely terminated, has still only been
    observed against the stand-in sccache the unit tests drive, because no CI
    step has blown its budget since. It stays *not yet confirmed* in the
    traceability table until one does.

    What deliberately did **not** change is the 1200s. The shard's own error
    text offers three ways out — speed up, repartition, or raise the budget with
    the job timeout — and the third is the one the standing clause forbids
    taking unasked. Repartitioning is the interesting one and is a decision
    rather than a cleanup: two thirds of this shard's budget is spent compiling,
    not testing, so a budget meant to bound test work is mostly bounding a cold
    build. Splitting it into a budgeted `cargo test --no-run` and a budgeted
    test run would give each phase a deadline that still expires before the
    35-minute cap, which is issue #1017's invariant. It would also raise the two
    budgets' sum above 1200s, which is why it is written here as a question for
    review and not applied.

35. **A cache that failed to download failed the job that could have run
    without it.** Verifying finding 34 produced its own evidence. Two workflows
    on commit `41c543964` — `Task Ladder` and `Question necessity ratchet` —
    went red in `Run ./.github/actions/setup-sccache`, before a line of
    repository code ran, because github.com's release CDN answered `504` to the
    sccache download three times in a row at 10:49Z
    (`logs/sccache-release-cdn-outage.log`). The action's own retry ladder waits
    13s and then 16s; the outage outlasted it. A re-run of each, same commit,
    nothing changed, was green — so this is an upstream hiccup and not a defect
    in this branch.

    What makes it worth writing down is the shape rather than the outage. The
    action's own log says `Enable sccache for Rust ... outcome=skipped`: when the
    install fails, `RUSTC_WRAPPER` is never exported, so the job would have
    compiled fine without the cache, only slower. The install step is fatal
    anyway, which is how a missing *accelerator* becomes a failed *job* — in 12
    step invocations across 8 workflows.

    The one-line change is obvious and is deliberately not taken here, because
    it is a decision rather than a cleanup, and finding 34 is exactly why. A job
    that silently proceeds without the compiler cache is a job compiling cold,
    and the budgeted steps on this branch are sized against warm builds — the
    specification shard has been observed at 603s, 620s, 838s and
    terminated-at-1200s on identical work. Tolerating a failed cache install
    would convert some fraction of these outages from an honest red step into a
    budget termination whose cause is one layer further away. It is worth doing
    *with* the counters from finding 34 in place, which is now true, and worth
    saying out loud before it is done.

36. **Ninety-nine security alerts, none of them about what the code does.** The
    `CodeQL` check on commit `800f5f7ff` failed with "99 new alerts including 98
    critical severity security vulnerabilities"
    ([run 96753666549](https://github.com/link-assistant/formal-ai/runs/96753666549)).
    Ninety-eight of them say the same sentence -- *This hard-coded value is used
    as a salt* -- and the ninety-ninth says *This operation writes session_id to
    a log file*. Both are produced by the spelling of an identifier and by
    nothing else.

    `rust/hard-coded-cryptographic-value` makes a sink out of every positional
    argument that reaches a parameter *literally named* `password`, `iv`, `nonce`
    or `salt` (`HeuristicSinks`, upstream
    `rust/ql/lib/codeql/rust/security/HardcodedCryptographicValueExtensions.qll`);
    nothing checks that the callee is cryptography. `sample_index(probabilities,
    impulse, salt)` in `src/translation/selection.rs` hashes with FNV-1a and uses
    the string only to make a draw reproducible, so all 98 alerts are the
    `temperature`, `guess_probability` and `questioning_rigor` floats -- `0.7`,
    `0.4`, `1.0` -- that 24 files write into a `FormalizationSelectionConfig`,
    each reported as a hard-coded cryptographic salt.
    `rust/cleartext-logging` classifies any name matching
    `session.?(id|key)` as account information
    (`HeuristicNames::nameIndicatesSensitiveData`, upstream
    `shared/concepts/codeql/concepts/internal/SensitiveDataHeuristics.qll`), and
    `src/cli_improve.rs` printed FNV-1a digests of recorded sessions out of a
    binding called `session_id`. Full evidence in
    `logs/codeql-name-heuristic-alerts.log`.

    The alerts predate the branch, which is worth stating because it is *not* a
    defence: `?pr=1027` and `?ref=refs/heads/main` return the same 101 alerts,
    identical on rule, severity and path, and the check's own summary says
    "Alerts not introduced by this pull request might have been detected because
    the code changes were too large" -- 1299 files change here. So the check is
    honest about the code base and uninformative about the diff. It is still
    red, and the alerts are what has to go.

    The fix renames the things after what they are: `salt` -> `seed` through
    `selection_seed`, `sample_index` and `seeded_unit_interval`, and
    `agent_session_ids` -> `agent_session_digests` through `src/promotion.rs`,
    `src/promotion/materialize.rs` and the evidence line `src/cli_improve.rs`
    prints. Nothing observable moves: the seed *strings* are assembled from the
    same fields in the same order, and `stable_id("promotion_agent_session", ..)`
    keeps its literal prefix, so every draw and every digest is byte-identical
    before and after. That is the whole change -- a rename is the correct fix
    when the finding is that a name claimed something the value is not.

    A rename holds only until someone spells it the old way again, so
    `tests/unit/ci-cd/codeql_sink_heuristics.rs` reads both heuristics out of the
    tree instead of pinning the two sites: it parses every Rust file CodeQL
    analyses -- the same set, minus the `docs/`, `dev/` and `experiments/`
    prefixes `.github/codeql/codeql-config.yml` ignores, which
    `the_scan_skips_the_same_directories_the_codeql_config_ignores` keeps in
    step -- and fails on any parameter named after a cryptographic sink or any
    account-shaped name handed to a logging macro, including through an inline
    `{capture}` in the format string. `experiments/issue-1021-codeql-name-heuristics/falsify.sh`
    reverts the renames and runs the guard against the tree that produced the
    alerts: it reports `src/translation/selection.rs:324` and `:352`,
    `tests/source/translation/selection.rs:297` and `:325`, and
    `src/cli_improve.rs:84` -- the exact two mechanisms behind the 99 -- and
    passes once they are put back.

    Two things were deliberately not done. The first is the two remaining
    `rust/cleartext-logging` alerts, in
    `tests/unit/docs_requirements_issue_917.rs` and `_918.rs`, where
    `let session_id = read(evidence.join("session-id.txt"))` really is an Agent
    CLI session id and the assertion it feeds is what proves the committed
    evidence came from a real session. The heuristic has correctly identified
    what the variable holds; renaming it would be spelling around a true
    positive, and dismissing the alerts would be moving the check rather than
    the code. They stay open and are named here. The second is the observation
    that `salt` probably does not belong in `HeuristicSinks` at all: upstream
    already excludes `key` from that same list for being too false-positive
    prone, and this branch is a 98-alert reproduction of the same objection for
    `salt`. That is an upstream report with a reproduction attached, not a
    change we can make, and it is proposed here rather than filed, because
    opening an issue in someone else's repository is a decision for review.

    A runner settled it. On 6149a639f both legs are green -- `CodeQL (rust)`
    job 96784436271 and `CodeQL (actions)` job 96784436191 -- and the aggregate
    check that had said *99 new alerts including 98 critical severity security
    vulnerabilities* now says *No new alerts in code changed by this pull
    request* (run 96784677379). Open alerts on the branch went 101 to 2, none
    critical, while `main` still carries all 101, which is what shows the count
    moved because the code changed and not because the query did. The 2 that
    remain are the two named above. The aggregate check spent nine minutes
    reporting "1 configuration not found" while the rust leg was still
    analysing, which is worth knowing before reading a red CodeQL check as a
    verdict: until every configured language has uploaded, the check reports
    the absence of an answer in the same slot where it later reports one.

    One thing the rename broke, and the tree caught it rather than a reviewer:
    `data/meta/self-ast/` holds a committed census of every owned module, keyed
    by content id, so renaming four files' symbols made four census documents
    stale and `issue_673_self_ast_census::committed_census_documents_match_what_the_sources_render`
    fail. `cargo run --example regenerate_self_ast_census` rewrote exactly those
    four. Worth naming because of where the failure surfaced: `Test (ubuntu-latest / full)`
    reports it in a *"Check self-AST census freshness"* step, and locally it is
    850 seconds into the unit suite, so a rename that looks complete because it
    compiles and passes the targeted tests is not yet complete. The lesson
    generalises past this rename -- a derived artifact committed next to its
    source is a second place every source edit has to land.

[agent-297]: https://github.com/link-assistant/agent/issues/297
