# Issue #1634 — false positives, false negatives, warnings and errors in CI/CD

Data in this folder: `issue.json` (the issue), `runs-summary.txt` (failing runs on `main` at
3888187b7 and related branches), `ci-logs/` (gzip-compressed raw logs of every failing run inspected, plus PR #1635
runs `pr-*.log`), `CI-CD-BEST-PRACTICES.md` (hive-mind reference used for the comparison).

## Timeline

| When | Event |
|------|-------|
| 284e24430 | `detect-code-changes.rs` gains `#[path = "github-output.rs"]` and a `#[path]` test module. The three Detect Changes jobs check out a sparse list that does not include them. |
| after that | Every push or PR run of CI/CD Pipeline, Coverage and Layered CI fails in Detect Changes with `couldn't find file scripts/github-output.rs`. Every downstream job is **skipped**, which hides other regressions (a false negative for the whole pipeline). |
| 99a1953c3 / later split | `algorithm_answers.rs` re-adds the removed `SORT_PYTHON` assertion. 75e31ea61 uses the private `formal_ai::concepts`. `prompt_variations.rs` reaches a private sibling import. The unit test target stops compiling (Question necessity ratchet, dependencies-latest). |
| upstream hive-mind 45c85a4 | Appends `HIVE_MIND_DEADLINE_AT_MS` and `HIVE_MIND_FORMAL_AI_DRAFT` to `setEnv`. The Hive Mind draft adapter fails with `shared diagnostic use site changed 0 !== 1`. |
| issue_558 test | Uses the `tempfile` crate, which is not a dev-dependency. The Benchmark Corpus Gate build breaks. |

## Root causes and fixes

| Failure | Root cause | Fix | Verification |
|---|---|---|---|
| Detect Changes (CI/CD Pipeline, Coverage, Layered CI) | The sparse checkout omits `#[path]` modules | Add `scripts/github-output.rs` and `scripts/detect-code-changes-tests.rs` to all three workflows. `release.yml` is regenerated through `scripts/generate-staged-release.mjs`, the source of its sparse list. | `node scripts/generate-staged-release.mjs --check-deployed`; `rust/tests/web/ci-change-detection-checkout.test.mjs` |
| The same class could recur anywhere | No guard existed | `scripts/check-sparse-checkout-rust-modules.mjs` follows `#[path]` recursively for every `.rs` file in every sparse block under `.github/`. It runs in `workflows.yml`. Set `FORMAL_AI_SPARSE_CHECK_VERBOSE=1` for verbose output. | `experiments/issue-1634/sparse-checkout-rust-modules.test.mjs`; the guard fails on the old workflows |
| Guarded staged release authority refusal, js tier release tests | A hand-edited `release.yml` differs from the generated candidate caller | Fix the generator template and regenerate `candidate-release-caller.yml` | 42 + 100 related node tests pass locally |
| Hive Mind draft | The adapter assumed the shared spread closes `setEnv` | Insert at the matching `}` of `setEnv` | `rust/tests/web/hive-draft-logs.test.mjs` with the upstream 45c85a4 fixture |
| Benchmark Corpus Gate | Undeclared `tempfile` crate | Use `std::env::temp_dir()` with explicit cleanup | CI |
| Question necessity ratchet / dependencies-latest compile errors | Private imports, a stale merge, a private module | `pub(super)` fixture re-export; the operation-required assertion from 99a1953c3; concept test moved to `tests/source/concept_query_tests.rs` (a `lib.rs` re-export broke 16 governed spec cases); witness `governed-baseline.v8` refreshed | CI (local rustc is OOM-killed on this 11 GB host even at `-j1`) |

## Items that are not code defects

- **Job Headroom / required status checks**: the default branch needs an administrator to configure the required status checks. The check already fails on `main` with an explicit message and passes on PRs, as the issue requires.
- **External Benchmarks (schedule)**: humaneval scored 12 against a ledger minimum of 20 ("answer contains no Python code"). This is a model-quality ratchet that works as intended, not a false signal. Recent push runs pass.
- **Issue 1028 ladder / Work-item qualification**: live agent runs that exceeded their 1200 s budget or whose hosted agents failed. These are external and time-bound. The workflows report them correctly.

## Template comparison

None of the 8 `link-foundation/*-ai-driven-development-pipeline-template` repositories uses
`sparse-checkout`, so the main bug class does not exist there and no upstream issue was filed. The rust
template's `scripts/test-scripts.sh` runs `rust-script --test` on every script, which matches the
`--test` invocation already used in `layered-ci.yml`.
