# Pull request #1188: bulk QA, reasoning and coding fixes

Pull request: <https://github.com/link-assistant/formal-ai/pull/1188>

Branch: `qa-reasoning-coding-bulk-fixes` (opened 2026-09-30, 191 commits ahead
of `main` at the time of writing).

Per-issue investigations live in each issue's case study
([#1161](../issue-1161/), [#1172](../issue-1172/), [#1173](../issue-1173/),
[#1174](../issue-1174/), [#1175](../issue-1175/), [#1176](../issue-1176/),
[#1177](../issue-1177/), [#1181](../issue-1181/)). This directory records what
is specific to the pull request: its scope, its CI history, the root causes
found while getting the full suite green, the decisions taken, and the
constraints the work ran under (CONTRIBUTING rule 13).

## Raw data

| File | Contents |
| --- | --- |
| [`raw-data/pull-request.json`](raw-data/pull-request.json) | `gh pr view 1188 --json title,body,comments,reviews,commits,createdAt,headRefName`, commits trimmed to short SHA, headline and date (GitHub lists the newest 100). No conversation comments or reviews existed. |
| [`raw-data/ci-cd-pipeline-runs.json`](raw-data/ci-cd-pipeline-runs.json) | Every "CI/CD Pipeline" run on the branch (51), one per line: run id, created, head SHA, status, conclusion. |
| [`raw-data/ci-cd-failed-jobs.json`](raw-data/ci-cd-failed-jobs.json) | For each failed run (20), the names of the jobs that failed. |
| [`raw-data/commits.txt`](raw-data/commits.txt) | `git log --reverse origin/main..HEAD`: short SHA, date, subject. |

## Scope

The description closes eight issues with GitHub closing keywords
(CONTRIBUTING rule 12):

| Issue | Scope |
| --- | --- |
| #1161 | clients registry `config_env` with JSON/CLI projection; release CLI binaries |
| #1172 | factual QA word-boundary subject match |
| #1173 | the fallback executes the search it offers; no canned "Web search requested" paragraph |
| #1174 | summarization and text-transform handler family |
| #1175 | routing by formalization: software-project phrase vocabulary, a 359-probe routing corpus with a coverage budget |
| #1176 | calendar offsets, statistics, unit conversion, word problems |
| #1177 | nine code-task handlers (review, debugging, explanation, refactoring, tests, regex/SQL synthesis, format conversion, shell compose) |
| #1181 | shared `cli_env` boolean parser and install scripts |

It also carries fixes for #1154, #1155, #1163, #1164, #1165, #1166 and #1184
found on the way, and four standing doctrines stated by the project owner
while the pull request was open:

- JavaScript first, full parity, then translate (R997-R1000);
- the recursive meta algorithm is the main path (R1001-R1012);
- the JavaScript server has full parity with the Rust server, guaranteed by
  CI (R1013-R1015), and no maintained file exceeds 1500 lines (R1016);
- work is delegated to Formal AI itself (R1017).

## Timeline

The full list is [`raw-data/commits.txt`](raw-data/commits.txt).

| Date | Commits | What landed |
| --- | --- | --- |
| 2026-09-30 | `7239c69bc` .. `05620d936` (78) | The bulk issue batch: handlers behind data-driven cue tables, registries, ledgers re-measured. The branch did not compile. |
| 2026-10-06 | `9e2f73b14` .. | Build restored (missing `version_resolution` module, lifetimes, the sha2 0.11 hex helper, 189 duplicated test-module declarations); js -> ts translator in JavaScript (`bc0986862`); 35 native-only handler rows ported to JavaScript; recursive meta reasoner. |
| 2026-10-07 early | `5a56705b5`, `8f1d3aaa2`, `77b7c86d2` | JavaScript server at Rust parity; 1500-line ceiling on every file (stylesheet, app, specs, requirements, changelog split); local web tests serialized. |
| 2026-10-07 | `ca1e24848`, `e00b3baa5`, `3836ea751` | Ladder rungs and small delegated tasks derive by general rules (R1009, R1017); coding-ladder floor corrected 24 -> 23. |
| 2026-10-07 | `46d45c9e1` .. `7688c93db` | Full-suite repairs, 234 test-target clippy fixes, workflow hardening. |
| 2026-10-07 | `7d4f110d2`, `6872307e4`, `a0e1786ee`, `49447ab5c` | HTML walker, role registry, agentic call arguments, REST issue URLs, quoted-literal masking. |
| 2026-10-07 | `2e1d6cb62` | Lean and Rocq keep the negation of a "No ..." statement. |
| 2026-10-07 | requirement-drafting batch | Every planned open row drafted: #1163/#1164 JS twins and working memory, #1172 subject-verified facts, #1176 word relations and Wikidata-checked unit factors, #1177 GROUP BY, #1180 history store, #1184 rule stage and JS twin, #1185 honest repair stops, #1186 deformalization and probe set, #1187 optional credentials. |
| 2026-10-07 | `a3340286e`, `065cb49af` and the next commit | #1177 breadth. The code debugger gets two more defect classes, a loop bound past the end of an indexed collection and an assignment inside an `if`/`while` condition (R1). The SQL composer reads a threshold after the grouping as HAVING, no longer emits a bogus `WHERE with > 5`, and joins on a stated shared column, keeping identifier underscores in both runtimes (R3). The shell composer gets a literal `sed -i` substitution from seeded cues (R4). |
| 2026-10-07 | fourth-pass bulk batch | Open rows drafted again in one batch. #1163: the six extractors on fixtures, the unmatched-need research route, the fetch path reading working memory, computed trust features, the `page_query_text` route, and HTML/Markdown document sources. #1172-3: a live Wikidata resolver (not yet wired). #1165-1/2: the procedure cache in the solver's write-program path. #1166-3: obligation-graph clause reading. #1168-6/8: no literal version left, and a JS live resolver. #1180-3/10/11: requirement and state evidence, a `repository_lineage` route, and JS twins. #1184-1/8: `derivation_id` on worker and server answers. #1185-1/2/7/8: a 14-language diagnostic table that found a PHP location bug, plus the TS root for `js/agentic` (175 `.mts` files). CI's first full run found a Wikidata-cache convention break in the P2370 table, so it moved to `data/cache/unit-conversion/`. |
| 2026-10-07 | fifth-pass batch | CI run 37587719623 on `1935f8f0d` was read in full before the next push. Lint failed on 21 clippy lints and a missing P856 cache pair; the specification lane failed on two pins; the full lane stopped at the census step. All are fixed in one commit. Same batch: #1187 R2/R4/R5 completed (the checks mode on every pull_request workflow except four with named reasons, a separate private repository at the App layer, one tracking issue at the default layer) and #1161 R7 drafted as a matrix case that starts each CLI from its relocated config. The full test lane now runs as four parallel shards. The drafting agents of this pass added the following. #1164: a Pascal CST grammar and a decomposition from freepascal.org, plus compile-and-run tests. #1172-3: live Wikidata facts wired into `fact_lookup`, and #1172-8 explanation research. #1163/#1164: byte-exact real captures pinned by SHA-256, and Agent CLI evidence. #1175-3: claim rows in the capability table. #1180-10: repository questions (status failure, function lineage). #1185-3: an applied, CST-validated repair. #1186-4/6: a real `rml` step recorded in the derivation. #1169-4: box-dind 2.10.2. JS mirrors: derivation persist/load (#1184-9) and the procedure-cache miss evidence (#1165-10). |

## CI history

51 "CI/CD Pipeline" runs: 20 failed, 30 were cancelled by a newer push
(the workflow's concurrency group), one in progress at the time of writing.

| Phase | Failed runs | Failing jobs per run |
| --- | --- | --- |
| Opened; branch did not build | 36707093714, 36742616682 (2026-09-30) | 4, then 14: build, lint, both test shards, all seven box-image projects, both E2E suites |
| Build being restored | 37470366082 .. 37513029093 (2026-10-06) | 12-14 while the lib-test build failed; 5-7 in the runs where it built (37485467245, 37489943444, 37491306927) |
| Full unit suite runs | 37516756207 .. 37541942966 | 5-7: lint, both test shards, both E2E suites, Docker, server parity once |
| Narrowing | 37549001684 .. 37570930929 (2026-10-07) | 4, 3, 2, 3, 3: only `Lint and Format Check` and `Test (ubuntu-latest / full)` remain |
| Follow-up | 37594789849, 37599699577 | clippy, rustfmt and the relocated agent case fixed |
| Green except one gate | 37605012854 (`4402656eb`) | 1: only `check_javascript_dependencies` in `Lint and Format Check`; every test shard, E2E, Docker and the agentic CLI matrix passed |

Inside the test jobs, the first complete unit run reported **113 test
failures plus 7 failing lint gates** (test counts as recorded in the pull request
description); after the full-suite repairs (`46d45c9e1` and the
lint follow-ups) the next complete run reported **18**. The latest completed
run, 37570930929 on `6872307e4`, still failed `Lint and Format Check` and
`Test (ubuntu-latest / full)`; those failures are addressed by `7d4f110d2`,
`6872307e4`, `a0e1786ee`, `49447ab5c` and `2e1d6cb62`, which the in-progress
run covers.

The failed-job sets per run are in
[`raw-data/ci-cd-failed-jobs.json`](raw-data/ci-cd-failed-jobs.json).

## Root causes found and decisions taken

Every failure was traced to a root cause; no test was relaxed to pass.

1. **Coding-ladder floor 24 -> 23, with a "corrected overcount" note**
   (`3836ea751`, gate in `46d45c9e1`). `multi.doc_and_requirement` printed
   the same open impasse as when the floor was recorded; its verify step
   grepped for `coding-task` in `REQUIREMENTS.md`, which the pre-split file
   happened to contain. After the 1500-line split the word was gone and the
   spurious pass disappeared. Decision: record the honest floor of 23, and
   make the gate accept a falling floor only when a "corrected overcount"
   note names the old floor, so a real regression can never be relabelled.
2. **Worker modules under the 1500-line limit** (`46d45c9e1`, R995/R999,
   R1016). `formal_ai_worker_20.js` grew with the JavaScript twins; its
   helpers moved into separate modules so it sits under the 1400-line warning
   band (1331 lines now), and `prompt_variations` was split. A worker module's budget shard may rise only
   when its rationale names the handler keys it ports.
3. **HTML walker** (`7d4f110d2`, #1163, #1164). `web_formalize` jumped from
   `<html>` to `</html>` instead of stepping into container tags, did not skip
   comments and opaque tags, and matched close tags by prefix, so `</p`
   closed a `<pre>`. Close tags now match by full name; the comment delimiters
   are named seed data so the literal ratchet holds (`a0e1786ee`).
4. **Role registry from `MEANING_FILES`** (`7d4f110d2`). The registry was
   generated from a different file list than the one the lexicon loads, so the
   roles of `software-project-phrases` were undeclared. It is now generated
   from exactly the files the lexicon loads.
5. **JSON-object call arguments** (`6872307e4`, #1154). A replayed tool call
   whose `arguments` is a JSON object, not a string, lost its command. Fixed
   in Rust and in the JS server.
6. **REST `/issues` URL** (`6872307e4`, #1155). `gh api` and `curl` reads of
   an issue map to `/issues/{n}`, the URL the plan targets; the test reads the
   two-call `gh api` template.
7. **Quoted-literal masking** (`6872307e4`, #1166). Sentence and clause
   splitting masks quoted literals, so an output literal like
   `"Hello, World!"` survives decomposition. Rust and JS.
8. **Lean and Rocq negation** (`2e1d6cb62`). The conjunctive-clause templates
   hardcoded an existential instead of reading `{quantifier}`, so "No cat
   that sleeps hunts" rendered positive. Lean reads the quantifier; Rocq gains
   a `clause_negative` template that parenthesizes the existential under `~`.
   Rust and JS twins.
9. **WASM worker refreshed from a CI artifact** (`7d4f110d2`). Without a local
   Rust toolchain the committed `js/formal_ai_worker.wasm` cannot be rebuilt on
   the workstation. The release workflow now uploads the CI-built WASM worker
   as an artifact, and the committed file is refreshed from that artifact
   rather than from a local build, so it matches the exact toolchain CI
   checks it with.
10. **Seed loaders one level too shallow** (2026-10-06). Ten handlers read
    `.lino` fields at the child level while seed files nest one wrapper deeper,
    so their cues silently never matched; the worker registry's permutation
    assertion threw inside a silent catch and emptied the lexicon. All ten
    loaders descend one level; the precedence seed lists every key.
11. **Rust follows JavaScript.** Porting the 35 native-only handlers exposed
    Rust handlers that misread their own seeds (TRIZ by record name, the
    formalization seed nesting, creative-composition roots, a double `≈` in
    unit conversion, code_debugging's whitespace scan); each was fixed to
    match the JS twin.
12. **Stale pins hidden behind earlier failures.** The full suite of run
    37574144432 failed only on the issue #918 metadata-gap pin (737, while the
    shards held 770): every earlier leg had died at the census-freshness step
    before the unit tests ran, so 33 reviewed-data gap rows from the seed
    growth of #1163 to #1187 accumulated unpinned. The pin now records the
    recount, including the eight meanings this batch added.
13. **Rules reach the derivation by data.** Instead of teaching
    `derivation.rs` the event names of the text-transform handlers, the
    derivation schema seed declares which event kinds a `rule` stage collects;
    a new handler joins `formal-ai explain` by naming its kind there (#1174 R9).
14. **A stopped loop says why.** The repair loop's ladder-exhausted and
    unresolved-need notes and its attempt chain existed but never reached an
    answer; the executor now appends them to the failure report in both roots
    (#1185 R4 to R6).

15. **A composite action cannot read `secrets`.** The credential resolver
    referenced `secrets.AUTOMATION_*` inside `.github/actions/automation-token`.
    GitHub rejects that at load time ("Unrecognized named-value: 'secrets'"),
    so the self-authored pull request workflow failed before its first step.
    The four callers now pass the secrets as inputs, and a test asserts the
    action reads none.
16. **One key per registry line.** `repository_lineage` was appended to
    another key's line in the worker's handler registry, probably to keep
    `formal_ai_worker_20.js` within its line ceiling. The registry reader
    takes one key per line, so the seed/worker permutation failed. The key
    has its own line now, and a comment line was folded to hold the ceiling.
17. **A supplied page outranks the capability table.** "The command in the
    paragraph that mentions compiler:" followed by a page mentions
    `hello.kt` and `hello.jar`. The capability table read those names as a
    batch file read and answered with a missing-`shell` gap before the
    `page_query_text` handler ran. `page_query_text_claims` now makes the gap
    yield, the same way `software_project_claims` does.
    Run 37594789849 found the same preemption for "Which issue introduced
    scripts/…?": `repository_lineage_claims` makes the gap yield to a cued
    lineage question that names an existing path.
18. **A gate failure no longer hides the suite.** The full lane's data and
    census gates ran before the unit tests, and their failure skipped the
    tests. That is how 33 metadata-gap rows went unpinned (item 12). The
    test steps now run `!cancelled()`, the lane is split into four shards
    over the prebuilt executables, and the runner reports every failing
    target before it exits.
19. **A silent 90-minute benchmark hang.** HumanEval at its full 164-case
    slice (run 37578344303) printed nothing after the dataset download and
    was cancelled at the job cap. The runner now prints each case before
    and after it is solved, and the dataset download has connect and
    transfer timeouts.

20. **The first sharded run found what the census step had hidden.**
    Run 37594789849 was the first full run in which every shard reached the
    unit tests.
    - A derivation test expected `kind grammar_correction`, but the record
      writer quotes every value.
    - The module map lacked `fact_live`.
    - `data/cache/unit-conversion/` read as an unregistered source. The
      P2370 table is a test fixture, not a source, and moved to
      `rust/tests/fixtures/unit-conversion/`.
    - The native-handler count did not include `repository_lineage`, which
      reads git history.
21. **A generic template must not read a cue as a noun.** The live-fact
    template filled `{relation}` with the relation's first cue phrase, so
    the answer read "the who wrote of War and Peace". A relation now seeds
    its own phrasing (`fact_live_answer_author_of_book`), and the generic
    template is the fallback in both runtimes. The browser test's source
    expectation moved from the value's Wikipedia page to the Wikidata
    snapshot, as R1172-3 specifies.
22. **The relocated agent ran on its own default model.** The matrix
    showed that `LINK_ASSISTANT_AGENT_CONFIG` really relocates the
    config. The CLI loaded the relocated file and found the `formalai`
    provider, but its built-in default model outranks the config's
    `model`. The case now passes the model as `formal-ai with` does, read
    from new `model_arg` fields of `formal-ai clients --format json`.

23. **The hang was a slow solver, not a stall.** With per-case progress,
    HumanEval run 37594798452 solved 89 of 164 cases in 81 minutes (55 s
    mean, `HumanEval/87` 164 s) and was cancelled before its grading pass,
    which runs only after the last case. `benchmark run --offset` and the
    workflow's `shard_size` input split one suite into concurrent read-only
    windows that each grade their own cases; a total job fails unless every
    case was graded, and shards cannot append to the ledger.

24. **New advisories fail an unchanged tree.** The dependency gate turned
    red on `4402656eb` with no lockfile edit: proxy-addr (critical),
    compression, and two advisories with no patched release, `sprintf-js`
    and `braces`. The patched ones are overrides; `sprintf-js` left with an
    override of `global-agent` to 4.1.3, which dropped `roarr`; `braces`
    left with `@vscode/vsce` 4.0.0, which also closes the #1169 currency
    gap. Pinning `electron-builder` 26.5.0 as `npm audit` suggested was
    tried and rejected: it reintroduced a critical `tar` advisory.

25. **Partial rows drafted to their real limit.** A seventh pass closed what
    this repository can close:
    - R1166-4 in all three roots, the browser worker included;
    - R1168-8 with an injected version set;
    - R1164-11 with a solver route to the decomposer;
    - R1184-9 with a JS server route;
    - R1172-7 with statement slots;
    - R1172-8 and R1165-10 with browser and server twins;
    - R1180-11 with the git and store twin.

    The rows left partial are partial for reasons the code cannot remove
    from here:
    - R1165-1/2/4-8 need verified cache rows, and each row needs a human
      reviewer's approval by design.
    - R1172-9 needs relation claims and Russian case forms that the
      Wikidata captures do not carry. A test now names each gap and fails
      once a capture closes it.
    - R1187-8 needs the merged workflow, because an `issues: labeled`
      event runs the default branch's file.

26. **A new meaning must speak every language.** The statement-projection
    meanings first carried en/ru/es only, and the language-parity ratchet
    (910) refused them. The hi/zh surfaces are now seeded. Both runtimes
    decline a sentence in a `verb_final` language, a flag read from
    `formal-targets.lino`, because such a language marks time and place
    with postpositions and the phrase reader expects prepositions. Chinese
    keeps only multi-character surfaces: a single character such as 在
    would have become a topic word in every Chinese composition prompt.

27. **A cached binary must be keyed by everything it embeds.** Server parity
    on `9d0bf539e` reported a new `bundle` divergence. Yet the JS and Rust
    bundle builders are line-for-line twins, the 206-file lists match, and
    the seed mirror is identical. The cause was the shared
    `formal-ai-binary` action. Its cache key hashed `rust/src`, the
    manifests and `build.rs`, but not `rust/embedded/` (308 `include_str!`
    seed files) nor `data/seed/api-cache/` (bundled by `build.rs`). A
    seed-only push such as `3e806b7db` (the zh stop words) therefore
    restored the binary of the previous seed. The Rust server served the
    old bundle, while the JS server read the new one. The key now covers
    both trees. The ratchet stays at 5.

28. **A spread runs at load time; a lookup runs at call time.** The
    numeric claim kinds were merged into `CLAIM_EVIDENCE` with
    `...NUMERIC_CLAIM_EVIDENCE`. In the worker, `formal_ai_worker_16.js`
    loads before the dispatch file, so this worked there. The handler-registry
    test loads the dispatch file alone, though, and the spread threw a
    `ReferenceError` there. The other evidence kinds call their helpers
    lazily, inside the arrow. The spread now falls back to an empty object
    when the numeric table is absent. MBPP's full 500-case slice, measured
    in twenty-five graded windows, rose from 60 to 68 online (R1177-12 is
    implemented).

## JavaScript parity gaps closed on 2026-10-08, and the ones left

The JavaScript-first doctrine (R997) asks for full parity on client and
server. Three known gaps were closed by porting the Rust code into the
JavaScript root and pinning it with tests that run the Rust tests' own
inputs and the committed fixtures.

| Gap | JavaScript twin | Pinned by |
| --- | --- | --- |
| #703 external agent orchestration: only the replay verifier had a twin | `js/agentic/crate/orchestration_{permission,workspace,runner,json_stream,attribution,dispatch,dispatch_error,incremental,session_file,analysis}.mjs`, `recursive_execution.mjs`, `task_decomposition_tree.mjs`, `client_contract_learning.mjs`, `seed_client_integrations.mjs`, and the CLI twin `js/agentic/orchestration_cli.mjs` (`node js/agentic/orchestration_cli.mjs run\|dispatch\|resume\|synthesize\|learn\|replay`) (`44fb58bb2`); the synthesis pipeline it ranks with: `summarization_{dedup,importance,recheck}.mjs`, `relative_meta_logic.mjs`, `statement_verification.mjs`, `translation_formalization.mjs` (`72fc70adc`) | `rust/tests/web/issue-0703-orchestration-{run,dispatch,committed,analysis}.test.mjs` (`33575f0a2`) and `issue-0703-synthesis-pipeline.test.mjs`: the Rust integration tests of #703, #991 and #1069 over a node stand-in agent and real `git`, and the committed #703/#924/#933 sessions, ledgers and `learning.lino`/`proposals.lino` recomputed byte for byte |
| #563 repository resource summarization: a 57-line sentence splitter | `js/agentic/crate/summarization{,_markdown,_dialog,_file,_resource,_identifier,_vocabulary,_meta_language}.mjs` (`24e0dcaec`, `83d303df1`, `b66454d31`) | `rust/tests/web/issue-0563-repository-summarization.test.mjs`: `summarization_pipeline.rs` and the source tests verbatim, the sampled repository files, and 14 conversation summaries captured from the prebuilt `formal-ai` 0.347.0 binary in en/ru/hi/zh |
| Browser worker coding catalog: Scala and Kotlin answered Hello World and copy stdin only (18 pairs missing) | the worker's `WRITE_PROGRAM_TASKS`/`WRITE_PROGRAM_TEMPLATES` tables are gone; `installSeedProgramTasks` installs every task and program from `data/seed/hello-world-programs.lino` (now carrying each task's Rust label), and the worker's Links Notation reader gained backtick strings and the `\t`/`\r` escapes of `rust/src/seed/parser.rs` (`2f169e78b`); a concrete catalog request runs the write-program rows before any promoted handler, as `is_concrete_write_program` does in `rust/src/solver.rs` (`7dd65ee74`) | `rust/tests/web/issue-0921-worker-coding-catalog.test.mjs`: tasks, language rows and all 151 task x language pairs against `data/meta/agentic-coding-catalog.lino` (generated from the Rust tables), Scala/Kotlin answers in english, russian, hindi and chinese, and the three `task_catalog.rs` prompts a promotion used to claim; `lino_parity.rs::lino_seed_task_rows_mirror_every_catalog_task` |

Gaps that remain, each with the reason it is open:

- **#703 synthesis claims are not pinned against a native run.** The council
  test's exact claims, verdicts and sources were derived from the algorithm:
  no prebuilt binary carries the `synthesize` command. The world-model layer
  the Rust merge tests need (`merge_into_context` in `context.rs`, `gather`
  in `gathering.rs`) and the temperature-selection tests of
  `specification/formalization.rs` are not ported; `formalizePrompt` reads
  the concept query and the unquoted translation surface through the booted
  worker realm, so it needs a host.
- **`formal-ai with` legs of #703.** `agent run` with the default Formal AI
  target and the six-CLI wrapper test go through `formal-ai with` and the
  loopback health server, which have no JavaScript twin; the JavaScript runner
  launches `$FORMAL_AI_CONTROLLER_PROGRAM` or `formal-ai` on `PATH` where Rust
  uses `current_exe`, and its tests pin the argv it builds with a stand-in
  controller instead.
- **#703 details no committed test reaches.** `TaskExecutor::split`'s default
  (`balanced_split`) is not ported (orchestration always overrides it);
  `Decomposition::from_links_notation` and the learning gate are not ported;
  serde's wording for malformed `--command`/`--verify` JSON is approximated;
  the JavaScript runner is asynchronous and kills a timed-out process group
  with POSIX signals only.
- **Meta-language parse evidence in summaries.** The Rust build's default
  `meta-language` feature parses 17 grammars (json/yaml/toml/ini/xml/html/css
  before tree-sitter) and adds a "parsed it as X with N syntax links"
  sentence to a code file's summary. The JavaScript root reproduces it for
  Rust only, over the vendored tree-sitter-rust, its four numbers are pinned
  by shape rather than by a native run, and no host installs that parser by
  default, so JavaScript summaries of code files match the featureless Rust
  build.
- **Summarization layers outside #563.** `describe_project` (no twin of the
  project registry), and the #844/#893 layers `context`, `pipeline`, `gathering` and
  `validation` have no JavaScript twin (`dedup`, `importance` and `recheck`
  were ported for the #703 synthesis).
- **Swift and R catalog rows.** The Rust catalog has `swift` and `r` language
  rows (issue #1167) with no template in either root; the worker has no such
  rows and answers Swift Hello World through its coding oracle and R with a
  skill gap. The native answer for those two languages could not be compared
  without a current Rust build.
- **Server parity, not re-measured.** `node scripts/check-server-parity.mjs
  --list` needs a Rust server built from this branch; the only local binary
  is the installed `formal-ai` 0.347.0 (the source is 0.352.1). Against it the
  run reports 132 identical, 13 content and 7 protocol divergences, but the
  protocol ones are fields newer than that binary (`derivation_id` on
  `/v1/responses`, `tool_calls` on `agent_commit_and_push`) and most content
  ones its older catalog and network, so they say nothing about this branch.
  The ratchet (`data/meta/server-parity-ratchet.lino`, ceiling 5:
  `responses_basic`, `responses_openai_path`, `responses_input_items`,
  `responses_stream`, `telegram_message`) is left as CI last measured it.

## Constraints

- **JavaScript first, full parity.** Every requirement lands in JavaScript
  first (or together with Rust); `scripts/check-js-parity.mjs` holds the
  native-only row count at 0, and `scripts/check-server-parity.mjs` compares
  the two servers on 147 requests with a falling ceiling of content gaps.
- **No local cargo.** Rust is built, linted and tested only in CI on push; the
  js -> ts translator was ported to JavaScript so `ts/` regenerates without a
  crate build, and the WASM worker comes from a CI artifact.
- **One node test at a time.** Local verification runs single test files
  serially (`node --test --test-concurrency=1`; `npm run test:web` is
  serialized, `77b7c86d2`) to keep CPU load down on the workstation; the full
  suite belongs to CI.
- **At most two or three sub-agents at once**, each making code changes only.
- **Files at most 1500 lines** (Rust 1000), including this case study and its
  raw data.

## Review checklist

- The description closes #1161, #1172-#1177 and #1181 with `Fixes`.
- Each ratchet (JS parity 0, worker literals 1106, server parity 5, routing
  misroutes js 22 / rust 54, closure gap 1045, ladder floor 23) is measured
  by CI, not asserted.
- `CI/CD Pipeline` is green on the final head SHA before the draft is marked
  ready.
- The Issue 1028 ladder's sixteen missing_proof leaves are a Rust parity gap: after a correct member-list write the native planner plans no observation and restarts on the client's continuation cue, while the JavaScript planner observes and answers.
