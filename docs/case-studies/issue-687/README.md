# Issue 687 case study — from symbolic fallback to agentic action

Issue #687 records four requests sent to Formal AI through an agentic CLI. Each
request named a capability the surrounding client already had, but Formal AI
either returned its unknown-answer text or described work without doing it:

1. `When next elections in the USA?`
2. `Report`
3. `What we were talking about?`
4. `Learn about it.`

The screenshot is preserved as
[`images/01-opencode-session.png`](images/01-opencode-session.png), and the
GitHub snapshots available when the investigation began are under
[`raw-data/`](raw-data/), including the issue, PR, conversation comments, inline
review comments, reviews, and recent workflow metadata. The later maintainer direction in
[PR comment 4977401752](https://github.com/link-assistant/formal-ai/pull/688#issuecomment-4977401752)
required a deeper, auto-learning-oriented implementation and execution of the
same task through Formal AI using Agent CLI.

## Reconstructed timeline

| Time (UTC) | Event |
| --- | --- |
| 2026-07-13 19:51 | Issue #687 filed with the four-turn screenshot and cross-environment requirements. |
| 2026-07-14 | Initial planner recipes, tests, and case study added to PR #688; CI follow-up fixed fixture and multilingual gaps. |
| 2026-07-15 06:01 | Maintainer rejected the shallow endpoint-specific solution and asked for deeper generalization, auto-learning alignment, and the exact task through Agent CLI. |
| 2026-07-15 | Branch merged current `main`, including issue #686's associative-learning architecture. Hard-coded request tables were replaced by Links Notation seed semantics shared across planner roles. |
| 2026-07-15 | The release Formal AI server was driven through the installed `@link-assistant/agent` in four continued invocations. It completed two research cycles, executed the report shell action, recalled the election topic, and completed at least nine OpenAI-compatible chat rounds. |
| 2026-07-15 | A real Chromium flow verified natural-language control for UI settings that previously had no message route. The test also exposed and fixed a stale-bundle bug in the Playwright server command. |

## What actually failed

The defect was not a single missing `if`. It was four architectural gaps:

- Agentic capability selection depended on local phrase tables rather than the
  project's seed knowledge, so new wording and languages required Rust edits.
- Recall and contextual learning did not share one history interpretation path.
- Tool progress was scanned globally, allowing an old tool result to satisfy a
  later user turn.
- The UI advertised controls whose state setters were not all reachable through
  message commands, and those commands had no declarative capability catalog.

See [`root-cause.md`](root-cause.md) for the traced control flow and
[`requirements.md`](requirements.md) for every issue requirement.

## Resulting architecture

The implementation makes the seed the extensibility boundary:

- `data/seed/meanings-agent-actions.lino` declares report, recall, and research
  semantics in English, Russian, Hindi, and Chinese.
- `data/seed/agent-info.lino` owns repository and issue-report templates.
- Report, recall, and research roles consume the same embedded seed registries.
- `solve_with_history` is the shared path for conversational recall and
  contextual follow-ups.
- Research ranks official government and educational URLs, fetches the chosen
  source, and cites it in the answer.
- Planner progress is scoped to the latest user turn, so a later `Learn about
  it.` cannot reuse an earlier web result.
- `data/seed/interface-capabilities.lino` maps natural-language phrases and
  values to UI preference keys. Adding an enum alias or another catalogued
  preference no longer requires another recognizer branch.

This builds on the associative persistent-learning foundation merged from issue
#686: learned conversation associations remain data, while the planner uses
seed-declared semantic roles instead of duplicating language policy in code.

## Reproduction and verification

The minimum deterministic reproduction is
[`tests/unit/issue_687.rs`](../../../tests/unit/issue_687.rs). It covers the four
reported turns, multilingual variants, official-source preference, source
citation, shell escaping, and the stale-progress regression.

The system reproduction is
[`experiments/agent_cli_e2e/run_issue_687.sh`](../../../experiments/agent_cli_e2e/run_issue_687.sh).
It starts the release Formal AI server and runs the installed Agent CLI four
times with `--continue --no-fork`. A PATH-local fake `gh` proves that the client
executes the generated command without creating a real GitHub issue. The run
completed with this invariant:

```text
issue #687 E2E OK: report executed, recall retained context,
follow-up researched it (9 rounds)
```

The browser reproduction is
[`tests/e2e/tests/issue-687.spec.js`](../../../tests/e2e/tests/issue-687.spec.js).
It verifies seed-backed commands for thinking detail, message animation,
follow-up probability, toolbar icon pack, and Full Auto mode.

The final Chromium state is preserved as
[`images/02-seed-interface-controls.png`](images/02-seed-interface-controls.png).
It shows the UI in Full Auto after the settings were changed through chat rather
than by directly manipulating the controls.

## Related evidence

- [`online-research.md`](online-research.md) records primary sources and the
  exact official election fact used to validate research behavior.
- [`upstream.md`](upstream.md) explains why no OpenCode or Agent issue was filed.
- No new Cargo dependency was needed; the implementation composes the existing
  planner protocol, seed parser, associative memory, client tools, and browser
  settings state.
