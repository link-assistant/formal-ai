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

The recording exposes two input hazards. The footer is not readiness, and the
adapter sent text and Enter at the same instant. Codex's pinned
[`PasteBurst` implementation](https://github.com/openai/codex/blob/rust-v0.147.0/codex-rs/tui/src/bottom_pane/paste_burst.rs)
suppresses submission for 120 ms after a text burst. Its
[`ChatComposer` handler](https://github.com/openai/codex/blob/rust-v0.147.0/codex-rs/tui/src/bottom_pane/chat_composer.rs)
inserts a newline instead of submitting during that window. Waiting for the
selected model alone therefore does not fix immediate text-plus-Enter delivery.

The contract now waits for `model: formal-ai`, allowing ANSI styling between
the label and value, then waits for the rendered draft before sending Enter as
a separate interaction after 200 ms of quiet output. This acknowledgment also
prevents a busy client from receiving text and Enter in the same later read.
It rejects the earlier footer, incomplete chunks and a different
model name. Other clients retain their existing single interaction. Startup
confirmation, the finite capture deadline, the real-client journey and the
independent dialog checks remain required. No new credentials or skipped
acceptance checks are introduced.

The actual public startup and draft frames are retained in
`experiments/agent_cli_e2e/issue_819_tui/fixtures/codex-startup.json` with their
source run, job, commit and complete-log digest. A mocked interactive process
replays them, rejects any prompt received during loading or Enter inside the
pinned paste window, and proves one settled prompt submission. Both old gates
are negative controls. The published terminal adapter executes this
regression in ordinary PR validation without installing or invoking external
agent clients. Lightweight Node tests also check the captured frames and partial
ANSI output. Native builds and real-client execution remain in CI.
