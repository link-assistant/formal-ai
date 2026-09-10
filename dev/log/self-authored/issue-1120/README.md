# Self-authored change for issue #1120

Run: https://github.com/link-assistant/formal-ai/actions/runs/34530199973

Task:

## Summary

`LOCKFILE_NAMES` in `scripts/self-hosting-attribution.rs` lists eight lockfiles
a package manager writes. `composer.lock` — PHP's, written by Composer and never
hand-authored — is not among them, so if one is ever committed it counts as
authored lines on both sides of the self-hosting share.

This is the same defect class #1091 fixed for `Gemfile.lock`, on the same list
and for the same reason: the list describes what a package manager writes, not
what this repository happens to track today.

## Task contract

The phrasing here is deliberate. Three defects found on #1113 shape it:

- **replacement**, not additive phrasing (#1115: `add X to the list` routes to
  web search rather than `edit`)
- a **single line** with no `\n` (#1116: `\n` is emitted as a literal backslash-n)
- `contains:` values are **unquoted** (#1117: the parser passes the author's
  quotes through verbatim, so a correct artifact fails verification)
- the target is **not** under `.github/workflows/` (#1118: `GITHUB_TOKEN` cannot
  push there)

task: In the file self-hosting-attribution.rs replace "    \"Gemfile.lock\"," with "    \"Gemfile.lock\",\n    \"composer.lock\","
seed: scripts
produces: self-hosting-attribution.rs
into: scripts/self-hosting-attribution.rs
contains: composer.lock
contains: LOCKFILE_NAMES
message: fix(metric): composer.lock is a lockfile the self-hosting share never counts

## How to test

- `cargo test --test unit self_hosting_metric` still passes.
- `is_non_authored_path("box/php/composer.lock")` is true.
- The pull request Formal AI opens carries a commit with the four self-hosting
  trailers and no human commit on its branch.

## Note

This task does use a `\n`, which #1116 records as broken. That is intentional:
it is the smallest honest way to add a list entry, and if #1116 is still live
the run will show it again on a task where nothing else can be blamed. If it
fails that way, the single-line fallback is to replace `"Gemfile.lock",` with
`"Gemfile.lock", "composer.lock",` on one line and let rustfmt split it.

