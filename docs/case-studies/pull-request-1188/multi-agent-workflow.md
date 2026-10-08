# Recommended multi-agent workflow for pull request #1188

The owner clarified this arrangement on 2026-10-09. It supersedes the older immediate-push and manual-requirement-edit advice in this case study. The objective is to make Formal AI substantially more capable at changing its own code while delivering every recorded requirement and a green, usable release.

## Roles and capacity

Keep the available slots busy. This environment admits four active agents including the coordinator, so three subagents run concurrently; a fourth subagent can take a slot after a completed batch. Use GPT-6.1-sol for subagents as requested. Never terminate an unfinished agent merely to replace it.

| Role | Continuous responsibility |
| --- | --- |
| Coordinator | Work independently, integrate batches, keep ownership clear, inspect evidence, verify production release paths and serialize pushes. |
| Formal AI delivery agent | Ask Formal AI to implement requirements through its JavaScript planner. If it fails, reproduce the defect, repair Formal AI generally, and retry the original request unchanged. Keep this role occupied. |
| Requirements drafting agents | Read issues, pull request and repository requirements; draft every undrafted requirement in bulk, lowest resolution first, with honest status and executable evidence. |
| CI/CD fixer | Continuously watch checks on the remote pull request head, immediately collect failures and draft their fixes in bulk. Keep monitoring while local commits accumulate. |

## Ask, repair, retry

1. Give Formal AI a concrete requirement or coding task and run its JavaScript source in-process where possible. Let it inspect the source, plan tools and execute changes.
2. Verify the actual files and behavior, retaining the original task and transcript. A final answer alone is not proof of a delivered change.
3. If Formal AI fails, record the failure and repair the general capability that blocked it. Prefer seed vocabulary or a shared mechanism over a case for the failing prompt. The dedicated agent may repair Formal AI after the failure; it must never implement the requirement itself manually.
4. Ask Formal AI to perform the original task again. Require the intended result and a regression that covers the class. A workaround does not close the capability gap without a successful retry.
5. Generate the other roots and derived records by their checked rules, run only the nearest JavaScript checks locally, and leave Rust compilation and full suites to CI.

All requirement, code, test and documentation changes go through Formal AI or the checked generation rules. A repaired planner must be exercised against the original task. Preserve failures and any exceptional repair authorship honestly in the dogfood tally.

Check JavaScript syntax after changing a module the driver imports, before other agents start a fresh driver. Helper modules must guard executable entry points so importing them cannot apply a draft twice. If an unfinished repair prevents boot, temporarily supply only that module from a known-good committed source through an in-memory Node loader, ask Formal AI to restore the reviewed source, and verify syntax plus an unmasked fresh-process retry. The loader is bootstrap infrastructure; all repository mutations still go through Formal AI. Keep the failed requests and actual tool effects.

## Commits and push barriers

Create many small, coherent commits as completed pieces become reviewable. Serialize commits with the shared atomic directory lock `/private/tmp/formal-ai-pr1188-commit.lock`: acquire it before reading HEAD or creating the private index, stage only owned files, confirm HEAD still equals the captured base, commit, refresh the default index and release the lock. A private index based on an older HEAD can otherwise revert another agent's committed work. Do not push each individual change.

The coordinator pushes only at one of two barriers: all contributing subagents have completed their current bulk batch, or every fix in the current CI/CD failure batch is fully committed. Before a push, inspect the complete committed batch and make required generated records current. New unrelated work can remain in the working tree.

After pushing, monitor the new remote head. Cancel superseded runs where appropriate, fix every new failure and repeat the barrier. Keep a CI monitor and a Formal AI delivery agent active while the coordinator continues work. Do not merge this pull request merely because local tests pass.

## Evidence, cost and completion

Record delegated tasks, Formal AI tool calls, verified successes, failures, general repairs and successful retries. Distinguish tasks planned by Formal AI from reviewed literal substitutions it merely applies. Never claim independent feature synthesis from a literal patch.

Delegating more execution and planning to Formal AI should reduce work performed by coordinating models. Measure that trend through recorded task outcomes and model usage where available; do not invent token savings or monetary costs from tool-call counts.

Completion requires every planned requirement to be delivered, all required checks green on the exact remote head and production artifacts verified. Exercise Docker, CLI, server, browser, desktop and package release paths, including the actual triggers and credentials. Keep post-merge-only observations explicitly pending until they can be observed.

The owner supplies the requirements through issues, the pull request and the repository. Work autonomously, resolve routine choices, record decisions and do not ask the owner questions.
