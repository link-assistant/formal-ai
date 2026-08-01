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
replays pass against `0.317.0`. The final complete score is recorded in the
canonical ladder result and this case study after the final run.

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
  four-leaf authorship accounting;
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
cargo build --release
ONLY=create.rust experiments/issue_847_coding_ladder/run_coding_ladder.sh
ONLY=search.find_stable_id experiments/issue_847_coding_ladder/run_coding_ladder.sh
ONLY=edit.add_devlog experiments/issue_847_coding_ladder/run_coding_ladder.sh
```

Run the complete real-client measurement with:

```sh
experiments/issue_847_coding_ladder/run_coding_ladder.sh
```

The runner starts every task from a clean repository, drives the installed
Agent CLI against Formal AI's temporary agent-mode server, verifies the
observable effect, reverts the task, reclaims client scratch space, and records
the result after every task. A server startup failure is `NOT MEASURED`, never
an ordinary failure.

## Same-task self-application

Formal AI served its model to the real external Agent CLI for one of four
reviewed smallest leaves. Session `ses_04160c59fffe3FDUKteR56kfQp` authored
the generalized coding-task execution invariant, wrote it through the client
tool, read the exact bytes back through the shell tool, and completed normally.
The canonical invariant is byte-for-byte identical to the Agent artifact.

One of four smallest leaves is therefore Formal-AI-authored: 25%. The replay
harness is
[`experiments/issue_848_self_authoring/run.sh`](../../../experiments/issue_848_self_authoring/run.sh).

## Residual boundary

This change establishes a trustworthy floor, not general autonomous software
engineering. Whole-issue L1 tasks remain 0/16, and refactoring remains 0/3.
Source generation intentionally covers only bounded shapes whose semantics can
be rendered and verified without guessing. Unsupported test bodies and
multi-file plans must continue to fail honestly until they gain data-backed
semantics and observed completion contracts.
