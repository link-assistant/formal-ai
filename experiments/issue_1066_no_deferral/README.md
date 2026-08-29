# Issue #1066 — the removed deferral budget, falsified

`an_ineligible_cycle_is_blocked_from_the_first_push` in
`tests/unit/specification/self_hosting_metric.rs` claims that no threshold can be
reintroduced into the release gate without the suite noticing. A guard that has
never been observed failing is a claim, not evidence, so this directory holds the
run that made it fail on purpose.

## What the script does

`falsify-reintroduced-budget.sh` patches `scripts/self-development-loop.rs` in
place with the smallest possible version of the budget #1065 added — a seven-day
threshold past which the refusal grows extra text — runs the single test,
asserts it goes **red**, restores the file, and asserts it goes **green** again.
The file is restored by an `EXIT` trap, so an interrupted run does not leave the
policy patched.

```bash
experiments/issue_1066_no_deferral/falsify-reintroduced-budget.sh
```

## Observed, 2026-08-29

With the threshold reintroduced, the aged fixture's refusal diverges from the
fresh one and the test fails on that exact difference:

```text
thread '...an_ineligible_cycle_is_blocked_from_the_first_push' panicked at
tests/unit/specification/self_hosting_metric.rs:246:5:
  left: "release cycle v1.0.0..HEAD has no merged Formal AI-authored pull request; ...
         non-merge commit. This deferral has outlived its budget: 60 days"
 right: "release cycle v1.0.0..HEAD has no merged Formal AI-authored pull request; ...
         non-merge commit"
test result: FAILED. 0 passed; 1 failed
```

With the file restored:

```text
test result: ok. 1 passed; 0 failed
```

So the guard is load-bearing in both directions, and the aged fixture really does
age the cycle: the reintroduced threshold measured 60 days from the tagged
baseline, which is what the fixture sets.

## Why the fixture ages the tag and not `HEAD`

The removed `cycle_age_days` read `git log -1 --format=%ct <since>`, and `since`
is the last release tag. A fixture that back-dates `HEAD` therefore leaves the
cycle looking brand new and would pass against a reintroduced day budget for the
wrong reason. The aged fixture amends the baseline commit and force-moves
`v1.0.0` onto it instead.
