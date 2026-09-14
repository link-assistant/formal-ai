# Self-authored change for issue #1120

Run: https://github.com/link-assistant/formal-ai/actions/runs/34530973176

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

task: In the file self-hosting-attribution.rs replace "uv.lock" with "uv.lock", "composer.lock"
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

## Attempt log

**Run 34530199973** — the first contract used `\n` to put the new entry on its
own line. Formal AI emitted every escape literally, `\"` as well as `\n`:

```rust
    \"Gemfile.lock\",\n    \"composer.lock\",
```

which does not compile (`error: unknown start of token: \`). The run still
reported success and opened #1121, because `contains: composer.lock` is a
substring check and the literal text is present. Recorded on #1116.

This second contract carries no escape sequences at all: it replaces the bare
token `uv.lock` with `uv.lock", "composer.lock` — no `\n`, no `\"` — and lets
rustfmt lay the list out.


