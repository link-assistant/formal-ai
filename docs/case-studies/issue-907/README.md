# Issue 907: the caller's framing is not the user's request

This case study records the reproduction, root cause, generalization, and
verification for the agent-mode intent hijack reported in issue 907: every
`gemini` run through `formal-ai serve --agent-mode` answered with
`run_shell_command({"command":"date"})` and silently dropped the actual request.

## Reproduction

The gemini CLI opens *every* turn with a block the caller cannot suppress:

```text
<session_context>
This is the Gemini CLI. We are setting up the context for our chat.
Today's date is Sunday, August 2, 2026 (formatted according to the user's locale).
My operating system is: linux
</session_context>

Write a hello world program in Python.
```

`experiments/issue-907/before.txt` is the captured sweep. The same request under
different framing, all other variables held constant:

```text
<session_context> … Today's date is Sunday, August 2, 2026 …  -> run_shell_command({"command":"date"})
<session_context> … (date sentence removed) …                 -> write_file(… main.py)
Today's date is Sunday, August 2, 2026.                       -> run_shell_command({"command":"date"})
The current time is 20:00.                                    -> run_shell_command({"command":"date"})
The date is Sunday.                                           -> write_file(… main.py)
Today is Sunday, August 2, 2026.                              -> write_file(… main.py)
date                                                          -> write_file(… main.py)
```

Deleting one sentence restored correct behaviour. Nothing else changed.

## Root cause

Two independent defects composed. Neither alone produced the failure, which is
why the reporter's elimination table (transport, prompt size, tool declarations,
message shape) came up empty — none of those was the variable.

1. **`user_request_text()` did not know what the client's framing looks like.**
   `src/protocol/content.rs` stripped exactly one marker, `<system-reminder>`,
   because Qwen Code was the client that had forced the question. gemini's
   `<session_context>` was therefore read as the user talking.
2. **`intent_shell_command()` matched cues by bare substring.** The *declarative*
   sentence "Today's date is …" contains the date intent's cue verbatim, so it
   fired an intent that only a question should fire.

The narrow reading of this is "add `<session_context>` to the strip list". That
is the shape of fix that produced the bug in the first place: `<system-reminder>`
was added the same way, one client at a time, and the next client re-broke it.
The reporter's own note — "`My operating system is: linux` is one `uname` pattern
away from the same bug" — is the general statement of the defect.

## Generalization

Three separations, each backed by seed data rather than by a branch in the
solver (CONTRIBUTING rule 7).

**1. Framing is data, not code.** `data/seed/caller-context.lino` declares every
client-injected block and the clients observed sending it:
`<system-reminder>` (claude, qwen), `<session_context>` (gemini),
`<environment_context>` (codex), `<env>` (agent, opencode),
`<environment_details>` (cline, roo). `src/seed/caller_context.rs` parses it and
`user_request_text()` removes them all. Registering the next client is an edit to
a `.lino` file, with no solver change and no new code path to regress.

**2. A statement is not a request — per sentence, not per cue.** Routing splits
the turn into sentences and classifies each: a sentence of the form
*&lt;cue&gt; &lt;copula&gt; &lt;value&gt;* states a fact and carries no intent.
Classification is at sentence granularity on purpose. Per *occurrence* — which is
what a cue-local copula check gives — "the current time is 20:00" suppresses the
cue `current time` but leaves the shorter cue `time`, riding inside the same
statement, free to route. Once the sentence is a statement, nothing in it routes.

The copulas, question words, subject determiners and request verbs live in the
same seed file across English, Russian, Hindi, Chinese and Spanish, so
`Текущее время — 20:00.` and `今天的日期是2026年8月2日。` are statements too —
while `今天的日期是什么？` still routes, because the question word marks it as a
question even with the mark stripped.

This is also the file that ended a duplication. Two different fixes for this
issue landed independently, one on `main` and one on this branch, and each grew
its own copy of the *is-this-asking-or-telling* vocabulary — one under
`shell-intents.lino`, one under `caller-context.lino`. Two copies of a natural
language list is exactly the drift this repository's seed rule exists to prevent,
so the merge keeps one home: `caller-context.lino`, read by any router that must
tell a request from the framing around it.

**3. A turn that carries a task gets the task.** When the turn requests a program
artifact, a built-in intent riding alongside it stands down. Answering the
smaller question and discarding the work is the worst available outcome, because
it exits 0 and reports success.

## Verification

### Per requirement, plus the whole task

`tests/unit/issue_907.rs` — one test per requirement in the issue plus the
whole-task test, all driven over the **real Gemini surface**
(`create_gemini_generate_content_response_with_solver_and_memory`), so a pass is
evidence about the reported behaviour rather than about an inner helper. Each
case uses a different phrasing (CONTRIBUTING rule 4), and
`asking_for_the_date_still_runs_date` exists so that a guard which simply
silenced the intent cannot pass the suite.

### Real client, end to end

The framing that caused this bug only exists once a real client injects it, so
the regression is guarded by the actual gemini CLI rather than by a fixture:
`experiments/agent_cli_e2e/run_issue_907.sh` boots
`formal-ai serve --agent-mode` and drives `gemini` against it over the native
Gemini routes, following `docs/testing/agentic-cli-tools.md`. Both directions
run in one leg — the reported request must leave `main.py` on disk and must never
emit `date`, and a real question must still reach `run_shell_command`. The step
is wired into the required `test-agent-cli-e2e` gate in
`.github/workflows/release.yml`, which now installs `@google/gemini-cli`
alongside the other mandatory clients.

Captured output is in `data/`.

## Related

- #904 — the caller's system prompt leaking into the plan's `goal` field, the
  same root pattern of treating caller framing as user content.
- #906 — naive keyword extraction in request parsing.
- link-assistant/hive-mind#2130 — where this was found; Hive Mind documented
  `gemini --model formal-ai` as blocked on it.
