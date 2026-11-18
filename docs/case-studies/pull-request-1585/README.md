# PR 1585 development and CI record

This is the CI and development record of
[PR 1585](https://github.com/link-assistant/formal-ai/pull/1585). The sections
below are in time order. The earlier rounds describe the lexical preview that the
generalized engine later replaced. The current state of every acceptance criterion
is in the [issue case study](../issue-1584/README.md) and the
[acceptance table](../../../experiments/issue-1584/acceptance.md).

The initial head was `552ffaf803ea4399dbe4806f8bfc3efd8797eecf`. Eleven
workflow runs created at 2026-11-17T11:37:01Z completed as `action_required`
without jobs. Log retrieval returned no available logs. Raw metadata and each
retrieval error remain in the initial issue evidence archive. These results
are not passing CI or evidence of source failures.

## First implementation CI round

The implementation commit is `409d72d08`; evidence commit `4ca5c0b57` was the
pushed head. Its 25 workflows, created at 2026-11-17T13:56:16Z, all completed:
21 succeeded, 3 failed, and 1 skipped. Every failed workflow log was downloaded.

| Workflow | Actual failure | Repair |
| --- | --- | --- |
| [CI/CD Pipeline 44097734939](https://github.com/link-assistant/formal-ai/actions/runs/44097734939) | Log line 7632: unused `SymbolicAnswer` import in `rust/src/main.rs:66`; warnings are denied | Remove the import left after extracting the thinking-trace module |
| [Layered CI 44097734472](https://github.com/link-assistant/formal-ai/actions/runs/44097734472) | Lines 8691/8709: worker rewrite exits around the reasoning procedure; four module-map assertions and three source-witness assertions also fail | Pass rewrite drafts through the procedure, retain two worker exits, regenerate the module map and explicitly review/refresh the complete witness |
| [Security 44097734306](https://github.com/link-assistant/formal-ai/actions/runs/44097734306) | Lines 742–751: unmaintained `chrono 0.4.45`, RUSTSEC-2026-0543 | Use the existing exact-version blocked-upstream proof; keep every other audit finding fatal |

The native draft already reaches the procedure inside its cascade closure.
Its census changes from 18 to 19 draft-producing methods; exits outside the
procedure remain forbidden. A new worker regression checks the exact rewrite,
absence of tool calls and presence of the actual procedure execution trace.
The HTTP regression also compares streaming content and its final output report
with the JSON response.

The witness comparison retains all 1185 native declarations: 221 executable
cases and 964 unsupported cases. One unchanged test moved to a split file;
the complete comparison permits that precise move and 111 source-hash changes
across five reviewed files. It rejects other record-shape, assertion or identity
changes. Both complete serialized records and the scalar delta are retained.
The maintained refresh command was used after review; tests never rewrite it.

## Upstream audit acknowledgement

RustSec provides no patched chrono version. Locked parents `command-stream
1.5.3` and `link-calculator 0.22.0` require it unconditionally. The latest
published versions checked on 2026-11-17, 1.13.0 and 0.24.0 respectively, still
require chrono. Existing upstream reports are
[command-stream 238](https://github.com/link-foundation/command-stream/issues/238)
and [calculator 265](https://github.com/link-assistant/calculator/issues/265).
Neither was created by this PR. This acknowledges a maintenance risk; it does
not repair chrono. The wrapper rechecks the all-target, all-feature graph and
fails once the exact crate leaves it. Mocked wrapper integration tests verify
expiry, rejection of a missing proof, and propagation of other audit failures.
The actual dependency graph and advisory database remain CI checks.

## Verification record

The pre-repair run recorded 86 local gate passes, 52 CI-only skips and no gate
failures, plus 20 focused test passes. The repair batch passed nine test files
(82 tests); the HTTP file was repeated after adding streaming assertions. The
first repair gate run found a stale runtime inventory and missing shellcheck
on PATH. The maintained generator refreshed the two changed source hashes;
its eight regression tests passed. The existing local shellcheck binary was
restored to PATH. Both failed checks and the final rerun are retained. The final complete run
passed 86 gates, skipped 52 CI-only gates and failed none. Native
behavior and WASM parity remain unverified; a type check is not execution.

[First-round raw evidence](raw-data/ci-round-one.tar.gz) preserves failed CI
logs, metadata, exact repair requests and complete witness captures; its
[manifest](raw-data/ci-round-one.manifest.json) records hashes and cutoff.
CI for the repair head has not run at this capture point. Later status belongs
to the PR checks for its exact SHA, not to the earlier run.

Supplied instructions and scripts count as zero accepted reasoning/authorship
units. The frozen inventory remains 8 units, 0 independently accepted.

## Second implementation CI round

Repair head `778a3a00c` completed 25 workflows: 23 succeeded, one skipped and
one failed. The native type-and-lint job and Security passed.
[Layered CI 44106197325](https://github.com/link-assistant/formal-ai/actions/runs/44106197325)
passed both JavaScript shards but failed TypeScript currency. Log lines
7504–7506 name the three stale generated worker files. The first repair missed
the ES projection. The maintained ES generator refreshed them; its check now
compares all 720 generated files. The reusable repair verifier explicitly runs
ES currency as well as the full gate set.

Final review separately reproduced a false E-Prime certificate for “’Tis
ready.” The old validator skipped its independent English checker after a
finite forbidden-word filter. Both JavaScript and native code now require that
checker. The new regression fails before the repair; it also checks explicit
positive and negative independent evidence. The old matrix’s two certificates
are withdrawn. New bounded captures contain no compliance certificates without
independent checkers; declared lexical identity is unchanged. Native test
assertions were strengthened, but native behavior remains unverified.

[Second-round evidence](raw-data/ci-round-two.tar.gz) and its
[manifest](raw-data/ci-round-two.manifest.json) retain the complete failed log,
before/after regression results, projection records and repair traces. The
archive cutoff precedes CI for this repair batch.

The latest full inventory passed 85 gates, skipped 52 CI-only gates and failed
only ShellCheck lookup on PATH. A targeted retry passed with the existing
ShellCheck binary available. The earlier inventory failed worker projection
currency; maintained worker and ES generation repaired that drift. Both failed
inventories and the successful retry are retained. All 86 runnable gates passed
across the final inventory and environment retry. Nine focused files passed
83 tests; the HTTP file passed again with an assertion that the public response
reports missing independent E-Prime evidence. ES currency and handwritten Rust
format checks also passed. No native behavior or WASM execution is claimed.

## Main integration and response contract regressions

The merge of `058e9bef1` retains request context, opt-in dialogue retention,
free-text summarization, and worker priority behavior from `main`. The
governed witness still records 221 cases and 964 unsupported declarations.
The merge moved one unchanged test body and changed 111 source hash records.
The later dependency backport changed 16 manifest fixture records, all
checked against the original and current manifest bytes.

The combined E-Prime and lexical-density regression previously rendered a
known forbidden form from a declared equivalent. Candidate selection now
rejects realizations with known violations. Independent E-Prime validation
remains required. The JSON regression reproduced a syntax error when the
dictionary notice became part of the message content. Fixed-format Chat
Completions now retain the body and carry the notice in projection metadata.
The checks also cover exact labels, streaming and tool-call token accounting.
The bounded matrix recorded 880 cases, zero declared identity drifts, and
zero compliance certificates. General semantic preservation remains open.

The complete local gate run passed 83 checks and skipped 52 CI-only checks.
The three failed checks passed targeted retries. ShellCheck needed its binary
on PATH and the source-model check needed the workflow's pinned grammar.
The dependency audit reproduced the new picomatch advisory on both unchanged
`main` manifests. This PR brings over the local patch from [PR #1581](https://github.com/link-assistant/formal-ai/pull/1581).
The package retains the MIT license and npm source hashes. Both lockfiles
pass `npm ci --dry-run` and their audits report zero vulnerabilities.

[Raw integration evidence](raw-data/main-integration.tar.gz) and its
[manifest](raw-data/main-integration.manifest.json) retain failing and passing
checks, the witness review, the projection capture, and the actual Formal AI
requests and traces. The planner executed the supplied merge and backport
commands. The reasoning attempt recovered the inventory without independently
resolving its gaps. The frozen inventory remains at 8 units and 0 independently
accepted. The [acceptance inventory](../../../experiments/issue-1584/acceptance.md)
retains all seven open criteria, including native and WASM behavior.

## Generalized engine round

The maintainer review replaced the preview with the generalized engine described
in the [issue case study](../issue-1584/README.md). The final local gate round
found these problems and fixed them:

- **Stale parity fixtures.** The check ids had been renamed from `_` to `-`
  after the fixtures were written, so the digests no longer matched. The fixtures
  were regenerated from the JavaScript engine.
- **Worker line budget.** The worker solve module grew past its budget. The
  solver hook moved into the shared crate module as `controlledSolve`, the twin of
  Rust `controlled_solve`, which brought the module back to its ceiling.
- **A test that passed trivially.** The density-switch test sent a bare
  `max-precision`. That is not a declared directive, so the solver answered
  `unknown`, which differs from the first answer, and the inequality assertion
  passed. Both twins now send "Use max-precision." and the declared switch
  "Rewrite in max-precision mode", and assert the exact re-rendered answer.
- **Generated surfaces.** The abbreviation ratchet, the planner-twin citation,
  the JS → Rust translation ledger, the runtime feature inventory and
  `docs/status.md` were brought back in line with their generators.
- **A controlled answer skipped the recursive procedure.** The Layered CI census
  of issue #1550 (`rust/tests/web/issue-1550-procedure-census.test.mjs`) failed in
  both runtimes. The hook returned a declared-fact answer or a clarification
  straight to the caller, so that turn never reached `metaRunProcedure` /
  `run_meta_procedure`. Now the hook returns a draft `{prompt, answer,
  projection}` (`ControlledSolve` in Rust) every time, and every draft goes through
  the impasse hand-off and the procedure, the same as a handler's answer. `projection` records
  the controlled text the draft already is, so the output projection does not
  render it twice. The governed specification witness and the debug stage sources
  were refreshed for the new `rust/src/solver.rs` and `controlled_solve` events.
