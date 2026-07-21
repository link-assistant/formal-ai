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
| Permissive industry slice | #304, #317 | [`industry-suite.lino`](../data/benchmarks/industry-suite.lino) | `issue_304_benchmark_suite_reports_pass_fail_counts` | 10 |
| Multilingual coding-modification | #362 | [`coding-modification-suite.lino`](../data/benchmarks/coding-modification-suite.lino) | `issue_362_multilingual_multi_turn_coding_modification_ratchet` | 4 |
| Text/code edit profile | #408 | [`text-manipulation-suite.lino`](../data/benchmarks/text-manipulation-suite.lino) | `issue_408_text_code_edit_profile_passes_local_ratchet` | 1440 |
| Procedural how-to / instruction-following | #444 | [`procedural-howto-suite.lino`](../data/benchmarks/procedural-howto-suite.lino) | `issue_444_procedural_howto_suite_routes_each_case` | 12 |
| Nemotron training-data sample ingestion | #482 | [`nemotron-training-samples.lino`](../data/benchmarks/nemotron-training-samples.lino) | `issue_482_nemotron_training_ingestion_ratchet_passes_all_samples` | 10 |
| bAbI-style world-state tracking | #702 | [`world-state-tracking-suite.lino`](../data/benchmarks/world-state-tracking-suite.lino) | `issue_702_world_state_suite_tracks_each_case` | 16 |

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
