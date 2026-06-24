# Issue 559 Online Research

Accessed: 2026-06-23.

This file records external research used for planning. The goal is not to import another framework directly, but to identify proven concepts that fit the repo's Rust, Links Notation, offline-capable, data-driven architecture.

## Reasoning And Acting Loops

Source: ReAct, "Synergizing Reasoning and Acting in Language Models"
URL: <https://arxiv.org/abs/2210.03629>

Relevant idea:

- Interleave reasoning state with actions against external tools or environments.
- Use observations from actions to update the plan.
- Improve interpretability by recording reasoning and action traces.

Fit for this repo:

- The current event log already records solver phases.
- Issue 559 should make "reason, act, observe, update" a general method loop rather than a behavior embedded in individual handlers.

## Search Over Candidate Solution States

Source: Tree of Thoughts, "Deliberate Problem Solving with Large Language Models"
URL: <https://arxiv.org/abs/2305.10601>

Relevant idea:

- Treat intermediate reasoning units as searchable states.
- Generate, score, keep, backtrack, or expand candidate paths.

Fit for this repo:

- The current solver already has candidate and validation events.
- A general problem frame can store candidate methods and validation scores in one format.

Source: GoT paper, "Solving Elaborate Problems with Large Language Models"
URL: <https://arxiv.org/abs/2308.09687>

Relevant idea:

- Model intermediate thoughts as linked states with dependencies.
- Allow aggregation, refinement, and feedback loops beyond a simple linear chain.

Fit for this repo:

- Large tasks should use a recursive task-link network rather than a flat handler selection.
- The existing decomposition and synthesis pieces can evolve toward a linked problem frame.

## Reflection, Feedback, And Learning

Source: Reflexion, "Language Agents with Verbal Reinforcement Learning"
URL: <https://arxiv.org/abs/2303.11366>

Relevant idea:

- Use task feedback to write reflections into memory instead of updating model weights.
- Reuse reflections on later attempts.

Fit for this repo:

- Matches the event log, source cache, memory, and self-improvement designs.
- Supports review-gated learning from failed unknown traces.

Source: Self-Refine, "Iterative Refinement with Self-Feedback"
URL: <https://arxiv.org/abs/2303.17651>

Relevant idea:

- Generate an initial answer, critique it, refine it, and repeat without extra training.

Fit for this repo:

- Can map to candidate -> validation -> simplification -> answer projection.
- Should be bounded and test-driven to avoid nondeterministic loops.

Source: Voyager, "An Open-Ended Embodied Agent with Large Language Models"
URL: <https://voyager.minedojo.org/> and <https://arxiv.org/abs/2305.16291>

Relevant idea:

- Maintain a growing library of executable skills.
- Improve programs using environment feedback, execution errors, and self-verification.
- Use an automatic curriculum to select useful next tasks for open-ended exploration.
- Store complex behaviors as code so they are interpretable, compositional, and reusable.

Fit for this repo:

- Supports a `.lino` method registry plus existing skill compiler and source cache.
- Useful precedent for storing reusable methods, but the repo should keep human-gated algorithm changes.
- Do not import Voyager as a runtime dependency: it is GPT-4-driven, embodied, and Minecraft-specific. The useful part is the architecture pattern.
- Translate automatic curriculum to a deterministic "novelty and coverage backlog" over unknown traces, benchmark gaps, dependency gaps, and unhandled prompt families.
- Translate the skill library to a link-native method/skill registry whose entries can point to Rust functions, standard-library calls, `.lino` rules, generated examples, and validated snippets.
- Translate iterative prompting to `execute -> observe -> validate -> patch candidate -> retry within budget`, using test results and source/evidence checks rather than neural self-critique.
- Translate self-verification to a critic method that checks the work unit's validation plan and marks it satisfied, blocked, or needing another recursive split.

## Link-Native Foundations

Source: link-foundation/meta-theory, "The Links Theory 0.0.2"
URL: <https://raw.githubusercontent.com/link-foundation/meta-theory/main/archive/0.0.2/article.md>

Relevant idea:

- Links theory reduces the data model to links; a point-like object is a self-referential link and a connection is also a link.
- Doublet links can represent objects, properties, relationships, sequences, and sentences.
- The article frames a link as recursively linking links.

Fit for this repo:

- The general solver should model problem frames, needs, evidence, work units, dependencies, methods, validation results, files, sequences, and algorithms as links.
- External literature may use other structural terms, but the formal-ai design should translate those into link networks.
- A recursive work unit should be a link with links to parent unit, child units, selected method, evidence, constraints, and validation result.

Source: LinksPlatform organization overview
URL: <https://github.com/linksplatform>

Relevant idea:

- LinksPlatform presents a modular framework for automation of automation.
- It treats algorithms as data in storage and describes a direction where programs can be created or edited from human-language descriptions.

Fit for this repo:

- Issue 559 should keep the long-term target of algorithm-as-data: the general algorithm is not just Rust control flow; it must be representable, inspectable, testable, and eventually editable as link data.
- Self-modification must stay review-gated in formal-ai, matching the repository's existing self-improvement loop.

Source: link-foundation/meta-language README
URL: <https://github.com/link-foundation/meta-language>

Relevant idea:

- `meta-language` advertises mutable link networks, source spans, lossless parse/reconstruction, generated source rendering, snapshots, query/replace, substitution, LiNo parsing, concept mappings, and cross-language reconstruction.
- It exposes storage-backed link stores and read-only/mutable access modes.

Fit for this repo:

- The planned `ProblemFrame`, `WorkUnit`, and method/skill registry can be represented in Links Notation now and later mapped into `meta-language` networks.
- Existing features are enough for the next phases: no upstream meta-language blocker is required before adding behavior-preserving frame and registry tests.

Source: Associative Model of Data paper archive
URL: <https://web.archive.org/web/20181219134621/http://sentences.com/docs/amd.pdf>

Relevant idea:

- The associative model is one of the historical sources behind links-style storage and link-centric data modeling.

Fit for this repo:

- It supports the plan's emphasis on a normalized associative memory layer rather than separate ad hoc structures for tasks, facts, methods, and evidence.
- The implementation should still be pragmatic: formal-ai can start with Rust structs plus `.lino` snapshots, then migrate more of the state into link-native stores as tests stabilize.

## Programmatic LM Pipelines

Source: DSPy, "Compiling Declarative Language Model Calls into Self-Improving Pipelines"
URL: <https://arxiv.org/abs/2310.03714>

Relevant idea:

- Express LM workflows as composable declarative modules and optimize against a metric.

Fit for this repo:

- The repo can borrow the separation between declarations and compiled execution.
- Direct adoption is not recommended because this project already has a Rust and Links Notation runtime.

## Agent Orchestration Frameworks

Source: LangChain stateful-agent documentation
URL: <https://docs.langchain.com/oss/python/langgraph/overview>

Relevant idea:

- Long-running stateful agents benefit from explicit state, durable execution, memory, human-in-the-loop, and observability.

Fit for this repo:

- Good architectural comparison for task-link state and checkpointing.
- Direct adoption is not recommended as core because this repo needs a Rust-native, offline-capable, Links Notation oriented solver.

Source: Microsoft Agent Framework overview
URL: <https://learn.microsoft.com/en-us/agent-framework/overview/>

Relevant idea:

- Distinguish open-ended agents from explicit workflows.
- Combine agent abstractions with type safety, middleware, telemetry, state, and explicit workflows.

Fit for this repo:

- Supports the plan to separate open-ended chat from explicit workflow execution through an evidence and control policy in the problem frame.

Source: AutoGen repository and docs
URL: <https://github.com/microsoft/autogen>

Relevant idea:

- Multi-agent systems can be built from composable conversable agents and tools.
- As of the accessed date, the repository README marks AutoGen as maintenance mode and directs new users to Microsoft Agent Framework.

Fit for this repo:

- Useful for multi-agent design patterns and benchmarking ideas.
- Not a recommended dependency for core behavior, especially because the current repo has its own solver, trace model, and data constraints.

## Planning Conclusions

1. A general solver should store problem state explicitly, not rely on one-shot intent routing.
2. Method selection should be data-described and validated, with current handlers preserved as executable methods during migration.
3. Big tasks should become recursive task-link networks with validation links.
4. Fresh-data needs should be represented as an evidence policy, not as separate ad hoc handlers.
5. Self-modification should use a proposal, verification, benchmark, and review gate.
6. Voyager strengthens the plan only if translated into deterministic formal-ai mechanisms: curriculum as coverage backlog, skills as registry entries, execution feedback as tests/tool observations, and self-verification as critic methods.
7. The link/meta-theory sources require the plan to model point-like and relation-like structures as links and to keep algorithm-as-data as an explicit migration target.
