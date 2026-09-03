# Issue #1069 — is the release cycle's denominator measuring the right thing?

**Result: no route to a release here. The change this directory measures is a
ratchet reduction, and issue #1069 forbids it. Recorded so it is not retried.**

## The observation that started it

`scripts/self-hosting-metric.rs` states its own intent in its module doc:

> Changed lines are additions plus deletions reported by `git show --numstat`;
> merge commits, binary files and captured artifacts do not contribute.

`METRIC_VERSION = 2` implements "captured artifacts" as a list of *extensions*
(`log`, `jsonl`, `diff`, `patch`, `stderr`, `stdout`) plus lockfile names. That
catches a CI log. It does not catch the `.md`, `.json`, `.ts` and `.js` files
sitting beside that log inside the same evidence bundle, nor the vendored
upstream templates checked into `dev/log/<issue>/pulls/<pr>/references/`.

In the open cycle the difference is not marginal. `replay.py` measures it:

| | v2 changed lines | with recorded history removed |
| --- | ---: | ---: |
| `v0.345.0..HEAD` | 423 418 | 92 146 |

**78.2% of the open cycle's denominator is recorded history** — the trees
CONTRIBUTING groups together as exempt from live policy:

> Recorded history under `docs/case-studies/`, `dev/log/`, and `experiments/`
> is exempt — a past run stays as it happened.

The single largest contributor is `dev/log/issues/1014/pulls/1015/raw-data/` at
51 580 lines, followed by 28 968 lines of a vendored third-party pipeline
template under `dev/log/issues/1012/pulls/1013/references/`. Nobody authored
those lines in this repository, and Formal AI cannot author them.

That looks like the issue #812 defect recurring one directory over: the
published share describing archive volume rather than authored work.

## Why it is nevertheless not a fix to make

The tempting conclusion is that a location-aware `METRIC_VERSION = 3` would be a
bug fix rather than a relaxation. Replaying every ledger cycle under both
definitions shows it is not:

    trailing target under v2 (last 3 recorded cycles, replayed): 12.76% (2255/17664)
    trailing target under v3 (last 3 recorded cycles, replayed):  1.66% (194/11685)

    authored lines the open cycle still needs under v2: 59685
    authored lines the open cycle still needs under v3:  1418

The exclusion does not only shrink the denominator. It shrinks the *numerator*
by more, because the historical attributed commits earned most of their credit
from exactly these trees — an attributed commit that files a large evidence
bundle counts every line of that bundle as self-authored. `v0.345.0` reads
17.73% under v2 and 1.98% under v3 for that reason.

So adopting v3 would carry a **1.66%** floor forward instead of **12.77%**, and
would reduce the open cycle's remaining obligation from 59 685 authored lines to
1 418 — a 98% reduction in the work required to ship. Whatever it is called in
the commit message, that is the second forbidden resolution in issue #1069:

> lowering, widening, or removing the ratchet target.

`target_from_rows` makes the trap sharper still. It selects rows by
`metric_version == METRIC_VERSION`, so merely bumping the constant without
rewriting every historical row would make `target_from_rows` return **0** and
the ratchet vacuous. A definition change that does not replay history is not a
redefinition at all; it is a reset.

## What the number actually is

Under the committed definition, and with a qualifying merged pull request
already in the cycle, the open cycle needs **59 685 further attributed lines**
before `Auto Release` can go green. `experiments/issue_1066_qualifying_pr`
measures the other half of the same gate:

    self-hosting target would fall from 12.77% to 0.89% for v0.345.0..HEAD

Both gates block. Gate 1 (a merged Formal AI-authored pull request) is the one
whose message surfaces first, but clearing it alone does not clear gate 2.

## Running it

    python3 experiments/issue_1069_denominator/replay.py

Roughly two minutes; it shells out to `git show` once per commit per cycle.
Nothing is written and nothing is committed — it only measures.
