## Summary

The weekly **External benchmarks** workflow has failed on `main` every week since 2026-09-28 because of a HumanEval ratchet violation:

```
Error: "ledger ratchet violated:\nhumaneval 2026-09-28: passed=12 is below the recorded minimum_pass_count=20"
```

This is a **true positive**. The solver lost 8 of the 20 ratcheted HumanEval cases between v0.351.0 and v0.352.1. Lowering `minimum_pass_count` would hide a real regression, so it was not lowered. This was found while auditing CI false positives and negatives in #1571 (PR #1572).

## Evidence (weekly runs of `external-benchmarks.yml` on `main`)

| Date | Run | Commit | humaneval |
|---|---|---|---|
| 2026-09-14 | 34830490180 | — | 1/20 |
| 2026-09-15 | — | 36c869226 "feat(coding): generalize benchmark discovery" sets `minimum_pass_count "20"` | — |
| 2026-09-21 | 35586279632 | efb6ddb60 (v0.351.0) | **20/20** |
| 2026-09-28 | 36412864432 | d209aac64 (v0.352.1) | **12/20** |
| 2026-10-05 … 2026-11-09 | 37303754028, 38246806902, 39234037390, 40305574816, 41445410045, … | — | 12/20 every week |

The same 8 cases fail every week:

- HumanEval/1, 3, 6, 7, 11, 17, 19: `answer contains no Python code`
- HumanEval/4: the emitted program fails `check(mean_absolute_deviation)` with a traceback

## Reproduction (no build needed)

Using the released static CLI v0.355.34 (`formal-ai-cli-x86_64-unknown-linux-musl`):

```bash
formal-ai benchmark run --suite humaneval --offset 3 --slice 1 --online --repository-root "$PWD"
# suite=humaneval passed=0 failed=1 total=1
# FAIL HumanEval/3 answer contains no Python code
```

The same command with `--offset 4` gives `FAIL HumanEval/4 answer contains no Python code` (locally the solver refuses; the CI run emitted a wrong program).

The answer text shows why. Run `formal-ai chat --thinking --prompt "<benchmark prompt for HumanEval/3>"`; the benchmark prompt is `Complete this Python function. Reply with the full implementation in a ```python code block.` followed by the `below_zero` stub:

```
Route to the write program skill gap handler.
I cannot write this program: no synthesis route reaches task "below_zero" in language "python".
I decomposed the request and tried every synthesis route I have, in order — catalog, blueprint_recipes, coding_oracle, seed_idiom_composer — and none of them derives it.
```

So none of the synthesis routes derives the task any more, and the honest skill-gap answer (#699) is returned.

## Bisect bounds

The regression lands between efb6ddb60 (v0.351.0, 20/20) and d209aac64 (v0.352.1, 12/20). The first-parent merges in that window are #1139 (worktree-issue-1138), #1140, #1141, #1144, #1146, #1148, #1152 and #1150.

Within #1139, the coding-synthesis rewrite is the strongest candidate:

- 9c31b0530 "land the codex continuation waves in one bulk state" deletes `src/coding/structural_composition.rs` (632 lines). It replaces `src/coding/composition.rs` and `src/coding/concept_discovery.rs` with `rust/src/coding/composition_search/*`, `program_ir*` and `ir_lowering/*`.
- c2d982ab5, 9b3b641a8, fd88a1745 and 4ab9e3e82 (2026-09-17 to 09-18) are follow-ups to the composition search.
- bcf1e8edb and e8533a522 (2026-09-20/21) change `data/seed/meanings-coding-structure.lino`, which is 518 lines smaller net across the window.

Only 300–400 MB desktop bundles are published for v0.351.0 and v0.352.1, so I did not bisect with binaries. Building each candidate commit would pin the exact commit.

## Expected

HumanEval slice 20 passes ≥ 20/20 again, through derivation rather than recited templates, so the ratchet holds. Alternatively, if a deliberate design change caused the drop, the ledger should record that decision explicitly instead of the workflow failing silently every week.
