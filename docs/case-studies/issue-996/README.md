# Issue #996 — A step to Formal AI being able to code itself, via Hive Mind

Issue: <https://github.com/link-assistant/formal-ai/issues/996>
Pull request: <https://github.com/link-assistant/formal-ai/pull/997>
Hive Mind companion issue: <https://github.com/link-assistant/hive-mind/issues/2146>

## Collected evidence

`raw-data/` preserves everything the analysis below is built on:

| File | Origin | Content |
| --- | --- | --- |
| `raw-data/hive-mind-log-XOq3KX-sanitized.log.txt` | gist `konard/d0553b8e1b5ed88f1b8241f539ba4907` | Hive Mind `solve` session where `--model formal-ai --tool agent` was requested |
| `raw-data/hive-mind-log-PhU3X2-sanitized.log.txt` | gist `konard/3df8fd313b592842d9dcf34bd265d7ca` | Session that produced no code changes and triggered auto-restart/resume |
| `raw-data/hive-mind-log-8GWnaH-sanitized.log.txt` | gist `konard/111bc2c200954b7f524e5f314e49f43e` | Second session with the same no-code-changes problem |
| `raw-data/github/issue-996.json` | GitHub API | The formal-ai issue body and metadata |
| `raw-data/github/hive-mind-issue-2146.json` | GitHub API | The Hive Mind companion issue |
| `raw-data/github/hive-mind-issue-2146-comments.json` | GitHub API | Its comment thread (the Hive Mind/Formal AI responsibility split) |
| `raw-data/github/open-issues.json` | GitHub API | Open formal-ai issues at analysis time (2026-08-10) |
| `raw-data/github/open-prs.json` | GitHub API | Open formal-ai pull requests at analysis time |

## Requirements extracted from the issue

The issue text decomposes into these requirements. The Hive Mind companion
issue #2146 contributes three Formal-AI-side sub-requirements (R1a–R1c),
because #996 explicitly names it as "one of the issues we are working on on
Hive Mind side".

| ID | Requirement | Status in PR #997 |
| --- | --- | --- |
| R1 | Advance the Formal-AI side of the Hive Mind self-coding loop (#2146) | see R1a–R1c |
| R1a | The driving model for a `formal-ai` run must be Formal AI only; the Agent CLI must not silently substitute another LLM | analyzed below; enforcement issue filed/planned upstream (Agent CLI side), Formal-AI-side surface documented |
| R1b | A Formal-AI-driven coding run must make it from start to end with real code changes — no empty runs that trigger Hive Mind auto-restart/resume loops | analyzed below |
| R1c | Formal AI final messages must wrap tab/space-indented text and code in fenced codeblocks so GitHub comments render correctly | fixed in this PR with tests |
| R2 | Take the most critical open formal-ai issues and close them in this single PR | fixed set listed below |
| R3 | Skip issues already executing in open PRs (#991, #990, #988, #710, #705, #651, #447, #483, #557 at analysis time) | respected |
| R4 | Architectural changes are welcome when they generalize logic (never specialize) | respected in each fix |
| R5 | Keep improving the associative technological stack: workarounds here must be paired with tracking issues; report upstream issues where separation of concerns benefits | documented below |
| R6 | Propose entirely new libraries/components if any look missing | proposals below |
| R7 | Compile all related data into `docs/case-studies/issue-996` | this folder |
| R8 | Deep case-study analysis, including online research | this document |
| R9 | List each and all requirements from the issue | this table |
| R10 | Propose solutions and solution plans for each requirement, checking existing components/libraries | per-requirement sections below |
| R11 | Plan and execute everything in this single pull request | PR #997 |

## Session-log analysis (R1, R1a, R1b, R1c)

_(filled in below from the three sanitized logs)_

## Most critical open issues and what this PR closes (R2, R3)

_(filled in below)_

## Solution plans per requirement (R10)

_(filled in below)_

## Existing components and libraries surveyed (R10)

_(filled in below)_

## Online research (R8)

The wider field confirms the direction #996 points at — agents that improve
the very tooling they run on:

- [Live-SWE-agent (arXiv:2511.13646)](https://arxiv.org/abs/2511.13646)
  demonstrates an agent that starts from a bash-only scaffold and evolves its
  own tools **while** solving issues, reaching 75.4% on SWE-bench Verified —
  no offline training, LLM-agnostic. The lesson for Formal AI: the
  self-coding loop does not need a finished toolbox up front; it needs a
  reliable start-to-end loop plus the ability to extend itself mid-run,
  which is exactly the gap the PhU3X2/8GWnaH logs expose (R1b).
- [OpenHands](https://github.com/OpenHands/OpenHands) (MIT-licensed,
  ex-OpenDevin) executes plans as code actions in a sandboxed Docker
  environment and iterates on test results — the same
  execute-verify-iterate contract CONTRIBUTING.md already mandates for the
  in-repo agentic driver.
- [SWE-agent](https://github.com/SWE-agent/SWE-agent) introduced the
  Agent-Computer Interface: a small, stable command surface an agent can
  reliably drive. Formal AI's OpenAI-compatible `serve` boundary plus the
  external `@link-assistant/agent` CLI is this project's ACI; keeping it
  small and deterministic is what makes symbolic self-coding testable.
- [SWE-EVO (arXiv:2512.18470)](https://arxiv.org/pdf/2512.18470) and
  [SWE-Chain (arXiv:2605.14415)](https://arxiv.org/pdf/2605.14415)
  benchmark long-horizon, multi-step evolution — evidence that "one issue,
  one PR, every requirement" (R11) is measured industry-wide as chained
  end-to-end completion, not single-patch success.

## Upstream reports and new-component proposals (R5, R6)

_(filled in below)_
