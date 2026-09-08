# Self-authored change for issue #1091

Run: https://github.com/link-assistant/formal-ai/actions/runs/34231781135

Task:

## Summary

`Gemfile.lock` is a lockfile a package manager writes, never hand-authored work, yet `scripts/self-hosting-attribution.rs` does not list it, so a Ruby box-image project's lockfile counts as authored lines on both sides of the self-hosting share. This is the first task Formal AI authors itself through the self-authored pull request workflow (issue #1085 D2.3).

A source change needs a changelog fragment beside it, so this task has two artifacts: the edited file and the fragment. Both are authored in one Agent CLI session.

## Task contract

task: Two files. First, edit the tracked file `self-hosting-attribution.rs`: add "Gemfile.lock" to the LOCKFILE_NAMES list, change only that file, and keep it valid Rust. Second, write a new file `fragment.md` whose first three lines are exactly `---`, `bump: patch` and `---`, then a blank line, then the line `### Fixed`, then a line starting with `- ` that says Gemfile.lock is now counted as a lockfile by the self-hosting metric and names issue #1091.
seed: scripts
produces: self-hosting-attribution.rs
produces: fragment.md
into: scripts/self-hosting-attribution.rs
into: changelog.d/20260908_140000_issue_1091_gemfile_lock.md
contains: "Gemfile.lock"
contains: LOCKFILE_NAMES
contains: bump: patch
message: fix(metric): Gemfile.lock is a lockfile the self-hosting share never counts

## How to test

- `cargo test --test unit self_hosting_metric` still passes; `is_non_authored_path("box/ruby/Gemfile.lock")` is true.
- The pull request Formal AI opens carries a commit with `Formal-AI-Session`, `Formal-AI-Model`, `Formal-AI-Evidence` and `Formal-AI-Pull-Request` trailers and no human commit on its branch.
- That pull request's own CI is green, including the changelog-fragment gate.

