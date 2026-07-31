# Issue 879: observable completion for coding clients

Issue [#879](https://github.com/link-assistant/formal-ai/issues/879) reported
three superficially different failures of `formal-ai with`: Agent reinterpreted
a Scala task into scratch planning, Claude stopped after `pwd`, and Codex
reached OpenAI's public Responses endpoint. All three runs produced no source
artifact, yet two looked like normal completion to the caller.

## Result

One seeded completion contract now applies to the six supported coding clients:
Agent, Claude, OpenCode, Codex, Qwen, and Gemini. A detected one-shot
software-authoring request:

1. selects that client's registered editing and machine-output profile;
2. snapshots the workspace while excluding wrapper scratch state;
3. accepts success only when a non-scratch file effect is observable;
4. walks a seeded ladder of recovery strategies, one strategy per retry, each
   with its own multilingual correction prompt, resuming the native session
   when known;
5. exits nonzero if the whole ladder still leaves no effect;
6. emits strict NDJSON and a final structured completion record with the model,
   configured endpoint, discovered actual endpoint, token usage, attempt count,
   reason, changed paths, and the recovery strategies it spent.

The contract is data-defined in
[`client-completion-contracts.lino`](../../../data/seed/client-completion-contracts.lino).
It declares the ordered recovery ladder, caps the run at one attempt per
strategy plus the original request, names the exact incomplete reason, and
lists the public vendor endpoints a local-server run must never reach.
Ordinary chat and explicitly interactive use remain unchanged.

## The self-learning loop

Retrying with the same prompt is not a different decomposition, so the issue
asked for the `(request, plan, outcome)` triple to be recorded and for the next
attempt to try a different one. Three seeded strategies escalate in order:
`restate_postcondition` asks for the observable postcondition in one line,
`name_target_artifact` forbids further planning and demands one concrete
relative path, and `decompose_into_leaf` asks for the smallest leaf of the task
that leaves a file behind.

Every attempt appends one `completion_recovery` Links Notation record — client,
postcondition, strategy, and whether an effect followed — to a durable ledger.
The next run reads that history back and ranks the ladder so a strategy that
has produced artifacts for this client is tried first and one that never has is
tried last. Learning can only *reorder* the seeded list: it cannot invent a
strategy, drop one, or widen the attempt budget, so a missing or corrupted
ledger degrades exactly to the seeded order.

The ledger is written under `$FORMAL_AI_STATE_DIR`, else `$XDG_STATE_HOME`,
else `~/.local/state`, never under the workspace the client was invoked in.
That is the same defect that made `.formal-ai/` scratch state pollute the
caller's tree, so the test asserts both the cross-run reordering and the
absence of any learning artifact from `git status`.

## Root cause

The wrapper previously delegated process completion to the client's exit
status. A zero exit could therefore mean "artifact produced", "model ended
after one orientation command", or "the request became an unrelated plan".
The wrapper also streamed the client's presentation format directly, so pretty
JSON broke line-oriented consumers, and it had no run-level place to publish
usage or endpoint evidence.

The fix separates process status from task completion. The original request is
classified with the existing seeded software-authoring role, and workspace
effects become the shared observable postcondition. This is intentionally a
run-level contract rather than six client-specific prompt patches.

## Requirements trace

| Issue requirement | Evidence |
| --- | --- |
| No false success without an artifact | `software_authoring_cannot_succeed_without_an_artifact` first failed against the old wrapper in commit `21a45ac6`, then passes with a two-attempt incomplete result |
| Diagnose and recover | `corrective_retry_reuses_the_native_session_and_can_complete` proves the correction carries the disproved claim and evidence and resumes `ses_issue879` |
| Retry differently | The reproducer asserts the three corrections escalate through distinct instructions, and `recovery.strategies_spent` names each one in the completion record |
| Self-learning across runs | `recovery_order_is_learned_from_what_actually_produced_artifacts` teaches a client that `name_target_artifact` works, then requires a later run to spend it first and the never-effective strategy last |
| Local endpoint is authoritative | The six-client test inspects every adapter's actual argv, environment, or temporary config; a separate Codex regression rejects a reported public OpenAI endpoint |
| Strict NDJSON | Every stdout line in the reproducer and conformance matrix is independently parsed as one JSON value |
| Populated metadata | The matrix requires model, endpoint, and nonzero input/output tokens; the live Agent run records 15,186 input and 1,553 output tokens |
| Clean workspace | `.formal-ai/` is ignored by snapshots and added to `.git/info/exclude`; each matrix workspace exposes only `Hello.scala` to Git |
| Uniform behavior | One deterministic test sends the identical Scala request through scripted entrypoints for all six seed-registered clients |

The deterministic suite uses scripted native entrypoints so CI can assert
argument and configuration parity without external credentials or quota. The
real end-to-end evidence below exercises Formal AI, the installed Agent CLI,
the local HTTP server, the filesystem tools, and the same public controller
surface.

## Reproduction and verification

The minimal reproducer installs a fake Agent executable that exits zero, emits
pretty JSON with empty metadata, and changes no workspace file:

```bash
cargo test --test integration issue_879_completion -- --nocapture
```

Before the fix, the wrapper exited zero and passed the child JSON through
unchanged. The reproducer was committed separately before implementation.
After the fix, the suite requires all four bounded attempts, a nonzero wrapper exit,
`completion_state: "incomplete"`, reason
`required_workspace_effect_missing`, and no Git-visible scratch files.

The same module also covers successful exact-session recovery, the six-client
contract, local endpoint configuration, nonzero usage, and fail-closed public
endpoint detection.

## Formal AI self-hosting evidence

Formal AI was used through the real Agent CLI while developing this change:

- Agent session `ses_04b793abaffet1ti9eyNHFImLN` authored the completion
  contract. Its canonical
  [session](formal-ai-authorship-agent-session-3.json) retains the raw tool
  transcript and workspace effects.
- Agent session `ses_04b6ac0c7ffeWr2CRJCMTiPqz0` authored the reviewed
  [requirements ledger](requirements.lino), captured in its canonical
  [session](formal-ai-authorship-requirements-session.json).
- After the implementation, Agent session
  `ses_04b64048cffeToyNFkGfSItJoz` created the
  [verification ledger](verification.lino) through the completion gate. Its
  [canonical session](formal-ai-authorship-verification-session.json) records
  the local endpoint, the created path, `completion_state: "complete"`, and
  15,186 input / 1,553 output tokens. The reviewed ledger normalizes the
  literal newline delimiters from the task argument; the canonical session
  preserves the unmodified client output.

The Rust implementation, regression tests, and prose documentation were
written with Codex assistance and are not claimed as Formal AI-authored.

## Live ladder evidence

The ladder was also exercised end to end against the installed Agent CLI and a
temporary local Formal AI server on port 18099, in an empty Git repository, with
a request the local model could not satisfy. Session
`ses_048503119ffe9cCHeMerd8wBVF` spent all three strategies, the wrapper exited
nonzero with `completion_state: "incomplete"` instead of reporting a false
success, and the workspace stayed empty to Git. The
[completion record](raw-data/live-recovery-completion-record.json) and the
[recovery ledger it wrote](raw-data/live-recovery-ledger.lino) are retained
verbatim; the ledger landed in `~/.local/state/formal-ai/`, outside the
repository the client ran in.

## Source evidence

The issue had no comments or embedded screenshots when collected on
2026-07-30. All three pull-request feedback surfaces were also empty before
implementation: conversation comments, inline review comments, and reviews.
Those raw empty API responses are retained under [`raw-data`](raw-data).

The primary external evidence remains the linked
[Hive Mind case study](https://github.com/link-assistant/hive-mind/tree/main/docs/case-studies/issue-2119),
including all 13 original logs, and
[Hive Mind PR #2120](https://github.com/link-assistant/hive-mind/pull/2120).
That caller-side work detects empty diffs and frames malformed JSON; this
change supplies the provider-side artifact gate, bounded recovery, endpoint
diagnostic, strict output, and metadata.
