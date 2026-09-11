# Self-authored change for issue #1123

Run: https://github.com/link-assistant/formal-ai/actions/runs/34542617225

Task:

## Summary

`LOCKFILE_NAMES` in `scripts/self-hosting-attribution.rs` lists the files a
package manager writes and nobody hand-authors. `go.sum` — Go's module checksum
file, written by `go mod` — is not among them, so if one is ever committed it
counts as authored lines on both sides of the self-hosting share.

Same defect class as #1091 (`Gemfile.lock`) and #1120 (`composer.lock`), on the
same list and for the same reason: the list describes what a package manager
writes, not what this repository happens to track today.

## Task contract

The phrasing avoids every defect measured so far:

- **replacement**, not additive phrasing (#1115: `add X to the list` routes to
  web search instead of `edit`)
- **no escape sequences at all** — no `\n`, no `\"` (#1116: escapes reach the
  file as literal source characters, producing a file that does not compile)
- `contains:` values are **unquoted** (#1117: the parser passes the author's
  quotes through verbatim, so a correct artifact fails verification)
- the target is **not** under `.github/workflows/` (#1118: `GITHUB_TOKEN` cannot
  push there)

The anchor token `composer.lock` occurs exactly once in the file, so the
replacement is unambiguous. rustfmt lays the list back out onto separate lines.

task: In the file self-hosting-attribution.rs replace "composer.lock" with "composer.lock", "go.sum"
seed: scripts
produces: self-hosting-attribution.rs
into: scripts/self-hosting-attribution.rs
contains: go.sum
contains: LOCKFILE_NAMES
message: fix(metric): go.sum is a lockfile the self-hosting share never counts

## Merge method

Merge this with a **merge commit**, not a squash. #1122 was squash-merged and
that silently stripped the four `Formal-AI-*` trailers: git only parses trailers
in the final paragraph of a message, and the squash left them mid-message under
a `Co-authored-by:` block. The self-development floor then counted nothing,
because it pairs an attributed commit with a real `Merge pull request #N`
commit — which a squash never produces. #1103 was merged correctly and counted.

## How to test

- `cargo test --test unit self_hosting_metric` still passes.
- `is_non_authored_path("box/go/go.sum")` is true.
- The pull request Formal AI opens carries a commit with the four self-hosting
  trailers and no human commit on its branch.

