# Dispatch pagination and documentation context receipts

The checks-only dispatch of source `b8383d52b10fb8be79f8fe7e42944b88343a07dc`
exposes a real GitHub CLI error: `--slurp` cannot be combined with `--jq`.
The maintained action conservatively dispatches the full target set after that
error, so documentation-only requests cannot select their lightweight profile.
Keep complete pagination and pass its raw page arrays to the existing classifier;
that classifier already ignores unrelated REST fields and validates the observed
count and both sides of every rename.

The regression executes the maintained action against a recording CLI that
rejects the observed incompatible flags. Before the correction, a documentation
request incorrectly dispatches a native workflow. All ten dispatch tests pass
afterward, including 101 files across two pages, a source rename, an incomplete
page set, an API failure and an explicit caller subset.

The hosted documentation-policy run also fails at log lines 375–429 because its
workflow simulator has no `github.sha` or `tested-sha` receipt. Supply the same
candidate identity that the real fast JavaScript job reports. All 37 documentation
tests then pass; missing and different-source receipts still reject native
admission, and the retained documentation-only case executes no native tool.
This changes the fixture, preserving the production admission condition.

Reproduce with one file at a time:

```sh
node --test --test-concurrency=1 rust/tests/web/issue-1187-release-dispatch.test.mjs
node --test --test-concurrency=1 rust/tests/web/documentation-only-ci-policy.test.mjs
```

The [lossless records](../../../experiments/issue-1580/evidence/checks/dispatch-pagination-repair/manifest.json)
retain the observed CLI error, before/after controls, the initial incorrect
fixture-field attempt, its correction and focused gates. No native build ran.
The hosted result for these corrections and the wider issue criteria remain
unfinished. Supplied controller and patch bytes receive zero authorship credit.
