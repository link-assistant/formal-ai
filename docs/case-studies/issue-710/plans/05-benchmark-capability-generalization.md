# Benchmark capability generalization plan

Status: implemented; wider release validation remains in Plan 04 L25  
Baseline captured: 2026-09-15  
Scope: the first 20 upstream-ordered cases in HumanEval and MBPP

## Evidence and constraints

The fresh online baseline is `humaneval passed=3 failed=17 total=20` and
`mbpp passed=1 failed=19 total=20`. HumanEval cases 0, 8, and 13 pass; MBPP case
2 passes. Every other case fails before upstream grading with `answer contains
no Python code`. The first problem is therefore missing structural composition,
not a collection of incorrect task-specific programs.

The accepted implementation reaches `humaneval passed=20 failed=0 total=20`
with an empty source cache. MBPP reaches `passed=18 failed=2 total=20` with an
empty source cache and `passed=20 failed=0 total=20` with live source discovery.
The two cache-empty gaps are exactly the source-dependent 3-by-n tiling
recurrence and named-number predicate; neither is guessed or embedded. Live
discovery resolves both through official OEIS JSON, follows bounded sequence
cross-references, reduces only a small arithmetic/linear-recurrence grammar,
and admits a candidate only after public-example execution and upstream
grading. A content-addressed runtime cache makes later offline runs replayable,
but it is ignored by Git and can be deleted without removing the discovery
algorithm.

The implementation must satisfy these constraints:

- recognize operations from formalized requirement language and explicit
  examples, never benchmark ids or stored canonical solutions;
- compose programs from source-grounded reusable operations, then accept only
  candidates that parse and pass the prompt's public examples in the bounded
  execution workspace;
- keep benchmark payloads and their canonical solutions outside production
  source, seed data, and committed tests;
- make each new family usable under renamed functions and parameters and on
  held-out values or wording;
- keep all discovery bounded and deterministic, and retain source URL and
  license evidence in the selected answer;
- allow the derived procedure cache to be deleted and recreated from the
  source-backed catalog and task evidence.

Authoritative grounding should use the Python documentation already represented
by `meanings-coding-structure.lino`: built-ins and expressions, list
comprehensions, `itertools`, `math`, `collections.Counter`, regular-expression
syntax, sequence/string methods, sorting, sets, and ranges. New meanings must
carry their own authoritative URL rather than relying on this plan as evidence.

## Complete failure taxonomy

Each failed case belongs to a reusable capability family. A case may exercise
more than one family; its primary family appears first.

| Family | HumanEval failures | MBPP failures | General operation |
|---|---|---|---|
| Scalar formula composition | 2, 4 | 14, 17 | arithmetic expression, mean, absolute deviation, fractional remainder, geometric formula |
| Stateful prefix scan | 1, 3, 6, 9 | 1 | balanced-group segmentation, prefix predicate, maximum nesting, running reduction, grid-path dynamic programming |
| Sequence map/filter/interpose | 5, 7, 14, 15, 17 | 7, 8, 15, 18 | transform or retain elements, insert separators, prefixes, tokenize, map explicit legends, exclude a set |
| Ordering and bounded selection | 12, 19 | 4, 10, 12, 13 | stable extrema, semantic key ordering, top/bottom N, row-key sort, frequency ranking |
| String window/symmetry/zip | 10, 11, 18 | 9, 11, 16 | palindrome suffix, aligned binary transform, overlapping windows, rotation period, boundary occurrence removal, whole-string pattern |
| Generic predicates | 16 | 3, 6, 19 | normalized distinct count, compositeness, one-set-bit comparison, duplicate existence |
| Recurrence or finite-state definition | — | 5, 20 | coupled recurrence and named integer-sequence predicate, discovered from trusted formal sources when prose is insufficient |

### Already discovered but not yet composed

The seed already formalizes `map_each`, `filter_only`, `reduce_max`,
`reduce_min`, `running_prefix`, `case_insensitive`, `sort_ascending`,
`sort_descending`, `count_overlapping`, `split_on_space`, `join_with_space`,
`membership`, and `reduce_len`. The composer currently ignores most of these or
does not combine them, explaining HumanEval 7, 9, 15, 16, and 18 and parts of
several other cases. These are the first implementation target because no new
domain knowledge is required.

### Missing formal structures

The structural catalog still needs source-grounded meanings for arithmetic
operators and averages; prefix scans and balanced delimiters; interposition;
substring membership; prefix enumeration; zip; reversal and palindrome tests;
keyed stable extrema/sort; explicit prose lookup tables; bounded top/bottom
selection; regular-expression matching/extraction; character exclusion;
duplicate and primality predicates; bitwise one-bit tests; and dynamic
programming. Named recurrences or sequences that cannot be derived from the
prompt must go through the trusted-source discovery route rather than a seed
containing their answer.

## Implementation order

1. Complete combinations of structures already present in the seed:
   case-normalized distinct counts, substring filtering, rolling reductions,
   overlapping-window counts, split/map/join, and ascending/descending bounded
   selection.
2. Add a small source-grounded expression and collection algebra: arithmetic,
   comparisons, membership, calls, mapping, filtering, reductions, ranges,
   zipping, slicing, stable key selection, and stateful scans. Candidate schemas
   are generated from the structures found in the requirements and are chosen
   only after executable example verification.
3. Parse explicit relations from prose as data: quoted symbol-to-value legends,
   ordered vocabularies, numeric constants, delimiter characters, and regular
   expression constraints. The parser must be generic over literals and must
   not know note names, numeral words, or benchmark function names.
4. Add recurrence/dynamic-programming construction only through a formal state,
   base-case, transition, and termination representation. Reuse Wikifunctions
   recurrence discovery for named sequences; report an honest blocked need when
   trusted evidence cannot establish the definition.
5. Run no-memorization checks, held-out tests, both benchmark slices, and then
   the wider coding and web regression suites. Refresh the benchmark ledger only
   with honest reproducible results.

All five implementation stages are complete. The reusable local families are
covered by renamed-parameter, paraphrased, held-out executions in
`tests/unit/coding_discovery/structural_composition.rs`; the official-sequence
catalog has separate synthetic-source fixtures proving name-independent formula
membership, recurrence cross-reference traversal, exact provenance, and
forget/replay behavior. The upstream corpus remains confined to the ignored
benchmark download cache.

## Held-out and anti-memorization proof

Tests will use invented function names, changed parameter names, paraphrased
requirements, and values absent from both upstream slices. Each schema test must
assert the derived composition trace, authoritative source links, Python CST,
and bounded execution result. A repository scan must reject benchmark ids,
canonical solutions, copied upstream assertions, and branches on benchmark
entry-point names in production paths. At least one forget-and-rebuild test must
delete the derived procedure artifact, reconstruct it from sources, and compare
the normalized digest.

## Measurement commands

Baseline and remeasurement use the same upstream-order slice and do not append
until the implementation is accepted:

```text
formal-ai benchmark run --suite humaneval --slice 20 --online --repository-root .
formal-ai benchmark run --suite mbpp --slice 20 --online --repository-root .
```

The benchmark harness must print every failed case and retain
`benchmark_unavailable` rather than substituting a local proxy. After acceptance,
the dated run may update the monotonic ledger and failure frontier through their
explicit CLI flags.

## Finite completion criterion

This benchmark increment is complete because all of the following are true:

- all 36 baseline failures are accounted for in this taxonomy and each
  implemented family has a held-out structural test;
- both slices improve above the committed floors without a regression in any
  previously passing case;
- every newly passing answer has a valid CST, passes isolated public-example
  verification and upstream grading, and cites source/license evidence;
- unresolved cases produce specific formal capability gaps rather than guessed
  code, and the refreshed frontier records those gaps;
- no-memorization, seed closure, native coding, browser coding, benchmark
  ratchet, formatting, lint, and release checks pass from a clean tree.
