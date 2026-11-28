# Self-maintained dependencies and patches

When a build step, the translator, or a layer check needs a feature a
self-maintained dependency has not released, the workaround is a
`[patch]`-style source install — and the debt it creates is tracked where
debt belongs:

- Every `[patch]` or source-install entry in `Cargo.toml` carries a comment
  beside it referencing, by URL, the issue that lists what is patched and
  what blocks using the latest release. The gate
  `rust/tests/unit/ci-cd/issue_1138_dependency_patches.rs` fails the build when
  an entry appears without one, and when the lockfile resolves git sources
  the manifest does not declare.
- One issue per dependency — not one issue per patch. The issue stays open
  until the last patch for that dependency is gone.
- Issues filed against dependencies are written **generally**: state the
  general case the dependency is missing, not what Formal AI happens to
  need. The general case moves into the dependency; this repository's own
  code shrinks. A patch is a bridge, and its issue is the plan to remove
  it — an untracked patch is invisible debt, because nothing fails when
  the upstream release lands and nothing ever removes it.

The same rule governs *held-back releases* (issue #1169): every dependency
must sit at its publisher's latest release, and `rust-script
scripts/check-dependencies-latest.rs` fails when one does not. The one
sanctioned way to hold a dependency back is a same-line annotation naming
the issue that tracks the hold-back — `links-notation = "0.16.1" # blocked:
https://github.com/link-foundation/lino-objects-codec/issues/60` in
`Cargo.toml` (the URL may be bare or `<…>`-bracketed), or a `"<name>//":
"blocked: <url> (reason)"` note key in the same `package.json`. The
annotation must open with an issue URL, or with the GitHub security advisory
(`https://github.com/advisories/GHSA-…`) when the hold-back waits on an
unpatched vulnerability; a hold-back without one is not a decision, only
drift, and the gate fails on it. The daily
`dependencies-latest` workflow re-checks every registry and opens one pull
request with everything that moved, so the gate being green is a fact about
yesterday's registries, not a permanent state anyone achieves once.
