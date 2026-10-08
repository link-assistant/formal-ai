## Why this issue exists

PR #888 (issue #710) merged on 2026-09-16 with the maintainer instruction quoted in `docs/case-studies/issue-710/plans/README.md`: no hard-coded solutions for any test, every solution discovered dynamically from trusted sources, the goal is "to know how to get to know anything when it is needed". This issue condenses what the repository, its ledgers, the eight issue-710 plans, `REQUIREMENTS.md`, `docs/requirements-traceability.md`, `docs/meta-algorithm.md`, `docs/benchmarks.md`, the 60 open issues, the 5 open PRs and ~45 closed coding/self-coding/learning issues say is still missing, ranked as bottlenecks. Each bottleneck names its evidence, why it blocks generality, and what "fixed" means under the standing doctrine (generalization over memoization, held-out paraphrases in en/ru/hi/zh/es, honest numbers, no budgets or bypasses).

Method: three parallel audits (plans, requirements/traceability/benchmarks, GitHub history) plus direct source inspection on `main` at `be8fd3174`.

## Measured state on `main` today

| Measure | Value | Source |
| --- | --- | --- |
| HumanEval upstream, first 20 of 164 | 20/20 (v0.349.2, 2026-09-15) | `data/benchmarks/external-results.lino` |
| MBPP upstream, first 20 of 500 | 20/20 online, 18/20 cold offline | same ledger; `docs/benchmarks.md` |
| GSM8K / MATH / BIG-bench object counting / CoEdIT | 2/20, 0/20, 0/20, 0/20 (v0.347.0, 2026-09-07) | same ledger |
| SWE-bench Lite | 0/1 | same ledger |
| Curated slices (industry 13/13, paraphrases 25/25, edits 1440/1440, equations 72) | all green, all self-authored | `docs/benchmarks.md` |
| Coding ladder over own issues (#848) | 65/130 | `REQUIREMENTS.md` R848 |
| Compiling ladder (#1085) | 15 of 32 leaves changed, no level fully passing | `REQUIREMENTS.md` R1085-9 |
| Self-hosting share | 0.15 % (58 of 38,373 behavior-changing lines) | plan 04 L26; `data/meta/self-hosting-ledger.lino` |
| Formal AI PR opened from a real `solve` run, green without human edits (R1021-22) | not achieved | traceability line 800 |
| Specialized handler files still pending migration to meta-methods | 37 files / 48 entries (#699 → #959) | plan 07; #959 |
| Allowlisted hard-coded prose literals | 1,286 | plan 07; `scripts/check-hardcoded-language.rs` |
| Unit tests | 3,542 pass, 494 source-placement, 378 integration | plan 07 |

The 20/20 coding rows are real and no longer come from per-task bodies (the three memorized HumanEval/MBPP functions were deleted in plan 03 L8, and a no-memorization gate scans `src/` and `data/seed/`). They are also the only upstream evidence that exists, on the first 20 tasks of each suite, and the plans themselves label it "finite slice evidence only" (plan 05 header, plan 06 first section).

## Bottlenecks, ranked by how much generality they block

### B1. The "understand each word" step has no live lookup in the coding path

- `src/coding/concept_discovery.rs` defines the `UnknownConceptLookup` trait, but the only implementation in the tree is `NoLookup` (line 87), and the production entry `discover()` (line 189) calls it. Nothing in `src/coding/`, `src/algorithm_discovery.rs`, `src/coding_research_learning.rs` or `src/solver_synthesis.rs` references Wiktionary, Wikipedia, WordNet, Stack Exchange or wikiHow, although `data/seed/sources-registry.lino` already declares 13 trusted source kinds with licenses, APIs and cache paths, and the how-to handler (#991) consults them live.
- The universal solver's own external-search step does no retrieval at all: `record_external_search` in `src/solver.rs:874` appends `policy:no_fetch_capability` and returns. Retrieval exists only inside specific handlers (how-to, coding function catalogs, research learning), never in the universal loop that every prompt passes through.
- Consequence: a coding task whose requirement uses a word outside the 82 seeded structural meanings in `data/seed/meanings-coding-structure.lino` cannot be understood, so it cannot be composed, so it fails before any algorithm search begins.
- Fixed means: one `UnknownConceptLookup` implementation that walks the sources registry (dictionary → encyclopedia → Q&A → docs), is bounded by depth and evidence rather than by a budget, caches content-addressed, and is used by the universal loop as well as the coding path. Held-out proof: a task with a word absent from every seed file resolves by lookup in all five languages.

### B2. Composition is a closed template catalog, not reconstruction from sources

- `src/coding/catalog/templates_core.rs`, `templates_extended.rs`, `templates_listing.rs`, `templates_stdin.rs` and `blueprint_programs.rs` hold roughly 150 named per-language programs (`factorial`, `fibonacci`, `fizzbuzz`, `reverse_string`, `sum_to_ten`, …); `src/knowledge.rs` `CodingOracle` still embeds 25 static snippets that plan 02 §2 calls "a cache with no way to be refilled"; `count_to_three` remains a fixed catalog program (plan 03 L12); `solver_search.rs` reaches arithmetic only (plan 01 A5).
- The structural families that lifted HumanEval and MBPP to 20/20 (expression algebra, scans, ordering, regex, grid-DAG, OEIS recurrence route; plan 05) are seeded idioms in Rust and `.lino`, selected by matching the requirement against the 82 structural meanings. They generalize across paraphrases of the same shape, not to shapes nobody seeded. Tasks 21–164 and 21–500 have never been run.
- Fixed means: the composer takes its parts from what B1 and the function catalogs retrieved (Python docs, Wikifunctions, OEIS, Rosetta, Stack Exchange accepted answers), reconstructs the step list from the retrieved procedure text, and the seeded idioms shrink to a bootstrap that can be deleted and rediscovered. Proof: `forget` the idiom seed, rediscover it online, replay offline, same content hash; then run the full 164 and 500 upstream suites and record the honest number, whatever it is.

### B3. There is no repository-grounded coding loop for real tasks

- The SWE-bench case builder (`src/external_benchmarks/cases.rs:152-166`) turns an instance into a single prompt, "Resolve this issue and reply with the fix as a unified diff patch", with no clone, checkout or base-commit step anywhere in `src/external_benchmarks/`. The solver never sees the repository, so 0/1 is structural, not a near miss.
- On its own repository: an open-ended refactor request produced a byte-identical file (plan 07 "Live source-edit probe"); an ordinary regression-authoring request read a nonexistent file and stopped (plan 06); the only attributed Formal AI commits are a 32-line `.lino` recipe, an index file and a two-line identifier rename. `solve --model formal-ai` is still not an authoring path (#1085 → hive-mind #2229), the #848 ladder is 65/130, the #1085 ladder has no fully passing level, and R1021-22 is "not achieved".
- Fixed means: one workspace protocol shared by SWE-bench, the #848 ladder and self-coding: clone at base commit, locate the files the requirement names (via the existing self-AST census for the own repo, via search for foreign repos), read, edit, run the named tests, produce the diff. Proof: SWE-bench Lite slice grows beyond 1 with an honest score, and one merged PR per release is authored by a real `solve` run.

### B4. Formalization is shallow: sentences are preserved, concepts and procedures are not extracted

- Memory-contract probes yield "zero concepts or procedures" and 2 of 9 protocol primitives (plan 06 "Remaining reconstruction edge"; plan 07 "Observed limits": "Deep formalization therefore remains an open prerequisite itself"). `coding_research_learning` accepts only a narrow typed procedure format. Unknown unquoted sources still reach a legacy fallback that once matched `рыбак`/`fisherman` to a cached fairy tale (plan 07).
- The agentic-coding recipe in `docs/meta-algorithm.md` (lines 186-262) recognizes tasks "against a small closed keyword set" and pins `SEARCH_QUERY`, `CANONICAL_SOURCE_URL`, `KB_PATH` as constants; the manual audit (traceability line 376) records that it "falls back to the seeded fairy-tale KB rather than reflecting custom --task".
- Consequence: the maintainer's recursion, "formalization itself may need recursive knowledge collection from trusted sources", cannot start because the formalizer has no way to say which concept it lacks and ask B1 for it.
- Fixed means: the formalizer emits an explicit need for every unresolved concept, B1 satisfies it, and the resulting links are concepts and procedures (not stored sentences) that the composer in B2 can consume. Proof: an unfamiliar requirement in en/ru/hi/zh/es formalizes to the same concept graph.

### B5. Obligations are marked satisfied by route selection, not by runtime evidence

- `NeedLedger` now distinguishes `Planned` from `Satisfied` (plan 07), but `task_obligations` still discards clauses without a recognized artifact, routes are marked satisfied without runtime evidence, and `REQUIREMENTS.md` R340-R344 states "a selected method is planned, not satisfied" and "runtime per-need verification feedback is still open".
- Consequence: the meta algorithm cannot tell a finished task from a routed one, so it cannot learn from its own failures or split a task further when a leaf did not actually run.
- Fixed means: every obligation node carries an execution record (command, exit code, observed output) before it may be satisfied, and an unsatisfied node triggers decomposition rather than completion prose.

### B6. Prerequisite and environment discovery is absent

- Kotlin sessions end at `kotlinc: command not found`, Scala CI at `scalac: not found`; "automatic setup discovery" is the open row of plan 07 (6 of 7 boxes unchecked). No docker execution pipeline exists for compile-before-answer (#930) or per-conversation containers with snapshot/replay (#937); both have nothing delivered. The browser has no Python runtime, so browser answers stay unverified (plan 02 §7).
- Fixed means: a failure such as "command not found" is itself a requirement fed back into B1/B4 (search the trusted publisher for the install procedure, install workspace-scoped, retry), with the same forget-and-rediscover proof as procedures.

### B7. Every learning loop is proposal-only, so nothing learned changes later behavior

- Self-improvement is gated off and never writes the recipe back (`docs/meta-algorithm.md` 137-141); method learning is "infer, then withhold" (#922); promotion is `--apply --confirm` and never pushes (#656); #364, #558, #701 each narrowed to one knowledge class; anticipatory dreaming (#705) is implemented in PR #887 but that PR has been conflicting since 2026-08-01. End-to-end automatic source-cache reconstruction is open (R710-R8), and forget-and-rediscover is proven only for the discovered-procedure ledger's content id.
- Fixed means: a learned item must demonstrably change the next answer (the #701 criterion) across the whole method registry, with the human gate applied at review time rather than by inertness.

### B8. Non-coding reasoning suites sit at the floor with no derivation path

- GSM8K 2/20, MATH 0/20, object counting 0/20, CoEdIT 0/20, plus 7 recorded loud-failure gaps in the equation corpus (irrational and complex roots, units, `Find x:` misrouted). These rows date from 2026-09-07 and were not re-measured after the discovery path landed.
- These are the same meta algorithm applied to word problems: understand each word, retrieve the method, reconstruct the steps, execute, verify. Their floor scores show the pipeline in B1–B5 is currently wired only to Python-function synthesis.
- Fixed means: the discovery path is invoked for any task with a verifiable expectation, not only for `CodingTaskSpec`, and the scheduled ledger records the new numbers.

### B9. The solver still runs on specialized Rust handlers rather than memory plus the meta algorithm

- 37 handler files / 48 entries pending migration (#699 → #959); 45 handler sources outside the minimal core under an 18,466-line ceiling; 1,286 allowlisted hard-coded prose literals; 18 hard-coded promotion predicates (#959); canned summaries and idiom handlers (#948); `VISION.md` "Current Direction" admits the solver "never read the doublets store"; the links network as the executable system of record is plan 01 C7, deferred.
- Consequence: each new capability tends to arrive as another handler, which is exactly the memoization the doctrine forbids, and the handlers are what the meta algorithm would have to modify to self-improve (B3).
- Fixed means: the ratchet in #959 turns strictly downward each release, and new capability from B1–B8 lands as seed data and registry methods, never as `try_*` arms.

### B10. Intent routing misroutes block capabilities that already exist

- #745 and #758 were re-measured still-broken by the maintainer on #710 (ladder 8/24; `src/seed/roles/intent.rs` has 15+ web-search roles and no local-search counterpart); the frontier queue #1087 (#720, #721, #722, #724, #869, #1063, #447) has nothing delivered.
- Fixed means: capability routing scored on 10–20 held-out paraphrases per intent in four languages with zero cross-tool misroutes, as #745 originally asked.

### B11. The record of truth disagrees with itself, which starves self-improvement of honest input

- `docs/requirements-traceability.md` has no rows at all for R710-R1..R10, R873, R919, R922, R924, R991 and R1085-2/3/11; R914-8 and R914-9 read "not delivered" while `REQUIREMENTS.md` 1945-1946 calls them implemented; the R67 row still describes the fabricated `example.org` provenance that #843 removed; `ROADMAP.md` lines 492 and 570 still say HumanEval/MBPP are 0/20; `VISION.md` cites only the curated 13/13 (#958); plan 01 B9 says composition is partial while plan 03 L7 says done; plan 04's 31/0/1 tally was never amended after plans 06–07 reopened items; #710 remains open although PR #888's title says it closes it.
- Fixed means: #1089 (render status from `data/meta` ledgers) and #958 (co-cite upstream numbers), so that a single generated table is the only place status lives.

### B12. The selection heuristics the vision names have no code

- Principle of least action (#491), TRIZ contradiction resolution (#901), 2-4-6 hypothesis search (#802) and moonshot task splitting (#453) all have zero delivered code and zero comments. These are how the meta algorithm is supposed to choose among several discovered algorithms and split a task it cannot solve directly.

### Stale open PRs

#887 (anticipatory dreaming), #652 (vision and roadmap refresh, sub-issues E35–E55) and #644 (small-model formalization fallback) are all conflicting and untouched since July or August; each either needs a rebase or a close with its requirement re-filed.

## Proposed order of attack

Generalization first, so each later step consumes the previous one rather than a new seed:

1. B1 live concept lookup through the sources registry, in the universal loop.
2. B4 formalizer that emits needs and stores concepts and procedures.
3. B2 composer rebuilt over retrieved parts; shrink the template catalog; run the full upstream suites.
4. B5 obligation nodes require execution evidence.
5. B6 prerequisite discovery as a requirement fed back through B1/B4.
6. B3 shared workspace protocol for SWE-bench, the #848 ladder and self-coding.
7. B7 learning loops that change behavior; rebase or re-file #887.
8. B8 route every verifiable task, not only coding, through the discovery path; re-measure GSM8K/MATH/CoEdIT/object counting.
9. B9, B10, B11, B12 as continuous ratchets alongside, with #959, #1087, #1089, #958 as the existing trackers.

## Related

#710, #651, #453, #491, #705, #930, #937, #939, #948, #955, #958, #959, #1087, #1089, #483, #901, #802, #1071, #843, #848, #873, #919, #922, #924, #1021, #1085; PRs #888, #887, #652, #644.

