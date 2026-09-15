# Plan 04 — Final requirements, release proof, and Formal AI self-improvement

This plan was written on 2026-09-15 before changing production code in the
second continuation. It starts at commit e065dc324, the exact remote head of PR
#888. The worktree was clean. The purpose is not merely to turn red checks
green: every failure is evidence that a requirement previously marked done may
not hold on a real surface, and must therefore be re-verified end to end.

## Exit condition

PR #888 is ready only when all of these statements are simultaneously true:

1. Every current CI failure has a reproduced cause and a regression test. No
   gate is weakened, skipped, reclassified, or given a larger timeout.
2. The generated requirements document, its source shards, the traceability
   table, the issue #710 audit, and the implementation agree on what is done,
   partial, superseded, or still owned elsewhere.
3. Coding answers are derived from task structure and source-backed parts.
   Tests may provide examples and expected behavior, but production data may
   not contain benchmark case names, copied task sentences, or case-specific
   bodies.
4. A cold system can forget procedure memory and rediscover the same verified
   construction from trusted cached or live sources.
5. HumanEval and MBPP are measured again through the real online discovery
   path. The current honest local floors, 3/20 and 1/20, may not regress.
   Failures are classified by missing general capability and at least the
   highest-leverage shared classes are implemented and remeasured.
6. The branch version of Formal AI performs additional real work through the
   external Agent CLI. Its failures are retained as evidence and fixed only
   when the fix generalizes beyond the requested example.
7. All pull-request workflows are green on one pushed head. The exact
   merge/release prerequisites are also exercised locally: package contents
   and size, changelog bump, security audit, full tests, browser E2E, Agent CLI
   E2E, box-language corpus, and release preflight.
8. The PR remains a normal fast-forward history, mergeable, and clean. The PR
   body reports measured facts and does not claim external publication can be
   guaranteed beyond repository credentials and third-party service
   availability.

## Evidence at the starting head

The final status snapshot for e065dc324 has four primary failures. Pipeline
Status is only their aggregate.

| Failure | Exact evidence | Requirement put back under review |
| --- | --- | --- |
| Test (ubuntu-latest / full) | data_files::meaning_definitions_are_unique finds sixteen byte-identical genus nodes in meanings-coding-structure.lino; data_files::meaning_seed_uses_id_fact_format finds bare surface any at line 40 | R710-D8, semantic closure, seed-as-IDs doctrine |
| Box Image Projects (python) | generate-box-language-corpus.sh reports that python/en carried no python fence; six other language images pass | R710-D1, R710-D11, R932 project generation, out-of-box Python |
| E2E Tests (local web app) | all three issue-334 Fibonacci tests receive no Python code block; 470 other tests pass | R710-D1, cross-runtime synthesis parity, recursive program discovery |
| Code Coverage | the instrumented suite fails before measuring coverage because issue_716_agentic_execution::responses_routes_program_creation_to_the_advertised_cli_write_tool emits no function call | agent-mode tool contract across OpenAI Responses |

Already-green evidence remains useful but does not cancel a red end-to-end
surface: all desktop build targets, Docker build/runtime, lint, security,
Actionlint, link checking, the Formal AI authorship gate, self-hosting evidence,
the agent CLI smoke job, and 470 browser tests passed.

## Architectural diagnosis to verify

The following are hypotheses, not conclusions. Each must be proved or struck
through with the observed answer and call path.

- The structural recognizer now claims broad write-program prompts before the
  established language-project catalog can answer. The new discovery composer
  currently renders only Python functions, so a request for a Python program
  may become a named gap rather than a fenced program.
- The same precedence change likely claims recursive Fibonacci. The correct
  repair is not a Fibonacci exception. Discovery needs source-backed recurrence
  and recursion parts: base cases, recursive calls on smaller arguments,
  combination of previous terms, and termination evidence. Fibonacci is one
  test; factorial, sum-to-N, and a held-out linear recurrence are the
  generalization tests.
- The Responses API failure may be the same prompt-claiming issue at a
  different boundary: the solver may return code directly instead of asking
  the advertised write tool. The server must preserve the rule that agent mode
  turns a verified program artifact into the best available file-write tool
  call, independent of whether the program came from a catalog or discovery.
- The sixteen structural genus meanings are placeholders rather than genuine
  definitions. They must be grounded and differentiated through real
  relations; adding arbitrary text only to appease uniqueness would violate the
  semantic-data contract. Bare surfaces must use the repository's ID/fact
  representation and remain discoverable through the lexicon.

## L17 — Freeze the plan and reproduce the four failures

Files: this plan and plans/README.md first; only then tests and production.

- [x] current remote head, clean worktree, and all CI conclusions captured
- [x] exact failure messages captured in this plan
- [x] reproduce the two LiNo integrity failures locally
- [x] reproduce the Responses tool-call failure locally
- [x] generate the Python box answer with a fresh isolated cache and retain it
- [x] run the three issue-334 tests against a fresh local web build and inspect
      the rendered answers, not only the missing locator

The local reproductions matched CI. The seed rejected `surface any` and found
sixteen definitions whose complete body was only `defined-by concept`. The
Responses request returned no `function_call` item. A clean Python box corpus
answered with `write program skill gap` and no fenced artifact. In the browser,
the structural recognizer parsed the relative pronoun in “function that
calculates” as the callable name `that`, then reported no discovered parts.

## L18 — Repair the semantic seed as knowledge, not decoration

Use existing repository ID/fact conventions. Each abstract structural genus
must have a distinct, grounded definition body or be merged with the meaning it
actually denotes. Each authored literal surface must use the current nested
`surface` / `text` fact shape (an imported lexical entity may instead use its
L-id) and resolve through the same production parser.

Tests:

- both failing data_files tests, first observed red and then green
- coding_discovery::concepts and coding_discovery::multilingual
- semantic closure, seed registry, hardcoded-language, and generated registry
  checks
- a held-out phrase for each changed structural family

No production consumer may learn raw test spellings as a side effect of the
format repair.

Completed locally in 7432a6807. Every structural surface now uses the nested
fact form and each formerly identical genus has a distinct official Python
documentation grounding. The two initially failing data-file tests now pass,
as do all 15 `meaning_` tests and the full 31-test coding-discovery unit slice.

## L19 — Preserve artifact-to-tool routing across every API

Separate two decisions:

1. discover or derive the requested artifact;
2. when agent mode advertises a compatible write tool, emit a tool call for
   that artifact.

The second decision must accept a verified artifact from any producer. It must
not key off count-to-three, Python, an endpoint name, or a benchmark prompt.
Chat Completions, Responses, Anthropic, and Gemini tests should share the same
contract table, with endpoint-specific encoding only at the boundary.

Tests:

- the failing issue-716 Responses test
- the other three issue-716 protocol tests
- a held-out dynamically composed Python function routed to write_file
- a non-coding answer never routed to a file-write tool

Completed locally in the runnable-program repair. A verified synthesis now
retains a typed `ExecutionRecipe`; the shared protocol boundary selects the
client-advertised write tool without scraping the rendered Markdown or knowing
which producer made the artifact. All six issue-716 tests pass: the four API
protocols, a held-out dynamically composed `count_vowels(text)` function, and
a non-coding explanation that must not write a file.

## L20 — Generalize discovery from functions to runnable programs and recurrences

### Runnable programs

Extend CodingTaskSpec with artifact shape (function, script/program, example,
or project file) discovered from seeded multilingual roles. Rendering chooses
the language requested by the spec and returns the correct fenced artifact.
Existing generic hello-world catalog parts can be source inputs, but the route
may not depend on the Python box test or its exact sentence.

Held-out checks: two output strings, five prompt languages, and at least Python
plus one non-Python target. The full seven-language box corpus remains the
acceptance test.

### Recurrences

Represent a recurrence as formal parts:

- initial or boundary conditions;
- a measure that decreases toward the boundary;
- recursive or iterative transition;
- combination of predecessor results;
- an observation/evaluation request.

Search trusted sources in order: local links and procedure cache, Python
documentation for function/call semantics, Wikifunctions for the mathematical
function/testers, then a licensed algorithm source. Record source, license,
content hash, match reason, and unknown concepts. Produce multiple recursive
and iterative candidates where the task permits them, validate their CST,
execute supplied or source-derived examples, and select by least action.

Fibonacci is an acceptance case, not seed knowledge. Generalization tests use
factorial and a held-out recurrence whose exact prompt and function name do not
appear in production sources. Forget-and-rediscover must reproduce the same
composition ID.

Browser behavior stays honest: if the browser cannot execute Python it may
show the discovered artifact and evidence as unverified, but it must not lose
an already established, deterministic program surface. Native execution must
remain bounded.

The first runnable-program slice is complete locally. `CodingTaskSpec` now
models function versus program artifacts and exact stdout; stdout is extracted
through seeded multilingual slots rather than prompt literals in code. Program
candidates compose seeded print/range/loop structures, execute in the bounded
workspace, and must match exact stdout. Held-out punctuation and cardinal-word
tests pass in English, Russian, Hindi, and Chinese. A fresh real Python box run
also exposed and fixed an outer-instruction-versus-fenced-payload precedence
bug. Recurrence discovery and the second target language remain open below.

## L21 — Make the real surfaces prove L19 and L20

- [x] Python box generation passes in en, ru, hi, and zh with byte-identical
      program artifacts
- [ ] all box-language project generation and container verification jobs pass
- [ ] all issue-334 tests pass locally, including agent decomposition and every
      supported language
- [ ] npm web tests and the entire local Playwright suite pass
- [ ] Agent CLI E2E passes with the branch binary

The isolated Python box corpus passed in all four prompt languages. Every
generated `main.py` was byte-identical with SHA-256
`51d2693342000ac090e8817796032592050e0f0b88d4d3a7ab1112058a169673` and
contained the independently derived `print("Hello, world!")` program. The
remaining checkboxes deliberately stay open until their complete real-surface
runs finish.

Generated bundles are rebuilt only with the pinned Bun version and verified
byte-for-byte after Playwright.

## L22 — Turn benchmark failures into general capability work

Run the current first-20 HumanEval and MBPP slices once with trace capture.
For every failure record:

- recognized spec or recognition gap;
- unresolved concepts and search queries;
- sources tried, source/license result, and depth/budget stop;
- candidate compositions;
- CST and execution failures;
- upstream grader result;
- reusable missing capability class.

Group failures by capability, not task ID. Expected shared classes include
sequence transforms, string normalization, aggregation, pairwise predicates,
recurrences, numeric constraints, and container-shape preservation. Implement
the smallest source-backed operators that solve more than one family and add a
held-out paraphrase/task for each operator before remeasurement.

- [ ] no-memorization scanner passes against both downloaded slices
- [ ] HumanEval first-20 online result is at least 3/20
- [ ] MBPP first-20 online result is at least 1/20
- [ ] final scores and failure-class counts are recorded as dated local
      measurements; scheduled history is not fabricated

## L23 — Use Formal AI through Agent CLI for real additional work

Serve the exact branch binary and use the external Agent CLI, with isolated
memory/source/workspace directories, for at least two bounded subtasks:

1. inspect a failure taxonomy and propose a language-neutral composition or
   recurrence record;
2. author one exact data/document/test leaf whose correctness can be checked
   independently.

If Formal AI returns the wrong route, malformed patch, non-exact payload,
unattributed knowledge, or a memorized case, first add a failing regression to
the general planner/authoring path, then repair and rerun. Keep the successful
leaf in its own commit with model, session, evidence, and PR trailers. Scan any
raw trace before upload; remember that an unlisted gist is readable by anyone
with its URL and therefore may contain no credential or private content.

The first subtask is complete and preserved as a three-run learning record.
Session `ses_f5d403d7effeUKzDDzRkbukTEg` read the supplied taxonomy but ended
without writing its distinct output obligation. After adding the generalized
read/derive/write transaction, session `ses_f5d359949ffer2uHBDol2bJpFh`
exposed a second general bug: Agent's decorated read envelope was parsed as
Links data. The regression now uses that real envelope. Session
`ses_f5d33586affebUtJm4fYjyNxy1` then derived all requested schema fields,
wrote the artifact, read it back, and verified it byte-for-byte. Formal AI's
leaf and successful trace are isolated in commit 6ce9fbac2; both failed traces
are retained with the repair. The repository's pinned secret scanner reports
`No secrets found` across all three evidence directories. The second bounded
authorship subtask remains open.

## L24 — Refresh and re-verify requirements

The previous audit covers #1–#1137 and 296 coding/benchmark records from the
944-record issue #957 corpus. Refresh issues and merged PRs created since that
snapshot, including comments that add acceptance criteria. Reconcile:

- plans/01 rows A, B, C, and D;
- both issue-710 requirement shards;
- the source shards for #395, #412, #559, #698, #848, #873, #914, #919,
  #922, #932, #991, #1071, and #1085;
- REQUIREMENTS.md, ROADMAP.md, VISION.md, docs/benchmarks.md, and
  docs/requirements-traceability.md.

Every done row must name a production path and an automated test. Partial rows
must name the missing behavior and owner. Manual confirmation stays explicitly
not recorded unless a human actually provides it. Regenerate assembled
requirements and run the requirements/document/traceability gates.

## L25 — Prove merge-to-release readiness

Reproduce every release prerequisite that can be proven before merge:

- changelog fragment schema and computed minor bump;
- cargo metadata locked, dependency audit, secrets scan, license checks;
- full Ubuntu test partitions and macOS-compatible focused tests;
- lint, format, generated bundles, package list, and crate-size ceiling;
- release build plus package extraction/smoke test without publishing;
- Docker runtime, box-language projects, local browser E2E, Agent CLI E2E;
- desktop dry-run artifacts already covered by the PR build matrix;
- merge simulation against the exact pinned base commit.

Inspect release.yml conditions to prove that a successful push to main with
the fragment reaches Auto Release, crate publication, containers, GitHub
release, and Pages deployment. External credentials, registries, GitHub, and
network availability cannot be guaranteed by code; the honest guarantee is
that no known repository condition prevents release and every dry-run path is
green.

## L26 — One final delivery and complete CI observation

- [ ] all changes committed in coherent, append-only commits
- [ ] self-hosting metric passes and attributes only Formal-AI-authored lines
- [ ] PR body reports refreshed scores, requirements verdicts, evidence,
      release proof, and exact limitations
- [ ] one normal fast-forward push after local validation
- [ ] every required and informational PR workflow finishes without failure
- [ ] remote head equals local HEAD; PR is mergeable; worktree is clean

Do not stop at “pending” for the final head. A failed job is new evidence and
re-enters the appropriate leaf. A cancelled superseded run is not evidence
about the final head.
