# Self-authored change for issue #1091

Run: https://github.com/link-assistant/formal-ai/actions/runs/34241163195

Task:

## Summary

`Gemfile.lock` is a lockfile a package manager writes, never hand-authored work, yet `scripts/self-hosting-attribution.rs` does not list it, so a Ruby box-image project's lockfile counts as authored lines on both sides of the self-hosting share. This is the first task Formal AI authors itself through the self-authored pull request workflow (issue #1085 D2.3).

The changelog fragment the source change needs is written by the bootstrap commit that opens the pull request: `changelog.d/` is excluded from both sides of the self-hosting share, so it is process record rather than authored behaviour.

## Task contract

task: Edit the tracked file `self-hosting-attribution.rs`: add "Gemfile.lock" to the LOCKFILE_NAMES list. Change only that file and keep it valid Rust.
seed: scripts
produces: self-hosting-attribution.rs
into: scripts/self-hosting-attribution.rs
contains: "Gemfile.lock"
contains: LOCKFILE_NAMES
message: fix(metric): Gemfile.lock is a lockfile the self-hosting share never counts

## How to test

- `cargo test --test unit self_hosting_metric` still passes; `is_non_authored_path("box/ruby/Gemfile.lock")` is true.
- The pull request Formal AI opens carries a commit with `Formal-AI-Session`, `Formal-AI-Model`, `Formal-AI-Evidence` and `Formal-AI-Pull-Request` trailers and no human commit on its branch.
- That pull request's own CI is green, including the changelog-fragment gate.

