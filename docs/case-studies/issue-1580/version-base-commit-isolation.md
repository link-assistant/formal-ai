# Version comparison uses the fetched base commit

The latest [PHP template repair](https://github.com/link-foundation/php-ai-driven-development-pipeline-template/pull/35)
isolates a base fetch from configured refmaps and concurrent fetches. Formal AI
has the same relevant gap: checking `git fetch` success is insufficient when the
following manifest read still uses `origin/main`.

`python3 experiments/issue-1580/version-base-git-probe.py` creates two finite local
Git worktrees and one bare origin. With an alternate remote refmap, the original
production fetch succeeds, `FETCH_HEAD` identifies the new remote commit, and
`origin/main` still identifies the old version. Comparing against that tracking
ref can accept an outdated PR version and reject a matching current version.

The guard validates the full branch ref, fetches into a process-specific temporary
ref with configured refmaps disabled, resolves that ref to a commit and attempts
cleanup before comparing manifests by immutable commit ID. It does not read or
overwrite `FETCH_HEAD` or rely on a remote-tracking ref. Fetch, resolution and
cleanup failures reject admission. The selected commit appears in the normal
check output for provenance.

The maintained compiled CLI suite adds three tests: both version outcomes under
a custom refmap, both outcomes when a Git interposer changes shared observations
immediately after fetch, and malformed branch references. Existing failed-fetch,
manifest-layout, parsed-version and merged-base cases remain intact. These tests
exercise the actual production CLI in hosted CI; they have not run locally.

The local Git probe passes and `rustfmt --edition 2024 --check
scripts/check-version-modification.rs` plus Python syntax validation pass. The
probe establishes Git behavior, not native guard success. Its
[lossless record](../../../experiments/issue-1580/evidence/checks/version-base-commit-isolation/manifest.json)
preserves that distinction. Hosted validation and the wider issue remain
unfinished. Supplied code and controllers receive zero authorship credit.
