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

- Acceptance: large or multi-step agentic tasks instantiate a task-link network or todo plan with progress events before execution.

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

R18. Integrate Voyager into the plan as a design reference, not as a neural dependency.

- Acceptance: the plan maps Voyager's automatic curriculum, skill library, execution feedback, and critic/self-verification ideas onto deterministic formal-ai mechanisms.
- Source: PR feedback on 2026-06-23 asked to check <https://voyager.minedojo.org> while explicitly avoiding neural-network dependence.

R19. Make the core problem-solving algorithm fully recursive.

- Acceptance: every non-atomic problem frame can recursively create smaller work units until a unit is small enough for a direct method call, standard-library call, existing Rust function, or newly reviewed skill.
- Source: PR feedback asked for recursive decomposition where small tasks become single-line or single-call solutions.

R20. Support both decomposition-first and construction-first reasoning.

- Acceptance: the plan includes one path that splits an original task into subproblems and another path that searches available components, libraries, cached facts, and skills to compose a solution upward.
- Source: PR feedback asks to go in both directions at the same time.

R21. Treat reusable skills as first-class, accumulated artifacts.

- Acceptance: the method registry can record existing handlers, Rust standard-library functions, repo functions, generated functions, examples, experiments, validation status, and reuse conditions as skills.
- Source: PR feedback compares Voyager's generated skill library with Rust and other standard libraries plus accumulated functions.

R22. Add a general evidence pipeline for fresh external data.

- Acceptance: the plan covers term/phrase/sentence/question search, multiple search providers, reranking, crawling, source extraction, contradiction checks, and hypothesis formation when the evidence policy requires fresh data.
- Source: PR feedback asks to search each term, phrase, sentence, and question online when needed and use the desktop web-search dependency.

R23. Stay link-native and avoid making a separate non-link ontology the core model.

- Acceptance: docs describe task, dependency, method, evidence, object, sequence, file, and algorithm structures as links or link networks. When external papers use other structural terminology, the plan explicitly translates it to links.
- Source: PR feedback asks to stay true to <https://github.com/link-foundation/meta-theory> where point-like and relation-like structures are represented as links.

R24. Treat algorithms as data that can be translated to code and back.

- Acceptance: the plan describes how meta-algorithm records, method registry entries, work units, validation policies, and generated patches become Links Notation or meta-language networks with source spans, provenance, and round-trip tests.
- Source: PR feedback references LinksPlatform, meta-language, and algorithms as data.

R25. Audit upstream dependencies owned by the related organizations.

- Acceptance: the case study lists relevant `link-assistant`, `link-foundation`, and `linksplatform` dependencies, their required features, open issues, and whether they block the next implementation phases.
- Source: PR feedback asks to check all organization dependencies and pause if needed features are missing.

R26. Create and list upstream dependency issues when blockers exist.

- Acceptance: if an upstream blocker is found, a GitHub issue is created in that dependency and listed in the PR comment. If no blocker exists, the PR comment states that no new upstream issues were created and lists existing relevant upstream issues.
- Source: PR feedback asks that upstream dependency issues be created and listed in a GitHub comment.

R27. Make the plan much more concrete and actionable.

- Acceptance: the revised solution plan includes concrete data structures, migration phases, tests, pause/go gates, owner-facing review checkpoints, and dependency conditions.
- Source: PR feedback asks for a plan at least twice as detailed and focused on missing pieces in the repository.

## Non-Goals For This First Session

NG1. Do not implement the architecture rewrite before the planning artifact is reviewed.

NG2. Do not remove specialized handlers immediately.

NG3. Do not replace the Rust solver with an external Python orchestration framework.

NG4. Do not silently enable self-modification of solver behavior.

NG5. Do not depend on Voyager, GPT-4, embeddings, or any other neural model as the core runtime for the general algorithm.

NG6. Do not replace link-native data structures with a separate non-link model; external structural ideas must be translated into links.
