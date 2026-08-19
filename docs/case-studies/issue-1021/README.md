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

## 3. Requirements

The full list, with IDs, lives in
[`docs/requirements/issue-1021-full-range-coding-and-contribution-artifacts.md`](../../requirements/issue-1021-full-range-coding-and-contribution-artifacts.md)
(assembled into `REQUIREMENTS.md` as R1021-1 … R1021-27) and each row has a
traceability entry in
[`docs/requirements-traceability.md`](../../requirements-traceability.md).

In short: seven reported prompts must be answered correctly (R1021-3 … R1021-9),
by fixing the rule rather than the prompt (R1021-2); a contribution must carry a
changelog fragment and a linked pull-request body (R1021-15, R1021-16); the
commands that publish it must sit on a ladder that refuses by default, with
`gh issue create` refused in both states (R1021-10, R1021-11); and all of it must
be test-covered, including the closed circle as a replayable session (R1021-18 …
R1021-21). Four requirements are reported **not delivered** — see §9.

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

**The minimal-script route** stands aside for a task it cannot render.
`names_no_task_beyond_the_minimal_script` in `src/solver_helpers/mod.rs` declines
when the prompt names a catalogued task other than the hello world, or one
`solve_numeric_list` can answer. It is a property of the route, not a phrase.

**The artifacts.** `src/contribution_artifacts.rs` composes the changelog
fragment and the pull-request body from `data/seed/contribution-artifacts.lino`,
holding no prose of its own. `src/contribution_write_path.rs` decides the
publishing commands on two rungs: refused unless `FORMAL_AI_CONTRIBUTION_WRITE=1`,
and never delegated at all (`gh issue create`, `gh pr merge`, the delete
commands). The ladder governs the write path Formal AI takes on its own behalf,
and deliberately not a command an operator names — #749 pinned `execute git push`
as explicit passthrough and #687 pinned "report this on GitHub"; refusing those
would be the over-refusal #824 reports.

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
```

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
3. **#723 is answered in PHP, not in Laravel (R1021-8).** A framework scaffold is
   a different capability from a language template, and inventing one to satisfy
   the reported prompt would be exactly the specialization the issue forbids.
4. **E94 and E95 are untouched (R1021-12, R1021-13).** Versioned recoverable
   memory and a stuck-recovery limit are properties of an unattended run;
   nothing here runs unattended, so there was no honest way to test them.
5. **The ladder does not govern operator-named commands**, by design — see §7.
6. **`мне нужен код` with no language named still reaches web search.** The
   asking verb is now grounded, but a coding request with no artifact and no
   language named is not yet recognised as one. It is outside the reported range
   and is left as found.
7. **`src/coding/catalog/mod.rs` has pre-existing drift from its `tests/source`
   mirror**, which is why the mirror compiles green while differing from `src/`.
   Not introduced here, not fixed here.
8. **No upstream defect was found (R1021-26).** Every root cause traced back into
   this repository; the one third-party constraint met — Links Notation having no
   comment syntax — is documented behaviour, not a defect.
9. **#863 and #862 are half delivered (R1021-6, R1021-7).** The misrouting they
   reported is gone: neither prompt reaches `cp` any more. Neither is *answered
   with code* either — both reach web search, measured and preserved in
   `logs/named-exercise-routing-after.log`:

   ```
   === Give me example of how to do copy stdin to stdout in Rust
   -- intent: web_search
   === Execute https://rosettacode.org/wiki/Copy_stdin_to_stdout in Rust
   -- intent: web_search
   === write a Rust program that copies stdin to stdout
   -- intent: write_program_skill_gap
   ```

   The third line is the honest half: asked directly, Formal AI says it cannot
   derive the program rather than guessing one. The blocker is structural, not a
   missing phrasing rule. `ProgramTask` in `src/coding/catalog/types.rs:31` is
   `{ slug, label, output }` — a task's verified output is a function of its
   source alone — and every path that runs a generated program sets
   `.stdin(Stdio::null())` (`src/agent.rs:358`,
   `src/client_integrations/global_verify.rs:305`,
   `src/orchestration/runner.rs:783`). A program whose output *is* its input
   cannot be stated, let alone verified, under that contract. Delivering it means
   giving `ProgramTask` an input fixture and threading it through the execution
   harness, then rendering the task in all thirteen catalogued languages — a
   capability, and out of scope for this branch. Adding a `copy_stdin_to_stdout`
   template for Rust alone would be exactly the specialization R1021-2 forbids,
   so it was not done.

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
    converts a slow success into a certain failure. Left for its own issue
    rather than folded in here, where it would arrive without the reproduction a
    stand-in mirror that is slow rather than stalled would need.
