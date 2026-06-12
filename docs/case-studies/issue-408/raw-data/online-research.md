# Issue 408 Benchmark Research

Collected on 2026-06-12 from primary benchmark sources.

## Benchmark Sources Referenced By PR 416

| Benchmark | Primary source | Task shape | Repository use |
| --- | --- | --- | --- |
| CoEdIT | https://github.com/vipulraheja/coedit | Instruction-guided text editing | Self-authored text edit examples inspired by the task family. |
| EditEval | https://github.com/facebookresearch/EditEval | Text editing evaluation | Self-authored style, grammar, and rewrite examples. |
| InstrEditBench / FineEdit | https://arxiv.org/html/2502.13358v2 and https://huggingface.co/datasets/YimingZeng/FineEdit_bench | Fine-grained instruction editing | Self-authored ordered edit examples. |
| CodeEditorBench | https://github.com/CodeEditorBench/CodeEditorBench | Code editing tasks | Self-authored code replacement and cleanup examples. |
| CanItEdit | https://github.com/nuprl/CanItEdit | Code edit ability evaluation | Self-authored program edit examples. |
| EDIT-Bench | https://github.com/waynchi/editbench | Editing benchmark tasks | Self-authored edit command examples. |
| HumanEvalPack | https://huggingface.co/datasets/bigcode/humanevalpack | Multilingual code generation and repair | Self-authored HumanEvalFix-style code edits. |
| SWE-bench | https://www.swebench.com/ and https://github.com/swe-bench/SWE-bench | Repository-level issue-to-patch tasks | Self-authored issue-style code edit examples only. |

## Additional Popular LLM Benchmarks (20)

| # | Benchmark | Primary source | Task shape | Issue 408 status |
| --- | --- | --- | --- | --- |
| 1 | HumanEval | https://github.com/openai/human-eval | Python code generation from docstring specifications | Already represented in the separate industry-suite slice, not an upstream 10% import. |
| 2 | MBPP | https://github.com/google-research/google-research/tree/master/mbpp | Basic Python programming problems | Already represented in the separate industry-suite slice, not an upstream 10% import. |
| 3 | GSM8K | https://github.com/openai/grade-school-math | Grade-school math word problems | Already represented in the separate industry-suite slice, not an upstream 10% import. |
| 4 | MATH | https://github.com/hendrycks/math | Competition-style math problems | Already represented in the separate industry-suite slice, not an upstream 10% import. |
| 5 | BIG-bench | https://github.com/google/BIG-bench | Broad task collection for language models | Already represented in the separate industry-suite slice, not an upstream 10% import. |
| 6 | MMLU | https://github.com/hendrycks/test | Multi-task knowledge questions | Source researched; no issue #408 edit tasks imported. |
| 7 | MMLU-Pro | https://github.com/TIGER-AI-Lab/MMLU-Pro | Harder multi-task knowledge questions | Source researched; no issue #408 edit tasks imported. |
| 8 | BIG-Bench Hard | https://github.com/suzgunmirac/BIG-Bench-Hard | Hard subset of BIG-bench tasks | Source researched; no issue #408 edit tasks imported. |
| 9 | HELM | https://github.com/stanford-crfm/helm and https://crfm.stanford.edu/helm/ | Holistic model evaluation harness | Source researched; no issue #408 edit tasks imported. |
| 10 | ARC | https://allenai.org/data/arc and https://github.com/allenai/ai2-arc | Science question answering | Source researched; no issue #408 edit tasks imported. |
| 11 | HellaSwag | https://rowanzellers.com/hellaswag/ | Commonsense completion | Source researched; no issue #408 edit tasks imported. |
| 12 | TruthfulQA | https://github.com/sylinrl/TruthfulQA | Truthfulness and misconception resistance | Source researched; no issue #408 edit tasks imported. |
| 13 | WinoGrande | https://github.com/allenai/winogrande | Commonsense coreference | Source researched; no issue #408 edit tasks imported. |
| 14 | DROP | https://allennlp.org/drop | Discrete reasoning over paragraphs | Source researched; no issue #408 edit tasks imported. |
| 15 | SQuAD | https://rajpurkar.github.io/SQuAD-explorer/ | Extractive reading comprehension | Source researched; no issue #408 edit tasks imported. |
| 16 | Natural Questions | https://ai.google.com/research/NaturalQuestions | Open-domain question answering | Source researched; no issue #408 edit tasks imported. |
| 17 | TriviaQA | https://nlp.cs.washington.edu/triviaqa/ | Reading comprehension and trivia QA | Source researched; no issue #408 edit tasks imported. |
| 18 | BoolQ | https://github.com/google-research-datasets/boolean-questions | Naturally occurring yes/no questions | Source researched; no issue #408 edit tasks imported. |
| 19 | CommonsenseQA | https://www.tau-nlp.sites.tau.ac.il/commonsenseqa | Commonsense multiple-choice QA | Source researched; no issue #408 edit tasks imported. |
| 20 | IFEval | https://github.com/google-research/google-research/tree/master/instruction_following_eval | Verifiable instruction-following prompts | Source researched; no issue #408 edit tasks imported. |

## Coverage Decision

The issue #408 implementation uses benchmark-derived examples only where they
exercise deterministic text or code editing behavior. It does not import the
external benchmark records above, does not run their official scoring scripts,
and does not claim to fully pass 10% of each benchmark. Any future PR that wants
to make that stronger claim must add dataset provenance, a runner, scoring,
license review, CI budget, and a pass-count ratchet for each benchmark.
