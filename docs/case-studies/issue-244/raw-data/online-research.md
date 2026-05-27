# Online Research — Issue #244 Vision Planning

This note collects external facts and prior art relevant to issue #244's vision:
a deterministic, link-native problem solver that learns a *universal problem
solving algorithm* and translates between natural and formal languages **without
using neural networks for the reasoning itself**. Citations are summarized, not
copied, per `NON-GOALS.md` ("research notes should not copy large external
texts; they should summarize and cite sources").

## 1. Neuro-symbolic and symbolic reasoning over knowledge graphs

Most 2024–2025 work on "reasoning over knowledge graphs" is *neuro-symbolic*:
it pairs a neural learner with a symbolic component to get robustness plus
interpretability. formal-ai deliberately takes the **symbolic-only** branch of
that spectrum — the knowledge graph (doublet links) *is* the model, and
inference is graph rewriting and rule substitution rather than learned weights.
The surveys below are useful to position the project and to borrow evaluation
framing (logic/reasoning ~35%, knowledge representation ~44% of studies), while
explicitly rejecting the neural learner.

- Neurosymbolic AI for Reasoning Over Knowledge Graphs: A Survey — <https://pubmed.ncbi.nlm.nih.gov/39024082/>
- Towards Unified Neurosymbolic Reasoning on Knowledge Graphs (2025) — <https://arxiv.org/abs/2507.03697>
- A review of neuro-symbolic AI integrating reasoning and learning — <https://www.sciencedirect.com/science/article/pii/S2667305325000675>

Takeaway: the literature treats symbolic KG reasoning as the *interpretable*
half of intelligence. Our differentiator is doing the whole loop symbolically
and keeping every step inspectable as Links Notation.

## 2. Abstract Wikipedia / Wikifunctions — translation through language-independent meaning

This is the closest public analogue to VISION.md's "translate through
link-native meanings" idea. Abstract Wikipedia stores **language-independent
abstract content** anchored on Wikidata items/lexemes; Wikifunctions hosts
**renderers** — functions that turn that abstract content into natural-language
sentences using the linguistic data in Wikidata. This validates our pipeline
`formalize → meaning → deformalize`: a meaning anchored on a Wikidata Q/P id can
be rendered into any language whose labels/lexemes exist.

- Building a Multilingual Wikipedia (CACM, Vrandečić) — <https://cacm.acm.org/opinion/building-a-multilingual-wikipedia/>
- Abstract Wikipedia (Meta-Wiki) — <https://meta.wikimedia.org/wiki/Abstract_Wikipedia>
- Abstract Wikipedia / NLG system architecture proposal — <https://meta.wikimedia.org/wiki/Abstract_Wikipedia/Natural_language_generation_system_architecture_proposal>
- Wikifunctions FAQ — <https://www.wikifunctions.org/wiki/Wikifunctions:FAQ>

Takeaway: we should keep treating Wikidata P/Q ids as the meaning anchor and
Wiktionary/Wikipedia as per-language surfaces, and watch Wikifunctions
renderers as a possible source of deterministic per-language generation rules.

## 3. OpenCog AtomSpace / Hyperon — associative hypergraph memory + self-modifying rules

OpenCog's AtomSpace is a (hyper)graph database whose query engine is a graph
rewriting system and whose rule engine is a generalized rule-driven inferencing
system. OpenCog Hyperon adds the **Distributed AtomSpace** plus **MeTTa**, a
language for introspective, self-modifying programs over that graph. This is the
single closest prior art to formal-ai's "the associative network is the AI" plus
the five rule shapes in VISION.md (pure-data rules → compiled handlers →
dynamically compiled code → natural-language skills).

- AtomSpace (graph DB + graph rewriting) — <https://github.com/opencog/atomspace>
- AtomSpace wiki — <https://wiki.opencog.org/w/AtomSpace>
- OpenCog Hyperon (DAS + MeTTa) — <https://hyperon.opencog.org/>

Takeaway: our doublet store + trigger/substitution computation model is a
restricted, reviewable cousin of AtomSpace. We use **doublets** (Link Foundation)
instead of triplets/hypergraph, and Links Notation as the human-reviewable text
layer. AtomSpace/MeTTa is the reference for rule-as-data and self-modification.

Cognitive-architecture context (Soar, ACT-R, Sigma, MicroPsi) is the lineage for
the "universal problem-solving loop" — symbolic production systems that run the
same recognize→decide→act cycle for every goal, which is exactly the
domain-agnostic loop VISION.md asks for. Newell & Simon's General Problem Solver
is the historical root of "one algorithm, any task via means-ends analysis".

## 4. Program synthesis & automated theorem proving — deterministic verification

For the "generate code + tests, then verify" and "formal reasoning that covers
the test cases" parts of the vision, the mature deterministic tools are SMT
solvers and interactive/automated theorem provers:

- Z3 (SMT solver, MIT-licensed) — used widely for symbolic analysis/verification.
- Lean (interactive theorem prover, Calculus of Inductive Constructions) — <https://galois.com/blog/2018/07/the-lean-theorem-prover-past-present-and-future/>
- Canonical: type-inhabitation/program synthesis in Lean (2025) — <https://arxiv.org/abs/2504.06239>
- Program Synthesis in Saturation (first-order prover synthesis) — <https://arxiv.org/pdf/2402.18962>

Takeaway: the current `src/proof_engine/` is a small classical-theorem registry.
The long-term "formal reasoning that covers all test cases and much more" goal
points toward integrating a real decision procedure (the repo already references
`link-assistant/relative-meta-logic`) and/or an SMT backend for arithmetic and
constraint checks, rather than expanding a hand-written theorem table.

## 4b. Substitution / rewrite systems as data (E24)

For the "behaviour as `replace x y` / `when n do m` rules over link CRUD" goal,
the reference design is `link-foundation/link-cli`: substitution operations where
both sides are link patterns, composed into conditional rewrites. The wider
lineage is term/graph rewriting (confluence + termination as the correctness
properties) and OpenCog AtomSpace's rule engine (rule-as-data, §3). Takeaway:
treat rules as inspectable Links Notation data attached to CRUD events, evaluated
deterministically with an explicit termination guard — not as opaque code.

- `link-foundation/link-cli` — <https://github.com/link-foundation/link-cli>

## 4c. Industry benchmark datasets under permissive licenses (E27)

The issue asks to "double check industry leading datasets … available in
permissible licenses" and import them as test cases. The survey below records the
license of each candidate (verified at the source repository) so E27 can import
only permissively-licensed data. Licenses must be re-checked at import time
against the exact source commit; this table is the starting point, not a waiver.

| Dataset | Domain | License | Source | Why it fits |
| --- | --- | --- | --- | --- |
| HumanEval | Programming (function synthesis from docstring + unit tests) | MIT | `openai/human-eval` | Deterministic unit-test grading matches our generate-then-verify loop. |
| MBPP (Mostly Basic Python Problems) | Programming (short tasks + tests) | CC-BY-4.0 | `google-research/google-research` (`mbpp`) | Broad, small tasks good for the parametric `write a program` intent. |
| GSM8K | Grade-school math word problems | MIT | `openai/grade-school-math` | Multi-step arithmetic reasoning, checkable final answer. |
| MATH (Hendrycks) | Competition math | MIT | `hendrycks/math` | Harder symbolic/algebraic reasoning with exact-answer checking. |

Takeaway: all four are permissively licensed (MIT or CC-BY-4.0) and grade by an
exact/unit-test check, which suits a deterministic symbolic solver — there is no
need for a learned scorer. They become deterministic `.lino` test cases; the
benchmark suite is allowed to start mostly red because it measures the gap the
general coding agent (E26) must close, not the current seed coverage. Avoid
copying any prose beyond the licensed dataset content itself, and vendor the
upstream license file alongside imported data.

## 5. Repository-internal references (the building blocks we already cite)

- `link-foundation/doublets-rs` — long-term doublet store backend (planned, not yet a dependency).
- `link-foundation/doublets-web` — browser-side IndexedDB mirror (planned).
- `link-assistant/calculator` (`link-calculator` crate) — already integrated for arithmetic.
- `link-assistant/relative-meta-logic` — future formal-reasoning integration point.
- `link-foundation/lino-i18n`, `lino-objects-codec`, `lino-arguments` — Links Notation tooling already in use.
- `link-foundation/link-cli` — reference for substitution-rule operations over links (E24).

## Summary of how the prior art shapes the plan

| Vision element | Closest prior art | What we adopt / reject |
| --- | --- | --- |
| "Associative network is the AI" | OpenCog AtomSpace / Hyperon | Adopt graph-rewriting + rule-as-data; use doublets + Links Notation; reject neural learner. |
| Translation via language-independent meaning | Abstract Wikipedia + Wikifunctions | Adopt Wikidata P/Q anchors + per-language renderers; keep Wiktionary/Wikipedia surfaces. |
| Universal problem-solving loop | GPS, Soar, ACT-R | Adopt one domain-agnostic recognize→decide→act loop; keep it deterministic + traceable. |
| Formal reasoning over test cases | Lean / Z3 / saturation synthesis | Integrate a real decision procedure (relative-meta-logic / SMT) instead of a fixed theorem table. |
| Reasoning without neural nets | Symbolic-only branch of NeSy surveys | Keep the whole loop symbolic and inspectable; use the web (Wikidata/Wikipedia/Wiktionary) as a cache, not a teacher. |
| Behaviour as substitution rules (E24) | link-cli / term-graph rewriting / AtomSpace rule engine | Adopt `replace x y` / `when n do m` as inspectable Links Notation data on CRUD; keep deterministic with a termination guard. |
| Measuring "can code / solve anything" (E27) | HumanEval, MBPP, GSM8K, MATH | Import permissively-licensed (MIT/CC-BY) benchmarks graded by exact/unit-test checks as `.lino` test cases; no learned scorer. |

---

# Issue 244 / 304 Benchmark Dataset Research (imported slice)

Date: 2026-05-26

This second section is the implementation record produced when E27
([#304](https://github.com/link-assistant/formal-ai/issues/304)) was merged. It
supersedes the planning table in §4c above with the exact datasets, licenses,
and source commits that were actually imported into
`data/benchmarks/industry-suite.lino`.

Issue #304 asks for a permissively licensed benchmark slice that covers
programming, math, and general problem-solving. This note records the online
license/provenance check used for the imported `.lino` fixtures in
`data/benchmarks/industry-suite.lino`.

## Imported Datasets

| Dataset | Domain | License | Size | Exact source | Imported case |
| --- | --- | --- | --- | --- | --- |
| HumanEval | Programming | MIT | 164 tasks | `openai/human-eval` commit `6d43fb980f9fee3c892a914eda09951f772ad10d`, `data/HumanEval.jsonl.gz` | `HumanEval/0` |
| Mostly Basic Python Problems (MBPP) | Programming | Apache-2.0 | 974 tasks; 427 sanitized tasks | `google-research/google-research` commit `1fa17414f56c3703d5adb3818338b6e35e0fd550`, `mbpp/mbpp.jsonl` | `task_id: 2` |
| GSM8K | General problem-solving | MIT | 7473 train tasks; 1319 test tasks | `openai/grade-school-math` commit `3101c7d5072418e28b9008a6636bde82a006892c`, `grade_school_math/data/test.jsonl` | test line 1 |
| MATH | Math | MIT | 12500 competition-math problems | Hugging Face dataset `qwedsacf/competition_math` commit `e839825f9ec5c6cfa585c654a59610969ec13993`; upstream code repo `hendrycks/math` commit `985bdc1696e88e8643f081a0ff4719da39f2ae2a` | train row 7 |
| BIG-bench `object_counting` | General problem-solving | Apache-2.0 | 1000 examples | `google/BIG-bench` commit `092b196c1f8f14a54bbc62f24759d43bde46dd3b`, `bigbench/benchmark_tasks/object_counting/task.json` | `examples[0]` |

## Selection Notes

- HumanEval and MBPP cover function synthesis from natural-language prompts
  with deterministic unit-test style expectations.
- GSM8K covers multi-step arithmetic word problems without requiring external
  facts.
- MATH covers competition-style symbolic math with a final answer that can be
  checked deterministically.
- BIG-bench `object_counting` covers non-math counting/reasoning and uses a
  deterministic exact-match target.
- The imported slice intentionally excludes canonical solutions and full
  dataset dumps. The repository vendors only five task prompts plus expected
  checks so the benchmark is reviewable and the full datasets remain upstream.

## Rejected Or Deferred Sources

| Dataset | Decision | Reason |
| --- | --- | --- |
| Full HumanEval | Deferred | The complete prompt/test corpus is permissively licensed, but issue #304 only needs a reviewable initial slice wired into the harness. |
| Full MBPP sanitized split | Deferred | Permissive Apache-2.0 source verified; one case is enough alongside HumanEval for this first programming benchmark slice. |
| Full GSM8K and MATH corpora | Deferred | Full imports would add thousands of prompts. The initial deterministic harness proves the schema and runner before scaling. |
| Non-canonical mirrors of these datasets | Rejected | License and source revision are harder to audit than the canonical OpenAI, Google Research, BIG-bench, Hendrycks, and Hugging Face sources listed above. |

## Verification Commands

```bash
gh repo view openai/human-eval --json licenseInfo,defaultBranchRef,url
git ls-remote https://github.com/openai/human-eval HEAD
curl -Ls https://raw.githubusercontent.com/openai/human-eval/6d43fb980f9fee3c892a914eda09951f772ad10d/data/HumanEval.jsonl.gz | gzip -dc | wc -l

gh repo view google-research/google-research --json licenseInfo,defaultBranchRef,url
git ls-remote https://github.com/google-research/google-research HEAD
curl -Ls https://raw.githubusercontent.com/google-research/google-research/1fa17414f56c3703d5adb3818338b6e35e0fd550/mbpp/mbpp.jsonl | wc -l

gh repo view openai/grade-school-math --json licenseInfo,defaultBranchRef,url
git ls-remote https://github.com/openai/grade-school-math HEAD
curl -Ls https://raw.githubusercontent.com/openai/grade-school-math/3101c7d5072418e28b9008a6636bde82a006892c/grade_school_math/data/test.jsonl | wc -l

gh repo view hendrycks/math --json licenseInfo,defaultBranchRef,url
curl -Ls https://huggingface.co/api/datasets/qwedsacf/competition_math

gh repo view google/BIG-bench --json licenseInfo,defaultBranchRef,url
git ls-remote https://github.com/google/BIG-bench HEAD
```
