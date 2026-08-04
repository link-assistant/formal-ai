# Benchmark Catalog

This is the single, central catalog of **every AI benchmark this repository has
ever touched** — across all issues and their solutions. It is produced by
scanning the executable benchmark fixtures under [`data/benchmarks/`](../data/benchmarks/),
their license provenance in [`data/benchmarks/LICENSES.md`](../data/benchmarks/LICENSES.md),
and the Rust ratchet tests that keep each suite honest.

The repository never vendors full upstream datasets. Each suite either pins the
exact upstream revision and license for a small reviewable slice, or records
source provenance for download-on-test integration. Only permissive licenses
(`MIT`, `Apache-2.0`, `CC-BY-4.0`) are imported.

## Suites at a glance

| Suite | Issue(s) | Fixture | Ratchet test | `minimum_pass_count` |
| --- | --- | --- | --- | --- |
| Permissive industry slice | #304, #317 | [`industry-suite.lino`](../data/benchmarks/industry-suite.lino) | `issue_304_benchmark_suite_reports_pass_fail_counts` | 13 |
| Multilingual coding-modification | #362 | [`coding-modification-suite.lino`](../data/benchmarks/coding-modification-suite.lino) | `issue_362_multilingual_multi_turn_coding_modification_ratchet` | 4 |
| Text/code edit profile | #408 | [`text-manipulation-suite.lino`](../data/benchmarks/text-manipulation-suite.lino) | `issue_408_text_code_edit_profile_passes_local_ratchet` | 1440 |
| Procedural how-to / instruction-following | #444 | [`procedural-howto-suite.lino`](../data/benchmarks/procedural-howto-suite.lino) | `issue_444_procedural_howto_suite_routes_each_case` | 12 |
| Nemotron training-data sample ingestion | #482 | [`nemotron-training-samples.lino`](../data/benchmarks/nemotron-training-samples.lino) | `issue_482_nemotron_training_ingestion_ratchet_passes_all_samples` | 10 |
| Held-out algorithm discovery | #531 | [`issue-531-algorithm-traces.lino`](../data/benchmarks/issue-531-algorithm-traces.lino) | `repeated_event_sequences_become_a_validated_parameterized_algorithm` | 1 |
| External (upstream) harness | #698 | [`external-results.lino`](../data/benchmarks/external-results.lino) | `external_benchmarks::recorded_upstream_pass_count_may_never_regress` | per suite, see below |
| bAbI-style world-state tracking | #702 | [`world-state-tracking-suite.lino`](../data/benchmarks/world-state-tracking-suite.lino) | `issue_702_world_state_suite_tracks_each_case` | 16 |
| Held-out computer-use generalization | #707 | [`computer-use-generalization.lino`](../data/benchmarks/computer-use-generalization.lino) | `every_synthesized_plan_executes_with_every_step_verified` | 12 |
| Search-fusion learning generalization | #709 | [`search-fusion-learning-generalization.lino`](../data/benchmarks/search-fusion-learning-generalization.lino) | `approved_recipe_round_trips_and_executes_a_held_out_task` | 1 |
| Multilingual local-path discovery | #819 | [`local-path-discovery-suite.lino`](../data/benchmarks/local-path-discovery-suite.lino) | `local_path_discovery_benchmark_routes_every_case_to_find` | 56 |
| Workspace-change learning generalization | #848 | [`workspace-change-learning-generalization.lino`](../data/benchmarks/workspace-change-learning-generalization.lino) | `only_a_green_named_review_promotes_and_replays_the_held_out_rewrite` | 1 |
| Equation-type corpus | #891 (from #406) | [`equation-type-corpus.lino`](../data/benchmarks/equation-type-corpus.lino) | `issue_891_equation_corpus_solves_every_type` | 67 (and ≥50 distinct verified types) |

Related earlier work: issue **#103** introduced the competitor-derived prompt
matrix in [`tests/unit/specification/prompt_variations.rs`](../tests/unit/specification/prompt_variations.rs)
(greetings, farewells, identity, clarification, concept lookups, capabilities,
hello-world, basic math, refusal, idioms across English/Russian/Hindi/Chinese).
It is a prompt-category matrix rather than an imported third-party dataset, so it
has no `data/benchmarks/` fixture, but it is listed here for completeness because
it is where systematic, benchmark-style regression coverage began. See
[`docs/case-studies/issue-103/`](./case-studies/issue-103/).

## Sources by suite

### Permissive industry slice — issues #304 / #317

Vendors five upstream task prompts plus five self-authored held-out variants
(anti-memorization). Provenance and pinned revisions live in
[`data/benchmarks/LICENSES.md`](../data/benchmarks/LICENSES.md).

| Source | License | Domain | Upstream |
| --- | --- | --- | --- |
| HumanEval | MIT | programming | <https://github.com/openai/human-eval> |
| Mostly Basic Python Problems (MBPP) | Apache-2.0 | programming | <https://github.com/google-research/google-research/tree/master/mbpp> |
| GSM8K | MIT | general problem solving | <https://github.com/openai/grade-school-math> |
| MATH | MIT | math | <https://github.com/hendrycks/math> |
| BIG-bench `object_counting` | Apache-2.0 | general problem solving | <https://github.com/google/BIG-bench> |

### Multilingual coding-modification — issue #362

Download-on-test manifest (external parquet files cached under
`target/formal-ai-benchmarks`, never checked in) plus four self-authored
multilingual `reverse_sort` prompts (en/ru/hi/zh).

| Source | License | Domain | Upstream |
| --- | --- | --- | --- |
| CanItEdit | MIT | code editing | <https://github.com/nuprl/CanItEdit> |
| HumanEvalFix (HumanEvalPack) | MIT | program repair | <https://huggingface.co/datasets/bigcode/humanevalpack> |
| EDIT-Bench | Apache-2.0 | code editing | <https://github.com/waynchi/editbench> |

### Procedural how-to / instruction-following — issue #444

Records source provenance with pinned `source_ref` revisions for six
instruction-following / assistant-dialog benchmarks. Twelve self-authored cases
(upstream-derived + held-out paraphrases) exercise the deterministic procedural
routing path.

| Source | License | Domain | Upstream |
| --- | --- | --- | --- |
| IFEval (Instruction-Following Eval) | Apache-2.0 | instruction following | <https://github.com/google-research/google-research/tree/master/instruction_following_eval> |
| Super-NaturalInstructions | Apache-2.0 | instruction following | <https://github.com/allenai/natural-instructions> |
| Self-Instruct | Apache-2.0 | instruction following | <https://github.com/yizhongw/self-instruct> |
| OpenAssistant Conversations (OASST1) | Apache-2.0 | assistant dialog | <https://huggingface.co/datasets/OpenAssistant/oasst1> |
| BIG-bench | Apache-2.0 | reasoning | <https://github.com/google/BIG-bench> |
| MMLU | MIT | knowledge | <https://github.com/hendrycks/test> |

### Nemotron training-data sample ingestion — issue #482

Records ten deterministic random samples from NVIDIA's Nemotron 3 Ultra legal
training-data shard. The fixture imports only compact row metadata, SHA-256
digests, and short excerpt previews; the sampler uses Hugging Face
datasets-server `rows` requests with `length=1` and does not download parquet
files or full splits.

| Source | License | Domain | Upstream |
| --- | --- | --- | --- |
| Nemotron Pretraining Legal v1 | CC-BY-4.0 | legal training-data ingestion | <https://huggingface.co/datasets/nvidia/Nemotron-Pretraining-Legal-v1> |

### Held-out algorithm discovery — issue #531

Records three self-authored event traces. Two support traces establish a
repeated `fetch → normalize → persist` episode while the held-out trace changes
the subject binding. The ratchet requires the link-native learner to infer the
shared dataflow, parameterize the changing value, reproduce the held-out trace
losslessly, and keep the resulting algorithm inert until explicit approval.
No third-party benchmark payload is imported.

### bAbI-style world-state tracking — issue #702

Sixteen self-authored dialogues in all four supported languages (en/ru/hi/zh),
each stating facts and a wish and then asking what is left; the solver must
answer from the current→target difference of its symbolic world model. Half the
cases are held-out paraphrases with different entities and query wordings. **No
upstream text is imported** — only the *shape* of the upstream task is
reproduced (the local-profile convention of issue #408), so the recorded licenses
are attribution for the task design, not for vendored data.

| Source | License | Domain | Upstream |
| --- | --- | --- | --- |
| bAbI tasks 1 / 2 / 6 | CC-BY-3.0 (shape only, no text imported) | state tracking | <https://github.com/facebookarchive/bAbI-tasks> |
| Everyday goal-directed assistant dialogues | CC-BY-4.0 | assistant dialog | <https://github.com/link-assistant/formal-ai> |

### Equation-type corpus — issue #891 (requirement from #406)

Sixty-seven self-authored equation types, each replayed through the production
entry point (`FormalAiEngine::answer`) and each carrying the **exact answer the
engine produced** — the expectations are observed, never hand-written
(`cargo run --example issue_891_equation_probe`). The ratchet fails below 50
distinct verified types or below the recorded pass count, which satisfies the
issue #406 requirement of at least fifty verified equation-type examples.
No third-party benchmark payload is imported.

| Category | Types | What it covers |
| --- | --- | --- |
| `linear_one_operation` | 10 | one inverse operation: `x + 2 = 5`, `100 - x = 42`, `-2 * x = 8`, decimal and fractional roots |
| `linear_multi_operation` | 12 | two or more steps: parentheses on both sides, like terms, fractional terms, unknown on both sides |
| `placeholder_unknown` | 8 | `?` and `*` placeholders standing in for the unknown, spaced and unspaced |
| `symbolic_multi_variable` | 7 | isolation with a symbolic right-hand side (`2 * x + 3 * y = 12` → `x = 6 - 1.5*y`) |
| `polynomial` | 14 | degree 2–5 with rational roots, double roots, pure powers, placeholder squares |
| `natural_language_wrapper` | 13 | equation-solving cues in all four supported languages (en/ru/zh/hi) |
| `evaluation_and_percent` | 3 | `2*2+2=?`, trailing `?`, `8% of x = 4` |

Recorded upstream / stack limitations (`benchmark_limitation` records — asserted
to keep failing *loudly*, never with a fabricated answer):

| Gap | Where | Example | Behaviour |
| --- | --- | --- | --- |
| Irrational roots | link-calculator | `x^2 - 2 = 0` | `calculation_error` (rational roots only) |
| Complex roots | link-calculator | `x^2 + 1 = 0` | `calculation_error` |
| Degenerate / contradictory | link-calculator | `0 * x = 5` | `calculation_error`, not "no solution" |
| Identity | formal-ai | `x = x` | `unknown` (no calculation signal) |
| Unit-carrying equations | link-calculator | `x kg = 1000 g` | `calculation_error` (units not converted before solving) |
| Named-unknown declarations | formal-ai | `What is x if x + 7 = 12?` | `calculation_error` (the `x if …` declaration is not stripped) |
| Command-shaped prompts | formal-ai | `Find x: 5 * x = 45` | `agent_suggestion` (`find` is claimed by the shell router) |

### Multilingual local-path discovery — issue #819

Records 56 self-authored prompts spanning English, Russian, Hindi, and Chinese,
the three local scopes, and file/directory targets. Every case must select the
shell tool, emit a bounded `find` command with the expected root and predicate,
and avoid web search. The suite has no imported payload or upstream license.

### Search-fusion learning generalization — issue #709

Records a self-authored held-out research task that is intentionally absent
from the two successful executions used to infer the search-fusion recipe. The
ratchet restores the reviewed recipe from its content-addressed ledger and
requires it to execute that unseen task with complete statement provenance,
ranked sources, semantic merging, and query-language deformalization. No
third-party benchmark payload is imported.

### Workspace-change learning generalization — issue #848

Records two self-authored training identifiers and one unseen equivalent
identifier rewrite. The ratchet verifies that observations with distinct task
and execution fingerprints produce only an inert candidate, then requires a
zero-failure gate and named human approval before the content-addressed recipe
can execute the held-out rewrite. No third-party benchmark payload is imported.

### Text/code edit profile — issue #408

The broadest map: 48 referenced edit/reasoning/coding/QA benchmarks recorded as
source provenance, each backed by 30 self-authored deterministic edit cases (no
upstream payloads vendored). The full list, in fixture order:

| Source | Domain | Upstream |
| --- | --- | --- |
| CoEdIT | text editing | <https://github.com/vipulraheja/coedit> |
| EditEval | text editing | <https://github.com/facebookresearch/EditEval> |
| InstrEditBench / FineEdit | fine-grained text editing | <https://arxiv.org/html/2502.13358v2> |
| CodeEditorBench | code editing | <https://github.com/CodeEditorBench/CodeEditorBench> |
| CanItEdit | code editing | <https://github.com/nuprl/CanItEdit> |
| EDIT-Bench | code editing | <https://github.com/waynchi/editbench> |
| HumanEvalFix (HumanEvalPack) | program repair | <https://huggingface.co/datasets/bigcode/humanevalpack> |
| SWE-bench | repository patch | <https://www.swebench.com/> |
| HumanEval | programming | <https://github.com/openai/human-eval> |
| Mostly Basic Python Problems (MBPP) | programming | <https://github.com/google-research/google-research/tree/master/mbpp> |
| GSM8K | math word problem | <https://github.com/openai/grade-school-math> |
| MATH | competition math | <https://github.com/hendrycks/math> |
| BIG-bench | broad language tasks | <https://github.com/google/BIG-bench> |
| MMLU | knowledge QA | <https://github.com/hendrycks/test> |
| MMLU-Pro | knowledge QA | <https://github.com/TIGER-AI-Lab/MMLU-Pro> |
| BIG-Bench Hard (BBH) | reasoning | <https://github.com/suzgunmirac/BIG-Bench-Hard> |
| HELM | evaluation harness | <https://github.com/stanford-crfm/helm> |
| AI2 ARC | science QA | <https://allenai.org/data/arc> |
| HellaSwag | commonsense completion | <https://rowanzellers.com/hellaswag/> |
| TruthfulQA | truthfulness | <https://github.com/sylinrl/TruthfulQA> |
| WinoGrande | commonsense coreference | <https://github.com/allenai/winogrande> |
| DROP | reading comprehension | <https://allennlp.org/drop> |
| SQuAD | reading comprehension | <https://rajpurkar.github.io/SQuAD-explorer/> |
| Natural Questions | open-domain QA | <https://ai.google.com/research/NaturalQuestions> |
| TriviaQA | reading comprehension | <https://nlp.cs.washington.edu/triviaqa/> |
| BoolQ | boolean QA | <https://github.com/google-research-datasets/boolean-questions> |
| CommonsenseQA | commonsense QA | <https://www.tau-nlp.sites.tau.ac.il/commonsenseqa> |
| IFEval | instruction following | <https://github.com/google-research/google-research/tree/master/instruction_following_eval> |
| GPQA | graduate reasoning QA | <https://github.com/idavidrein/gpqa> |
| MuSR | multi-step reasoning | <https://github.com/Zayne-sprague/MuSR> |
| LiveCodeBench | live coding | <https://github.com/livecodebench/livecodebench> |
| Berkeley Function Calling Leaderboard (BFCL) | tool calling | <https://gorilla.cs.berkeley.edu/leaderboard.html> |
| SimpleQA | factuality | <https://openai.com/index/introducing-simpleqa/> |
| MMMU | multimodal reasoning | <https://mmmu-benchmark.github.io/> |
| RULER | long context | <https://github.com/NVIDIA/RULER> |
| LongBench | long context | <https://github.com/THUDM/LongBench> |
| AlpacaEval | instruction following | <https://github.com/tatsu-lab/alpaca_eval> |
| MT-Bench | chat evaluation | <https://github.com/lm-sys/FastChat/tree/main/fastchat/llm_judge> |
| Arena-Hard | chat evaluation | <https://github.com/lm-sys/arena-hard-auto> |
| WildBench | instruction following | <https://github.com/allenai/WildBench> |
| MATH-500 | competition math | <https://github.com/openai/simple-evals> |
| AIME | competition math | <https://artofproblemsolving.com/wiki/index.php/AIME_Problems_and_Solutions> |
| MGSM | multilingual math | <https://github.com/google-research/url-nlp/tree/main/mgsm> |
| HumanEval+ | programming | <https://github.com/evalplus/evalplus> |
| MBPP+ | programming | <https://github.com/evalplus/evalplus> |
| MultiPL-E | multilingual programming | <https://github.com/nuprl/MultiPL-E> |
| APPS | programming | <https://github.com/hendrycks/apps> |
| DS-1000 | data science code | <https://github.com/xlang-ai/DS-1000> |

## External (upstream) results

Every suite above scores the solver against a small, reviewable slice that this
repository curates. Issue **#698** adds the opposite kind of measurement: a
harness that fetches the *unmodified upstream* case set at run time and reports
`passed / total` over the first N cases **in upstream order**, with no curated
subset and no invented floor. A low number is published as a low number.

The harness lives in [`src/external_benchmarks/`](../src/external_benchmarks/),
its provenance and results ledger is
[`data/benchmarks/external-results.lino`](../data/benchmarks/external-results.lino),
and the scheduled job that refreshes it is
[`.github/workflows/external-benchmarks.yml`](../.github/workflows/external-benchmarks.yml)
(weekly, plus `workflow_dispatch` with configurable core/SWE-bench slices).
Every pull request also compares the ledger with its fetched base revision.
Cached payloads are accepted only when their URL, immutable source revision,
byte length, and content id match the adjacent provenance record.

### Honest current numbers

Recorded `2026-08-03`, solver version `0.323.0`, slice `20` upstream cases per
core suite and a separately bounded one-case SWE-bench slice, offline
deterministic solver (`temperature = 0.0`):

| Suite | License | Grading | Passed | Total |
| --- | --- | --- | ---: | ---: |
| HumanEval | MIT | upstream unit test executed | 0 | 20 |
| MBPP | Apache-2.0 | upstream `test_list` asserts executed | 0 | 20 |
| GSM8K | MIT | final number vs. `####` gold | 2 | 20 |
| MATH (`prm800k` 500-problem split) | MIT | final `\boxed{...}` vs. gold | 0 | 20 |
| BIG-bench `object_counting` | Apache-2.0 | final number vs. target | 0 | 20 |
| CoEdIT | Apache-2.0 | edited text vs. gold target | 0 | 20 |
| SWE-bench Lite (dev) | MIT | official upstream instance tests executed | 0 | 1 |
| EditEval | — | `benchmark_unavailable` | — | — |

`2 / 20` on GSM8K, `0 / 20` on the other scored core suites, and `0 / 1` on
SWE-bench Lite are the real measurements of the current offline solver against
unmodified upstream cases. They are recorded exactly as measured; the ratchet
makes them the floor these numbers may never fall below.

The original SWE-bench row was withdrawn: it compared output with the gold
patch, which is not the SWE-bench pass criterion. Scheduled runs now use the
pinned official harness (`f7bbbb2…`) to apply a candidate patch in the upstream
container and execute the instance tests; the current `0 / 1` score came through
that evaluator. An evaluator, Docker, or parquet decoder failure becomes
`benchmark_unavailable`; it is never counted as a solver failure and never
replaced by an exact-diff proxy.

EditEval is recorded as `benchmark_unavailable` rather than being replaced by a
local proxy: the upstream repository ships an evaluation harness with no task
payload, and its constituent corpora fail the permissive-only policy (ASSET is
CC BY-NC 4.0, JFLEG is CC BY-NC-SA 4.0). The instructed-text-editing task family
is independently measured by the Apache-2.0 CoEdIT suite; that score is never
recorded as an EditEval result. Runtime download, decode, or upstream-schema
failures likewise produce a concrete `benchmark_unavailable` row so scheduled
runs do not silently lose the reason that no score exists.

### Ratchet

`external_benchmark_suite.minimum_pass_count` only ever rises: a run that scores
higher raises the floor, a run that scores lower is a failure, and a pull request
that rewrites a recorded pass count downwards or deletes a recorded row is
reported as a regression by `external_benchmarks::ratchet::regressions`.
The pull-request workflow invokes `benchmark ratchet --base-ref
origin/${GITHUB_BASE_REF}`, so this comparison is exercised rather than merely
exposed as a library function.

Each scheduled run also writes proposal-only associative learning reports from
the failed case ids and evaluator details. These reports use Formal AI's shared
learning substrate and remain `awaiting_human_review`; no observed failure
automatically changes solver behavior or raises a floor.

### Running it

```sh
# List every upstream suite with license, provenance, and grading mode.
cargo run --bin formal-ai -- benchmark list

# Run 20 real upstream HumanEval cases end to end (network + python3 required).
cargo run --bin formal-ai -- benchmark run --suite humaneval --slice 20

# Refresh every suite locally. SWE-bench additionally needs the pinned official
# Python harness and Docker; scheduled CI bounds it separately to one case.
cargo run --bin formal-ai -- benchmark run --suite all --slice 20 --append

# Verify the monotonic ratchet without running any suite.
cargo run --bin formal-ai -- benchmark ratchet

# Compare the current ledger with a real git baseline.
cargo run --bin formal-ai -- benchmark ratchet --base-ref origin/main

# The same end-to-end run as an ignored test (network + python3 required).
cargo test --test unit external_benchmarks -- --ignored --nocapture
```

## How to run

Each suite is an executable ratchet — CI fails if the derived pass count drops
below the recorded `minimum_pass_count`.

```sh
# Industry slice (#304/#317)
cargo test --test unit issue_304_benchmark_suite_reports_pass_fail_counts -- --nocapture

# Multilingual coding-modification (#362)
cargo test --test unit issue_362_multilingual_multi_turn_coding_modification_ratchet -- --nocapture
# Optional network download-on-test integration:
FORMAL_AI_BULK_BENCHMARK=1 cargo test --test unit issue_362_external_edit_datasets_download_on_test_only -- --ignored --nocapture

# Text/code edit profile (#408)
cargo test --test unit issue_408_text_code_edit_profile_passes_local_ratchet -- --nocapture

# Procedural how-to / instruction-following (#444)
cargo test --test unit issue_444_procedural_howto_suite_routes_each_case -- --nocapture

# Nemotron training-data sample ingestion (#482)
cargo test --test unit issue_482_nemotron_training -- --nocapture

# Multilingual local-path discovery (#819)
cargo test --test unit local_path_discovery_benchmark_routes_every_case_to_find -- --nocapture

# Review-gated workspace-change generalization (#848)
cargo test --test unit only_a_green_named_review_promotes_and_replays_the_held_out_rewrite -- --nocapture

# Equation-type corpus (#891)
cargo test --test unit issue_891_equation_corpus -- --nocapture
```

## Conventions

- **Permissive only.** `MIT`, `Apache-2.0`, `CC-BY-4.0`. New sources must record
  their license and pinned revision before import.
- **No vendored datasets.** Slices pin a handful of upstream prompts; bulk data
  is downloaded on test into `target/formal-ai-benchmarks` (a build artifact).
- **Anti-memorization.** Each upstream-derived case ships a self-authored
  held-out / paraphrased variant so passing requires generalization, not recall.
- **Ratchet, never regress.** `minimum_pass_count` only rises after new cases
  pass locally and in CI.
- **Adding a benchmark.** Add its provenance record to the suite `.lino`, add
  cases, update [`data/benchmarks/LICENSES.md`](../data/benchmarks/LICENSES.md)
  when a payload slice is vendored, and add a row to the tables above so this
  catalog stays the complete index.
