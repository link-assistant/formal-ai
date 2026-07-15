# Issue 715: contextual code changes must mutate the workspace

## Result

Formal AI now treats generated source as a mutable workspace artifact whenever
an agentic client advertises file tools. A request such as “Give me a hello
world program in Rust” emits a real `write` call for `main.rs`; a later “Change
the output message to `Hello 2`” discovers that artifact from tool-call history,
reads the current file, applies a bounded mutable rewrite program, and writes the
updated source. The response no longer repeats stale prose.

The implementation is driven by the existing coding catalog, not a Rust-only
branch. Automated coverage exercises generation and follow-up modification for
all ten catalog languages and forty follow-up phrasings across English, Russian,
Hindi, and Chinese.

## Reproduction

The original screenshot shows OpenCode returning a Rust snippet inline and then
ignoring the requested output change. The preserved issue payload and validated
PNG are in [`raw-data`](raw-data/); the pre-fix test trace is
[`test-logs/reproducer-red.log`](test-logs/reproducer-red.log). All four original
regression axes failed with `expected one tool call, got None`.

Run the focused regression suite:

```sh
cargo test --test issue_715 -- --nocapture
```

Run the real two-turn OpenCode replay:

```sh
cargo build --bin formal-ai
experiments/agent_cli_e2e/run_issue_715_opencode.sh
```

The replay boots the OpenAI-compatible server with request tracing, invokes the
installed OpenCode CLI twice in one session, verifies the CLI created `main.rs`,
then verifies that the second turn replaced the old output in that actual file.
Its durable transcript is written to [`opencode-run`](opencode-run/).

## Timeline

| Time (UTC) | Event |
| --- | --- |
| 2026-07-13 | Issues 680 and 681 introduced general explicit file intent routing and write-before-read ordering. |
| 2026-07-14 | Issue 712 reported remaining regressions in those explicit routers. |
| 2026-07-15 | Issue 715 reported the multi-turn code-artifact failure with an OpenCode screenshot. |
| 2026-07-15 | The mandatory `solve --tool agent` self-coding entry point was attempted; it stopped before execution because that wrapper rejected the configured model name. The failure is preserved and reported on the issue. |
| 2026-07-15 | A red integration test reproduced missing generation tools, missing contextual reads, stale edits, and arbitrary-fragment changes. |
| 2026-07-15 | The artifact planner, current-turn progress scoping, mutable rewrite graph, catalog-wide tests, and real OpenCode replay were added. |

## Requirements and evidence

| Requirement | Implementation | Verification |
| --- | --- | --- |
| Use real file tools in agentic CLIs | Generation emits write; contextual mutation emits read then write. Tool names are classified by capability, so `read`, `read_file`, `write`, and `write_file` share the path. | `issue_715` integration tests and OpenCode replay |
| Work for all programming languages | Filename and source are selected from `PROGRAM_LANGUAGES` and its task templates. | Ten-language generation and modification loops |
| Work across natural-language variation | Follow-ups derive the requested value from quoted structure and prior artifact state instead of English keyword lists. | Ten phrasings each in English, Russian, Hindi, and Chinese |
| Support general changes | Two quoted fragments form an arbitrary old/new rewrite; one quoted fragment targets the artifact's last output literal. | Arbitrary Rust statement replacement test |
| Use a mutable meta-language representation | Changes compile to conditional, replacement, jump, and halt instructions; backward jumps are supported and execution has a safety bound. Each applied plan renders as Links Notation in the final trace. | Planner test asserts the emitted graph and the OpenCode server trace preserves it |
| Preserve current workspace state | The planner reads before modification, decodes known CLI read envelopes, and uses only the current read result as rewrite input. | Read-before-write assertions and actual OpenCode file result |
| Apply consistently to APIs/CLIs | The behavior sits in `plan_chat_step`, shared by the OpenAI-compatible API and every CLI using that endpoint. | Planner-level protocol tests plus OpenCode HTTP round trip |
| Deep case study and raw evidence | Issue/PR payloads, related reports, screenshot, red/green logs, research, server trace, CLI JSONL, and final file are retained here. | This directory |

## Root cause

There were three interacting defects:

1. The planner routed only explicit file instructions from the latest user
   message. A code-generation answer could be produced by the symbolic solver,
   but the planner never converted the catalog result into a workspace write.
2. A follow-up naturally omitted the filename and old text. The explicit edit
   router therefore had nothing to match even though the prior assistant tool
   history could identify both.
3. Generic recipe progress scanned the entire conversation. A write result from
   an earlier user turn could incorrectly make a later operation look complete.

The visible stale response was therefore not a code-generation problem. It was
an artifact identity and conversation-progress problem at the agentic planning
boundary.

## Solution design

The planner first looks for the most recent code-bearing write call before the
latest user message. That call is the artifact record: its normalized path and
full content are independent of any client's display prose. If none exists, a
recognized coding request resolves through the established language/task/template
catalog and emits a write.

For an existing artifact, quoted request data compiles to a mutable instruction
graph. The graph can branch on content, replace source, and jump backward; the
interpreter caps execution steps. The first turn requests the current file. The
next turn runs the graph over the read result and writes the complete updated
source. Tool results are considered only after the latest user turn.

This design deliberately keeps filesystem authority in the agentic client. The
server plans calls and consumes results; it does not bypass the client's tool
permissions or directly touch the client's workspace.

## Limitations and follow-up boundary

This change handles deterministic literal/source-fragment rewrites grounded in
conversation and file state. It does not claim that arbitrary semantic refactors
can be inferred without a concrete transformation. The mutable graph is capable
of iterative conditional rewriting, but every execution is intentionally bounded.
Requests without file tools continue through the existing prose solver, which
preserves web, desktop, and chat clients that do not expose a workspace.

## Data inventory

- `raw-data/issue-715.json` and `issue-715-comments.json`: authoritative issue
  description and discussion snapshot.
- `raw-data/issue-715.png`: downloaded attachment; PNG signature
  `89 50 4e 47 0d 0a 1a 0a` was verified before inspection.
- `raw-data/issue-{680,681,712,714,716}.json`: adjacent routing and agentic-mode
  reports used for regression analysis.
- `raw-data/pr-727.json`: initial prepared-PR metadata snapshot.
- `test-logs/`: red reproduction, green planner/meta-language checks, and the
  mandatory self-coding wrapper failure.
- `research.md`: external behavior contracts consulted during design.
