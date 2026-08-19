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
