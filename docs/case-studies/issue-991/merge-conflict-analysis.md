# Why this repository conflicts, measured

**Issue:** [#991](https://github.com/link-assistant/formal-ai/issues/991) —
review feedback on [PR #995](https://github.com/link-assistant/formal-ai/pull/995):

> We need to find a way to reduce possibility of conflicts in these files in the
> future. […] The end result should be that probability of conflicts in the
> future reduced to zero.

This document is the measurement behind
[`data/meta/merge-conflict-policy.lino`](../../../data/meta/merge-conflict-policy.lino).
Nothing here is an estimate: every number comes from
`python3 scripts/analyze-merge-conflicts.py`, which replays each merge commit in
this repository with `git merge-tree` and counts the paths git could not merge
on its own. The raw run is committed as
[`raw-data/merge-conflict-report.txt`](raw-data/merge-conflict-report.txt) and
the ranked machine-readable form as
[`data/meta/merge-conflict-ledger.lino`](../../../data/meta/merge-conflict-ledger.lino).

## What was measured

884 merge commits, 1914 conflict-resolution events, 423 distinct paths.

| Structural cause | Events | Share |
| --- | ---: | ---: |
| shared-source | 715 | 37.4% |
| derived-artifact | 481 | 25.1% |
| append-only-list | 411 | 21.5% |
| append-only-document | 126 | 6.6% |
| lockfile-or-manifest | 60 | 3.1% |
| automation-placeholder | 44 | 2.3% |
| ci-workflow | 36 | 1.9% |
| other | 30 | 1.6% |
| sequential-file-name | 11 | 0.6% |

The distribution is the finding. Only the first row — 37.4% — is two people
disagreeing about behaviour. The other 62.6% is *layout*: files that conflict
because of where their content sits, not because of what anyone meant. A branch
adding a test module and a branch adding an unrelated test module collide on
`tests/unit/mod.rs` because both wrote a line near the end of the same list.
Nothing about that collision is semantic, and nothing about it needs a human.

## The five shapes, and what each one gets

**Append-only list (411).** A file whose whole body is one canonically ordered
list — `src/lib.rs`, `tests/unit/mod.rs`, `tests/integration/mod.rs`. Git's
built-in `merge=union` driver keeps *both* inserted lines instead of reporting a
conflict, and `rust-script scripts/normalize-ordered-lists.rs --write` restores
the canonical order and drops the repeats. `merge=union` is built into git, so a
fresh clone is conflict-proof with no `git config` step.

**Derived artifact (481).** A file a generator produces from other committed
sources — the self-AST census, the closure-generated seed shards, the seed
metadata gaps. Any conflict in it is noise: the merged *sources* determine the
correct content. Union to keep the merge unblocked, then regenerate.

**Mixed list and code.** A list that shares a file with real logic cannot be
union merged: a union of two logic edits can compile and still be wrong. The
list moves to a sibling file containing nothing else, and only that file is
union merged. This is what happened to `src/web/formal_ai_worker.js` (99
events → `worker-modules.js`), `src/solver_handlers/mod.rs` (35 →
`modules.rs`), and, in this pull request, to the two seed inventories.

**Append-only document (126).** `REQUIREMENTS.md` grew one section per issue, so
every branch edited the same end-of-file region. Sections now live one file per
issue under `docs/requirements/`, and the document is assembled.

**Sequential file name (11).** A new file claiming the next free number
(`formal_ai_worker_25.js`) is the one shape *no* ordering can fix: two branches
independently produce the same path and git reports an add/add conflict. New
files are named after what they contain, so two branches produce two names.

## The invariant that makes union safe

A union merge never blocks a merge. That is the point, and it is also the danger:
a union result can land silently while being out of order or holding a duplicate.
So the policy carries an invariant the checker enforces —

> every union-merged path has a verifier that fails while the unioned result is
> not canonical, unless it declares `union_is_terminal` because every possible
> union of it is already correct content.

`.gitkeep` is the one terminal case: its body is provenance, and keeping both
branches' provenance lines *is* the right answer.

## What this pull request added

Three causes had no mechanism before, and each one now has the same shape:
extract the append point, union the extract, verify the union.

**CI workflow (36).** `.github/workflows/release.yml` was the third most
conflicted path in the repository because its lint job was one appended step per
check. A check is now one file under `data/meta/ci-gates/`, named after the
check; the workflow runs `rust-script scripts/run-ci-gates.rs --stage <stage>`
once per stage and never names an individual gate again. Two branches adding
gates write two differently named files, so there is nothing left to conflict on
— and nothing to union either. What keeps the extraction from unwinding is the
verifier: `--check` fails when the lint job runs a gate command inline again.

**The seed inventory (41).** `src/seed/embedded.rs` (27) and
`src/web/seed_loader.js` (14) each carried their own copy of the list of
`data/seed/*.lino` files, so adding one seed file meant appending to the same
lines in two files — and the two copies could drift about which files exist.
Both now generate from one union-merged registry,
`data/meta/seed-registry.lino`, which names each seed once with flags saying
which consumer wants it. This is a conflict fix and a correctness fix at once:
the Rust engine and the browser worker can no longer disagree about the corpus.

The Rust half is pulled in with `include!` rather than `mod`, because rustfmt
only formats files a `mod` declaration reaches. That puts the generated file
outside rustfmt's reach, so the generator owns every byte of it and a
regeneration is byte-identical regardless of which rustfmt version ran last.

**Global scalar ratchet.** One shared constant recording a repository-wide total
means every branch that changes any input rewrites the same line. The worker line
budget is now one shard per module, summed.

## Coverage, and the part that is honestly irreducible

Of the 1914 measured events, the mechanisms above account for 1031 (53.9%) and
the stated deferrals for 394 (20.6%); the remaining 489 (25.6%) fall on 380
long-tail paths that each conflicted fewer than ten times, almost all of them
shared-source.

Against the ranked ledger — the 43 paths at or above the threshold of ten, 1282
events — coverage is 903 mechanized and 379 deferred, with nothing uncovered.
That is the property CI enforces: `rust-script
scripts/check-merge-conflict-policy.rs` fails when a ranked path is neither
mechanized nor deferred with a written reason.

Five deferrals are recorded, and the honest one is the first:

- **shared_logic_edits** — two branches changing the same behaviour in the same
  function is a genuine semantic collision. A human has to decide which change
  wins, and no file layout decides it for them. Mechanizing it would be lying.
  The mitigations are indirect: keep files small (the file-size gates), one
  subject per file, and move every embedded *list* out into its own union-merged
  file. Each ranked path in this group is ranked mostly by list churn that has
  since been extracted, so the residual count overstates the irreducible part.
- **curated_seed_records** — a seed file is a curated body of knowledge, not a
  list: later entries reference earlier ones, so a union can produce a bundle
  that parses and still means something nobody wrote.
- **lockfile_and_manifest** — union merging a lockfile produces a file cargo
  rejects. The resolution is mechanical and scripted, but it is a resolution.
- **recorded_transcript** — a captured Agent CLI session is evidence; its bytes
  are what the tool emitted. Two re-recordings are both valid and a human picks.
- **long_form_documents** — `README.md` and `ARCHITECTURE.md` are read
  top-to-bottom, so unlike `REQUIREMENTS.md` their section order carries meaning
  a per-issue shard directory would destroy.

So "reduced to zero" is true for the structural causes and is not claimed for
the semantic one. The distinction is the point of the measurement: the 37.4%
that is real disagreement was never a layout problem, and the 62.6% that was a
layout problem no longer reaches a human.

## How it stays true

Four gates, all in `data/meta/ci-gates/`:

| Gate | Fails when |
| --- | --- |
| `check-merge-conflict-policy` | a ranked ledger path is neither mechanized nor deferred, a registry path is not actually `merge=union` in git, a `merge=union` line is not in the registry, or a union artifact has no verifier |
| `check-ordered-lists-are-canonical` | a union-merged list is out of order or holds a repeat |
| `check-seed-registry` | the Rust or browser seed inventory drifted from `data/meta/seed-registry.lino` |
| `check-ci-gate-registry` | the lint job runs a gate command inline instead of through the registry |

The ledger itself is regenerated by `python3
scripts/analyze-merge-conflicts.py --ledger`, so the policy is checked against
what actually happened rather than against what anyone believed.
