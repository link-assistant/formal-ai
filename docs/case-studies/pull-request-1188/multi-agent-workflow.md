# Recommended multi-agent workflow for pull request #1188

The owner updated this arrangement on 2026-10-10; the 2026-10-09 arrangement is retained below as dated history. It supersedes the older immediate-push and manual-requirement-edit advice in this case study. The objective is to make Formal AI substantially more capable at changing its own code while delivering every recorded requirement and a green, usable release.

## Roles and capacity

Keep the available slots busy. Current owner authorization (2026-10-10): up to 12 GPT-6.1-sol subagents, with 13 active agents including the coordinator. Reuse completed slots; do not create extra agents beyond this capacity. Never terminate an unfinished agent merely to replace it. Keep at least one CI watcher and one whole-ask Formal AI delivery agent active throughout.

Historical capacity (2026-10-09): this environment admitted four active agents including the coordinator, so three subagents ran concurrently and a fourth role waited for a completed slot. This historical capacity is not the current limit.

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
5. Automate the other roots and derived records through maintained JavaScript ↔ TypeScript and meta-language/Rust general translation or transformation rules with `--check`. Record temporary workarounds and the upstream blockers that retire them; unsupported native ports are not automated translation proof. Run only the nearest JavaScript checks locally, and leave Rust compilation and full suites to CI.

All repository content changes go through actual maintained JavaScript Formal AI; the coordinator is the sole repository writer. Subagents produce scratch drafts and source-qualified deterministic transformation rules with generation and `--check`, frozen preimages, exact postimage hashes and inverses. The coordinator asks Formal AI to apply only the reviewed finite source cohort. A repaired planner must be exercised against the original task. Preserve failures and any exceptional repair authorship honestly in the dogfood tally.

Check JavaScript syntax after changing a module the driver imports, before other agents start a fresh driver. Helper modules must guard executable entry points so importing them cannot apply a draft twice. If an unfinished repair prevents boot, temporarily supply only that module from a known-good committed source through an in-memory Node loader, ask Formal AI to restore the reviewed source, and verify syntax plus an unmasked fresh-process retry. The loader is bootstrap infrastructure; all repository mutations still go through Formal AI. Keep the failed requests and actual tool effects.

## Enforce execution authority before planning effects

Give every independent whole ask an explicit scratch directory and only the adapters it needs. Use `drive(..., {tools, allowedCommands: []})` when Bash is unnecessary. For a reviewed execution step, supply an exact allowlist containing only the inspected command. The maintained JavaScript driver copies the policy before planning, snapshots call arguments, and refuses the complete batch before any effect when a Bash command is unauthorized. The same guard applies to fallthrough plans; retain denied calls and original failures.

Do not rely on prompt instructions or an outer success marker as an execution boundary. Actual T4326 overwrote the version helper, wrote an unauthorized test, invoked local rustc and ran its produced binary despite scratch-only instructions. Treat that original ask as a failure. T2817 restored the exact original source and removed only the independently frozen unauthorized files through Formal AI; full source, binary bytes and tool trace are retained. No autonomous coding credit applies. The later T2827 independent guard-authoring ask also failed; the separately supplied generic driver repair passed22 original and8 new physical controls.

An exact command allowlist authorizes that command; it is not an operating-system sandbox. Review the allowed program and its child commands, and verify actual repository scope, process status and source identities. Never run cargo, rustc, rust-script or generated native binaries locally. Leave native compilation and execution to CI.

## Commits and push barriers

Create many small, coherent commits as completed pieces become reviewable. Serialize commits with the shared atomic directory lock `/private/tmp/formal-ai-pr1188-commit.lock`: acquire it before reading HEAD or creating the private index, stage only owned files, confirm HEAD still equals the captured base, commit, refresh the default index and release the lock. A private index based on an older HEAD can otherwise revert another agent's committed work. Do not push each individual change.

The coordinator pushes only at one of two barriers: all contributing subagents have completed their current bulk batch, or every fix in the current CI/CD failure batch is fully committed. Before a push, inspect the complete committed batch and make required generated records current. Freeze the finite committed cohort before the barrier; exclude every incomplete next cohort from staging and pushing, even while its drafts continue. New unrelated work can remain in the working tree.

After pushing, monitor the new remote head. Cancel superseded runs where appropriate, fix every new failure and repeat the barrier. Keep a CI monitor and a Formal AI delivery agent active while the coordinator continues work. Do not merge this pull request merely because local tests pass.

## Evidence, cost and completion

Record delegated tasks, Formal AI tool calls, verified successes, failures, general repairs and successful retries. Distinguish tasks planned by Formal AI from reviewed literal substitutions it merely applies. Never claim independent feature synthesis from a literal patch. Supplied patches, generators and reviewed transformations receive zero autonomous authorship or amplification credit, even when Formal AI executes them.

Delegating more execution and planning to Formal AI should reduce work performed by coordinating models. Measure that trend through recorded task outcomes and model usage where available; do not invent token savings or monetary costs from tool-call counts.

For coding and self-coding, prefer concise goals that yield larger useful validated net code changes. Keep deletion and short repairs valuable, and impose no output-growth floor on mathematics. Follow [compact inputs and useful coding work](coding-amplification.md): record all retries, source reads, reviewed patches, exact accepted changes and actual model usage separately; reject padding and exclude literal patch replay from autonomous amplification. Declare the task category before execution: feature/test construction and detailed proof/explanation target expansion; decisions/calculations/extraction/summaries target compression; repair/refactoring/deletion/translation have unrestricted size. Report exceptions and a complete cohort; the near-universal improvement objective is95% accepted matching tasks over at least20 attempts per category. The initial CI checks verify accounting, not a claimed autonomous success baseline.

Completion requires every planned requirement to be delivered, all required checks green on the exact remote head and production artifacts verified. Exercise Docker, CLI, server, browser, desktop and package release paths, including the actual triggers and credentials. Keep post-merge-only observations explicitly pending until they can be observed.

The owner supplies the requirements through issues, the pull request and the repository. Work autonomously, resolve routine choices, record decisions and do not ask the owner questions.
