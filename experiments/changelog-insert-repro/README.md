# Reproducing the changelog shape defect (issue #736)

`release.yml` runs `node experiments/issue_711_rebuild_changelog.mjs --check`,
which rebuilds `CHANGELOG.md` from git history and compares it byte for byte
with the committed file. Every release commit made that comparison fail in two
ways at once:

* the marker was left followed by **two** blank lines instead of one, and
* the file's **trailing newline** was stripped.

`run.sh` reproduces both against the real script and the real `CHANGELOG.md` as
it stood immediately before the `v0.296.0` release commit (`b2064b2a`), which is
the commit that introduced the divergence currently on `main`.

Run it against the code *before* the fix to see the defect, and after to see the
canonical output. The equivalent assertion now lives as a permanent regression
test, `release_writes_the_changelog_exactly_as_reconstruction_expects`, in
`scripts/version-and-commit.rs`.
