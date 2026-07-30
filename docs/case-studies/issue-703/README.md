# Issue 703: Formal AI as a multi-agent coding controller

Issue [#703](https://github.com/link-assistant/formal-ai/issues/703) asked
Formal AI to orchestrate external coding agents safely, compare several
solutions, compose verified changes, and retain replayable provenance. Pull
request [#876](https://github.com/link-assistant/formal-ai/pull/876) implements
that controller without changing the conservative behavior of `formal-ai
with`.

## Root cause and design

Formal AI already had a data-driven client registry and wrappers, but those
wrappers were designed for chat and defaulted to read-only behavior. There was
no capability grant for an editing agent, no common run record, and no
isolation or composition layer above the clients.

The new path has four boundaries:

1. `AgentRunPermission` is denied by default and grants one canonical
   workspace.
2. The seed registry supplies each CLI's non-interactive editing and structured
   output arguments. The existing wrapper receives these only from the hidden
   orchestration overlay.
3. Every candidate runs in its own workspace copy. Verification commands use
   JSON argv and must be executable-allowlisted.
4. Canonical session JSON records output, status, elapsed time, hashed effects,
   verification, and an append-only hash chain of events. Replay rejects schema,
   chain, digest, or byte-canonicalization changes.

The comparison rule is deterministic for a recorded ledger: passing status,
smaller diff, shorter wall time, CLI id, then session path. Decomposed runs
preflight conflicts and never compose changes from a failed candidate.

## Requirements trace

| Issue requirement | Delivered evidence |
| --- | --- |
| Permission-gated `run_agent`, isolated workspace, timeout, output and provenance | `src/orchestration/permission.rs`, `runner.rs`, `workspace.rs`, and focused integration tests |
| Six data-driven CLI adapters and Formal AI/vendor targets | `data/seed/client-integrations.lino`, registry rendering, and six-client CLI E2E |
| Universal bounded decomposition and composition | `src/orchestration/dispatch.rs` using the existing universal task decomposer |
| Parallel same-task comparison and winner ledger | `agent dispatch --compare`, canonical sessions, and `comparison-ledger.json` |
| Hive Mind dependency | Upstream [Hive Mind #2059](https://github.com/link-assistant/hive-mind/issues/2059) is closed by [#2108](https://github.com/link-assistant/hive-mind/pull/2108); the installed solver reaches command preparation |
| Safe errors and no silent retries | Default-denied, allowlisted verification, one process-start event, timeout/failure regression tests |
| Byte-for-byte replay | Canonical serializer plus replay tests that reject altered bytes or event chains |
| Self-hosting authorship evidence | Formal AI + Agent CLI session `ses_050646852ffetdnQ73vR1yZ8la`, captured below |

## Reproduction and verification

The completed controller path was exercised against a real local Formal AI
server and the real Agent CLI:

```bash
formal-ai agent run \
  --cli agent \
  --task "Create file controller-proof.lino containing controller_proof and the phrase replayable workspace edit." \
  --workspace /tmp/formal-ai-issue703-controller-agent \
  --base-url http://127.0.0.1:8704 \
  --session /tmp/formal-ai-issue703-controller-agent/controller-session.json
```

Agent CLI session `ses_05046b1c9ffe59CvG0N3QrrsV4` used model
`formalai/formal-ai`, invoked the write and shell tools, created
`controller-proof.lino`, and verified it with `cat`. The controller captured
both file effects and the complete client stream in the canonical
[session](controller-agent-run/controller-session.json). CI replays those
exact committed bytes and pins the [generated
artifact](controller-agent-run/controller-proof.lino) to its recorded hash.
The invocation also disables Agent CLI's rate-limit retries, so the controller
and client path have no hidden retry layer.

The exact acceptance command completed through every real client:

```bash
FORMAL_AI_ISSUE_703_LIVE=1 \
FORMAL_AI_ISSUE_703_CODEX_UNSANDBOXED=1 \
KEEP_MATRIX_ROOT=1 \
experiments/issue_703_orchestration/run_live_cli_matrix.sh
```

The 2026-07-29 run used Agent CLI 0.25.3, Claude Code 2.1.220, Codex
0.145.0, Gemini CLI 0.52.0, Qwen Code 0.21.0, and OpenCode 1.18.7. For each
client the script created a fresh workspace, invoked `formal-ai agent run
--task "add a README badge"`, and asserted all three observable results:

1. the real client exited successfully;
2. its canonical session contains a `README.md` workspace-effect event; and
3. the isolated README contains the generated `img.shields.io` badge.

The concise [live matrix record](live-cli-matrix.log) contains the versions,
durations, command, and assertions. The real client streams remain in each
generated `session.json`; they are intentionally opt-in because running them
consumes client quota. The deterministic CI gate runs the same public command
against recorded client entrypoints, stores each session, and replays its exact
canonical bytes.

Codex selected the product's default `workspace-write` sandbox. This container
blocks the user namespaces that sandbox needs, so the live harness used its
explicit `FORMAL_AI_ISSUE_703_CODEX_UNSANDBOXED=1` escape hatch only inside the
already externally sandboxed job. Normal orchestration continues to pass
`--sandbox workspace-write`, which the deterministic wrapper test asserts.

The real matrix also found three client-boundary defects that API-only tests
would miss:

- Gemini advertises file read/write tools but no shell tool, so the planner
  safely projects the bounded seeded append command into `read_file` followed
  by `write_file`.
- Qwen's default `auto` permission mode hides mutation tools, so its temporary
  seed configuration selects noninteractive `auto-edit`.
- OpenCode reads `PWD` when choosing its project. The process runner now
  exports the canonical granted workspace as `PWD` as well as setting the
  actual current directory.

The deterministic integration test
`all_six_cli_entrypoints_run_a_scripted_repo_task_through_the_real_wrapper`
uses the same public command for all six clients. Each scripted client edits
`README.md`; the test then verifies success, the recorded effect, and exact
session replay. Additional tests cover permission denial, process and
verification timeout, no retry, failed-candidate isolation, decomposition,
parallel comparison, output containment, worker shutdown, deterministic
selection, and canonical replay.

A reproducible two-client run made Codex and Claude edit isolated copies of the
same scripted repository and ran the same allowlisted `test -s README.md`
check in each copy. Both passed; Codex's 21-byte effect beat Claude's 53-byte
effect under the recorded rule. The committed
[comparison ledger](comparison/comparison-ledger.json) and its referenced
canonical sessions are generated by
[`run_comparison.sh`](../../../experiments/issue_703_orchestration/run_comparison.sh)
and asserted in CI.

Run the focused suite with:

```bash
cargo test --all-features --test integration issue_703_orchestration -- --nocapture
```

The reusable library walkthrough is
[`examples/issue_703_orchestration.rs`](../../../examples/issue_703_orchestration.rs).

## Formal AI authorship boundary

The change was reviewed as five leaves:

1. permission and adapter registry;
2. session/event capture and replay;
3. parallel dispatch, comparison, and composition;
4. CLI, tests, and documentation;
5. the declarative orchestration safety invariant.

Formal AI through Agent CLI genuinely authored leaf 5, meeting the required
one-in-five self-hosting threshold. The exact output is promoted to
[`data/meta/orchestration-safety-invariant.lino`](../../../data/meta/orchestration-safety-invariant.lino)
and byte-pinned against the captured artifact. The complete
[Agent CLI log](self-hosting-authorship/agent-cli.log), [Formal AI server
log](self-hosting-authorship/formal-ai.log), and [generated
artifact](self-hosting-authorship/orchestration-safety-invariant.lino) are
committed as provenance.

The controller implementation, tests, documentation, and harness were written
manually with Codex assistance and are not claimed as Formal AI-authored. Two
earlier boundary attempts failed before producing a reviewed artifact: one
used the unsupported model selector `formalai/formal-ai`, and one exposed a
task-routing limitation. They were discarded rather than counted.
