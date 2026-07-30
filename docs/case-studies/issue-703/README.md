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
| Explicit control of another CLI, TUI entrypoint, Bash adapter, or local model frontend | `agent run --command`, multi-command `agent dispatch`, and a separate executable grant that cannot be bypassed by a registered CLI label |
| Correct a disproved result in its original conversation | Native ids and parent digests in canonical sessions, six seed-defined resume contracts, mismatch rejection, and the live same-session chain below |
| Formalize, cross-check, summarize, and answer in the requested language | `orchestration::analysis`, `agent synthesize`, dispatch synthesis, correction requests, and verified translator-session provenance |
| Learn from completed work | Orchestration observations feed the existing evidence-linked, proposal-only client-contract learner; promotion remains human-gated |

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

## Maintainer follow-up: correction, synthesis, and learning

The maintainer's
[follow-up comment](https://github.com/link-assistant/formal-ai/pull/876#issuecomment-5127332907)
asked the controller to reach beyond the initial six adapters: explicitly
control any requested CLI/TUI or Bash-backed model, use several agents,
formalize and summarize their answers, cross-check facts, answer in a requested
language, learn from the work, and resume a model's exact session when evidence
disproves it.

The public surface now separates those responsibilities:

1. `--command` supplies JSON argv for a requested custom adapter, while
   `--allow-agent-command` independently grants its exact executable. Dispatch
   accepts several `CLI=JSON_ARGV` mappings and applies the same isolation and
   evidence protocol to every candidate.
2. Native session ids and resume argv are discovered from real client output
   and seed data. `agent resume` binds a correction to the parent digest,
   carries the disproved claim and evidence, and fails if a client switches
   session ids.
3. `agent synthesize` extracts only answer events, formalizes them in the
   meta-language, performs statement-level deduplication, evidence ranking,
   contradiction reporting and recheck, summarizes them, and emits
   evidence-linked correction requests.
4. Requested output language is verified rather than assumed. A summary that
   is not in `en`, `ru`, `hi`, or `zh` as requested remains marked
   `translation_required`; an independently recorded translator session must
   detect as the target language before its bytes are accepted.
5. `agent learn` converts recorded runs into the existing client-contract
   learner. It proposes evidence-linked contract facts but cannot promote
   itself.

The fact-checking claim is deliberately bounded. The synthesis report calls
its scope `cross_agent_evidence_preflight`: independent model outputs can expose
agreement and contradictions, but are not primary evidence about the outside
world. External factual guarantees still require captured sources and the
production fact-checking path.

### Live mistake-to-correction chain

A real run used Formal AI as controller, the real Agent CLI 0.25.3 as client,
and Formal AI as that client's model. The initial model turn exited
successfully but did not create the requested invariant. Formal AI then resumed
native session `ses_04e25ba4cffeibfMekv188DNLX` with that proof. The first
correction created a contaminated artifact, exposing a correction-template
ordering defect. After moving the task to the final prompt position, the
controller supplied the new evidence and resumed the same native session once
more. The final artifact contains exactly:

```text
orchestration_continuation resume_exact_native_id.
```

The three canonical sessions, their SHA-256 parent chain, actual Agent CLI
`--resume ID --no-fork` argv, failure observations, and final artifact digest
are preserved in the
[exact-session correction evidence](followup-authorship/README.md). The focused
suite byte-replays the evidence and rejects any changed native id, parent hash,
argv contract, canonical bytes, or artifact.

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

For the maintainer follow-up, Formal AI through the real Agent CLI also authored
the exact-session invariant in
[`data/meta/orchestration-continuation-invariant.lino`](../../../data/meta/orchestration-continuation-invariant.lino).
The commit carries the required `Formal-AI-Session` and `Formal-AI-Evidence`
trailers, and the evidence is the final canonical session from the live
correction chain. Failed earlier turns are retained rather than presented as
successful authorship.

The controller implementation, tests, documentation, and harness were written
manually with Codex assistance and are not claimed as Formal AI-authored. Two
earlier boundary attempts failed before producing a reviewed artifact: one
used the unsupported model selector `formalai/formal-ai`, and one exposed a
task-routing limitation. They were discarded rather than counted.
