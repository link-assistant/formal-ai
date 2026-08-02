# Issue 847: task decomposition as a working meta-method

Issue [#847](https://github.com/link-assistant/formal-ai/issues/847) asks
Formal AI to treat decomposition as a task it can perform, inspect, execute,
and improve. The first PR draft recognized direct requests such as “split this
task”, but it still certified an issue-sized single clause as atomic. It also
rebuilt no executable tree from the inspected result and had no evidence-gated
path from a failed run to a reusable strategy.

The replacement defines atomicity operationally: a leaf is independently
checkable only when it has an observable completion contract, has no pending
children, and was not stopped by the depth bound. Direct edits derive that
contract from semantic roles and a concrete target. Requests that name a work
item but omit their own operation contract descend through reviewed,
data-backed lifecycle strategies.

## Evidence inventory

Authenticated GitHub API snapshots are retained under [`raw-data/`](raw-data/):

- issue 847 and every issue comment;
- PR 857 metadata, all conversation comments, all inline review comments, and
  every review.

Neither the issue nor its comments contain an image attachment, so there is no
screenshot to preserve. This is an execution and reasoning defect, not a
visual defect.

Reproducible runtime evidence is kept beside this README:

- [`self-hosting-authorship/`](self-hosting-authorship/) contains the raw
  external Agent CLI transcript, Formal AI server trace, exact authored
  invariant, and four-leaf authorship accounting for this same task;
- [`self-hosting-evidence/`](self-hosting-evidence/) retains the earlier
  whole-repository source projection as complementary self-model evidence;
- [`requirements.md`](requirements.md) maps the issue and maintainer feedback
  to implementation and executable evidence.

## Root cause and prior architecture

The original splitter equated grammatical splittability with task atomicity.
That works for “edit A and edit B”, but an issue URL or a broad one-clause
request has no coordinating conjunction to split. It therefore reached the
single-need base case even though no observable operation contract existed.
At a depth bound the root could also be a leaf while still being marked
atomic, making an incomplete plan look complete.

Formal AI already had three useful general mechanisms:

- `recursive_execution` executes a caller-supplied `RecursiveTask` tree and
  records blocked attempts;
- skill procedures use explicit completion contracts and bounded recursion;
- learning ledgers use deterministic records and human review to control
  durable activation.

The fix composes those mechanisms instead of adding issue-specific parser
branches. The decomposition tree is content-addressed and round-trippable, its
exact nodes adapt directly to `RecursiveTask`, and failed execution can create
a typed proposal. Promotion requires both a green regression gate and an
explicit reviewer. The lifecycle strategy and its shipped approval live in
[`task-decomposition-strategies.lino`](../../../data/meta/task-decomposition-strategies.lino),
not in a Rust phrase table.

## State transitions

```text
task
  -> direct semantic split with observable targets
  -> or missing-operation-contract strategy from reviewed data
  -> recursively inspect children until atomic or visibly depth-bounded
  -> content-addressed artifact with exact child identifiers
  -> exact tree adapter -> recursive execution
  -> blocked leaf -> deterministic strategy proposal
  -> green regression evidence + human review -> durable activation
```

A proposal alone never changes planning behavior. An empty ledger demonstrates
the failure and emits a proposal; only the reviewed shipped ledger enables the
general lifecycle strategy. Restored ledgers reject changed proposal IDs,
review IDs, content IDs, failed gates, absent reviewers, duplicate reviews,
or unknown strategies.

## Implementation

`task_decomposition::strategy` detects a missing operation contract from
semantic roles, concrete code targets, and repository work-item references.
It loads localized requirements, regression, implementation, and verification
stage templates for English, Russian, Hindi, and Chinese. These stages are a
general verified-change method: they contain no issue-847 filenames or desired
patch.

`SubTask::is_independently_checkable` requires an atomic leaf, no children, and
a resolved completion criterion. A depth-bounded unresolved root is therefore
never reported as atomic. The bounded result remains visible in the answer and
artifact so termination cannot masquerade as completion.

`Decomposition::to_recursive_task` traverses the already-inspected nodes. It
does not invoke the splitter a second time. The artifact records content
identifiers for the tree and every child edge; parsing rejects changed content,
orphans, cycles, broken edges, invalid paths, and inconsistent depths.

`TaskStrategyProposal::from_failed_run` derives the first candidate from an
actual blocked leaf and its last execution evidence. `TaskStrategyLedger`
accepts it only when the registered strategy is known, the regression suite
has at least one pass and zero failures, and a named reviewer grants approval.
The durable ledger preserves the failed task, failure evidence, suite,
result, reviewer, and content-addressed identities.

The Agent-authored
[`task-decomposition-invariant.lino`](../../../data/meta/task-decomposition-invariant.lino)
is embedded as the public decomposition contract. The shipped strategy ledger
loads only when all three atomicity, execution, and learning invariants parse.
An executable test pins the embedded bytes to the raw Agent output.

## Reproduction

The minimum pre-fix regressions are:

1. decompose the one-clause request “Implement issue #847 …” with an empty
   ledger and observe that it has no independently checkable completion
   contract;
2. decompose the same request at depth zero and verify that the unresolved
   root is not atomic;
3. inspect a decomposition artifact, adapt it to recursive execution, and
   verify every ID, goal, and child edge is unchanged.

Run the focused suite with:

```sh
cargo test --test unit specification::task_decomposition -- --nocapture
cargo test --test unit issue_847_task_decomposition -- --nocapture
experiments/issue_847_self_authoring/run.sh
```

Run the repository gates with:

```sh
cargo test --test unit -- --nocapture
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
sh src/web/wasm-worker/build.sh
GITHUB_BASE_REF=main npm run --prefix tests/e2e check:language-test-coverage
rust-script scripts/check-file-size.rs
rust-script scripts/check-hardcoded-language.rs
scripts/sync-seed.sh --check
```

All unit and integration fixtures are deterministic and offline. The external
Agent replay is a separate explicit proof and requires the configured Agent
CLI.

## Same-task self-application

Formal AI served the `formal-ai` model to the external Agent CLI for the issue
847 objective. Session `ses_05171239affeLG6WcrsR27Rb8U` planned the general
change, used the client-owned write tool to create the task-decomposition
invariant, read it back through the shell tool, and completed without error in
four chat rounds. The canonical data file is byte-for-byte identical to the
generated artifact.

The reviewed decomposition has four smallest leaves. Three implementation
leaves are human-authored; the generalized contract leaf is Agent-CLI-authored.
That is one of four leaves, or 25%. The harness is
[`experiments/issue_847_self_authoring/run.sh`](../../../experiments/issue_847_self_authoring/run.sh).

The earlier whole-repository projection remains useful evidence that Formal AI
can inspect this branch, but it is not counted as same-task authorship. The
session above is the narrower, replayable authorship claim.

## Residual limits

The shipped strategy is intentionally review-gated and currently supplies one
general verified-change lifecycle. A failed run can propose a registered
strategy, but it does not autonomously invent and activate arbitrary new
stages. New strategies still require regression evidence and repository
review. Semantic-role coverage can also be expanded as new languages and task
domains are reviewed. Finally, a depth bound guarantees termination, not
completion; unresolved bounded leaves remain explicit and non-atomic.
