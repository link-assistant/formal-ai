# Defect D8 -- the evidence gate had no way out on a branch that cannot be rewritten

`Self-Hosting Evidence Check` (`.github/workflows/release.yml:100-124`) runs
`scripts/self-hosting-metric.rs` over `merge_base..HEAD` on every pull request
under `EvidencePolicy::Strict`. A commit that carries exactly one of
`Formal-AI-Session` / `Formal-AI-Evidence` is a hard error, on the stated
premise that a commit which is still in review can still be amended.

That premise is false in this repository.

## The failure

Run [34075819541](https://github.com/link-assistant/formal-ai/actions/runs/34075819541),
job 101601575998, at `ef3bb4aff`. Full log in
`../ci-logs/self-hosting-evidence-101601575998.log`; line 1570 is the verdict:

```
self-hosting metric error: commit bd511432a6f9aa83446c155ff94dbff009403dae must record both Formal-AI-Session and Formal-AI-Evidence
```

A **true positive**. `git show -s --format=%B bd511432a` carries
`Formal-AI-Evidence` and `Formal-AI-Pull-Request` and genuinely no
`Formal-AI-Session`, so it claims authorship it cannot prove. It is my own
mistake, made earlier in this pull request. The gate is right.

It is not the only one. The walk is `rev-list --reverse` and it stops at the
first error, so `bd511432a` hid a second: `265d1339580d4c4ec9cb8f2d206d10c882449304`
records `Formal-AI-Pull-Request` with neither evidence trailer, which
`commit_has_formal_ai_evidence` also rejects -- a commit may be silently
unattributed, but it may not name a pull request while proving nothing. Two
commits in this range are malformed, and the second only became visible after
the first was withdrawn.

## Why the prescribed remedy does not exist

The message the check implies -- amend the commit -- cannot be carried out:

```
$ gh api repos/link-assistant/formal-ai/rulesets/21300712 \
    --jq '{target,enforcement,conditions,bypass:.bypass_actors,rules:[.rules[].type]}'
{"bypass":[],"conditions":{"ref_name":{"exclude":[],"include":["~ALL"]}},
 "enforcement":"active","rules":["deletion","non_fast_forward"],"target":"branch"}
```

`~ALL` with an empty `bypass_actors` means no branch in this repository accepts
a non-fast-forward push from anyone. Confirmed by doing it: a `git filter-branch`
rewrite that produced a tree-identical history passing the metric locally
(`0.00% (0/52776 changed lines; 0/14 commits)`) was rejected on push --

```
remote: error: GH013: Repository rule violations found for refs/heads/issue-1079-92eb3dd8a0fa.
remote: - Cannot force-push to this branch
```

A pushed commit message is therefore permanent, and a strict gate keyed to
commit-message content is a gate whose only remedy is unavailable. That is the
same deadlock shape as issues #796 / #810 / #812, which
`a_malformed_historical_evidence_record_cannot_deadlock_a_release` already
removed from the *release* path -- it survived on the pull-request path only
because nobody had yet mis-trailered a commit there.

Classing it: this is a **true positive that is nevertheless a CI/CD defect**.
The finding is correct; the check is unactionable. Under R1 both halves count.

## Fix: a strictly one-directional retraction trailer

```text
Formal-AI-Retract: <full 40-character sha of the commit being withdrawn>
```

A later commit in the same measured range withdraws the earlier claim.
`retracted_commits` in `scripts/self-hosting-retraction.rs` collects them before
the walk; a retracted commit is not attributed, and is not consulted for
evidence at all, so a malformed trailer on it can no longer fail the run.

The mechanism is designed so that it cannot be used to cheat, and the design is
what makes it safe to add to a gate that exists to keep a number honest:

* it only ever moves a commit **out** of the numerator. There is no trailer
  that moves one in without the session evidence that has always been
  required, so the measured share can never rise because of a retraction;
* the target must be a full 40-character sha (a short sha or a branch name is
  an error, not a silent miss);
* the target must be inside the range being measured -- a retraction withdraws
  a claim made in the same range, it cannot reach into history;
* a commit may not retract itself;
* malformedness follows the existing policy split: `Strict` (the PR gate)
  errors, `Lenient` (release recording) warns and ignores, so a bad retraction
  cannot deadlock a release either. `Lenient` drops **only** the trailer it
  cannot resolve, never the whole set: a release range that begins after a
  retraction's target makes that trailer permanently stale, and discarding its
  siblings along with it would put the commits they withdraw back into the
  numerator -- the one direction a retraction must never move in.

It is applied on **both** walks -- `measure_with_policy` in
`scripts/self-hosting-metric.rs` and `merged_self_authored_pull_requests` in
`scripts/self-development-loop.rs`, both reading the one implementation in
`scripts/self-hosting-retraction.rs`. Applying it on one only would let a commit
sit outside the measured share while still counting toward the release floor,
which is the arbitrage the floor exists to prevent.

## Tests

`tests/unit/specification/self_hosting_metric/retraction.rs`:

| test | pins |
| --- | --- |
| `a_retraction_unblocks_a_branch_whose_history_cannot_be_rewritten` | the reported failure: a half-trailered commit plus a retraction passes `Strict` |
| `a_retraction_can_only_lower_the_measured_share` | the measured share after a retraction is <= the share before it |
| `a_retraction_also_withdraws_the_commit_from_the_release_floor` | the same commit stops counting in `self_development_release_status`, which reports `Blocked` |
| `a_retraction_must_name_a_full_sha_inside_the_measured_range` | a short sha and an out-of-range sha are both errors |
| `a_stale_retraction_does_not_restore_the_claims_the_others_withdraw` | a `Lenient` reader drops the unresolvable trailer alone, and the commit its sibling withdraws stays out of the numerator |
| `an_indented_example_of_a_trailer_is_not_a_trailer` | an indented `Formal-AI-*` line in a commit message is prose, matching `git interpret-trailers --parse` |

## Applied to this pull request

The commit that lands this mechanism carries a retraction for each of the two
mis-trailered commits:

```text
Formal-AI-Retract: bd511432a6f9aa83446c155ff94dbff009403dae
Formal-AI-Retract: 265d1339580d4c4ec9cb8f2d206d10c882449304
```

That is the intended use: both commits stay in history, stay in the
denominator, and no longer claim authorship they cannot prove. This pull
request therefore records no Formal AI attribution at all, which is the honest
outcome -- it disowns a claim, it does not make one.

## D9, found by using the fix

The first commit carrying a retraction also documented the trailer's format in
its own message, as an indented example. The gate answered:

```
self-hosting metric error: Formal-AI-Retract in commit f7d9ccd76... must name a full 40-character sha, found <full 40-character sha>
```

`trailer_values` `trim()`s every line before matching the key, so an indented
line counts as a declaration. Git does not agree:

```
$ printf 'subject\n\nbody\n\n    Formal-AI-Retract: abc\n' | git interpret-trailers --parse
$
```

Nothing. The parser deliberately scans the whole body rather than using git's
`%(trailers)` placeholder -- issue #796, where a blank line between two
trailers hid one of them -- but that reason never applied to indentation. It
now skips lines beginning with a space or a tab, so documenting a trailer is
no longer indistinguishable from using one, and the blank-line tolerance #796
needs is untouched. Pinned by
`self_hosting_metric::retraction::an_indented_example_of_a_trailer_is_not_a_trailer`.

## Two more things the fix ran into

**A stale retraction is the normal case, not the exceptional one.** The first
draft treated a retraction that no longer resolves as fatal to the whole set:
under `Lenient` it warned and then dropped *every* retraction in the range.
That is backwards. A release range begins at the previous tag, so as soon as a
retraction's target falls behind that tag the trailer stops resolving --
permanently, because history cannot be rewritten. Dropping its siblings with it
would put the commits *they* withdraw back into the numerator, which is the one
direction a retraction must never move in. `Lenient` now drops only the trailer
it cannot resolve. `a_stale_retraction_does_not_restore_the_claims_the_others_withdraw`
fails with `left: 1, right: 0` against the first draft and passes against the
second.

**The fix crossed the file-size ceiling.** `check_file_size` failed the
`Lint and Format Check` job at `087d5fb9b`:

```
tests/unit/specification/self_hosting_metric.rs: 1118 lines (exceeds Rust limit of 1000)
scripts/self-hosting-metric.rs: 1021 lines (exceeds Rust limit of 1000)
```

A correct report about this branch, not a pipeline defect -- the gate caught
exactly what it exists to catch. The retraction logic moved to
`scripts/self-hosting-retraction.rs` (included with `#[path]`, the same way
`self-development-loop.rs` already is, so `rust-script` still runs the metric
from one entry point) and its tests to
`tests/unit/specification/self_hosting_metric/retraction.rs`, beside the
`authorship_composition` submodule that was already there. Both files are back
under the ceiling: 960 and 898 lines.
