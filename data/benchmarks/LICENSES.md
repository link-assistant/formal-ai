# Benchmark License Provenance

This directory imports a reviewable benchmark slice for issues #304 and #317.
The `.lino` fixture records the exact upstream revision, source path, and
license URL for every source. Issue #317 adds self-authored held-out variants
that preserve the licensed source task structure while changing wording,
numbers, or distractors.

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
