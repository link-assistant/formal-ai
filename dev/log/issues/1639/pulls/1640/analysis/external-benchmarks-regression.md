# External Benchmarks (scheduled) — ledger-ratchet regression (issue #1639)

Workflow: `.github/workflows/external-benchmarks.yml` (weekly, Monday). The job
fails in the `Run upstream suites` step because `formal-ai benchmark run ... --append`
refuses to append a slice-20 row below the best row already recorded for the same
`(suite, slice)` (`rust/src/external_benchmarks/ratchet.rs:179`,
`historical_floor_violations`). Last failure (run 45197910485, main `03a636334`,
2026-11-23):

```
ledger ratchet violated: humaneval 2026-11-23: passed=12 is below the recorded minimum_pass_count=20
ledger ratchet violated: mbpp      2026-11-23: passed=10 is below the recorded minimum_pass_count=20
ledger ratchet violated: gsm8k     2026-11-23: passed=1  is below the recorded minimum_pass_count=2
```

Raw logs: `ci-logs/run-45197910485-failed.log`, older runs under
`/tmp/ci-art-1639/extbench/` (`run-35586279632.log` = last pre-#1139 run,
`run-36412864432.log` = first post-#1139 run, `summary.txt` = per-run score lines).

## 1. Timeline (scheduled runs)

| Date | Run | main commit | Result |
|---|---|---|---|
| 09-07 | 34105840056 | 6039c4d94 | success (humaneval 0, mbpp 0, gsm8k 2; floors were 0/0/2) |
| 09-14 | 34830490180 | de88ca251 | fail: humaneval 1/20 vs old floor rows (unrelated, earlier rows) |
| 09-15 | — (PR 36c869226) | — | ledger gets humaneval **20** (`runner ... --slice 20`, *offline*, no `mode` → parsed as online) and mbpp **20** (`--online`), version 0.349.2 |
| 09-21 | 35586279632 | efb6ddb60 (before #1139) | humaneval **20/20**; mbpp 18 (MBPP/5, /20 failed — live OEIS); `set -e` stopped there |
| 09-26 | — | bad87cddf | **merge of PR #1139** (issue-1138 bulk) |
| 09-28 | 36412864432 | d209aac64 (after #1139) | humaneval **12**, stopped |
| 10-05 | 37303754028 | d209aac64 | humaneval 12, stopped |
| 10-12 | 38246806902 | f943c9ba8 | humaneval 12, stopped |
| 10-19 | 39234037390 | fd5bfaced | humaneval 12, stopped |
| 10-26 | 40305574816 | c4137c0e7 | humaneval 12, stopped |
| 11-02 | 41445410045 | 658cd102a | humaneval 12, stopped |
| 11-09 | 42621251488 | 70935ad8e | humaneval 12, stopped |
| 11-16 | 43870207134 | b68d21647 | (issue #1569: all suites now run) humaneval 12, mbpp 11, gsm8k 1, math 0, object_counting 0, coedit 0, egg_math 20, ascent 5/5 |
| 11-23 | 45197910485 | 03a636334 | humaneval 12, mbpp 10, gsm8k 1, others unchanged |

Between 09-28 and 11-09 only HumanEval was measured (the step stopped at the first
violation), so the MBPP/GSM8K falls were invisible until #1569 (11-16).

## 2. Introducing commits

`efb6ddb60..d209aac64` has 231 commits; on the first-parent line the only
capability-bearing merge is **`bad87cddf` (PR #1139, issue #1138)**
(`git merge-base --is-ancestor bad87cddf efb6ddb60` → false; `... d209aac64` → true).
Inside it:

* **`9c31b0530`** (2026-09-17, "land the codex continuation waves in one bulk state"):
  deletes `src/coding/structural_composition.rs` (632 lines of authored structural
  drafts), replaced by the seed-driven composition search over
  `data/seed/coding-composition-fragments*.lino`. The plan's own pre-deletion
  baseline on 09-19 was already HumanEval 13 and MBPP 15, against 20/20 on 09-15
  (36c869226, an ancestor of the PR branch). So the larger part of the fall (20 → 13)
  happened in the PR branch's coding commits of 09-16..09-19. The most prominent of
  these is 9c31b0530; others are 6d864ebfd "pay for answers with vocabulary, not
  memory", c2d982ab5 and 4ab9e3e82. This part was **not bisected**, because local
  `cargo build` is not allowed here (3 GB cgroup).
* **`bcf1e8edb`** (2026-09-20, plan 02 leaf L15): deletes the seven
  algorithm-shaped templates from `data/seed/coding-discovery-runtime.lino`
  (aligned_binary_xor, explicit_value_mapping, explicit_ordering,
  regex_minimum_word_length, regex_lowercase_chunks, regex_lowercase_underscore,
  composite_number).

The fall was measured and *knowingly recorded* by the authors —
`docs/case-studies/issue-1138/plans/02-composition-from-sources.md:1293-1322`
("2026-09-19 L15 template-deletion control (fall recorded, leaf stays open)"):

> humaneval ... baseline `passed=13 failed=7`, after deletion `passed=12 failed=8`.
> HumanEval/19 (`sort_numbers`) fell ...
> mbpp ... baseline `passed=15 failed=5`, after deletion `passed=11 failed=9`. Four fell:
> MBPP/3 `is_not_prime` (composite_number), MBPP/7 `find_char_long`
> (regex_minimum_word_length), MBPP/15 `split_lowerstring` (regex_lowercase_chunks),
> MBPP/16 `text_lowercase_underscore` (regex_lowercase_underscore).
> Per plan 00 §9 X3 the falls are recorded and not repaired by restoring a template.
> The tick criterion "both controls hold" is not met while composition cannot derive regex
> construction, primality-by-divisor, or word→rank ordering as typed compositions; that
> generalization is the remaining L15 scope

L15 is still `- [ ]` (plan 02 line 1194). The same plan (line 1128) also says
"The slice-20 rows may not fall; a fall is a regression to fix in capability, never by
lowering the floor", and plan 00 §9 X3 (line 529) gates the deletion on the controls
staying green. So the deletion landed with its own gate unmet, and the ratchet —
which has no way to *record* a fall — has failed every week since.

**Not a network effect** (confirmed by the per-mode runs in §3). HumanEval's 20/20 was produced offline (09-15 ledger runner
has no `--online`) and online (09-21 CI). The post-#1139 HumanEval failures are
"answer contains no Python code" (HumanEval/1, 3, 6, 7, 11, 17, 19) and one Python
traceback (HumanEval/4) — composition finds no program; nothing is fetched.
MBPP's 20 *was* partly a network artifact: on CI before #1139 it was already 18
(MBPP/5 and /20 depend on a live OEIS lookup), so `minimum_pass_count 20` for MBPP
is not reproducible on CI even without #1139.

### Secondary defects seen in the failing cases

* **MBPP/7 answers in JavaScript** for a Python prompt (11-23 log line 3150):
  `function solution(input) {` → `SyntaxError: invalid syntax`. The language request
  ("Reply with the Python code in a ```python code block") is not honoured once the
  regex template is gone. The trace on main shows `coding_search: language=python ...
  surviving_programs=256` and `coding_composition: task=find_char_long needs=2`, with no
  verified Python program. The answer then comes from the general meta reasoner
  (`rust/src/meta_reasoner/integration.rs`: `resolve_impasse` / `try_meta_discovery`).
  Its only renderer is `Catalog::render` (`rust/src/meta_reasoner/catalog.rs:751`),
  which always emits `function solution(input) {` JavaScript, and nothing checks the
  requested programming language. Score-neutral (the case fails either way), but the
  answer is wrong.
* **GSM8K/0 (14 instead of 18).** The word-relations handler added 2026-10-07
  (`d2b6a7e68`/`3925784af`, issue #1176 R2,
  `rust/src/solver_handlers/statistics/word_relations.rs`) folds only digit tokens
  (16 − 2 = 14) and preempts `solver_synthesis::compose_remainder_sale`, which reads
  number words ("three", "four") and gives 18. GSM8K was 2/20 on every scheduled run up
  to 09-07; the two passing cases were GSM8K/0 and GSM8K/4 (run 34105840056 log, saved
  as `/tmp/ci-art-1639/extbench/run-34105840056.log`). On 11-16 and 11-23 it was 1/20,
  with only /4 passing.
* **GSM8K/4 regressed on current main.** On the main binary 0.357.0 (`513e4d9`),
  GSM8K/0 passes again, but GSM8K/4 ("Wendi feeds each of her chickens three cups ...
  15 cups ... 25 cups ... flock is 20 chickens") now fails. The cause is that the
  capability table routes it to `concept_measurement_lookup` and the answer comes back
  as `status "no_grounded_quantity"`. The score is still 1/20, now made up of a
  different case. `formal-ai chat --thinking` shows "Route to the concept measurement
  lookup handler". The decline guard in
  `rust/src/meta_method_dispatch.rs::try_capability_route`
  (`"compose_from_sources" | "concept_measurement_lookup"`) only steps aside for:
  - two units of one dimension (`names_arithmetic_quantity`);
  - a derivable unit conversion;
  - a calendar duration;
  - a statistic.

  A word problem that *states* the asked unit with numbers ("15 cups", "25 cups" →
  "how many cups") is not declined, although its answer is computed from the text, not
  looked up. This regression landed after `03a636334` (11-23). It was not bisected,
  because no build is available: ~800 commits separate `03a636334` and `513e4d9`.

## 3. Reproduction (prebuilt main binary 513e4d9, v0.357.0)

**Caveat — the prebuilt binary needs `FORMAL_AI_DATA_DIR`.**
`FragmentCatalog::bootstrap()` reads `bootstrap_seed_directory()`
(`rust/src/coding/fragment_catalog.rs:255`) =
`$FORMAL_AI_DATA_DIR` or `env!("CARGO_MANIFEST_DIR")/../data/seed`, i.e. the CI build
path `/home/runner/work/formal-ai/formal-ai/rust/../data/seed` baked in at compile time.
Outside that path the catalog is silently empty (`coding_search: ... structures=0
ranked=[]`) and *every* coding case fails (HumanEval offline 0/20). This is not the CI
cause (CI runs in that path), but it makes any local reproduction with a downloaded
binary misleading, and `--repository-root` does not influence it.

Setup (tree exported from main, no checkout):

```bash
git archive 513e4d954 data | tar -x -C /tmp/extbench-1639/main-data
mkdir -p /tmp/extbench-1639/root3 && ln -s /tmp/extbench-1639/main-data/data /tmp/extbench-1639/root3/data
# dataset cache copied to /tmp/extbench-1639/root3/target/formal-ai-benchmarks/*.jsonl
export FORMAL_AI_DATA_DIR=/tmp/extbench-1639/main-data/data/seed FORMAL_AI_LOG_LEVEL=trace
ulimit -v 1200000
# offline
FORMAL_AI_NETWORK=denied timeout 2400 formal-ai benchmark run --suite humaneval --slice 20 \
  --repository-root /tmp/extbench-1639/root3
# online (as in the workflow, without --append)
timeout 2400 formal-ai benchmark run --suite humaneval --slice 20 --online \
  --repository-root /tmp/extbench-1639/root3
```

Single-case checks:

* HumanEval/0 without `FORMAL_AI_DATA_DIR`: fails, `structures=0 ranked=[]`.
* HumanEval/0 with it: passes (`structures=3`, ranked `pair_destructuring_pattern,
  pairwise_distinct, predicate_abs_diff_lt, quantifier_any`, 256 candidate programs).
* GSM8K/0, `--online` and `FORMAL_AI_NETWORK=denied`: `passed=1/1` in both modes.

Slice-20 per mode on main 513e4d9:

| Suite | `FORMAL_AI_NETWORK=denied` (no `--online`) | `--online` | Ledger floor | Failing ids (identical in both modes) |
|---|---|---|---|---|
| humaneval | 12/20 | 12/20 | 20 | HumanEval/1, 3, 4, 6, 7, 11, 17, 19 |
| mbpp | 10/20 | 10/20 | 20 | MBPP/1, 5, 7, 9, 12, 13, 14, 15, 16, 20 |
| gsm8k | 1/20 | 1/20 | 2 | all except GSM8K/0 |

* HumanEval and MBPP reproduce the 11-23 CI failure exactly, with the same ids and the
  same MBPP/7 `function solution(input) {` JavaScript answer, so main is unchanged since
  then. The network makes **no** difference to any of the three scores. Online and
  offline differ only in the refusal text for GSM8K: "Fused 0 statement(s) from 0
  captured source(s)" online versus "I haven't learned ..." offline.
* MBPP/5 and MBPP/20, the live-OEIS cases that passed on the local 09-15 run, fail in
  both modes now as well.
* GSM8K: main answers GSM8K/0 (18), which failed on 11-23, but now fails GSM8K/4,
  which passed on 11-23. The total stays 1.

Raw outputs: `external-benchmarks-main-513e4d9/{humaneval,mbpp,gsm8k}-{online,offline}.txt`
(next to this file); trace logs stayed local (`/tmp/extbench-1639/logs/v-*/serve-*.lino`). Each run took 25-45 min at
`FORMAL_AI_LOG_LEVEL=trace`.

## 4. Root cause

1. **Capability removal (HumanEval, MBPP).** PR #1139 (`9c31b0530` deleted the
   structural drafts; `bcf1e8edb` deleted the seven templates) took away answers
   that the general composition search cannot yet derive (regex construction,
   primality by divisor, word→rank ordering, balanced-group parsing, MAD, ...). The
   authors measured and recorded the fall (plan 02 L15) but did not record it in the
   ledger, because the ledger cannot record a fall.
2. **Policy deadlock.** Plan 00 §9 X3 says to record the fall rather than restore
   the template. The ratchet (`historical_floor_violations`) rejects any slice-20 row
   below the best earlier row, so the fall cannot be recorded. The scheduled job will
   fail every week until capability returns to 20/20.
3. **Mode-blind ratchet.** Rows carry `mode` (bc3dd83c0), but
   `historical_floor_violations`, `best_pass_count` and `stagnant` group by
   `(suite, slice)` only. The HumanEval 20 floor comes from an *offline* run, the
   MBPP 20 floor from an *online* local run with live OEIS that CI never
   reproduced (18 on 09-21).
4. **GSM8K** 2 → 1 comes from routing regressions, not from the network. The score
   is identical online and offline.
   - Through 11-23, GSM8K/0 failed: word_relations (10-07) preempted the
     remainder-sale composition.
   - On current main, GSM8K/0 is answered, but GSM8K/4 fails: the
     `concept_measurement_lookup` capability route claims an arithmetic word
     problem (§2).
   - Both regressions are general routing defects in which a lookup/relation route
     wins over computing from stated quantities. Neither is a missing capability.

## 5. Fix

No solver change is made in this PR. Restoring HumanEval/MBPP to 20 honestly means
finishing plan 02 L15: deriving regex construction, primality-by-divisor and
word→rank ordering as typed compositions from the fragment catalog. That is a
multi-leaf capability project, cannot be verified here without `cargo build`, and
restoring the templates is explicitly forbidden by plan 00 §9 X3. Prompt-keyed
patches for the failing ids would violate AGENTS.md. The ratchet is **not**
lowered here.

Proposed changes (for the coordinator / maintainers; this agent did not apply
them because workflows, `main-only-regressions.lino` and the ledger floors are out
of scope):

1. **Make the fall recordable without lowering anything silently.** Add a reviewed
   *regression record* type to `data/benchmarks/external-results.lino`, e.g.
   `external_benchmark_regression_humaneval_2026_09_26` with `suite`, `slice`,
   `from 20`, `to 12`, `introduced_by bad87cddf (PR #1139, plan 02 L15)`,
   `evidence docs/case-studies/issue-1138/plans/02-composition-from-sources.md:1293`,
   `tracking <issue>`. `historical_floor_violations` would then reset the running floor
   at that record's date to `to`, and only for that `(suite, slice, mode)`. Every
   later row must still meet the new floor, and the record cannot be added without a
   cited commit and plan entry. Same for MBPP (20 → 10). GSM8K (2 → 1) should **not**
   get a record: its fall is two routing defects (proposal 6 and the word_relations
   number-word gap), which can be fixed in code.
2. **Key the ratchet on `(suite, slice, mode)`.** Change `historical_floor_violations`,
   `best_pass_count`/`check_floor` and `stagnant` to group by mode as well, and
   backfill `mode "offline"` on the 09-15 HumanEval row (its runner has no `--online`).
   Optionally have the workflow also run an offline slice-20 pass, which is the stable
   ratchet series (no OEIS/network variance). Online stays informational.
3. **Add a `main-only-regressions.lino` entry** `external-benchmarks-slice20-capability-fall`
   (runs 36412864432 … 45197910485, introduced by bad87cddf) next to the existing
   `external-benchmark-ratchet-discards-failure-report` entry.
4. **Seed path.** Resolve `bootstrap_seed_directory()` from `--repository-root` /
   the executable's location before the compile-time `CARGO_MANIFEST_DIR`, or fail
   loudly when the catalog is empty, so released binaries do not silently score 0.
5. **MBPP/7 language bug.** The meta reasoner should not take a turn (via
   `try_meta_answer`, `try_meta_discovery` or `resolve_impasse`) with a JavaScript
   program when the request names another target language. Use the language the
   coding request spec already resolves (the value traced as
   `coding_search: language=python`), and mirror the change in
   `js/worker/formal_ai_worker_meta_reasoner.js`. This is score-neutral, so it is a follow-up, not part of the ratchet fix.
6. **GSM8K/4 routing.** In `try_capability_route`, the `concept_measurement_lookup`
   row should also decline in one more case. The case is when the noun the quantity
   question counts is itself quantified by stated numbers in the prompt ("15 cups",
   "25 cups" → "how many cups"). "cup" is not a seed `measurement_unit`, so
   `names_arithmetic_quantity`, which needs two units of one dimension from that role,
   cannot see it. The interrogative comes from the routing vocabulary that already
   selects `quantity_question`, and the numbers come from digits plus the seed role
   `cardinal_number_word`, so no prompt literals are needed. Add a Rust regression
   test with an invented word problem (not the GSM8K text), plus the JS twin. This is not applied
   here: the same block is being edited concurrently by another #1639 change (unit
   conversions), and without a build the change cannot be verified to restore an
   answer of 20 rather than another misroute.
7. **Capability (real fix).** Finish plan 02 L15. Its tick criterion is that both
   controls return to their pre-deletion values with typed compositions. When it
   lands, the regression records from (1) are superseded and the floor returns to 20.
   The same applies to GSM8K: fixing both GSM8K/0 and GSM8K/4 routing would give 2/20
   again, which meets the current floor without any ledger change.

## 6. Verification status

* Root cause: **verified** from the ledger, the CI logs of all 12 scheduled runs, git
  ancestry, and the authors' own control measurements in plan 02.
* Current-main measurement: **done**, all three suites in both modes (§3). The ratchet
  would still fail on main today for humaneval (12 < 20), mbpp (10 < 20) and gsm8k
  (1 < 2), whatever the network setting.
* Fix: **not applied** (blocked by policy and scope, see §5). The scheduled workflow will
  keep failing until proposal 1 (or the capability work of 7) lands for
  HumanEval/MBPP **and** GSM8K regains 2/20 (proposal 6).
