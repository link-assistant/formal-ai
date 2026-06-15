# Issue 408 Benchmark Research

Collected on 2026-06-12 from primary benchmark sources.

## Benchmark Sources Referenced By PR 416

| Benchmark | Primary source | Task shape | Repository use |
| --- | --- | --- | --- |
| CoEdIT | https://github.com/vipulraheja/coedit | Instruction-guided text editing | Listed in `data/benchmarks/text-manipulation-suite.lino` with 30 local text/code edit variations. |
| EditEval | https://github.com/facebookresearch/EditEval | Text editing evaluation | Listed in `data/benchmarks/text-manipulation-suite.lino` with 30 local text/code edit variations. |
| InstrEditBench / FineEdit | https://arxiv.org/html/2502.13358v2 and https://huggingface.co/datasets/YimingZeng/FineEdit_bench | Fine-grained instruction editing | Listed in `data/benchmarks/text-manipulation-suite.lino` with 30 local text/code edit variations. |
| CodeEditorBench | https://github.com/CodeEditorBench/CodeEditorBench | Code editing tasks | Listed in `data/benchmarks/text-manipulation-suite.lino` with 30 local text/code edit variations. |
| CanItEdit | https://github.com/nuprl/CanItEdit | Code edit ability evaluation | Listed in `data/benchmarks/text-manipulation-suite.lino` with 30 local text/code edit variations. |
| EDIT-Bench | https://github.com/waynchi/editbench | Editing benchmark tasks | Listed in `data/benchmarks/text-manipulation-suite.lino` with 30 local text/code edit variations. |
| HumanEvalPack | https://huggingface.co/datasets/bigcode/humanevalpack | Multilingual code generation and repair | Listed in `data/benchmarks/text-manipulation-suite.lino` with 30 local text/code edit variations. |
| SWE-bench | https://www.swebench.com/ and https://github.com/swe-bench/SWE-bench | Repository-level issue-to-patch tasks | Listed in `data/benchmarks/text-manipulation-suite.lino` with 30 local text/code edit variations. |

## Additional Popular LLM Benchmarks (20)

| # | Benchmark | Primary source | Task shape | Issue 408 status |
| --- | --- | --- | --- | --- |
| 1 | HumanEval | https://github.com/openai/human-eval | Python code generation from docstring specifications | Listed in the issue #408 local profile with 30 deterministic edit variations. |
| 2 | MBPP | https://github.com/google-research/google-research/tree/master/mbpp | Basic Python programming problems | Listed in the issue #408 local profile with 30 deterministic edit variations. |
| 3 | GSM8K | https://github.com/openai/grade-school-math | Grade-school math word problems | Listed in the issue #408 local profile with 30 deterministic edit variations. |
| 4 | MATH | https://github.com/hendrycks/math | Competition-style math problems | Listed in the issue #408 local profile with 30 deterministic edit variations. |
| 5 | BIG-bench | https://github.com/google/BIG-bench | Broad task collection for language models | Listed in the issue #408 local profile with 30 deterministic edit variations. |
| 6 | MMLU | https://github.com/hendrycks/test | Multi-task knowledge questions | Listed in the issue #408 local profile with 30 deterministic edit variations. |
| 7 | MMLU-Pro | https://github.com/TIGER-AI-Lab/MMLU-Pro | Harder multi-task knowledge questions | Listed in the issue #408 local profile with 30 deterministic edit variations. |
| 8 | BIG-Bench Hard | https://github.com/suzgunmirac/BIG-Bench-Hard | Hard subset of BIG-bench tasks | Listed in the issue #408 local profile with 30 deterministic edit variations. |
| 9 | HELM | https://github.com/stanford-crfm/helm and https://crfm.stanford.edu/helm/ | Holistic model evaluation harness | Listed in the issue #408 local profile with 30 deterministic edit variations. |
| 10 | ARC | https://allenai.org/data/arc and https://github.com/allenai/ai2-arc | Science question answering | Listed in the issue #408 local profile with 30 deterministic edit variations. |
| 11 | HellaSwag | https://rowanzellers.com/hellaswag/ | Commonsense completion | Listed in the issue #408 local profile with 30 deterministic edit variations. |
| 12 | TruthfulQA | https://github.com/sylinrl/TruthfulQA | Truthfulness and misconception resistance | Listed in the issue #408 local profile with 30 deterministic edit variations. |
| 13 | WinoGrande | https://github.com/allenai/winogrande | Commonsense coreference | Listed in the issue #408 local profile with 30 deterministic edit variations. |
| 14 | DROP | https://allennlp.org/drop | Discrete reasoning over paragraphs | Listed in the issue #408 local profile with 30 deterministic edit variations. |
| 15 | SQuAD | https://rajpurkar.github.io/SQuAD-explorer/ | Extractive reading comprehension | Listed in the issue #408 local profile with 30 deterministic edit variations. |
| 16 | Natural Questions | https://ai.google.com/research/NaturalQuestions | Open-domain question answering | Listed in the issue #408 local profile with 30 deterministic edit variations. |
| 17 | TriviaQA | https://nlp.cs.washington.edu/triviaqa/ | Reading comprehension and trivia QA | Listed in the issue #408 local profile with 30 deterministic edit variations. |
| 18 | BoolQ | https://github.com/google-research-datasets/boolean-questions | Naturally occurring yes/no questions | Listed in the issue #408 local profile with 30 deterministic edit variations. |
| 19 | CommonsenseQA | https://www.tau-nlp.sites.tau.ac.il/commonsenseqa | Commonsense multiple-choice QA | Listed in the issue #408 local profile with 30 deterministic edit variations. |
| 20 | IFEval | https://github.com/google-research/google-research/tree/master/instruction_following_eval | Verifiable instruction-following prompts | Listed in the issue #408 local profile with 30 deterministic edit variations. |

## Additional Current/Common LLM Benchmarks (20)

| # | Benchmark | Primary source | Task shape | Issue 408 status |
| --- | --- | --- | --- | --- |
| 1 | GPQA | https://github.com/idavidrein/gpqa | Graduate-level Google-proof science QA | Listed in the issue #408 local profile with 30 deterministic edit variations. |
| 2 | MuSR | https://github.com/Zayne-sprague/MuSR | Multi-step soft reasoning | Listed in the issue #408 local profile with 30 deterministic edit variations. |
| 3 | LiveCodeBench | https://github.com/livecodebench/livecodebench and https://livecodebench.github.io/ | Continuously collected coding evaluation | Listed in the issue #408 local profile with 30 deterministic edit variations. |
| 4 | Berkeley Function Calling Leaderboard | https://gorilla.cs.berkeley.edu/leaderboard.html | Function/tool calling | Listed in the issue #408 local profile with 30 deterministic edit variations. |
| 5 | SimpleQA | https://openai.com/index/introducing-simpleqa/ and https://github.com/openai/simple-evals | Short factuality QA | Listed in the issue #408 local profile with 30 deterministic edit variations. |
| 6 | MMMU | https://mmmu-benchmark.github.io/ and https://github.com/MMMU-Benchmark/MMMU | Multimodal college-level reasoning | Listed in the issue #408 local profile with 30 deterministic edit variations. |
| 7 | RULER | https://github.com/NVIDIA/RULER | Long-context retrieval and reasoning | Listed in the issue #408 local profile with 30 deterministic edit variations. |
| 8 | LongBench | https://github.com/THUDM/LongBench | Long-context language tasks | Listed in the issue #408 local profile with 30 deterministic edit variations. |
| 9 | AlpacaEval | https://github.com/tatsu-lab/alpaca_eval | Instruction-following response comparison | Listed in the issue #408 local profile with 30 deterministic edit variations. |
| 10 | MT-Bench | https://github.com/lm-sys/FastChat/tree/main/fastchat/llm_judge | Multi-turn chat evaluation | Listed in the issue #408 local profile with 30 deterministic edit variations. |
| 11 | Arena-Hard | https://github.com/lm-sys/arena-hard-auto | Hard chat prompt evaluation | Listed in the issue #408 local profile with 30 deterministic edit variations. |
| 12 | WildBench | https://github.com/allenai/WildBench | Wild instruction-following evaluation | Listed in the issue #408 local profile with 30 deterministic edit variations. |
| 13 | MATH-500 | https://github.com/openai/simple-evals | Competition-math evaluation slice | Listed in the issue #408 local profile with 30 deterministic edit variations. |
| 14 | AIME | https://artofproblemsolving.com/wiki/index.php/AIME_Problems_and_Solutions | Contest math problems used in LLM evaluation | Listed in the issue #408 local profile with 30 deterministic edit variations. |
| 15 | MGSM | https://github.com/google-research/url-nlp/tree/main/mgsm | Multilingual grade-school math | Listed in the issue #408 local profile with 30 deterministic edit variations. |
| 16 | HumanEval+ | https://github.com/evalplus/evalplus | Augmented HumanEval tests | Listed in the issue #408 local profile with 30 deterministic edit variations. |
| 17 | MBPP+ | https://github.com/evalplus/evalplus | Augmented MBPP tests | Listed in the issue #408 local profile with 30 deterministic edit variations. |
| 18 | MultiPL-E | https://github.com/nuprl/MultiPL-E | Multilingual code-generation evaluation | Listed in the issue #408 local profile with 30 deterministic edit variations. |
| 19 | APPS | https://github.com/hendrycks/apps | Python programming problems | Listed in the issue #408 local profile with 30 deterministic edit variations. |
| 20 | DS-1000 | https://github.com/xlang-ai/DS-1000 | Data-science code completion/editing tasks | Listed in the issue #408 local profile with 30 deterministic edit variations. |

## Coverage Decision

The issue #408 implementation records every source above in
`data/benchmarks/text-manipulation-suite.lino` and generates 30 deterministic
repository-local edit variations per source. The local ratchet passes 1,440 of
1,440 profile checks, and the test reports per-source pass counts so each source
must pass 30/30. This is the complete issue #408 benchmark claim for PR #416:
repository-local executable text/code edit coverage with source provenance, not
an external leaderboard publication.
