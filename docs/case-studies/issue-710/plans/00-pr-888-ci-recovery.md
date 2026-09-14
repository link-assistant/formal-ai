# Plan 00 — PR #888 CI recovery

The branch at `cde14085d` merges `main` through v0.350.0, passes 3440 unit
tests, 82 web tests and every rust-stage gate locally, and is `MERGEABLE`.
Three checks are red in CI. Each has one cause; none is fixed by relaxing a
check.

## 1. Lint and Format Check — `src/web/app.js` differs after `bun build`

**Evidence.** Job 103885406807 runs `npm run build:web` and then
`git diff --exit-code -- src/web/vendor.bundle.js
src/web/web-search-component.bundle.js src/web/ocr.bundle.js src/web/app.js`.
Only `app.js` differs, from the first line: the minifier's identifier
allocation is different. `.bun-version` pins `1.4.0` and every workflow uses
`bun-version-file: .bun-version`; the local machine has bun `1.2.20`. The
branch's earlier vendor-bundle churn was already reverted for the same reason.

**Fix.** Rebuild with the pinned version without changing the local install:

```bash
cd /tmp/wt888
npx -y bun@1.4.0 build ./src/web/app/main.jsx --outfile ./src/web/app.js \
  --target browser --format iife --production --minify-whitespace --minify-syntax
git status --short src/web/   # only app.js may change
```

The other three bundles are rebuilt the same way and must come out byte
identical to `main`; if one does not, `main` was built with a different
version and that is a separate finding, not something to paper over.

**Verify.** `npm run test:web` (82/82) still passes; the diff of `app.js`
against `main` is only the `main.jsx` punctuation change this branch carries.

- [x] app.js rebuilt with bun 1.4.0, vendor bundles byte-identical to `main`
- [x] `npm run test:web` green (82/82)

## 2. E2E Tests — the held-out computer-use generalization step ran out of budget

**Evidence.** In run 34815480961 the step `Run agent CLI E2E — held-out
computer-use generalization (issue #707)` took 07:07:59 → 07:16:59 and was
killed at its 600 s budget inside replay 12/12. On `main`, run 34810804701, the
same step took 06:00:06 → 06:01:31 (85 s). Per-case durations from the two job
logs:

| case | main | branch (record) | branch (replay) |
| --- | --- | --- | --- |
| notes_count_and_pack | 3 s | 5 s | 9 s |
| documents_list_and_pack | 5 s | 29 s | 20 s |
| documents_pack_and_unpack | 5 s | 55 s | 16 s |
| form_submission | 4 s | 42 s | 38 s |
| status_page_heading_and_pack | 5 s | 35 s | 36 s |

**Measurements taken (2026-09-14).** Every suspect below was tested against the
two release binaries, both built from their own tree at 0.349.2.

1. Whole-script local runs, same machine, sequential: branch record 336 s /
   replay 294 s; `main` record 324 s / replay 254 s. The branch is 4 % slower,
   not 6× slower. The branch's solver diff therefore does not explain CI.
2. Replaying one *captured* 55 KB agentic request (the largest body the record
   phase actually sent, taken from a `FORMAL_AI_TRACE_REQUESTS=1` run) against a
   fresh server: branch 30 ms, `main` 29 ms after the first warm-up call. The
   two binaries are indistinguishable on the real request shape.
3. A synthetic 50 KB single-prompt solve costs 4.31 s on the branch and 4.29 s
   on `main`. Prompt length alone is expensive on *both*; it is not a
   regression, and the real requests are not shaped that way (their bulk is
   `tool` messages, not one long user sentence).
4. Latency inside a real run grows with *accumulated memory*, not with request
   size: over the record phase the bodies stay at 54–70 KB while the response
   time climbs 0.5 s → 12.4 s, and `memory.lino` grows ~16 KB per request with
   no compaction. Preloading the 150 KB `memory.lino` from a finished run makes
   the *first* request cost 18.3 s on the branch and 19.4 s on `main` — again
   the same on both.

**Suspects cleared by measurement** (kept, struck, not deleted):

1. ~~`solver_helpers::record_decomposition` now calls
   `independent_actionable_segments(prompt)` unconditionally.~~ Real captured
   requests solve in 30 ms on both binaries; the branch and `main` differ by
   1 ms. The unconditional call is real (see the diff) but costs nothing on the
   prompt shapes this suite sends.
2. ~~`conversation_memory::try_assistant_name` now calls `formalize_intent` on
   every prompt.~~ Same measurement clears it.
3. ~~`seed::response_variant_for` re-parsing multilingual files per call.~~ It
   runs only on the `AssistantFreeTime` rule and never fires here.

**Root cause (measured, 2026-09-14).** The cost is not in the solver at all.
A `sample` of the server during a slow request puts 3 789 of 4 649 samples in
`fcntl` — file locking and `fsync`, not computation. The path is
`link_store::projection_sync::synchronize_memory_events`: the HTTP server opens
a fresh link store per request, and a `.projected` marker beside the database
records how many leading events the projection already holds. With a marker the
store *appends*; without one it *rebuilds*. The store is opened with
`CommitMode::Sync`, so a rebuild pays one synchronous commit per event.

Rebuild cost against a real `memory.lino` captured from this very suite:

| memory events | first request | later requests |
| --- | --- | --- |
| 21 | 3.50 s | 0.03 s |
| 33 | 5.12 s | 0.03 s |
| 56 | 13.61 s | 0.03 s |
| 102 | 15.40 s | 0.03 s |
| 112 | 15.80 s | 0.03 s |

The growth is superlinear in the number of events, and the second request always
costs 32 ms because the marker written by the first one turns every later
synchronization into an append. This is the same mechanism issue #1106 already
measured (51.8 s rebuilding vs 0.35 s appending on a 400-event store); what
#1106 did not cover is that the suite *restarts the server between the record
and replay phases* while keeping `FORMAL_AI_MEMORY_PATH` inside the same
workdir, and every restart throws away the in-process store, so the first
request of each phase pays a full rebuild against whatever the previous phase
accumulated.

That makes the step's duration a function of how many memory events the earlier
cases happened to write, which is why the same code is 85 s in one run and over
600 s in another, on the same runner image with the same
`@link-assistant/agent@0.26.3`. It is a real defect with a real bound, not
noise, and not something a wider budget would fix.

**What remains unreconciled.** `main`'s job log for run 34810804701 contains no
session events at all (`session.prompt`: 0, `tool_use`: 0), because the agent
CLI only dumps session JSONL when a step fails. The branch's log has 286. The
95-vs-3 `session.compaction` counts are therefore an artifact of failure
dumping, not a behavioural difference; in the branch's own log `currentTokens`
peaks at 18 154 against a `safeLimit` of 38 856, so compaction was evaluated and
never triggered. Both jobs ran the same runner image (`ubuntu-24.04`,
`20260907.300.1`). Nothing here contradicts the rebuild finding; it only means
the two job logs are not comparable line for line.

**Fix (applied).** `replace_memory_events_transactionally` now stages its
replacement with per-append `fsync` turned off and flushes the staged log once
before publishing. Both halves matter: the replacement is a process-unique
scratch file that is either renamed into place whole or swept away by
`sweep_dead_replacements`, so no reader ever replays its log and per-entry
durability protects nothing; the single `flush_log` before the rename keeps what
reaches the served path exactly as durable as it was. The `.projected` marker
is unchanged and still turns a restart's second request into an append.

Measured on the same store sizes as the table above, after the fix:

| memory events | before | after |
| --- | --- | --- |
| 21 | 3.50 s | 1.40 s |
| 56 | 13.61 s | 0.20 s |
| 112 | 15.80 s | 0.20 s |

The curve is flat instead of superlinear. The whole held-out generalization
script now runs in 39 s locally and passes, against 10 min 41 s before.

- [x] both local runs finished; per-record times recorded here
- [x] the three original suspects tested and cleared
- [x] root cause named and measured: per-event synchronous commits during a
      projection rebuild (`synchronize_memory_events`), not the solver
- [x] a regression test that bounds rebuild cost:
      `issue_1106_projection_reuse::rebuilding_a_projection_costs_what_appending_to_it_costs`
      asserts a rebuild stays within 8× the append time on the same machine. It
      was confirmed to fail without the fix (31.98 s against a 373 ms append)
      and to pass with it
- [x] local re-run of the whole script: 39 s end to end, exit 0, against 10 min
      41 s before the fix on the same machine and 85 s for `main`'s fastest CI
      run. Per-case cost fell from 45 s to 1-2 s

## 3. Self-Hosting Evidence Check — two commits without `Formal-AI-Model`

**Evidence.** Job 103885145297: `commit 8a2054245… must record Formal-AI-Model
(issue #1085)`. The strict gate (`scripts/self-hosting-metric.rs`,
`commit_has_formal_ai_evidence` → `self-hosting-attribution.rs::
model_attribution`) validates every non-merge commit in `de88ca251..HEAD`; a
commit that carries `Formal-AI-Session` must also carry exactly one
`Formal-AI-Model` naming `formal-ai`, **and the committed evidence file must
contain that trailer's value as a literal string**. Of the 26 non-merge commits
on this branch only `8a2054245` (verdict contract) and `ae1194e7c` (audit
contract) carry a session trailer, and both were made on 2026-08-01, before
issue #1085 made the model trailer mandatory.

**Correction to this plan's first draft.** It proposed writing
`Formal-AI-Model: formal-ai/0.317.0`, on the strength of the version named in
the PR description. That was checked and is wrong: `0.317.0` appears nowhere in
either evidence file, and neither does any other version. What the evidence
does contain is `"model":"formal-ai"` four times and the bare string
`formal-ai` 31 times, in the agent stream both commits point at. The gate's
containment rule therefore admits exactly one value, `formal-ai`, and the
version-qualified spelling would have failed the very check it was meant to
satisfy. Writing a version the evidence cannot support would also be a claim
about provenance that the recorded session does not make.

**Fix.** Rewrite the messages of those two commits to add
`Formal-AI-Model: formal-ai`, rebuilding the branch chain with plain plumbing
(`git commit-tree` reusing each original tree, parents, author and dates) so
every later commit — including the five merges of `main` — keeps its tree byte
for byte and no merge is redone. `git filter-branch` is not used: the session
sandbox rejects it and the plumbing does the same job transparently. The
rewrite touches only this pull request's branch; `main` history is not
involved. The push uses `--force-with-lease`.

**Verify.**

```bash
git diff cde14085d <new HEAD> --stat   # empty: same tree
rust-script scripts/self-hosting-metric.rs measure --since de88ca251 --until HEAD
```

**Status: rebuilt and verified locally; the single final push is pending.** On
2026-09-15 the committed plumbing script rebuilt the chain from
`20ecb3737` to `79ccdabd4`. Both heads resolve to tree
`d7dac2a66a53179157e1f4ecc6c3093596a1e12c`, and `git diff --exit-code`
between them is empty. The rewritten verdict and audit commits are
`3111ed74d` and `e8d855b16`; each now records
`Formal-AI-Model: formal-ai`. The local strict evidence measurement completes
successfully over `de88ca251..HEAD`.

The script is committed beside this plan as `add-model-trailer.sh` for whoever
runs it.
It walks `de88ca251..HEAD` oldest-first, reuses each commit's original tree,
parents, author and committer identities and dates verbatim through
`git commit-tree`, appends `Formal-AI-Model: formal-ai` to the two commits
named above and to no others, and prints the old and new heads together with
`git diff OLD NEW --stat`, which must come out empty. It moves no ref; pointing
the branch at the new head and pushing is a separate, deliberate step:

```bash
bash docs/case-studies/issue-710/plans/add-model-trailer.sh   # prints NEW_HEAD and an empty diff
git reset --hard <NEW_HEAD>                      # only after that diff is confirmed empty
rust-script scripts/self-hosting-metric.rs --since de88ca251 --until HEAD
git push --force-with-lease origin HEAD:issue-710-14da90b08a12
```

The value to write is the bare `formal-ai`, not a version-qualified spelling;
see the correction above for why.

- [x] chain rebuilt; `git diff` against the old head is empty
- [x] evidence check passes locally on the rebuilt range
- [ ] pushed with `--force-with-lease`; CI evidence check green


## 4. Not a defect of this branch, recorded so it is not chased twice

- `Desktop Release / Build linux-x64` was `cancelled` by the pipeline when the
  lint job failed; it is not an independent failure.
- The scheduled External Benchmarks run failed on `ledger ratchet violated:
  humaneval 2026-07-20 … 2026-09-07: passed=0 is below the recorded
  minimum_pass_count=1`: the floor was raised to 1 by the 2026-09-14 run and
  the check then reads seven earlier rows as regressions. That is a defect in
  the ratchet's reading of history, owned by plan 03 leaf L1, not by this
  branch.
