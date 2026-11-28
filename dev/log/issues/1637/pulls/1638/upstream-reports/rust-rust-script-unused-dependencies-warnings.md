# [rust template] Release scripts declare unused dependencies — cargo 1.100 `unused_dependencies` warnings in Release Script Tests

Target repo: `link-foundation/rust-ai-driven-development-pipeline-template` (HEAD `1d29f55`)
Found while auditing: link-assistant/formal-ai#1637 (formal-ai copied these scripts and
got the same warnings on main: `version-and-commit` serde, `publish-crate` regex,
`wait-for-crate` regex; fixed downstream in formal-ai commit 836fe921e).

Duplicate check (2026-11-25): no existing issue mentions `unused dependency` /
`unused_dependencies` / `CARGO_BUILD_WARNINGS`. Related but different: #239, #179 (closed).

## Problem

Cargo 1.100.0 (2026-11-09) added the `cargo::unused_dependencies` lint (warn by
default). The "Release Script Tests" job (`rust-script --test scripts/<name>.rs`) now
emits warnings that nobody sees because warnings do not fail the job:

| Script | Warning | Cause |
|---|---|---|
| `scripts/check-cargo-lock.rs` | unused dependency `regex` | declared, never used |
| `scripts/check-crate-size.rs` | unused dependency `regex` | declared, never used |
| `scripts/wait-for-crate.rs` | unused dependency `ureq` | declared, never used |
| `scripts/version-and-commit.rs` | unused dependency `ureq` | declared, never used |
| `scripts/version-and-commit.rs` | unused dependency `chrono` | only used in `#[cfg(not(test))] fn collect_changelog` |
| `scripts/create-github-release.rs` | unused dependency `serde` | only used via `#[cfg_attr(not(test), derive(Serialize))]` |

Seen in run 45090928431, job 137434276912 (Release Script Tests).

Note: cargo reports this lint only for test builds (`cargo test` / `rust-script
--test`); a plain `cargo build` of the same package is silent. The last two rows are
therefore "unused in the test build only" — deleting `chrono`/`serde` would break the
real script. The first four are genuinely dead.

## Reproduction (local, verified 2026-11-25)

```bash
git clone --depth 1 https://github.com/link-foundation/rust-ai-driven-development-pipeline-template t && cd t
rustup toolchain install 1.100.0 --profile minimal
cargo install rust-script
for s in check-cargo-lock check-crate-size create-github-release version-and-commit wait-for-crate; do
  echo "== $s"; RUSTUP_TOOLCHAIN=1.100.0 rust-script --test scripts/$s.rs 2>&1 | grep 'unused dependency'
done
# Make it fatal, as it should be in CI:
CARGO_BUILD_WARNINGS=deny RUSTUP_TOOLCHAIN=1.100.0 rust-script --test scripts/check-cargo-lock.rs; echo "exit=$?"   # exit=1
```

Minimal standalone repro:

```rust
#!/usr/bin/env rust-script
//! ```cargo
//! [dependencies]
//! regex = "1"
//! ```
fn main() {}
#[cfg(test)] mod t { #[test] fn ok() {} }
```

`RUSTUP_TOOLCHAIN=1.100.0 rust-script --test s.rs` → `warning: unused dependency
`regex``; with `CARGO_BUILD_WARNINGS=deny` it exits non-zero.

## Workaround (downstream)

Remove the dead dependency lines from the scripts' embedded `//! [dependencies]`
manifests; ignore or allow the two test-only cases.

## Suggested fix

1. Delete `regex` from `check-cargo-lock.rs` and `check-crate-size.rs`, and `ureq`
   from `version-and-commit.rs` and `wait-for-crate.rs`.
2. Make the test build use the test-only deps too, so the lint is accurate:
   - `create-github-release.rs`: `#[derive(Serialize)]` unconditionally instead of
     `#[cfg_attr(not(test), derive(Serialize))]` (or add a test that serializes the
     payload — it is worth testing anyway);
   - `version-and-commit.rs`: route the date through a function that is also used in
     tests (e.g. `fn today() -> String { chrono::Utc::now()... }` plus a trivial test),
     or remove `chrono` and get the date from `git log -1 --format=%cs`/`date -u`.
3. Fail on warnings in the job so the next regression is caught on the PR that
   introduces it:
   ```yaml
   script-tests:
     env:
       CARGO_BUILD_WARNINGS: deny
   ```
   (requires the toolchain to be ≥ 1.100, which the template already pins).
4. Optionally add a unit test in `tests/unit/ci-cd/` asserting the `script-tests` job
   sets `CARGO_BUILD_WARNINGS: deny`.
