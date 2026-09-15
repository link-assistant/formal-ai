# Plan 02 — Dynamic discovery: the meta algorithm applied to a coding task

The instruction, in the architect's words: "make this algorithm to discover
enough knowledge in the internet to understand each word/concept and
reconstruct from the formalized knowledge collected in the internet the step by
step guide/algorithm. Formalization itself may need recursive knowledge
collection from trusted sources. Please try to reproduce how previously humans
did manual programming/research […] searched for parts of ready algorithms
[…] search the info on how to do each part, or what each part means […]
dynamic discovery algorithms, that use primarely trusted sources. […] the goal
to know how to get know anything when it is needed."

This document says what the system does with one coding task, which existing
modules do each step, which sources are trusted for what, and where the
behaviour is data rather than code. Plan 03 splits it into leaves.

## 1. One task, end to end

Take HumanEval/13, exactly as the upstream harness sends it:

```
Complete this Python function. Reply with the full implementation in a ```python code block.

def greatest_common_divisor(a: int, b: int) -> int:
    """ Return a greatest common divisor of two integers a and b
    >>> greatest_common_divisor(3, 5)
    1
    >>> greatest_common_divisor(25, 15)
    5
    """
```

A person who knew nothing about the domain would: read the signature and the
sentence; search "greatest common divisor"; find a definition and a ready
implementation; check it against the two examples; write it down; remember
where it came from. The universal problem-solving loop of `VISION.md` names
the same steps. The system does them in this order, and every step appends its
event to the log:

| Step (`VISION.md` loop) | What happens for the task | Existing module / new leaf |
| --- | --- | --- |
| 1 Impulse | the prompt is recorded | `event_log.rs` |
| 2 Formalization → `CodingTaskSpec` | language `python`; name `greatest_common_divisor`; parameters `a: int, b: int`; return `int`; imports (none); requirement sentences: ["Return a greatest common divisor of two integers a and b"]; examples: `(3, 5) → 1`, `(25, 15) → 5` | `src/coding/python_signature.rs` (exists) + `src/coding/task_spec.rs` (L2) |
| 3 Context and domain | surface, language of the prose (en), domain `program_synthesis`; recognised structurally — a `def` with a docstring, or task text with `assert name(...) ==` lines — before any lexical route can claim the prompt | `src/coding/task_spec.rs::recognise` (L2), dispatch order (L3) |
| 4 History lookup | the discovered-procedure ledger is asked for a procedure whose spec identity matches (same normalized requirement and signature shape); a hit is replayed and recorded as `cache_hit` | `src/coding/discovered_procedures.rs` (L9) |
| 5 Decomposition into needs | each requirement sentence becomes a need; the sentence is split into concept phrases: "greatest common divisor", "two integers" | `meta_frame.rs` (exists: `ProblemFrame`, `Need`, `NeedLedger`) + `src/coding/concept_discovery.rs` (L6) |
| 6 Unknown concepts → search local links first | seed meanings (`meanings-*.lino`), the discovered-procedure ledger, the standard-library documentation index already cached | L6 |
| 7 … then external trusted sources | Wikifunctions label search for "greatest common divisor" → `Z13612` (Function, `match_rate 1`); fetch `Z13612` → arguments `Z13518, Z13518` (integers), implementations `Z14707 Z14857 Z29084 Z13642 Z13639`, testers `Z13613 …`; fetch implementations → `Z14857` is Python (`Z610`): `import math; return math.gcd(a, b)`; `Z13642` is Python: the Euclid loop; `Z14707`/`Z29084` are JavaScript (`Z600`) | `src/coding/function_catalog/wikifunctions.rs` (L4) through `CachedSourceClient` (exists) |
| 8 Convert findings into links with source metadata | one `function_part` record per implementation: catalog id, label(s), argument arity/types, language, code, license (`Apache-2.0` for implementations, `CC0-1.0` for definitions), source URL, sha256, fetched_at; the words of the label are linked to the concept phrase that found it | `links_format.rs` (exists), L4 |
| 9 Recursive collection when a finding contains unknown words | a label or description whose words are not in the lexicon is looked up (Wiktionary/Wikipedia/Wikidata chain, `VISION.md` "Formalization And Temperature"), bounded by depth and page limits like `how_to_guide::GuideBounds` | L6 |
| 10 TDD: tests | the docstring examples become assertions (`assert greatest_common_divisor(3, 5) == 1`); MBPP's `test_list` is used as given; a conversational request with no examples gets the testers the catalog function carries (Wikifunctions `Z20`) translated to calls | L2, L4 |
| 11 Drafts | k candidate programs: (a) wrap each Python implementation found, renamed to the task's signature; (b) a standard-library part whose documentation matches the sentence (`math.gcd`: "Greatest Common Divisor."); (c) structural compositions (see §4) | `draft_portfolio.rs` (exists) + `src/coding/composition.rs` (L7) |
| 12 Selection by tests | each draft runs in the bounded workspace against the tests; survivors are ranked by least action (shortest passing source); the comparison is recorded | `agent.rs::AgentWorkspace` (exists), `draft_portfolio.rs` |
| 13 Composition | for a multi-need task the passing parts are composed per §4 and the whole is verified again | L7 |
| 14 Recovery | a failing whole descends: retry alternative parts, widen the search (next label matches, next source), then stop with a named gap | `recursive_execution.rs` (exists), L7 |
| 15 Simplification | the shortest passing draft wins; the answer carries the code, the execution status, and the provenance of every part | `render_python_answer` (exists, extended) |
| 16 Record | the spec identity → parts → composition → tests → result is appended to the discovered-procedure ledger, content-addressed; the trace links name every source | L9 |
| 17 Answer | the code block, `Execution status: tests passed …`, and `Sources:` lines with license; or, when nothing passed, the named skill gap **with the research trail** (what was searched, what was found, which draft failed which example) instead of the unknown opener | L8 |

The same steps handle HumanEval/8 (`sum_product`): the sentence "return a
tuple consisting of a sum and a product of all the integers in a list" yields
needs *sum of a list* and *product of a list*; the standard-library index
answers `sum` ("Return the sum of a 'start' value plus an iterable of
numbers") and `math.prod`; Wikifunctions answers "product of list"; the
structural meaning *tuple of (A, B)* composes them; the two examples select the
draft that returns `(0, 1)` on `[]`. Nothing in that path knows the words
`sum_product` or `HumanEval/8`.

## 2. What exists and what is wrong with it

| Component | State | Consequence |
| --- | --- | --- |
| `solver_handlers/program_synthesis.rs` | recognises a Python-function request through seed roles (good), then dispatches on the task slug to one of three literal bodies | every other task returns `None`; the prompt then falls to arithmetic / concept lookup and answers nonsense (plan 01 B4) |
| `external_benchmarks::benchmark_solver()` | `offline: true` | discovery cannot happen in the scheduled job |
| `coding_research_learning.rs` (#919) | the loop is right (query → cached fetch → formalize → verify → gated ledger) but `captured_procedure` accepts only pages in the repository's own `formal_ai_coding_procedure_v1` format | it has never learned from a real page |
| `knowledge.rs` `CodingOracle` | 25 embedded snippets attributed to Rosetta/Wikifunctions/Hello World/SO | a cache with no way to be refilled; not discovery |
| `coding-idioms.lino` + composer (#395) | real composition from seed idioms for list operations | the right shape; limited to one operation family, no source provenance |
| `draft_portfolio.rs`, `solver_search.rs`, `recursive_execution.rs`, `meta_frame.rs`, `algorithm_discovery.rs` | general engines | the synthesis leaf does not feed them |
| Browser worker (`formal_ai_worker_06.js`, `_13.js`) | mirrors the three literal bodies in JavaScript | JavaScript logic that must go (doctrine); no Python runtime in the browser |

## 3. Trusted sources and what each may be used for

The policy is data: `data/seed/sources-registry.lino` gains a `use` field per
source, read by the discovery code. No source is consulted for a purpose its
license does not allow, and every answer names its sources.

| Source | License | May contribute | May not |
| --- | --- | --- | --- |
| Wikifunctions (`wikifunctions.org` API: `wikilambdasearch_labels`, `wikilambda_fetch`) | CC0-1.0 definitions/labels; Apache-2.0 implementations | ready function parts with code in the target language, argument types, testers, multilingual labels (the labels are how a Russian or Hindi phrase finds the same function) | — |
| Python official documentation (`docs.python.org/3/library/…`) | PSF-2.0 | the standard-library parts index: builtins, `str`/`list`/`dict`/`set` methods, `math`, `itertools`, `heapq`, `re`, `collections`, with their one-sentence descriptions as the words that match a requirement sentence | — |
| Wikipedia / Wikidata / Wiktionary / Open English WordNet (registry entries exist) | CC BY-SA / CC0 / CC BY-SA / CC BY | meaning of a word or concept when a sentence or a label contains one the lexicon does not know (the recursive collection); algorithm *descriptions* (Wikipedia "Euclidean algorithm") that the structural composer can follow | verbatim code |
| Rosetta Code (`rosettacode.org` MediaWiki API) | GFDL-1.2 | task descriptions and per-language example sections, quoted **with attribution and license** when the user asks for an example (#863), executed in the bounded workspace when the user asks to run one (#862) | silent inclusion in a generated solution |
| Stack Exchange (registry entry exists) | CC BY-SA 4.0 | procedure steps in prose, as #991 already uses them | verbatim code |
| The local interpreter (`python3 -c "help(sum)"`) | PSF-2.0 (same text as the documentation) | offline fallback for the standard-library index when the documentation pages are not cached, with provenance `python3.<minor> <symbol>.__doc__` | — |

Every capture goes through `CachedSourceClient` (content-addressed, `fetched_at`,
sha256, TTL); live access is opt-in (`with_online`), offline runs replay the
cache. This is the existing #873/#991 contract; nothing new is invented for
transport.

## 4. Composition: reconstructing the algorithm from parts

Composition is where "reconstruct from the formalized knowledge the step by
step algorithm" happens. It must be general, so its vocabulary is seed
meanings, each grounded in a documentation source, never a table of task
names.

**Structural meanings** (`data/seed/meanings-coding-structure.lino`, new; en, ru,
hi, zh, es lexemes; each carries the Python idiom and the documentation URL
that defines it):

| Meaning | Cue words (en shown; the other four languages are lexemes of the same meaning) | Python idiom | Grounding |
| --- | --- | --- | --- |
| `quantifier_any` | any, some, at least one, whether there exist | `any(<pred> for … in …)` | `builtins.any` docs |
| `quantifier_all` | all, every, each … must | `all(…)` | `builtins.all` |
| `map_each` | for each, list of … for every, transform | `[<expr> for x in xs]` | list displays (tutorial "List Comprehensions") |
| `filter_only` | only those that, filter … for, which contain | `[x for x in xs if <pred>]` | same |
| `reduce_sum` / `reduce_product` / `reduce_max` / `reduce_min` / `reduce_count` / `reduce_len` | sum, total; product; largest, maximum; smallest, minimum; how many, number of; length | `sum(xs)`, `math.prod(xs)`, `max(xs)`, `min(xs)`, `sum(1 for …)`, `len(xs)` | builtins / `math` docs |
| `pairwise_distinct` | any two, each pair, two … each other, between every two consecutive | `itertools.combinations(xs, 2)` / `zip(xs, xs[1:])` | `itertools` docs |
| `predicate_abs_diff_lt` | closer than, differ by less than, within | `abs(a - b) < t` | `builtins.abs` |
| `tuple_of` | a tuple consisting of A and B, return (A, B) | `(A, B)` | tuples |
| `running_prefix` | rolling, so far, until given moment, cumulative | `itertools.accumulate(xs, f)` | `itertools.accumulate` |
| `distinct_elements` | distinct, unique, different | `set(xs)` | `set` |
| `case_insensitive` | regardless of case, ignoring case | `.lower()` | `str.lower` |
| `sort_ascending` / `sort_descending` | sorted from smallest to largest / reverse | `sorted(xs)` / `sorted(xs, reverse=True)` | `sorted` |
| `count_overlapping` | count overlapping cases | `sum(1 for i in range(len(s)) if s.startswith(sub, i))` | `str.startswith` |
| `split_on_space` | space-delimited, separated by spaces | `s.split()` | `str.split` |
| `join_with_space` | space-delimited result, joined with spaces | `' '.join(…)` | `str.join` |
| `range_inclusive` | from 0 up to n inclusive, count to n | `range(0, n + 1)` | `range` |

These are the meanings of English (and Russian, Hindi, Chinese, Spanish) words
as used in specifications; they are language semantics, not benchmark answers.
The rule that keeps them honest: **no meaning may be added whose cue is a
function name or a whole docstring from any benchmark**; leaf L11 pins this
with a gate that greps `src/` and `data/seed/` for the upstream case names and
docstring sentences of the HumanEval and MBPP 20-slices.

**Composition procedure** (L7), given the spec's needs with their found parts:

1. Bind the signature: parameters are the free variables; the return type is
   the shape the composition must produce (`bool` → a quantifier or predicate;
   `List[T]` → a map/filter/sort; `Tuple[A, B]` → `tuple_of`; `int`/`float` →
   a reduction or a function part; `str` → a string construction).
2. For each need, enumerate part candidates: catalog function parts whose
   labels matched the need's phrase (arity must match the phrase's arguments),
   standard-library parts whose description tokens overlap the phrase, and
   structural meanings whose cues occur in the phrase.
3. Build drafts by least action first: a single part whose arity and return
   shape fit the whole signature; then one structural meaning over one part;
   then two; depth bounded (default 3, `SolverConfig::max_decomposition_depth`).
4. Every draft is rendered, CST-validated through the meta-language network,
   and executed against the tests in the bounded workspace. Only a passing
   draft can be selected; among passing drafts the shortest source wins and the
   comparison is recorded (`draft_portfolio.rs`).
5. When no draft passes: descend — split the need at its coordinating
   conjunction ("a sum and a product"), solve the halves, recompose; widen —
   next label matches, next source; stop at the depth bound with a named gap
   whose message lists the searched phrases, the parts found and the failing
   example.

## 5. Verification

Unchanged contract: `AgentWorkspace` (cleared environment, 5 s command
budget, temporary directory), `python3 solution.py` with the assertions
appended, exit code decides (#908: exit code, not output presence). The answer
states what ran. A draft that imports a module the tests did not need is still
allowed; a draft that reads the network or the filesystem is not (the
workspace already blocks it).

## 6. Remembering, forgetting, rediscovering

A passing composition is appended to `discovered-procedures.lino` under the
cache directory (`FORMAL_AI_CACHE_DIR`, the same root the source cache uses):

```
discovered_procedure <content id>
  spec_identity <hash of normalized requirement sentences + signature shape>
  language python
  need "greatest common divisor of two integers"
    part wikifunctions Z14857
      license Apache-2.0
      source_url https://www.wikifunctions.org/wiki/Z14857
      sha256 …
      fetched_at …
  composition direct_wrap
  tests_passed 2
  verified_at …
```

- **Reuse:** step 4 of §1 replays it (`cache_hit`) when the spec identity
  matches; the code is regenerated from the parts, not stored as an opaque
  string, so a later part change is a visible diff.
- **Forget:** deleting the file (or the cache directory) loses nothing that
  cannot be rediscovered: the test `forgotten_procedures_are_rediscovered_from_the_same_sources`
  clears the ledger, replays the same captures offline, and asserts the same
  content id is produced.
- **Deduplicate:** the ledger's sequences (`search → part → compose → verify`)
  are event traces; `algorithm_discovery.rs` already turns repeated traces into
  proposals, and the promotion protocol (#656) is the only way a proposal
  becomes seed. Nothing here promotes by itself.
- **Promotion to seed** stays a reviewed step (`formal-ai improve --promote`);
  the scheduled benchmark job never commits learned knowledge, only the ledger
  row and the frontier record it already commits.

## 7. Where it runs

| Surface | Sources reachable | Behaviour |
| --- | --- | --- |
| Library / CLI, offline (default; `FORMAL_AI_OFFLINE`, PR-time tests) | cache and committed fixtures only | full path over cached captures; a miss is a named gap that says the source was not cached |
| Library / CLI, online (`FORMAL_AI_LIVE_FETCH=1`, or the harness's explicit online flag) | live through `CurlSourceTransport`, captures written to the cache | full discovery |
| `formal-ai benchmark run` in the scheduled External Benchmarks job | online (the job already has network for the datasets) | honest upstream numbers with discovery; the PR-time `benchmark ratchet` never runs a suite |
| OpenAI-compatible server for agentic clients | the client's own `web_search`/`web_fetch` tools (planner) or the server's cache when the client has none | first slice: the server-side path above; a later leaf lets the planner ask the client to fetch the same URLs so the evidence is the client's |
| Browser worker | no Python runtime, no server | reports the boundary: it can formalize the spec, discover and show the parts with provenance, and must say the result is **unverified**; the doctrine says this JavaScript logic is transitional and the three literal bodies are removed rather than extended |

## 8. Multilingual by construction

The recogniser and the structural meanings are seed lexemes in en, ru, hi, zh
and es. The held-out paraphrase sets under `data/benchmarks/` (L12) hold one
conversational coding request per language per family ("напиши функцию на
Python, которая возвращает наибольший общий делитель двух чисел", "एक Python
फ़ंक्शन लिखो जो …", "写一个 Python 函数 …", "escribe una función de Python
que …"); none of those sentences appears in any seed file. Wikifunctions labels
are multilingual, so the Russian phrase finds `Z13612` through its Russian
label when one exists and through the English label of the lexeme's meaning
otherwise.

## 9. Honesty rules that bind every leaf

1. No benchmark case id, function name or docstring sentence in `src/` or
   `data/seed/` (gate, L11).
2. The recorded upstream number is the measured number; a run that discovers
   less than the previous run is red (existing ratchet), and the fix is
   capability, never the floor.
3. An answer without a passing execution never says "tests passed"; an answer
   without a source never says "Source:".
4. A source is named with its license; a share-alike or GFDL source is never
   silently pasted into generated code.
5. Deterministic: same prompt, same cache, same answer; live captures are
   content-addressed so a replay reproduces the run.

## 10. What is measured when this is done

- `formal-ai benchmark run --suite humaneval --slice 20 --online` and the same
  for `mbpp`, locally, before the PR is marked ready; the numbers go into the
  PR description and into `docs/benchmarks.md` as *local measurements dated
  and labelled as such*; the ledger row is written only by the scheduled job.
- The curated 13/13 slice stays 13/13 **through discovery** (the three memorized
  bodies are gone).
- `cargo test --test unit coding_discovery` (new module) runs offline over
  committed captures and covers: spec extraction for three prompt shapes in
  five languages, catalog formalization, standard-library index, structural
  composition, selection by tests, the named gap with a research trail, the
  forget-and-rediscover round trip, and the no-memorization gate.
