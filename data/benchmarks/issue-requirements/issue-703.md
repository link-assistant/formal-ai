Parent: #651

## Motivation and evidence

Today Formal AI is the **server** side of agentic coding: external CLIs (codex, opencode, gemini, qwen, claude, agent) drive it through the OpenAI-compatible API (`src/agentic_coding/`). The vision — and #651's headline ("code itself using itself via Agent CLI, directed by Hive Mind") — also requires the opposite direction: **Formal AI as the orchestrator/controller of other AI agents**:

- Issue #655 (E36) closed with the headline Hive-Mind-dispatched end-to-end solve **never run** (upstream blocker [hive-mind#2059](https://github.com/link-assistant/hive-mind/issues/2059) "Invalid model name"); only the inner loop was verified.
- Issue #439's comment asked to connect Formal AI to [link-assistant/agent](https://github.com/link-assistant/agent) as `--model formal-ai` with user docs and output comparison against claude-sonnet — none of it appears in closing PR #471.
- Issue #385 / R385 asks for Agent-CLI self-hosting; today Formal AI can *be driven*, but cannot *drive*.
- The maintainer's standing direction (issues #651, #538, #558): tasks should be solvable by Formal AI directing agent sessions, with recorded session JSON reproducing the change.

## Requirements

1. **Agent-runner capability**: a permission-gated `run_agent` tool (registered in `data/seed/tools.lino`, agent-mode only, isolated workspace) that launches an external agent CLI — `agent`, `claude`, `codex`, `gemini`, `qwen`, `opencode` — with a task prompt, deterministic config (model, cwd, timeouts, allowlisted commands), captures the full session (stdout/JSON stream), and records it as append-only events with provenance.
2. **CLI adapter registry as seed data**: per-CLI invocation shape (binary, flags for non-interactive mode, session-capture format, how to pass an OpenAI-compatible base URL) lives in a `.lino` registry, so adding a CLI is a data edit. Include pointing any of them **back at Formal AI itself** (`--model formal-ai` via the local server) — closing the #439 ask — and at vendor models when the user configures credentials.
3. **Task dispatch protocol**: decompose a repository task (per the universal solver) into sub-tasks, dispatch each to a configured agent, collect results, verify them with generated tests in the bounded workspace, and compose — the map-reduce shape of the problem-solving methodology (konard/problem-solving).
4. **Parallel dispatch + comparison**: optionally dispatch the *same* sub-task to N different agents/CLIs, verify each result against the same tests, record a comparison ledger (pass/fail, diff size, wall time), and select the winner — evidence-linked, deterministic given recorded sessions.
5. **Hive-Mind path**: unblock and complete #655's end-to-end scenario — Hive Mind dispatches an issue, Formal AI (as the model behind Agent CLI) or Formal AI (as orchestrator invoking Agent CLI) produces the branch/PR; follow up on hive-mind#2059 upstream and record status here.
6. **Honesty and safety**: agents run only in agent mode with explicit permission grants and isolation (Docker via the existing runtime or workspace sandbox); every external call and file effect is an event; failures are surfaced, never retried silently.
7. Offline reproducibility: recorded sessions replay byte-for-byte in CI (the recording-proxy discipline of #671 / PR #631 reused for the client direction).

## Acceptance criteria

- `formal-ai agent run --cli codex --task "add a README badge" --workspace <tmp>` completes a real session, produces the diff, and stores the session events; the equivalent works for `agent`, `gemini`, `qwen`, `claude`, `opencode` (recorded/replayed in CI, live behind env gates).
- A dispatch of one task to ≥ 2 CLIs in parallel yields a committed comparison ledger and a selected, test-verified winner.
- One end-to-end recorded run exists where Formal AI orchestrates Agent CLI to fix a scripted repository issue, reproducible via replay in CI (completes the spirit of #655).
- Docs: an "Orchestration" page describing the adapter registry, permissions, and the `--model formal-ai` loop-back setup.

## Dependencies

- Blocked by #681/#682-class wire fixes (PRs #684/#685) and benefits from #671 (E52 matrix + recording proxy infrastructure).
- Blocks #657 (E38 self-hosting metric needs attributable orchestrated sessions) and the #651 self-coding goal.
- Related: #654/#655 (closed E35/E36), #687 (meta-requests in agentic mode).

## Process

Collect data to `docs/case-studies/issue-{id}` (per-CLI invocation research, session-format survey, upstream hive-mind#2059 status, comparison-methodology notes); single PR per milestone until every requirement is addressed.

