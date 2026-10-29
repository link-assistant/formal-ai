# Issue #1455 and the incomplete PR #1456

Base: `3c9195899c627c0e6f05771e1c059bddac2c9ee9` (released v0.355.17).
Working branch: `fix/issue-1455-docs-and-ci-20261029`.
Pull request: https://github.com/link-assistant/formal-ai/pull/1461.

## Failure and scope

PR #1456 fetched the issue through `gh issue view`, then returned
`planned_not_executed`. It did not inspect the repository or update the guide;
its diff contained session logs and a `.gitkeep` change. The failure is missing
execution synthesis, not a successful implementation or an unbounded retry.
A secondary log-upload permission failure does not explain the missing edit.
The minimal public fixture records the original log SHA-256 and source commit;
the raw session log is deliberately not added to this PR.

## Changes

- The associative stack guide documents all three actual integrations and their
  source/manifest, target/feature and host/browser boundaries.
- A comment-preserving JavaScript generator adds explicit docs filters to heavy
  automatic workflows. Lightweight checks, all retained job bodies, schedules
  and manual controls remain. Bot check dispatch also classifies complete file
  pages and rename preimages; unknown data selects normal checks.
- The native detector prevents docs-only automatic release admission even with
  a pending fragment, and treats other Markdown as possible runtime input.
- The JavaScript Formal AI work-item executor reads explicit repository paths
  before an unsupported terminal gap. Its native twin records the same bounded
  observations and decomposition. Failed/partial/policy-refused reads stop.
  JavaScript/TypeScript files are synchronized through `translate-es.mjs`.

The preflight does **not** autonomously author the guide, bind every requirement
or independently verify an implementation. Those stages remain explicitly
unfinished in its result. General evidence-to-edit synthesis is a development
gap; this PR must not receive autonomous task-completion credit for reading files.

## Verification and continuation

Targeted JavaScript files: `documentation-only-ci-policy`, `work-item-evidence`,
`fetch-work-item-observations`, `release-pending-pipeline`,
`fast-pull-request-production-profile` and `ci-speed` under `rust/tests/web`.
Requirement assembly/status, package-data mirroring, TypeScript translation and
the deployed staged-release projection are checked with JavaScript tools.
Rust test/compilation belongs exclusively to CI; only the existing source
formatter was used locally. Disk remained about 19 GiB free; no local products,
Rust compilation, Docker builds or broad test suites were run.

Open the combined PR, inspect failures in a batch, and qualify the exact head.
This PR changes runtime/CI source, so it is not itself a docs-only change and may
produce a release after merge. Confirm main and its configured publication chain
before claiming production delivery. Future docs-only changes produce no release
by default; GitHub path-filter diff limits may sacrifice CI savings, and the
detector remains the additional automatic-release guard. Before protecting a
filtered workflow, account for GitHub's pending-check behavior.

The first hosted CI batch identified an unlocked parser installation, abbreviated
new names and the desktop workflow crossing its source line limit. The fixes use
a tiny dedicated dependency lockfile, full-word names and a semantically identical
compact path list. No retained job payload or CI acceptance criterion is removed.
Two unrelated native-compiling PR qualifications (provenance performance and the
real Agent CLI failure-report replay) now use the existing full-PR/manual opt-in
profile. Their step payloads are fingerprinted in the retained-jobs manifest;
default PR deferral earns no execution credit. Main's production compilation and
native tests remain selected. These two separate qualifications are available
through manual dispatch, or `FORMAL_AI_FULL_PR_CI=run` on PRs.

## Full-suite compatibility follow-up

PR #1461 merged as `690fb930005600d0a98c4c990e9f76bf5c5094c4` after its
exact-head fast checks passed. The broader main suite then found three gaps:
the publication-path projector rejected the exact new documentation negatives;
the twin scanner required a separately named native path-parser function; and
two original auxiliary-safety tests assumed the old English rendering.

The follow-up retains only the shared, exact documentation exclusions, inserts
new positive declarations before them and still rejects arbitrary negation,
traversal and dynamic source authority. Original bytes/comments and CRLF framing
remain reversible. The native parser is extracted without changing its logic.
The auxiliary tests assert the structured schema, missing-read-tool reason,
no-authorship/no-verification and unfinished implementation, while preserving
all their no-journal/no-unauthorized-write checks. All three regression files are
now part of fast PR coverage. No native tests were run locally.

The first main Rust gate lane also identified a missing backtick in a native
documentation comment and dependency currency for the isolated parser/action
versions. The follow-up corrects the comment, uses the repository's v7 action
convention, locks the parser to the current official npm registry answer and
records that one publisher observation without changing the older snapshot
cohort's refresh time. The single documentation-policy JavaScript regression
passes against the isolated current parser as well as the existing test parser.

The earlier main native unit run subsequently found two admission regressions:
the filename allowlist still omitted unknown runtime Markdown, and the new
read-only documentation workflow lacked concurrency ownership. A further scoped
follow-up includes Markdown after the explicit documentation exclusion, protects
main while cancelling superseded PR checks, and runs the classifier's small
standalone Rust unit suite in CI before tier selection. That suite is CI-only;
local verification remains one targeted JavaScript file at a time.
The same native run also confirmed the documentation checkout must explicitly
drop credentials and an original compound-work-item test must assert the
structured policy-refusal gap. The latter keeps its exact two retrieval steps
and prohibition on plan-journal effects; no acceptance criterion is relaxed.
The deprecated-action check reported there is already fixed by PR #1462.
Final results from that earlier main run also required preserving the push/main
association in the status-trigger test despite intervening documentation filters,
updating one native CLI original-prose assertion to the structured protocol, and
moving the inline native regression out of production `src` into the real unit
integration target. Its replacement exercises the public planner and actual
temporary-workspace reads. Evidence now carries the caller's work-item URL,
without accepting a replacement origin from fetched prose; UTF-8 goal hashes
and original requirement spans remain bound to the captured source.
The intermediate main Clippy run also flagged a filesystem-extension heuristic
on the root Markdown predicate. The predicate now expresses the exact
case-sensitive GitHub suffix rule (including a root `.md` filename), while
unknown runtime Markdown of either case reaches code admission. No lint is
suppressed and no documentation path policy is broadened.

The final repaired main source then exposed one residual test-only Clippy error:
the old expected-plan assertion had been removed, leaving its `task.clone()`
redundant. The follow-up moves the final owned task into its message and keeps
all no-effect/source-identity assertions. It also reuses the native-compatible
Unicode whitespace splitter for source references: NEL is whitespace and BOM is
not, matching Rust without changing original goal text or source spans.

The same follow-up preserves literal Git filenames using NUL-delimited diff and
tree records. Unicode and embedded line breaks in documentation paths must not
be mistaken for code because Git quoted a filename; diagnostics escape these
paths. A focused classifier unit test runs only in CI, with no local Rust build.
