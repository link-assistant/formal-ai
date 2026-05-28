# Issue 244 / 304 / 317 Benchmark Dataset Research

Date: 2026-05-26

Issue #304 asked for a permissively licensed benchmark slice that covers
programming, math, and general problem-solving. Issue #317 grows that slice
with held-out variants and a pass-count ratchet. This note records the online
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

## Issue #317 Expansion

| Source | Held-out variant | Anti-memorization change |
| --- | --- | --- |
| HumanEval | `humaneval_0_has_close_elements_paraphrase` | Rewords the function specification while preserving the threshold-pair relation. |
| MBPP | `mbpp_2_similar_elements_paraphrase` | Rewords the tuple-intersection task and requires deterministic sorted tuple output. |
| GSM8K | `gsm8k_held_out_hen_eggs` | Renumbers the egg-sale word problem and changes entities so the old answer `18` is wrong. |
| MATH | `math_held_out_algebra_substitution` | Changes assignments and expression so substitution must recompute `17`. |
| BIG-bench `object_counting` | `bigbench_held_out_object_counting_instruments` | Adds a utensil distractor so category filtering, not list length, determines the answer. |

The fixture records `minimum_pass_count "10"`. The benchmark test reports the
full pass/fail breakdown and fails CI when the derived pass count drops below
that recorded floor, while individual cases still retain
`allow_current_failure "true"` for future unsolved expansion cases.

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
  dataset dumps. The repository vendors only five upstream task prompts, five
  self-authored held-out variants, and expected checks so the benchmark is
  reviewable and the full datasets remain upstream.

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

cargo test --test unit issue_304_benchmark_suite_reports_pass_fail_counts -- --nocapture
cargo test --test unit issue_317_held_out_benchmark_variants_pass_by_derivation
```
