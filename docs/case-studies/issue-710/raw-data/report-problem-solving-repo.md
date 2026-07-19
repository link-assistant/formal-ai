# Digest: github.com/konard/problem-solving

Researched 2026-07-14. Full shallow clone at:
`/private/tmp/claude-501/-Users-konard-Code-Archive-link-assistant-formal-ai/f67d6e80-2941-4ce4-949e-6a37db9ce2d0/scratchpad/problem-solving-repo`

Repo state: no open or closed GitHub issues, no PRs (verified with `gh issue list --state all` and `gh pr list --state all` -- both empty). License: Unlicense. Content is mostly English; one document (`automation-article-draft.md`) is in Russian.

## 1. Repository inventory

| Path | Content |
|---|---|
| `README.md` | Top-level 2-step algorithm + embedded diagrams + "Progress" links to deep-foundation implementations |
| `problem-solving/problem-solving.md` / `.dot` / `.svg` | Canonical 2-step formulation (Problem Formulation <-> Solution Implementation) with feedback edge "If necessary" |
| `ttd-problem-solving.md` / `.dot` / `.svg` | 9-step TDD-flavored variant of the algorithm |
| `mapReduce.png` | Map/Reduce analogy diagram (input -> parallel map -> reduce -> output) |
| `automation-article-draft.md` | Russian-language draft: a debugging/escalation ladder for solving an unknown error |
| `js/` | Most complete "Problem Solving Automation" implementation (Orchestrator + Decomposer + TestGenerator + SolutionSearcher + Composer + GitHubClient + LLMClient, tests incl. e2e sqrt pipeline) |
| `js-poc-1` ... `js-poc-9` | Nine parallel proof-of-concept implementations of the same "Universal Algorithm" (see section 6 -- the repo itself practices parallel candidate solutions) |

Note: `problem-solving.md` links to `problem-formulation/problem-formulation.md` and `solution-implementation/solution-implementation.md`, but those sub-documents do not exist in the repo (dead links / planned content).

## 2. The core methodology: two-step universal algorithm

From `problem-solving/problem-solving.md` (and its .dot graph):

1. **Problem Formulation** -- fully understand and define the problem; define it in clear, precise terms; break it into smaller, manageable parts (decomposition); and, crucially, **formulate failing tests based on the problem definition**. The tests are the concrete goals a solution must achieve. The output of this step is "a well-defined set of failing tests that clearly express what a solution to the problem should look like."
2. **Solution Implementation** -- formulate a plan to pass the tests; write **just enough** code to pass them (anti-overengineering); then refine via debugging and refactoring without changing behavior; document the solution. Iterative: may return to step 1 if new information arises or tests must change.

Global invariant (README note): *at any step the problem definition may change or need to change for a solution to be possible or efficient; in that case restart from decomposition (the first step).* The .dot graph encodes exactly two nodes with a forward edge and a back edge labeled "If necessary".

## 3. The 9-step TDD variant (`ttd-problem-solving.md` / `.dot`)

1. **Define the Problem** -- understand it thoroughly, express in clear, concise terms (read briefs, talk to stakeholders, research).
2. **Analyze the Problem** -- break into smaller manageable parts (**decomposition**).
3. **Formulate a Plan** -- decide how to tackle each part; consider tools, algorithms, data structures.
4. **Write a Failing Test** -- the "Red" phase; the test passes when the problem is solved.
5. **Implement the Minimum Code to Pass the Test** -- the "Green" phase.
6. **Refactor** -- improve without changing behavior (readability, remove duplication).
7. **Review and Reflect** -- could it be more efficient/readable? *What can be learned and applied to future problems?* (explicit self-improvement step).
8. **Document the Solution** -- process, decisions, issues encountered, how solved.
9. **Iterate** -- return to step 4 with new/remaining problems; each cycle improves on the previous solution.

Graph: linear 1->...->9 with a back edge 9->4 labeled "If necessary".

## 4. The "Universal Problem Solving Algorithm" diagram (hand-drawn, embedded in README)

The richest artifact is the hand-drawn diagram (README attachment IMG_3275). It shows a **pipeline of four column-stages with two labeled axes**:

- Top axis (artifacts): **tasks -> tests -> drafts -> solutions**
- Bottom axis (operations): **decomposition -> experiment -> selection -> composition**

Structure of the drawing:
- **Problem** (single red node) fans out via **decomposition** into a *recursive tree of tasks* (red nodes decompose into further red nodes -- decomposition is hierarchical/recursive, not one-shot).
- Each leaf task gets a **test** (yellow node) -- the "experiment" that operationalizes the task's definition of done.
- Each test fans out into **multiple drafts** (white nodes -- 2-3 candidate implementations per test). This is the explicit generate-many-candidates step.
- **Selection** picks, per test, which draft(s) pass/are best -> they become **solutions** (green nodes).
- Selected partial solutions are merged pairwise/hierarchically via **composition** (green nodes converge) into the final single **Solution**.

A second hand-drawn diagram (IMG_1141) shows the simpler linear version: problem -> part -> test -> solution (two parallel lanes) -> composed solution. The **mapReduce.png** diagram makes the computational analogy explicit: decomposition = *map* (parallelizable, independent parts), composition = *reduce* (merging partial results). So the intended execution model is map-reduce over subproblems: subtasks can be solved **in parallel**, and results are compared/selected and reduced.

Summary of the full flow as one algorithm:

```
solve(problem):
  if problem definition unclear/wrong at any point -> reformulate & restart from decomposition
  tasks    = decompose(problem)                 # recursive; may build a tree
  for each task (parallelizable, map phase):
      test     = formulate_failing_test(task)   # experiment / definition of done
      drafts   = generate_candidate_solutions(task, test)  # several per test
      solution = select(drafts, test)           # keep the one(s) that pass / are best
  final = compose(selected partial solutions)   # reduce phase; must satisfy top-level test
  refactor, review/reflect (extract lessons), document
  iterate if needed
```

## 5. The automated implementation ("Universal Algorithm" pipeline in `js/` + POCs)

The `js/` package ("Problem Solving Automation") automates the whole methodology with an LLM + GitHub as the shared workspace/audit trail. Orchestrator flow (`js/src/orchestrator.js`, READMEs of js and js-poc-2..7):

0. **Test Repository Creation** -- create a disposable, uniquely-named repo (`problem-solving-test-<...>`) as a sandbox; delete it on success, **keep it on failure for investigation**.
1. **Task Decomposition** -- LLM breaks the main task into 3-5 (configurable, max `UNIVERSAL_ALGORITHM_MAX_SUBTASKS`) specific, actionable subtasks; a main GitHub issue plus linked sub-issues are created (issues = externalized task tree, with dependencies).
2. **Test Generation (Red)** -- for each subtask the LLM generates comprehensive failing tests (multiple cases, edge cases, assertions); each test is pushed on a branch and opened as a **test PR**. Tests are "the definition of done".
3. **Test approval gate** -- human/simulated approval of the test PR before any solution is attempted (approve the *specification* first).
4. **Solution Search (Green)** -- LLM analyzes the failing tests, generates minimal production-ready code to pass them; each attempt is a **solution PR**; retry loop up to `UNIVERSAL_ALGORITHM_MAX_SOLUTION_ATTEMPTS` (default 3) with "learning from failures" (failed attempt feedback goes into the next prompt in the fuller POCs).
5. **Solution approval gate** -- approval of solution PR; unapproved subtasks are skipped, pipeline continues with the rest (graceful partial success).
6. **Composition** -- LLM combines all approved partial solutions into one coherent final implementation ("resolve conflicts and dependencies", generate documentation), validated against the main task, opened as a final PR on the main issue.
7. **Cleanup + summary** -- success = all subtasks solved; report N/M completed.

Operational features worth copying: dry-run mode (execute pipeline without side effects), debug/verbose logging, config via env (`UA_MAX_SUBTASKS`, `UA_MAX_SOLUTION_ATTEMPTS`, `UA_ENABLE_COMPOSITION`, `UA_DRY_RUN`), exponential-backoff retry on LLM/API errors, robust JSON parsing with fallback strategies, status/interactive CLI commands (`solve`, `decompose`, `solve-subtask N`, `compose N`, `status N`, `check-config`).

### Exact LLM prompts used (js/src/llm/llmClient.js)

- **decomposeTask**: "Decompose this task into 3-5 specific, actionable GitHub-style subissues. Each subtask should be focused and implementable. Return only a JSON array of strings."
- **generateTest**: "Generate comprehensive Jest test code for this task. Include multiple test cases, edge cases, and proper assertions. Return only the test code."
- **generateSolution**: "Write production-ready code that passes this test. ... Return only the implementation code." (given task + test code)
- **composeSolutions**: "Combine these individual solutions into one coherent, production-ready implementation. Ensure the final code ... integrates all components properly." (given all partial solutions joined by `---` separators)

### Two decomposition engines (js-poc-1)

- **RuleBasedDecomposer** (`js-poc-1/src/ruleBasedDecomposer.js`) -- pure symbolic decomposition by pattern matching: numbered lists, bullet points, `Step N:` markers, `Prerequisites:`, and a "First ... then ..." sequential pattern producing prerequisite/dependent pairs; falls back to a single `atomic` task; validates decomposition (non-empty, no circular dependencies in sequential chains).
- **LLMBasedDecomposer** (`js-poc-1/src/llmBasedDecomposer.js`) -- structured-output decomposition with a Zod schema: each subtask has `description`, `type in {atomic, sequential, parallel, prerequisite}`, optional `dependencies[]` and `estimatedComplexity in {low, medium, high}`; metadata carries `confidence in [0,1]` and `reasoning`. Validation includes schema check, **cycle detection over the dependency graph** (DFS with recursion stack), and rejection when `confidence < 0.5`. On failure, falls back to treating the whole task as atomic.

The subtask **type vocabulary {atomic, sequential, parallel, prerequisite} + dependency graph + complexity + confidence + reasoning** is the closest thing in the repo to a formal task ontology.

## 6. Meta-level: the repo practices its own algorithm

There are **nine parallel proof-of-concept implementations** (`js-poc-1` ... `js-poc-9`) of the same Universal Algorithm plus the consolidated `js/` package. This is the "drafts -> selection" stage applied to the algorithm's own implementation: multiple independent candidate implementations of the same spec, compared, with the best ideas consolidated into `js/`. Each POC varies the architecture slightly (flat vs `core/` module layout, Octokit vs raw REST, Jest vs Bun test, retry loop placement, CLI shapes). This is also the repo's implicit **self-improvement / meta-algorithm** stance: the problem-solving system is itself a problem to be solved by the algorithm (the "Progress" checklist in README tracks automating each stage: task decomposition, task->function-type, (task,function-type)->test, (task,function-type)->solution, composition of solutions -- all implemented via deep-foundation, e.g. npm `@deep-foundation/chatgpt-tasks`).

The "task to function type" step (deep-foundation issue #141) is notable: before generating tests/solutions, a task is translated into a **typed function signature** -- i.e., an informal natural-language task is mapped into a formal type -- then tests and solutions are generated against (task, type). Pipeline: task -> function type -> test -> solution -> composition. That is the repo's concrete bridge from natural language to a formal system.

## 7. The debugging/escalation ladder (`automation-article-draft.md`, Russian)

A worked example (error `EMPTY_RECORDING`) written as a Discord dialogue; it defines a strict escalation ladder for solving an unknown error -- effectively a **prioritized search strategy over knowledge sources and interventions**:

1. Walk the stack trace to find *who throws the error* (locate the responsible code/file). (Caveat noted: stack traces are reliable in C#/Java, flaky in JS.)
2. Set breakpoints there; observe where the empty/invalid value comes from.
3. Search the error: Google, ChatGPT, GitHub, and your own organization's repositories (the author's explicit search order: "google, chatgpt, github, and our repositories").
4. Read the library's documentation, if any.
5. Ask the maintainers (GitHub issues / support).
6. Read the library's source code until you understand what the error means.
7. Return to your own code: determine what *you* do that triggers it; can it be avoided or prevented? E.g. pass different options/arguments; or guard with if/else so the call happens only after required initialization (recognize the function's implicit preconditions: environment objects, argument sets).
8. If the error can't be influenced from outside: **fork the library and fix it**.
9. If the fork doesn't help: **find an alternative library**.
10. If nothing helps: **write your own code** that does what the library should have done -- ideally immediately as its own library.

This encodes: root-cause localization first; cheapest-knowledge-first search; precondition inference; and a monotone escalation of intervention cost (configure -> guard -> fork -> replace -> rewrite).

## 8. Key concepts, mapped

- **Decomposition** -- recursive splitting of a problem into a task tree/graph with typed edges (sequential/parallel/prerequisite dependencies); both symbolic (regex patterns) and LLM implementations; validated (acyclicity, non-emptiness, confidence threshold). Map-phase of map-reduce.
- **Tests / experiments** -- every task is converted into failing executable tests *before* solving; tests are the machine-checkable definition of done and the selection criterion; the diagram literally labels this stage "experiment".
- **Drafts / candidates** -- multiple candidate solutions ("drafts") are generated per test; retry loop with max attempts; each attempt is preserved as a PR (audit trail of the search).
- **Selection** -- drafts are filtered by the tests (+ human approval gates for both tests and solutions); only passing/approved drafts become solutions.
- **Composition** -- partial solutions are merged (reduce phase) into the final solution, which must satisfy the top-level acceptance test; composition itself is an LLM step with conflict/dependency resolution.
- **Parallel solution attempts & comparison** -- at two levels: drafts-per-test within a run, and whole parallel implementations (js-poc-1..9) across runs, later consolidated. mapReduce.png sanctions parallel execution of independent subtasks.
- **Self-improvement** -- TDD step 7 "Review and Reflect" (extract lessons for future problems); iterative refinement loops at every level; solution-caching and reuse listed as planned enhancement; the project bootstraps itself (uses its own methodology to build its own automation).
- **Meta-algorithm / formal systems** -- the algorithm is "universal": problem-agnostic, defined over artifacts (task, test, draft, solution) and operations (decompose, experiment, select, compose); the deep-foundation pipeline formalizes tasks into typed function signatures before test/solution generation; the .dot/graph representations give the process itself a formal, machine-readable structure.
- **World models** -- not addressed explicitly; the closest analogues are (a) the sandbox test repository as a disposable "world" in which experiments run and whose state is kept for post-mortem on failure, and (b) precondition inference in the debugging ladder ("the code works only in a certain situation -- environment objects and argument sets").
- **Translation between languages** -- the repo itself is bilingual (EN methodology, RU debugging article); "Multi-language support (Python, Java, C#...)" is a stated future enhancement; the deeper translation idea is NL task -> formal function type -> tests -> code, i.e. staged translation from informal to formal representations.

## 9. Actionable capabilities for a symbolic AI system

Each item is a concrete mechanism the repo describes or implements:

1. **Two-phase solve loop**: represent every problem as (formulation, implementation) with a feedback edge; allow the problem definition itself to be revised mid-solve, triggering restart from decomposition.
2. **Recursive task decomposition** into a typed task graph: node types {atomic, sequential, parallel, prerequisite}, explicit `dependencies[]`, `estimatedComplexity`, plus `confidence` and `reasoning` metadata on each decomposition.
3. **Dual decomposer strategy**: cheap symbolic/rule-based decomposer (numbered lists, bullets, "Step N:", "Prerequisites:", "First...then..." patterns) tried alongside or before an LLM decomposer; fall back to `atomic` when neither applies.
4. **Decomposition validation**: schema check, dependency-cycle detection (DFS), non-empty check, confidence threshold (reject < 0.5); invalid decompositions are re-done, not silently used.
5. **Test-first goal encoding**: convert every (sub)task into failing executable tests *before* attempting a solution; tests are the machine-checkable success criterion ("definition of done") and the only selection oracle.
6. **Spec-approval gate before solution search**: validate/approve the *test* (the formalized goal) before spending effort on solutions -- catches misformalized goals early.
7. **Task -> formal type translation**: map each natural-language task to a typed function signature first; generate tests and solutions against (task, type) rather than raw prose.
8. **Bounded candidate search ("drafts")**: generate k candidate solutions per test; iterate with a max-attempts budget; feed failure information from each attempt into the next.
9. **Selection by oracle**: run all drafts against the tests; keep only passing (or best-scoring) drafts; support an optional human-approval gate on the selected solution.
10. **Hierarchical composition (reduce)**: merge selected partial solutions pairwise/hierarchically into a whole; treat composition as its own problem with its own top-level acceptance test; resolve conflicts/dependencies during merge.
11. **Map-reduce parallelism**: schedule independent subtasks concurrently (map), compose results (reduce); dependencies from the task graph determine what can be parallelized.
12. **Portfolio of parallel implementations**: for important problems, spawn N independent full solution attempts (the repo's own js-poc-1..9 pattern), compare them, and consolidate the best elements into a final version.
13. **Externalized, auditable search state**: persist the whole search -- task tree, tests, every draft, every selection decision -- as durable linked artifacts (issues/PRs/branches); every intermediate is reviewable and resumable (`solve-subtask N`, `compose N`, `status N` entry points).
14. **Sandboxed experiment worlds**: run each solving episode in a disposable isolated environment; destroy on success, **preserve on failure** for diagnosis.
15. **Dry-run mode**: execute the entire pipeline without side effects for verification of the plan itself.
16. **Graceful partial success**: skip failed/unapproved subtasks, continue with the rest, report N/M completion instead of failing atomically.
17. **Refactor + reflect + document steps**: after tests pass, improve the solution behavior-preservingly; extract reusable lessons ("what applies to future problems?") and store documentation of process, decisions, and encountered issues -- the hook for a self-improvement memory.
18. **Solution caching/reuse**: before searching, check for previously solved similar (sub)tasks (listed as planned enhancement -- "Solution Caching: Reuse of similar solutions").
19. **Debugging escalation ladder** (for failures with unknown cause): locate the thrower via stack trace -> observe with breakpoints where the bad value originates -> search knowledge sources in cost order (web, LLM, code hosting, own repos) -> read docs -> ask maintainers -> read source -> infer preconditions and adapt own call (options/guards) -> fork & patch dependency -> swap dependency -> reimplement. Each rung is a discrete, orderable action with increasing cost.
20. **Robustness engineering around stochastic components**: exponential-backoff retries, structured-output schemas (Zod-style) with parse-fallback strategies, configuration-validated startup (`check-config`), and configurable resource budgets (max subtasks, max attempts).
21. **Process as data**: keep the methodology itself in machine-readable form (.dot graphs of the algorithm) so the meta-level process can be inspected, executed, and improved like any other artifact.

## 10. Diagrams (textual reconstruction)

**problem-solving.dot**: `Problem Formulation -> Solution Implementation`, back edge labeled "If necessary".

**ttd-problem-solving.dot**: linear chain Define -> Analyze -> Plan -> Failing Test -> Minimal Code -> Refactor -> Review -> Document -> Iterate, with back edge Iterate -> Failing Test ("If necessary").

**Universal Problem Solving Algorithm (hand-drawn)**:
```
axes:   tasks    ->  tests      ->  drafts    ->  solutions
        decomposition  experiment    selection    composition

Problem --decompose--> task tree (recursive)
each leaf task --> failing test --> {draft1, draft2, ...}
tests select passing drafts --> partial solutions
partial solutions --compose (hierarchical merge)--> Solution
```

**mapReduce.png**: Input data -> [map, map, map] -> [reduce, reduce] -> Output data (decomposition := map, composition := reduce, subtasks independent/parallel).

**IMG_1141 (hand-drawn)**: problem -> {part -> test -> solution} x 2 lanes -> single composed solution.
