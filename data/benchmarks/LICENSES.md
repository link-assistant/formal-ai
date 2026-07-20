# Benchmark License Provenance

This directory imports a reviewable benchmark slice for issues #304 and #317.
The `.lino` fixture records the exact upstream revision, source path, and
license URL for every source. Issue #317 adds self-authored held-out variants
that preserve the licensed source task structure while changing wording,
numbers, or distractors.

Issue #362 adds a coding-modification manifest. It records external benchmark
provenance for download-on-test integration while keeping full upstream
datasets out of the repository.

## Imported Sources

| Source | License | Upstream revision | Upstream cases | Held-out variants | Suite cases |
| --- | --- | --- | --- | --- | --- |
| HumanEval | MIT | `6d43fb980f9fee3c892a914eda09951f772ad10d` | 1 | 1 | 2 |
| Mostly Basic Python Problems | Apache-2.0 | `1fa17414f56c3703d5adb3818338b6e35e0fd550` | 1 | 1 | 2 |
| GSM8K | MIT | `3101c7d5072418e28b9008a6636bde82a006892c` | 1 | 1 | 2 |
| MATH | MIT | `e839825f9ec5c6cfa585c654a59610969ec13993` | 1 | 1 | 2 |
| BIG-bench object_counting | Apache-2.0 | `092b196c1f8f14a54bbc62f24759d43bde46dd3b` | 1 | 1 | 2 |
| Arithmetic reachability search (self-authored) | CC-BY-4.0 | `issue-662` | 0 | 1 | 2 |

Issue #662 adds a self-authored, search-only slice ("Arithmetic reachability
search"). Its two cases have no reusable part or single rule that derives the
answer, so they pass only through the step-7 budget-driven random/evolutionary
search that combines the given numbers with the allowed operators to reach the
target. Because the prompts are self-authored rather than imported, the source
carries no upstream dataset; it is released under CC-BY-4.0.

## License Texts

The upstream license files are canonical:

- HumanEval MIT license: <https://raw.githubusercontent.com/openai/human-eval/6d43fb980f9fee3c892a914eda09951f772ad10d/LICENSE>
- MBPP Apache-2.0 license: <https://raw.githubusercontent.com/google-research/google-research/1fa17414f56c3703d5adb3818338b6e35e0fd550/LICENSE>
- GSM8K MIT license: <https://raw.githubusercontent.com/openai/grade-school-math/3101c7d5072418e28b9008a6636bde82a006892c/LICENSE>
- MATH MIT license: <https://raw.githubusercontent.com/hendrycks/math/985bdc1696e88e8643f081a0ff4719da39f2ae2a/LICENSE>
- BIG-bench Apache-2.0 license: <https://raw.githubusercontent.com/google/BIG-bench/092b196c1f8f14a54bbc62f24759d43bde46dd3b/LICENSE>

Only five upstream task prompts, six self-authored held-out variants (including
the issue #662 search-only source), two self-authored search-only prompts, and
their expected deterministic checks are vendored here. Canonical solutions and
full external datasets are intentionally not copied into the repository. The
benchmark suite records `minimum_pass_count "12"` so CI fails if the current
derived pass count drops below the recorded floor.

## Issue #362 Coding-Modification Sources

| Source | License | Upstream revision or file hash | Download mode |
| --- | --- | --- | --- |
| CanItEdit | MIT | `74d15ea7e6207cb9fdeeecd761907371d4cc2e26`; HF `3c07f38b1f9385f3214fcea94d4664c79df0d36a`; LFS `9f78b1a2378b96b24d158a6fe83d69aa18e43a360ae3b7d0891c02f660cc6222` | ignored network test |
| HumanEvalFix from HumanEvalPack | MIT | `e17a8f6470264286bc6a52eb8263582083bf3bf6`; HF `9a41762f73a8cb23bb5811b73d5aab164efcf378`; LFS `ed5f15d789156e21222bfcd556c425a39042355c84ae1e8b058abd6a3d7f8075` | ignored network test |
| EDIT-Bench | Apache-2.0 | `2ecd13159711d2d5bbdf36700119b4278f387dc0`; HF `0d41afafcfe7c759adcd2eaceabfa486ab6eb0e2`; LFS `0245660f5422cc1404da044f612d2aa9511c7feec252416cbda447c9fe0ee531` | ignored network test |

The canonical license references are:

- CanItEdit MIT license: <https://raw.githubusercontent.com/nuprl/CanItEdit/main/LICENSE>
- HumanEvalPack dataset card with MIT metadata: <https://huggingface.co/datasets/bigcode/humanevalpack>
- EDIT-Bench Apache-2.0 license: <https://raw.githubusercontent.com/waynchi/editbench/main/LICENSE.md>

The issue #362 deterministic ratchet vendors only four self-authored
multilingual prompts and deterministic trace checks. The network benchmark
downloads the external parquet files into `target/formal-ai-benchmarks`, which
is a build artifact cache rather than checked-in source.

## Issue #482 Nemotron Training-Data Samples

| Source | License | Upstream revision | Sampled rows | Download mode |
| --- | --- | --- | --- | --- |
| Nemotron Pretraining Legal v1 | CC-BY-4.0 | HF `3d91d58a5c0c46fe9944300ec46719f97a385b13` | 10 | Hugging Face datasets-server `rows`, `length=1` |

Canonical source:
<https://huggingface.co/datasets/nvidia/Nemotron-Pretraining-Legal-v1>.

The issue #482 suite vendors only compact metadata, SHA-256 digests, and short
excerpt previews for ten deterministic random rows. The sampler records row
offsets and provenance URLs in
`docs/case-studies/issue-482/raw-data/nemotron-random-samples.json` and never
downloads upstream parquet files or full splits.

## Issue #698 External (Upstream) Benchmark Harness

The issue #698 harness runs the *unmodified upstream* case set at run time. It
vendors nothing: every payload is fetched into `target/formal-ai-benchmarks`
(a build artifact) and only the honest `passed/total` score is committed, to
`data/benchmarks/external-results.lino`. Only permissively licensed suites are
fetched; the harness refuses to substitute a repository-local proxy for a suite
it may not or cannot fetch.

| Suite id | Source | License | Upstream revision | Download mode |
| --- | --- | --- | --- | --- |
| `humaneval` | [openai/human-eval](https://github.com/openai/human-eval) | MIT | `6d43fb980f9fee3c892a914eda09951f772ad10d` | gzipped JSONL over HTTPS |
| `mbpp` | [google-research/mbpp](https://github.com/google-research/google-research/tree/master/mbpp) | Apache-2.0 | `1fa17414f56c3703d5adb3818338b6e35e0fd550` | JSONL over HTTPS |
| `gsm8k` | [openai/grade-school-math](https://github.com/openai/grade-school-math) | MIT | `3101c7d5072418e28b9008a6636bde82a006892c` | JSONL over HTTPS |
| `math` | [openai/prm800k](https://github.com/openai/prm800k) 500-problem split | MIT | `7ecc794703b2877f63226f2477a49b34f9b25163` | JSONL over HTTPS (Git LFS media endpoint) |
| `object_counting` | [google/BIG-bench](https://github.com/google/BIG-bench/tree/main/bigbench/benchmark_tasks/object_counting) | Apache-2.0 | `092b196c1f8f14a54bbc62f24759d43bde46dd3b` | BIG-bench `task.json` over HTTPS |
| `coedit` | [grammarly/coedit](https://huggingface.co/datasets/grammarly/coedit) | Apache-2.0 | HF `e9a255c33ef910bc33a9d2b522653fa87521583e` | Hugging Face datasets-server `rows` |
| `swebench_lite` | [princeton-nlp/SWE-bench_Lite](https://huggingface.co/datasets/princeton-nlp/SWE-bench_Lite) dev split | MIT | HF `6ec7bb89b9342f664a54a6e0a6ea6501d3437cc2` | Hugging Face datasets-server `rows` |
| `editeval` | [facebookresearch/EditEval](https://github.com/facebookresearch/EditEval) | CC0-1.0 (harness code only) | `main` | **not fetched** — recorded as `benchmark_unavailable` |

License texts:

- HumanEval MIT: <https://raw.githubusercontent.com/openai/human-eval/6d43fb980f9fee3c892a914eda09951f772ad10d/LICENSE>
- MBPP Apache-2.0: <https://raw.githubusercontent.com/google-research/google-research/1fa17414f56c3703d5adb3818338b6e35e0fd550/LICENSE>
- GSM8K MIT: <https://raw.githubusercontent.com/openai/grade-school-math/3101c7d5072418e28b9008a6636bde82a006892c/LICENSE>
- MATH / prm800k MIT: <https://raw.githubusercontent.com/openai/prm800k/7ecc794703b2877f63226f2477a49b34f9b25163/LICENSE>
- BIG-bench Apache-2.0: <https://raw.githubusercontent.com/google/BIG-bench/092b196c1f8f14a54bbc62f24759d43bde46dd3b/LICENSE>
- CoEdIT Apache-2.0: <https://huggingface.co/datasets/grammarly/coedit>
- SWE-bench MIT: <https://raw.githubusercontent.com/SWE-bench/SWE-bench/main/LICENSE>
- EditEval CC0-1.0 (harness code only): <https://raw.githubusercontent.com/facebookresearch/EditEval/main/LICENSE>

**Why EditEval is not fetched.** The upstream repository ships an evaluation
harness with no task payload (`configs/dataset_paths.json` points at per-corpus
download directories), and its constituent corpora fail the permissive-only
policy: ASSET is CC BY-NC 4.0 and JFLEG is CC BY-NC-SA 4.0. The harness records
this as an explicit `benchmark_unavailable` entry with the reason and executes
the instructed-text-editing task family through the Apache-2.0 CoEdIT suite.
