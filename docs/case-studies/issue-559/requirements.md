# Issue 559 Requirements

These requirements are derived from the issue body, existing repo direction, and the first-session planning constraint.

## Planning Requirements

R1. Create a detailed plan before implementation.

- Acceptance: this case study captures requirements, architecture inventory, research, solution options, and phased implementation steps.
- Source: issue statement asks the first working session to be detailed planning only.

R2. Collect issue and PR data under `docs/case-studies/issue-559`.

- Acceptance: raw GitHub issue, PR, comments, reviews, code search, related PR search, image, and online research are stored locally.
- Source: issue data collection requirement.

R3. Read and document the current architecture before changing it.

- Acceptance: [architecture-inventory.md](architecture-inventory.md) names the relevant docs, source files, data files, and tests.
- Source: issue statement asks to read most docs/code and document architecture before planning.

R4. Research existing components and libraries.

- Acceptance: [raw-data/online-research.md](raw-data/online-research.md) summarizes relevant papers and current framework docs.
- Source: issue statement asks for online research and component/library checks.

## Product Requirements

R5. Replace hardcoded specific intents with a general meta algorithm.

- Acceptance: future implementation routes every prompt through a general problem frame and data-described method registry before selecting any specialized execution path.

R6. Translate each message into the meta language.

- Acceptance: every user message produces a formal `ProblemFrame` or equivalent Links Notation object with impulse, needs, constraints, candidates, validation plan, selected method, and evidence policy.

R7. Detect all questions, requirements, and needs in a prompt.

- Acceptance: the frame records multiple need types from the same prompt instead of collapsing the prompt to one handler intent.

R8. Address every detected need in the response.

- Acceptance: final answer composition references the frame's detected needs and records satisfied, deferred, blocked, or intentionally rejected status for each need.

R9. Use todo/task planning for big tasks in agentic mode.

- Acceptance: large or multi-step agentic tasks instantiate a task graph or todo plan with progress events before execution.

R10. Give meaningful chat-mode answers with fresh internet data when needed.

- Acceptance: chat-mode frames contain an evidence policy that can require fresh external data for time-sensitive, factual, or recommendation-like questions.

R11. Merge specific algorithms into one general meta algorithm.

- Acceptance: existing recipes, handlers, and algorithms become methods callable from one solver frame rather than separate top-level control flows.

R12. Support future self-reasoning and self-modification of the algorithm.

- Acceptance: algorithm changes are proposed as data/rule changes, gated by tests, benchmarks, and human review.

## Compatibility Requirements

R13. Preserve backward compatibility.

- Acceptance: existing behavior tests continue to pass and migration tests prove parity for representative prompts from each specialized handler family.

R14. Add tests rather than rewriting existing behavior coverage.

- Acceptance: new tests cover classes of prompts and routing behavior; existing tests remain useful.

R15. Preserve cache, overrides, meanings, and `.lino` architecture.

- Acceptance: new method metadata and problem frames reuse the existing data architecture instead of replacing it with an unrelated framework.

R16. Keep Rust and browser worker parity.

- Acceptance: mirrored worker behavior remains covered when routing or formalization logic changes.

R17. Keep changes reviewable in one PR through steps.

- Acceptance: each phase can be committed and reviewed independently, with tests and PR body updates after each meaningful step.

## Non-Goals For This First Session

NG1. Do not implement the architecture rewrite before the planning artifact is reviewed.

NG2. Do not remove specialized handlers immediately.

NG3. Do not replace the Rust solver with an external Python orchestration framework.

NG4. Do not silently enable self-modification of solver behavior.
