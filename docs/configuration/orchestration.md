# Multi-agent orchestration

`formal-ai agent` is the explicit controller for external coding agents. It
keeps the conservative `formal-ai with` defaults unchanged while providing a
separate permission-gated path for editing, verification, parallel comparison,
bounded decomposition, composition, and byte-replayable provenance.

## Run one agent

Choose one seed-registered adapter and grant an existing workspace:

```bash
formal-ai agent run \
  --cli codex \
  --task "add a README badge" \
  --workspace /tmp/example-repository \
  --session /tmp/codex-session.json
```

The library API denies execution by default. The CLI canonicalizes
`--workspace` and creates a capability for that exact path. The controller
records the child process output, status, wall time, and file effects. Client
editing controls still apply: for example, the Codex adapter selects its
`workspace-write` sandbox. The runner sets both the process current directory
and `PWD` to the canonical workspace so clients cannot accidentally select the
controller's repository as their project. A timeout kills the external process
and its process group on Unix, then records one failed attempt; the controller
never retries implicitly.

The six orchestration adapters are `agent`, `claude`, `codex`, `gemini`,
`qwen`, and `opencode`. Their non-interactive editing and structured-output
arguments live in `data/seed/client-integrations.lino`.

An explicitly granted command makes the same controller available to another
CLI, TUI's noninteractive entrypoint, local neural model, or fixed Bash
adapter. JSON argv must contain `{task}`; the placeholder is replaced as one
argument, without controller-side shell parsing:

```bash
formal-ai agent run \
  --cli local-shell \
  --command '["sh","-c","printf \"%s\\n\" \"$1\" > answer.txt","formal-ai-agent","{task}"]' \
  --allow-agent-command sh \
  --task "record this exact task" \
  --workspace /tmp/example-repository
```

`--command` never inherits trust from `--cli`. Even if a custom command is
labelled `codex`, its executable needs a separate exact
`--allow-agent-command` grant. The task placeholder is mandatory, empty argv is
rejected, and the runner still applies the workspace, deadline, effects,
verification, and provenance boundaries.

The capability grant limits what Formal AI will launch and compose; filesystem
containment is supplied by each client policy or by the environment running
Formal AI. Codex uses `workspace-write`; Agent, Claude, Gemini, Qwen, and
OpenCode use their noninteractive edit policies rooted at the selected
workspace. Run untrusted client binaries in a container or equivalent host
sandbox as an additional boundary.

By default, the selected client talks to the local Formal AI model:

```bash
formal-ai serve --host 127.0.0.1 --port 8080
formal-ai agent run \
  --cli agent \
  --task "create release-notes.md" \
  --workspace /tmp/example-repository
```

Use `--model` or `--base-url` to override that target. `--target vendor` skips
the Formal AI wrapper and uses the client's existing vendor authentication;
pair it with `--model` to select a model that vendor provides. The permission,
timeout, effects, verification, and provenance boundaries remain active.

## Control several custom agents

Dispatch accepts one custom command mapping per CLI id, so registered agents,
local model frontends, and fixed Bash adapters can participate in the same
parallel comparison:

```bash
formal-ai agent dispatch \
  --cli shell-a,shell-b \
  --compare \
  --command 'shell-a=["sh","-c","printf \"a: %s\\n\" \"$1\" > answer.txt","a","{task}"]' \
  --command 'shell-b=["sh","-c","printf \"b: %s\\n\" \"$1\" > answer.txt","b","{task}"]' \
  --allow-agent-command sh \
  --task "propose a release title" \
  --workspace /tmp/example-repository
```

Every candidate gets its own copy and canonical session. A custom executable
has exactly the same deny-by-default rule in single, decomposed, and comparison
runs.

## Require verification

Verification is argv-based and default-denied. Each executable must be
allowlisted separately:

```bash
formal-ai agent run \
  --cli codex \
  --task "fix the failing parser test" \
  --workspace /tmp/example-repository \
  --allow-command cargo \
  --verify '["cargo","test","parser"]'
```

Repeat `--allow-command` and `--verify` for additional checks. Commands run
without a shell, use the same hard timeout, and retain stdout, stderr, exit
status, and timeout state in the session. A process or verification failure is
visible to the caller and is never treated as a passing result.

## Compare the same task

Comparison copies the workspace once per CLI, runs all candidates in parallel,
and composes only a passing winner:

```bash
formal-ai agent dispatch \
  --cli codex,claude,opencode \
  --compare \
  --task "add a README badge" \
  --workspace /tmp/example-repository \
  --allow-command cargo \
  --verify '["cargo","test","--workspace"]'
```

The recorded selection order is passing status, smaller diff, shorter wall
time, CLI id, then session path. Candidate workspaces, canonical sessions, and
`comparison-ledger.json` are written below
`.formal-ai-orchestration/`. A custom `--output-dir` must remain strictly
inside the granted workspace and is excluded from candidate copies.

## Decompose and compose a task

Omit `--compare` to use the universal task decomposer:

```bash
formal-ai agent dispatch \
  --cli codex,claude \
  --task "add a README badge and write a release note" \
  --workspace /tmp/example-repository
```

Bounded leaves are assigned round-robin to isolated candidate copies.
Composition preflights overlapping effects and applies changes only from
passing sessions. Identical overlapping effects are deduplicated and
conflicting effects are rejected. Immediately before composition, the
controller revalidates both the original pre-run hashes and the candidate
post-run hashes, preventing workspace or candidate drift from being composed.
Failed candidates remain isolated. Duplicate CLI ids are rejected before any
worker starts. `--max-depth` controls the decomposition bound.

## Split only what actually fails

`--decompose` plans before it has evidence: it splits the task whether or not
the CLI needed the split. `--incremental` inverts that order. The whole task is
attempted first, and only a failure justifies a split:

```bash
formal-ai agent dispatch \
  --cli codex,claude \
  --incremental \
  --task "add a README badge and write a release note" \
  --workspace /tmp/example-repository
```

To turn each verified effect into an independently reviewable commit, run in a
clean Git worktree and bind the dispatch to its pull request:

```bash
formal-ai agent dispatch \
  --cli agent \
  --incremental \
  --pull-request https://github.com/example/project/pull/123 \
  --task "implement the reviewed issue" \
  --workspace /tmp/example-repository
```

`--pull-request` is intentionally opt-in and requires `--incremental`. Before
starting an agent, the controller rejects a malformed URL, a non-Git workspace,
or any tracked, staged, or untracked work. Each passing session with an effect
must expose its native session id. The controller then stages only that
session's verified paths and canonical session JSON, checks the exact staged
set, and creates a commit carrying `Formal-AI-Session`, `Formal-AI-Evidence`,
and `Formal-AI-Pull-Request`. Passing verification-only sessions and passing
sessions with no file effect do not create empty commits. Without
`--pull-request`, incremental dispatch keeps its compose-only behavior.

The protocol is the repository's own failure-driven controller
(`solve_recursively`) with the repository's own splitter behind its split hook,
one level per split, so every deeper split is justified by a failure that
actually happened. A passing attempt's effects are applied to the workspace
before the next attempt starts, so the pieces build on each other and the
parent's retry sees their work. A task that the splitter cannot shrink is
irreducible; it is escalated to the next CLI in `--cli`, and when that list is
exhausted it is reported blocked rather than silently retried.

The report gains an `incremental` section listing every attempt, every split
with the failure evidence that caused it, and every blocked task — the exact
input a reviewer needs to decide what capability is missing. Exit status
reflects the root task only: pieces are allowed to fail on the way to solving
it.

Each blocked task also becomes a proposal, mirrored to `proposals.lino` in the
output directory:

```lino
incremental_proposals
incremental_proposal "incremental_proposal_cdea3001ce44c46f"
  task "Add dev/log/ to the excluded_folders array."
  tried_cli "codex"
  tried_cli "claude"
  failure_evidence "cli:codex status:Failed exit:7 verification:"
  failure_evidence "cli:claude status:Failed exit:7 verification:"
  status "human_review_required"
```

A run cannot approve its own extension, so the status is always
`human_review_required` — the same gate a learned decomposition strategy passes
through before it may be used. A run that solved everything still writes the
document, empty but for its header, because "nothing to propose" and "the run
never got that far" are different answers.

## Resume a disproved result in the exact native session

Registered adapters retain a stable, workspace-scoped client home under
`.formal-ai-orchestration/native-sessions/<cli>`. A completed controller
session records the native client session id and the registry-derived resume
command. Supply the recorded parent, the disproved statement, and reviewable
evidence:

```bash
formal-ai agent resume \
  --parent /tmp/first-session.json \
  --task "correct the parser and rerun its focused test" \
  --workspace /tmp/example-repository \
  --disproved-claim "the parser test passes" \
  --evidence "cargo test parser failed at tests/parser.rs:42" \
  --session /tmp/corrected-session.json
```

The correction prompt is rendered from seed data in English, Russian, Hindi,
or Chinese. It carries the parent session SHA-256, inherits the parent's
client, target, model, and base URL, and records a `continuation` link in the
child session. A client that reports a different native id fails the run.
Clients that do not repeat the id retain the verified parent id.

The exact resume contracts are data, not Rust branches:

| CLI | Resume argv | Primary client documentation/source |
| --- | --- | --- |
| Agent CLI | `--resume ID --no-fork` | [stdin-mode documentation](https://github.com/link-assistant/agent/blob/main/docs/stdin-mode.md) |
| Claude Code | `--resume ID` | [CLI reference](https://docs.anthropic.com/en/docs/claude-code/cli-usage) |
| Codex | `exec resume ID` | [Codex exec CLI source](https://github.com/openai/codex/blob/main/codex-rs/exec/src/cli.rs) |
| Gemini CLI | `--resume ID` | [CLI reference](https://github.com/google-gemini/gemini-cli/blob/main/docs/cli/cli-reference.md) |
| Qwen Code | `--resume ID` | [TypeScript SDK/CLI equivalence](https://github.com/QwenLM/qwen-code/blob/main/packages/sdk-typescript/README.md) |
| OpenCode | `--session ID` | [run command source](https://github.com/anomalyco/opencode/blob/dev/packages/opencode/src/cli/cmd/run.ts) |

For a custom command, repeat `--command` and its executable grant; resumed
custom argv must contain both `{session_id}` and `{task}`.

## Formalize, cross-check, summarize, and translate

Add synthesis to dispatch, or synthesize existing canonical sessions:

```bash
formal-ai agent dispatch \
  --cli codex,claude \
  --compare \
  --synthesize \
  --response-language ru \
  --task "explain the parser failure" \
  --workspace /tmp/example-repository

formal-ai agent synthesize \
  /tmp/codex-session.json /tmp/claude-session.json \
  --response-language ru
```

The report extracts only answer-bearing JSON/JSONL events, preserves the
complete streams in their sessions, formalizes each answer into the
meta-language, deduplicates statements, ranks their evidence, records
contradictions, rechecks presentation, and emits correction requests for
denied claims. Its `fact_check_scope` is deliberately
`cross_agent_evidence_preflight`: agreement between models is not external
proof. Claims needing factual guarantees must still use captured primary
sources and the production fact-checking pipeline.

The summarizer supports `en`, `ru`, `hi`, and `zh`. If its deterministic output
does not detect as the requested language, the report remains explicit with
`translation_required: true`; it never relabels untranslated text. Run a
translator as another recorded agent and pass that canonical session with
`--translation-session`. The result is accepted only when language detection
matches the request, and the translator session digest is retained.

## Learn from recorded client behavior

Repeated orchestration sessions can feed the existing client-contract learner:

```bash
formal-ai agent learn \
  /tmp/codex-session.json /tmp/codex-corrected-session.json
```

This produces an evidence-linked Links Notation report. Learning remains
proposal-only and human-gated: observations can recommend adapter facts, but
cannot silently edit the seed registry or approve their own promotion.

## Replay a session

```bash
formal-ai agent replay /tmp/codex-session.json
```

Sessions use canonical pretty JSON with a terminal newline. Replay validates
the schema, event sequence, previous-event links, SHA-256 event digests, and
exact serialization bytes before printing the record. This detects reordered,
edited, truncated, or noncanonical evidence.

For an end-to-end controller run and a committed two-client comparison, see
the [issue 703 case study](../case-studies/issue-703/README.md).

## Real-client compatibility gate

The six installed CLIs can be exercised with the exact acceptance task:

```bash
FORMAL_AI_ISSUE_703_LIVE=1 \
experiments/issue_703_orchestration/run_live_cli_matrix.sh
```

This gate is opt-in because it launches authenticated clients and may consume
vendor quota. Each client receives a fresh workspace; the gate requires a
successful canonical session, a recorded README effect, and the actual badge
in that workspace.

Codex's native sandbox requires user namespaces. If the surrounding CI job is
already externally sandboxed but blocks those namespaces, the harness alone
supports `FORMAL_AI_ISSUE_703_CODEX_UNSANDBOXED=1`. This does not change the
product default and must not be used on an unsandboxed host.
