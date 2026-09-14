# Issue #1066 — what the release gate still refuses (#1066, item 4)

Acceptance item 3 asks for a *merged* pull request in the open release cycle
whose every introduced non-merge commit is validly attributed. No branch can
satisfy it: the merge is the thing being asked for, and CONTRIBUTING is explicit
that the attribution trailers must not be added to a commit that was not
authored through the loop. Item 4 — `Auto Release` going green — is stated as
following from item 3.

"Follows from" is a claim, and this directory measures it instead of assuming
it. `dry-run.sh` builds the missing pull request in a throwaway clone: an
authored commit carrying the three canonical trailers, with the session id read
out of the committed evidence bundle rather than pasted, merged with a `--no-ff`
merge whose subject matches the release scripts' pull-request pattern. It then
runs the two gates against that clone.

Nothing is pushed, nothing is committed here, the pull request number named in
the trailer does not exist, and the clone is deleted on the way out.

    experiments/issue_1066_qualifying_pr/dry-run.sh
    AUTHORED_LINES=40000 experiments/issue_1066_qualifying_pr/dry-run.sh

`AUTHORED_LINES` is the variable worth turning. The preflight
(`check-self-development-release.rs`) asks only whether a qualifying pull
request exists; the ledger's self-hosting ratchet asks how much of the cycle it
accounts for, so a qualifying pull request that is small still leaves
`Auto Release` red. The script exits with the preflight's own status, so it can
be read as a check as well as a measurement.
