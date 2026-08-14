## Issue #80 Software Project Request Requirements

Issue [#80](https://github.com/link-assistant/formal-ai/issues/80) reports that
an Owlbear/D&D extension request for HP, Protection/Resistance mitigation, and
cooldown tracking returned `intent: unknown`. The maintainer asked for a
generalization across similar tasks instead of a memoized prompt answer, and
then clarified that the first step must formalize the message into Links
Notation meaning before deriving reasoning and plan steps.

| ID | Requirement | Status |
| --- | --- | --- |
| R156 | Open-ended requests to build or write a software artifact (extension, plugin, bot, app, service, website, or tool) must route to a typed software-project intent instead of `unknown`. | Implemented by `try_software_project_request` in `src/solver_handlers/software_project.rs`, mirrored by `trySoftwareProjectRequest` in `src/web/formal_ai_worker.js`, and registered in `data/seed/intent-routing.lino`. |
| R157 | The first response must formalize the user text into a reviewable Links Notation meaning record before producing project steps. | Implemented by the software-project meaning parser and `software_project_request` Links Notation block rendered in Rust and browser responses. |
| R158 | The meaning record must model the task as a graph of requirements and subtasks, grouped by general categories instead of by one-off memorized prompts. | Implemented by requirement extraction, category assignment, subtask derivation, `Requirement model`, and `Subtasks` sections in Rust and browser responses. |
| R159 | The response must derive deep reasoning steps and a detailed implementation plan from the meaning record, then wait for user approval before converting the plan into code, scripts, manual instructions, or execution steps. | Implemented by the plan response asking for `approve plan`, by the approval follow-up handler that uses the prior user request and assistant plan, and by explicit delivery-mode/approval-gate fields. |
| R160 | Game-unit tracker requests must receive a concrete implementation starter for HP, Protection, Resistance, damage mitigation, and cooldown ticking only after the user approves the plan. | Implemented by the game-unit tracker approval branch, which emits a TypeScript `UnitState` core with `mitigateDamage`, `setStacks`, and `tickCooldowns`. |
| R161 | Tests must show at least 20 full dialogue examples, including popular programming-task requests, so reviewers can inspect the text -> meaning -> reasoning -> plan -> approval flow. | Covered by the `software_project_dialogue_examples_formalize_plan_then_implement_after_approval` regression matrix and the dedicated software-project prompt matrix. |
| R162 | The browser demo must handle the reported Owlbear prompt and generalized software-project prompts without showing the unknown-intent fallback. | Covered by `software_project_request_returns_reviewable_plan`, `software_project_variations_do_not_return_unknown`, the full dialogue matrix, and the Playwright regression `Owlbear extension request returns a software project plan`. |
| R163 | Software-task formalization must expose user autonomy preferences: manual instructions, code generation, script generation, immediate execution intent, and approval gates for task formalization, requirements, plan, code, each step, and shell/WebVM commands. | Implemented by `delivery_mode` and `approval_gate` fields in the Links Notation meaning record plus matching reasoning and plan text. |
| R164 | Approval follow-up must generate language-aware starter code where the prompt names a common programming language, and tests must assert the generated code surface. | Implemented by TypeScript, JavaScript, Python, and Rust starter cores selected from the formalized meaning and checked by unit regressions. |
