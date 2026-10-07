# Issue #1165 — Discovery on the Production Path

Issue [#1165](https://github.com/link-assistant/formal-ai/issues/1165) (E130):
the machinery for coding by discovery was test-only, `write_program` answered
from stored template sets, and nothing on the production path could rediscover
what it knows.

## What this slice delivers

The rediscoverable procedure cache, its data-driven policy, and the bootstrap
conversion of the compiled-in snapshot store:

- `rust/src/discovery_production.rs` — `ProcedureCache` (load / lookup /
  store / delete_all) over `data/cache/coding-procedure-cache.lino`,
  `RediscoverableRecipe` with the FNV-1a `content_id` recomputed on store and
  verified on load, and `cached_or_research`, the miss path that runs
  `research_coding_skill_gap`, stores the verified procedure with its
  rediscovery query and source, and answers the second ask from the cache.
- `data/seed/program-cache-policy.lino` (mirrored under `rust/embedded/`) —
  the policy as data: which fields a row must carry, the hash that addresses
  a program, and the `bootstrap` record whose `active` flag still fronts the
  embedded `ORACLE_SNAPSHOTS`.
- `rust/src/knowledge.rs` — `lookup` and `knows_language` gate on
  `bootstrap_cache_active()`: retiring the bootstrap tier is a seed edit that
  stops every snapshot answer with no Rust change.

## Rediscovery from documentation (2026-10-08)

The miss path answers from documentation instead of a stored program:

- `data/seed/coding-documentation-captures.lino` is pre-cached source data.
  Each capture names a documentation page (URL, rediscovery query, mime), the
  byte-for-byte fixture it was captured into and that fixture's SHA-256, and
  holds the code blocks the page formalizer reads from those bytes. The
  blocks are written by `scripts/generate-coding-documentation-captures.mjs`
  and re-derived from the fixtures by a pin test in each root, so no program
  in the seed was authored.
- The formalizer keeps code indentation (`strip_tags_keep_lines`,
  `pageStripTagsKeepLines`); before the fix every recomposed program came back
  unindented, which is why the #1164 recomposition could not answer.
- On an unmodified `write_program` miss, `answer_cache_miss`
  (`rust/src/discovery_production_documentation.rs`, JS twin
  `rediscoverDocumentedProgram`) decomposes each captured page's example,
  binds the task's expected output into its literal, keeps the shortest
  program, and verifies it twice: decomposing it again must find the same
  output call printing the expected output, and it must declare the names the
  catalog run contract invokes. The derivation logs `procedure_cache
  outcome=discovered` with the page and content id; a runtime cache
  (`FORMAL_AI_PROCEDURE_CACHE`) stores the row and the next request hits it.
  The committed cache file is never written by a solve.
- Result on the committed captures: Rust (Rust Book), Go (go.dev) and Kotlin
  (kotlinlang command-line page, shorter than the tour's example with its
  comment line) are rediscovered byte for byte equal to the retired
  templates; Scala is refused, because the Scala book's `object hello` cannot
  run as the catalog's `scala Main`.
- The Rust, Go and Kotlin Hello World programs left
  `data/seed/hello-world-programs.lino` (their rows now carry
  `program_source "documentation_route"`) and the worker's template table; the
  no-memorization ratchet fell from 14 to 11, and the captures are counted on
  their own ratchet (7 page literals).

## Every catalog language (2026-10-08, fifth pass)

Official pages were fetched live for every remaining catalog language and
each rediscovered program was saved under its run contract and run:

| Language | Page | Result |
| --- | --- | --- |
| Python | wiki.python.org BeginnersGuide/Programmers/SimpleExamples | `print('Hello, world!')` |
| JavaScript | MDN console (mdn/content markdown) | `console.log("Hello, world!");`, the template byte for byte |
| TypeScript | TypeScript handbook Basics (markdown) | `hello.ts` with its comment line |
| C | Microsoft C runtime `puts` reference (markdown) | `crt_puts.c` with `puts( ... )` |
| C++ | learn.microsoft.com vscpp-step-1-create | `std::cout << ... << std::endl` |
| C# | learn.microsoft.com tour-of-csharp hello-world | `Console.WriteLine(...)` top-level statement |
| Ruby | ruby-lang.org examples/hello_world | comment header plus `puts` |
| Swift | swift-book GuidedTour.md | `print(...)` plus its comment, answered by the coding oracle (the Swift snapshot is retired) |
| Java | docs.oracle.com Java tutorial (Unix) | `class HelloWorldApp`; the name binds `HelloWorldApp.java` and the page states `javac HelloWorldApp.java` and `java HelloWorldApp` |
| Scala | Scala 3 book taste-hello-world | `object hello`; the `entry_container` row binds `hello.scala`, `scalac hello.scala`, `scala hello` |
| PHP | php.net tutorial first page | `echo "...";` then `?>`; answered with `documentation_deviation trailing_newline=absent` recorded |
| R | none | no official page has a lone `print`/`cat` example |
| Laravel | none | no grammar row; the Artisan docs show no Hello World command |

Later the same day the run contract learned to bind a documented program's
own name (a captured command line, else an `entry_container` keyword), every
command became runtime data with its source recorded in the derivation, and
the coding oracle began reading the documentation route, which brought Java,
Scala, PHP and Swift in: only Laravel's Hello World is still stored.

The ten documented Hello World programs are no longer stored anywhere:
`ProgramTemplate.code` became runtime data and the Rust catalog table appends
the documentation route's program for every retired pair when it is first
read. The no-memorization ceiling fell from 11 to 4. A captured block tagged
`js`, `ts` or `c++` names its language through the `program_language`
meaning surfaces. R1165-6 now derives each catalog command from the
captures (a documented file name bound to the catalog's) and records which
languages' pages state none.

## Wiring plan (the integration commit's part)

1. `pub mod discovery_production;` in `rust/src/lib.rs`.
2. The `SelectedRule::WriteProgram` branch of `rust/src/solver.rs` calls
   `cached_or_research` on the template path; `plan_work_item_execution` in
   `rust/src/agentic_coding/general_execution.rs` does the same for a
   template-backed `execution_recipe` (R1/R2).
3. After the `FORMAL_AI_LIVE_FETCH` rediscovery run covers every language
   row: delete `data/seed/hello-world-programs.lino`, prune
   `entry`/`operation`/`ci_setup`/`documented_source` from
   `data/meta/stdout-program-contracts.lino`, delete the `check_command`
   constants and `ORACLE_SNAPSHOTS`, and retire the bootstrap record in the
   policy seed (R4–R6).
4. Register the policy seed in `data/meta/seed-registry.lino` and regenerate
   the registry surfaces; extend the no-memorization gate (R8) and add the
   three-roots parity cases (R10).

## CI evidence

- Offline: `cargo test --test unit issue_1165_discovery_production` (store
  policy enforcement, round-trip, content-address drift rejection,
  delete-all totality, the offline miss path on the issue #919 capture
  transport, grammar-seed reading, bootstrap gating).
- Online (gated on `FORMAL_AI_LIVE_FETCH`, lands with the wiring): the
  six-language delete-and-rediscover run proving `content_id` stability for
  Kotlin, Scala, Rust, Python, Go, and Pascal.
