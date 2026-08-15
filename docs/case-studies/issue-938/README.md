# Issue 938 Case Study

Issue [#938](https://github.com/link-assistant/formal-ai/issues/938) asks the
coding-task handlers to execute one shared meta-algorithm builder instead of
merely describing compatible construction stages. The implementation extracts
the private installation-conversion mechanism into
`src/meta_algorithm_builder.rs`, migrates every named Rust and browser handler,
and proves that each path emits the same seven-stage trace exactly once.

## 1. Reproduction and root cause

The red-first test is `tests/unit/issue_938_meta_algorithm_builder.rs`. Before
the implementation, its three tests all failed:

- the source-usage test stopped at
  `src/solver_handlers/installation_conversion.rs still owns bespoke construction logic`;
- installation conversion exposed the meta-algorithm and seven stages but no
  active-surface marker; and
- program synthesis, coding catalog, numeric-list, and rule synthesis exposed
  no shared builder trace at all.

The focused pre-fix run ended with `0 passed; 3 failed` and the post-fix run
with `3 passed; 0 failed`. This was an architectural ownership defect, not a
recognition defect. `installation_conversion.rs` privately owned
`AlgorithmConstructionStage`, `CodingSurfaceProjection`, both constant tables,
the event recorder, the Links Notation projection, and the prose renderer.
Other handlers constructed valid domain IRs with separate code paths, while
PR #424 had only listed them as theoretically compatible surfaces. No import or
executable symbol connected them.

## 2. Evidence timeline

The primary GitHub snapshots are preserved under `raw-data/`.

- 2026-06-11T08:35:29Z: issue
  [#412](https://github.com/link-assistant/formal-ai/issues/412) requested a
  repo-wide meta-algorithm for coding tasks while reporting the numeric-list
  follow-up defect.
- 2026-06-11T13:27:46Z: PR
  [#413](https://github.com/link-assistant/formal-ai/pull/413) opened. It
  delivered the numeric-list compositional builder and was merged at
  2026-06-11T16:54:12Z.
- During #413, the maintainer explicitly rejected narrowing the request to one
  handler and asked for the widest general solution
  ([comment](https://github.com/link-assistant/formal-ai/pull/413#issuecomment-4681963053)).
  The implementation later documented that catalog, program synthesis,
  software projects, and the oracle still used bespoke paths, then asked
  whether to expand the PR or land incrementally
  ([comment](https://github.com/link-assistant/formal-ai/pull/413#issuecomment-4682494177)).
  The question received no answer before merge.
- 2026-06-12T08:04:03Z: issue
  [#423](https://github.com/link-assistant/formal-ai/issues/423) requested
  README/install-script conversion plus a meta algorithm for constructing
  algorithms. PR [#424](https://github.com/link-assistant/formal-ai/pull/424)
  opened at 08:04:50Z and merged on 2026-06-13T09:26:40Z with the private
  installation tables.
- 2026-06-13T08:36:51Z: issue
  [#433](https://github.com/link-assistant/formal-ai/issues/433) reopened the
  cross-coding-surface expectation. PR
  [#434](https://github.com/link-assistant/formal-ai/pull/434), merged at
  11:14:58Z, mapped numeric-list conceptually to the seven stages but still did
  not introduce a shared executable symbol.
- 2026-06-13T15:08:39Z: PR
  [#448](https://github.com/link-assistant/formal-ai/pull/448) introduced a
  further per-topic grounded recipe and merged on 2026-06-14T00:04:48Z. This
  third local representation demonstrated the cost of leaving ownership
  unresolved.
- 2026-08-04T13:49:32Z: issue #938 made the missing extraction and migrations
  explicit acceptance criteria.

## 3. Requirement and migration matrix

| Requirement | Implementation | Verification |
| --- | --- | --- |
| One reusable builder | `src/meta_algorithm_builder.rs` owns event, Links Notation, and prose rendering; both runtimes read the stage/projection definition from `data/seed/coding-idioms.lino` | source-usage test rejects local builders; language lint records four fewer debt entries |
| Installation conversion | Replaces its private tables/record/render functions with `MetaAlgorithmBuilder` | existing installation suite plus shared trace test |
| Program synthesis | Records the builder only after sandbox verification succeeds | direct and en/ru/hi/zh/es synthesis cases |
| Coding catalog | Catalog exposes its construction call; `solver.rs` invokes it for ordinary `WriteProgram` routes | Rust and worker hello-world cases |
| Numeric-list | Records the builder after a complete typed solution exists | code-plus-result list case |
| Rule synthesis | Records the builder after candidate construction and verification | conversation follow-up case |
| Exactly one active path | Catalog invocation is suppressed when a verified rule-synthesis candidate already owns the `WriteProgram` result | seven-stage count must equal 7, never 14 |
| Browser parity | `formal_ai_worker_11.js` parses the same seed definition and all five worker paths call its helpers | focused `worker-mirror.test.mjs` case |
| Same trace shape | Every path records the meta-algorithm ID, active surface, seven ordered stages, and all known projections | unit, worker, and manual example checks |
| Release/roadmap | Changelog fragment and ROADMAP status identify #938 | ordered-list and repository checks |

The shared builder deliberately does not replace domain logic. Each handler
still recognizes its own problem class, builds its own IR, projects its own
target, and verifies its own result. The builder standardizes the construction
protocol and observable evidence so later work can move those operations into
data-driven interpreters without first reverse-engineering five trace formats.

## 4. Resolution of the #413 expand-or-increment question

The answer is: **#413 should have expanded in the same PR.** The original issue
and maintainer feedback explicitly made repo-wide construction the acceptance
criterion, the PR itself identified the remaining bespoke consumers, and no
technical blocker justified changing that contract. Merging the numeric-list
slice without a tracked dependency converted an acknowledged requirement into
two months of architectural drift: installation conversion, #433/#434, and
#448 each added another local representation before #938 restored one owner.

Incremental migration is acceptable only after the shared abstraction already
exists, or when the current PR creates a named blocking follow-up before merge
and states plainly that the original requirement is partial. Each increment
must still add a shared-symbol assertion and an executable trace-shape test for
its consumer. For #938 the module and all four requested migrations are one
cohesive, independently testable change, so splitting them would recreate the
same ambiguity.

## 5. Manual and automated verification

`examples/issue_938_meta_algorithm_traces.rs` is the manual verification
harness. Running `cargo run --example issue_938_meta_algorithm_traces` produced:

```text
installation_conversion: meta_algorithm=shared stages=7 active=installation_conversion
program_synthesis: meta_algorithm=shared stages=7 active=program_synthesis
coding_catalog: meta_algorithm=shared stages=7 active=coding_catalog
numeric_list: meta_algorithm=shared stages=7 active=numeric_list
rule_synthesis: meta_algorithm=shared stages=7 active=rule_synthesis
```

The focused Rust test repeats program synthesis with English, Russian, Hindi,
Chinese, and Spanish wrapper text, covering the issue's four requested locales
plus the repository's current fifth supported locale. The worker test executes
all five browser paths. Existing installation-conversion tests remain the
regression contract for the extracted Lino and prose projections.

## 6. Formal-AI self-coding evidence

The work was decomposed into five smallest independently verifiable leaves:
red-first test, Rust builder/extraction, remaining Rust migrations, browser
mirror, and case-study/release documentation. The real Formal AI server and
real Agent CLI authored the red-first test leaf, satisfying the one-of-five
(20%) authorship floor. Session `ses_ffc90c6ceffewpKNqZTYUy2nYF` completed with
an observed workspace effect; its exact task, run log, and exported session are
stored beside this document. Commit `8a1f8f76` carries the required
`Formal-AI-Session` and `Formal-AI-Evidence` trailers.

The Agent protocol also exposed a useful tool-authoring constraint: exact
literal file requests must preserve the final newline through the protocol's
request parser or format verification can reject byte-correct content. The
successful leaf used the Agent's native file tool and then repository formatting
before the red test run.

This change has no visual UI component, so before/after screenshots and visual
regression tests are not applicable.
