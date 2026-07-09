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

## License Texts

The upstream license files are canonical:

- HumanEval MIT license: <https://raw.githubusercontent.com/openai/human-eval/6d43fb980f9fee3c892a914eda09951f772ad10d/LICENSE>
- MBPP Apache-2.0 license: <https://raw.githubusercontent.com/google-research/google-research/1fa17414f56c3703d5adb3818338b6e35e0fd550/LICENSE>
- GSM8K MIT license: <https://raw.githubusercontent.com/openai/grade-school-math/3101c7d5072418e28b9008a6636bde82a006892c/LICENSE>
- MATH MIT license: <https://raw.githubusercontent.com/hendrycks/math/985bdc1696e88e8643f081a0ff4719da39f2ae2a/LICENSE>
- BIG-bench Apache-2.0 license: <https://raw.githubusercontent.com/google/BIG-bench/092b196c1f8f14a54bbc62f24759d43bde46dd3b/LICENSE>

Only five upstream task prompts, five self-authored held-out variants, and
their expected deterministic checks are vendored here. Canonical solutions and
full external datasets are intentionally not copied into the repository. The
benchmark suite records `minimum_pass_count "10"` so CI fails if the current
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
