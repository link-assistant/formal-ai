# Issue 698 — survey of existing benchmark harnesses

The issue asks for a survey of what already exists, in this repository and
upstream, before adding another harness.

## What this repository already had

| Suite | Issue | What it measures | Why it is not an upstream score |
| --- | --- | --- | --- |
| `industry-suite.lino` | #304, #317 | A permissively licensed slice of industry prompts with self-authored held-out variants. | The cases are a curated handful (one upstream case per source), chosen and paraphrased in-repo. |
| `coding-modification-suite.lino` | #362 | Multilingual multi-turn code editing, four self-authored prompts. | External datasets are recorded for provenance and downloaded by an ignored test, but the *scored* cases are local. |
| `text-manipulation-suite.lino` | #408 | A large local text/code edit profile (`minimum_pass_count` 1440). | Entirely self-authored. |
| `procedural-howto-suite.lino` | #444 | Instruction following on how-to tasks. | Self-authored. |
| `nemotron-training-samples.lino` | #482 | Ingestion of ten sampled Nemotron rows. | Measures ingestion, not task accuracy. |

The pattern worth reusing was **download-on-test** from issue #362 (`curl` into
`target/formal-ai-benchmarks`, never commit the payload) and the
**datasets-server `rows`** access from issue #482 (JSON access to parquet-only
datasets without a parquet decoder). The pattern worth *not* reusing was scoring
a curated local slice and calling it a benchmark result.

## Upstream harnesses considered

| Harness | What it is | Why it was not adopted wholesale |
| --- | --- | --- |
| [`openai/human-eval`](https://github.com/openai/human-eval) | The reference HumanEval runner: reads `HumanEval.jsonl.gz`, executes completions against `check(entry_point)`. | Python-only, and it expects a completions file. Its *data file and grading contract* are adopted directly — this harness performs the same concatenate-and-execute step. |
| [`bigcode-project/bigcode-evaluation-harness`](https://github.com/bigcode-project/bigcode-evaluation-harness) | Broad code-benchmark runner over HF `transformers`. | Requires a Python model-serving stack; the solver here is a Rust library. |
| [`EleutherAI/lm-evaluation-harness`](https://github.com/EleutherAI/lm-evaluation-harness) | The de facto multi-task harness (GSM8K, MATH, BIG-bench tasks). | Same reason: it drives Python model objects or an OpenAI-compatible endpoint, and it would pull a large dependency tree into CI. Its *task definitions* informed the grading modes chosen here (final-number for GSM8K, `\boxed{}` for MATH). |
| [`openai/simple-evals`](https://github.com/openai/simple-evals) | Deliberately minimal reference implementations of GSM8K/MATH/HumanEval graders. | Closest in spirit; its grading rules (last number, last boxed expression) are what `grade.rs` implements in Rust. |
| [`google/BIG-bench`](https://github.com/google/BIG-bench) | `task.json` documents with `examples: [{input, target}]`. | The JSON task format is consumed directly; the Python harness is not needed. |
| [`SWE-bench/SWE-bench`](https://github.com/SWE-bench/SWE-bench) | Agentic repository-patch benchmark with containerised per-instance environments. | Full SWE-bench evaluation requires building a Docker image per instance and applying the gold test patch — far beyond a weekly CI slice. The dev split's `patch` field is compared as a diff instead, which is a strictly *harder* criterion than the upstream test-based one, so the recorded score cannot overstate ability. This is stated in the ledger's `grading_note`. |
| [`facebookresearch/EditEval`](https://github.com/facebookresearch/EditEval) | Instruction-based text-editing benchmark. | Ships no task payload, and its corpora are non-commercial licensed. Recorded as `benchmark_unavailable`; see `solution-plans.md` § R698-06. |
| [`grammarly/coedit`](https://huggingface.co/datasets/grammarly/coedit) | Apache-2.0 instructed text editing, `src`/`tgt` pairs. | Adopted as the runnable instructed-text-editing suite. |
| [`huggingface/evaluate`](https://github.com/huggingface/evaluate) | Metric library (exact match, BLEU, code-eval). | A metric library, not a task runner; the metrics needed here (exact match, final number, boxed answer, diff equality) are a few dozen lines of Rust. |

## Conclusion drawn

Adopt the upstream **data formats and grading contracts** — which is what makes a
score comparable — while keeping the runner in Rust so it can drive the solver
directly, needs no Python model stack, and adds no new crate dependency. Python
is used only where the upstream criterion *is* Python execution (HumanEval,
MBPP), and a machine without `python3` records `benchmark_unavailable` rather
than a fabricated zero.
