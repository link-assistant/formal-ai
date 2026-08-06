# Captured output — real gemini CLI run

Produced by `ARTIFACT_DIR=docs/case-studies/issue-907/data
experiments/agent_cli_e2e/run_issue_907.sh` on 2026-08-06 against
`formal-ai serve --agent-mode` at `http://127.0.0.1:8907/api/gemini`.

- `formal-ai.log` — the server's request trace and planner outcomes. Lines are
  cut to 800 characters so the prompt bodies do not bury the trace; the
  `[trace] agentic_outcome:` lines are intact.
- `gemini-task.log` — leg 1, `gemini -p "Write a hello world program in Python."`.
  The turn carries the CLI's `<session_context>` framing with "Today's date is
  Thursday, August 6, 2026 …" and still produces `main.py`; no `date` command is
  planned anywhere in this leg.
- `gemini-question.log` — leg 2, `gemini -p "what is the date?"`. The intent
  still fires: the server plans
  `run_shell_command({"command":"date"})` and answers with its output.
