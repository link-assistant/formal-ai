**Problem.** Formal AI must be well integrated with link-assistant/
hive-mind through agentic harness CLIs and TUIs (issue #914). The pieces
exist — the orchestration module (#703) dispatches external CLIs, the
server speaks the OpenAI and Anthropic protocols, and hive-mind#2059
specifies `solve ISSUE_URL --tool agent --model formal-ai` — but no gate
proves a full circle in either direction.

**Approach.** Build one replayable end-to-end scenario per direction and
keep both as permanent gates. Direction one: hive-mind drives the Agent
CLI with Formal AI as the model behind the OpenAI-compatible server, and
the gate asserts an observed workspace effect landing as a commit,
following the byte-exact replay pattern of
`experiments/issue_890_agent_cli.sh` and its CI job. Direction two:
Formal AI's orchestrator dispatches an external agent CLI on a
hive-mind-shaped issue task, with the hash-chained session replayed in
CI. Evidence lands as case-study folders.

**Existing components.** `src/orchestration/` (#703);
`src/server.rs` protocol namespaces; the `formal-ai with` wrapper and
`data/seed/client-integrations.lino`;
`scripts/mine-hive-mind-dataset.rs`; the #890 Agent-CLI evidence and
replay-gate pattern; closed groundwork in #655; hive-mind#2059.

**Acceptance criteria.**
- Both directions have committed evidence folders and deterministic
  replay scripts gated in CI.
- Direction one succeeds through the exact hive-mind invocation shape
  from hive-mind#2059, with an observed workspace effect.
- Failures anywhere in the chain propagate honestly (no narrated
  success), reusing E69's exit-code guarantees.
- Depends on E69.

---

Part of the issue #914 vision-planning batch (E69-E77). Parent: #914. Case study, gap analysis, and design rules: `docs/case-studies/issue-914/` on the issue #914 branch (pull request #915).

