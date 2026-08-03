# Issue 848: executable coding tasks through the real Agent CLI

Issue [#848](https://github.com/link-assistant/formal-ai/issues/848) asks where
Formal AI can perform coding work rather than describe it. The expanded ladder
contains 130 real repository tasks in 16 families, from reading and search to
test authoring, targeted edits, multi-file deliverables, and whole issues. Each
task runs through `formal-ai with agent`; a write passes only when the requested
workspace effect is observed.

The issue baseline on v0.303.0 was 38/130. The prepared branch before this fix
measured 45/130, with `test_authoring` and `targeted_edit` both at zero. The
first complete post-fix run measured every task and passed 61/130:

```text
L1  0/16
L2  4/12
L3  8/28
L4 49/74
```

The required boundaries are now nonzero: compiler-valid source creation,
`test_authoring`, `targeted_edit`, L2, L3, focused repository search, grounded
collection edits, and multilingual execution. Two correct version answers in
that first run were scored against the stale literal `0.30`; the benchmark now
derives their oracle from `Cargo.toml`, and both English and Chinese filtered
replays pass against `0.317.0`.

The second complete pre-learning run measured 130/130 tasks with no unavailable
servers and passed 62/130:

```text
L1  0/16
L2  4/12
L3  8/28
L4 50/74
```

Its acceptance-boundary families passed `create` 6/6, `search` 6/8,
`test_authoring` 1/8, `targeted_edit` 2/7, and `multilingual` 7/11. The two
corrected version reads added two passes, while a variable Hindi decomposition
answer lost one pass relative to the first run. The canonical result therefore
records the observed net increase of one rather than extrapolating from the
filtered replays.

The final v0.320.0 issue run measured all 130 tasks and passed 65/130:

```text
L1  0/16
L2  5/12
L3 10/28
L4 50/74
```

Its family boundaries include `create` 6/6, `test_authoring` 1/8,
`targeted_edit` 3/7, `multilingual` 8/11, `refactor` 1/3, and `multifile` 2/4.
The new grounded rename and module-registration cases both pass in the complete
run and in filtered real-Agent replays, with observed effects and no refusal or
timeout. One full-run result was initially labelled `NOT MEASURED` even though
its server started and its edit verified: the task read seed data containing the
literal startup-error message. The harness now also requires absence of the
launcher success marker, and the corrected replay passes, leaving a complete,
internally consistent canonical result.

## Evidence inventory

Authenticated GitHub snapshots under [`raw-data/`](raw-data/) retain the issue,
all issue comments, PR metadata, conversation comments, inline review comments,
and reviews. None contains an image attachment, so there is no screenshot to
preserve; this is an execution and measurement defect rather than a visual one.

Reproducible runtime evidence consists of:

- the complete result in
  [`results.json`](../../../experiments/issue_847_coding_ladder/results.json),
  with a 130/130 completeness marker and per-task output tails;
- the generator and real-Agent runner beside that result;
- [`self-hosting-authorship/`](self-hosting-authorship/), containing the raw
  Agent CLI transcript, Formal AI trace, exact authored invariant, and reviewed
  authorship accounting;
- [`self-hosting-workspace-learning/`](self-hosting-workspace-learning/),
  containing three additional real Agent CLI transcripts and traces for the
  canonical learning contract, execution policy, and held-out fixture;
- [`requirements.md`](requirements.md), mapping each acceptance boundary to
  implementation and executable regression evidence.

## Root cause

The original planner lowered a broad literal-file pattern before it understood
the requested artifact. A prompt such as “create a Rust test …” could therefore
become the exact bytes of the new `.rs` file. A verifier that grepped for a
function name accepted that prose as code. Search had a related grounding
problem: it passed the whole conversational sentence to the repository search
tool instead of the named symbol or string. Collection edits had no transition
from reading existing bytes to transforming the named array.

Two deeper routing failures remained visible in the complete result. The broad
shell meaning of “rename” captured identifier refactors and attempted to move a
file, even when the request named a constant inside that file. A composite
“create a module and export it” request was treated as one source artifact: it
either refused to synthesize or repeatedly read the not-yet-created target,
with no representation of the second workspace effect.

The measurement harness amplified those defects. It originally accepted
narration, existing branches, mixed log text, and a substring resembling Rust.
It could also overwrite the canonical score with a filtered replay or compile
an unrelated pre-existing rust-script file. Each discovered false green now
has a structural guard.

## Implementation

`agentic_coding::code_task` runs before literal writes. It recognizes source
artifact concepts from seed data, extracts the requested path, identifier,
visibility, operation, and values, renders executable Rust bytes, writes them,
then reads the exact target back through the client-owned shell tool. It covers
bounded functions, constants, and numeric tests in English, Russian, Hindi,
and Chinese.

Repository search extracts the named code subject and issues one focused query.
`agentic_coding::structured_edit` reads the requested file, transforms the
existing collection bytes, writes the whole result, and observes the exact file
through `cat`. No task is credited from narration alone.

`agentic_coding::workspace_change` now routes grounded identifier rewrites
before broad shell actions. It reads the named target, applies a bounded Normal
Markov substitution through the shared workspace-rewrite executor, then uses a
compact native edit for a unique occurrence or one validated replace-all command
for a repeated identifier. It finishes only after observing the exact expected
SHA-256 digest, avoiding a second full-file payload through Agent. Composite Rust
module requests use an ordered transaction: render and observe the module, then
read the registration target, apply a compact unique suffix edit, and verify its
digest. Write-only clients retain the full-write/exact-read fallback. The
meanings, roles, and module-registration template are seed data, including
English, Russian, Hindi, and Chinese surfaces.

The reusable procedure path is proposal-only. `workspace_change_learning`
separates task identity from execution identity, accepts only exact successful
observations, and forms a candidate only after two distinct tasks. Candidates
have no execution surface. A zero-failure gate plus named human approval is
required before a content-addressed Links Notation recipe enters the approved
ledger; only that ledger can execute the same bounded transformation on an
unseen equivalent task. The active identifier-refactor route and the learning
tests call the same executor rather than maintaining a benchmark-only copy.

The ladder snapshots source-file existence before each task and invokes
`rustc` on every newly generated Rust target. Full and filtered measurements
have separate output paths, and the canonical JSON records dataset size,
measured size, completeness, and filter. Release-dependent answer expectations
are resolved from a named file and capture regex; a malformed oracle fails
closed.

## Reproduction

Build the release binary and run the focused deterministic tests:

```sh
cargo test --test unit issue_848_coding_ladder -- --nocapture
cargo test --test unit issue_848_workspace_change_learning -- --nocapture
cargo build --release
ONLY=create.rust experiments/issue_847_coding_ladder/run_coding_ladder.sh
ONLY=search.find_stable_id experiments/issue_847_coding_ladder/run_coding_ladder.sh
ONLY=edit.add_devlog experiments/issue_847_coding_ladder/run_coding_ladder.sh
ONLY=refactor.rename_const experiments/issue_847_coding_ladder/run_coding_ladder.sh
ONLY=multi.module_and_export experiments/issue_847_coding_ladder/run_coding_ladder.sh
```

Run the complete real-client measurement with:

```sh
experiments/issue_847_coding_ladder/run_coding_ladder.sh
```

The runner starts every task from a clean repository, drives the installed
Agent CLI against Formal AI's temporary agent-mode server, verifies the
observable effect, reverts the task, reclaims client scratch space, and records
the result after every task. A server startup failure is `NOT MEASURED`, never
an ordinary failure; diagnostic text read from a task file cannot impersonate a
startup failure once the launcher has emitted its success marker.

## Same-task self-application

Formal AI served its model to the real external Agent CLI for four of seven
reviewed smallest leaves. Session `ses_04160c59fffe3FDUKteR56kfQp` authored
the generalized coding-task execution invariant. Sessions
`ses_03d2e0597ffeAUZhq3qAtj2I4U`, `ses_03d2df24effeijLfzPXiUeV4pG`, and
`ses_03d2ddb1cffeQkS5gxWjpMojc6` authored the learning contract, execution
policy, and held-out generalization fixture. Each session wrote through the
client tool, read the exact bytes back through the shell tool, and completed
normally; the canonical runtime inputs match the Agent artifacts.

Four of seven smallest leaves are therefore Formal-AI-authored: 57%. The
replay harnesses are
[`experiments/issue_848_self_authoring/run.sh`](../../../experiments/issue_848_self_authoring/run.sh)
and
[`run_workspace_learning.sh`](../../../experiments/issue_848_self_authoring/run_workspace_learning.sh).

## Residual boundary

This change establishes a trustworthy floor, not general autonomous software
engineering. Whole-issue L1 tasks remain outside the bounded executor. Source
generation, identifier substitution, and module registration intentionally
cover only shapes whose semantics can be rendered from data and verified
without guessing. Unsupported refactors, test bodies, and multi-file plans must
continue to fail honestly until they gain data-backed semantics and observed
completion contracts; an observed procedure is not automatically approved.
