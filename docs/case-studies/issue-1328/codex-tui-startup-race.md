# Codex TUI readiness race

Main run `40109825719`, job `120922662089`, failed the grounded-action
journey after Codex's TUI timed out. The same local-search task passed through
Agent, OpenCode, Claude and Codex's command-line interfaces, and through the
OpenCode and Claude TUIs. Codex's failed TUI server log contained only a health
request: the prompt never reached Formal AI.

The captured terminal events establish the cause. The `formal-ai default`
footer appeared at 0.232 seconds while the banner still showed `model: loading`.
The harness submitted the prompt and Enter at 0.237 seconds. The selected-model
banner appeared at 0.423 seconds; the text remained an unsubmitted draft.

The readiness contract now waits for `model: formal-ai`, allowing ANSI styling
between the label and value. It rejects the earlier footer, incomplete chunks
and a different model name. The existing startup confirmation, finite timeout,
real-client journey and independent dialog checks remain required. No new
credentials or skipped acceptance checks are introduced.

The two actual public frames are retained in
`experiments/agent_cli_e2e/issue_819_tui/fixtures/codex-startup.json` with their
source run, job, commit and complete-log digest. A mocked interactive process
replays them, rejects any prompt received during loading and proves one prompt
is submitted after readiness. The published terminal adapter executes this
regression in ordinary PR validation without installing or invoking external
agent clients. Lightweight Node tests also check the captured frames and partial
ANSI output. Native builds and real-client execution remain in CI.
