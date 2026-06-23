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

Source: Graph of Thoughts, "Solving Elaborate Problems with Large Language Models"
URL: <https://arxiv.org/abs/2308.09687>

Relevant idea:

- Model intermediate thoughts as graph vertices with dependencies.
- Allow aggregation, refinement, and feedback loops beyond a simple linear chain.

Fit for this repo:

- Large tasks should use a task graph rather than a flat handler selection.
- The existing decomposition and synthesis pieces can evolve toward a graph-shaped problem frame.

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
URL: <https://arxiv.org/abs/2305.16291>

Relevant idea:

- Maintain a growing library of executable skills.
- Improve programs using environment feedback, execution errors, and self-verification.

Fit for this repo:

- Supports a `.lino` method registry plus existing skill compiler and source cache.
- Useful precedent for storing reusable methods, but the repo should keep human-gated algorithm changes.

## Programmatic LM Pipelines

Source: DSPy, "Compiling Declarative Language Model Calls into Self-Improving Pipelines"
URL: <https://arxiv.org/abs/2310.03714>

Relevant idea:

- Express LM workflows as composable declarative modules and optimize against a metric.

Fit for this repo:

- The repo can borrow the separation between declarations and compiled execution.
- Direct adoption is not recommended because this project already has a Rust and Links Notation runtime.

## Agent Orchestration Frameworks

Source: LangGraph documentation
URL: <https://docs.langchain.com/oss/python/langgraph/overview>

Relevant idea:

- Long-running stateful agents benefit from explicit graph state, durable execution, memory, human-in-the-loop, and observability.

Fit for this repo:

- Good architectural comparison for task graph state and checkpointing.
- Direct adoption is not recommended as core because this repo needs a Rust-native, offline-capable, Links Notation oriented solver.

Source: Microsoft Agent Framework overview
URL: <https://learn.microsoft.com/en-us/agent-framework/overview/>

Relevant idea:

- Distinguish open-ended agents from explicit workflows.
- Combine agent abstractions with type safety, middleware, telemetry, state, and graph workflows.

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
3. Big tasks should become graph-shaped tasks with validation edges.
4. Fresh-data needs should be represented as an evidence policy, not as separate ad hoc handlers.
5. Self-modification should use a proposal, verification, benchmark, and review gate.
