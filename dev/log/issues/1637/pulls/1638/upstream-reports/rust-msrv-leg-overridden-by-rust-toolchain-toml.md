# [rust template] MSRV test leg runs Rust 1.100.0, not 1.85.0 — `rust-toolchain.toml` overrides the matrix toolchain

Target repo: `link-foundation/rust-ai-driven-development-pipeline-template` (HEAD `1d29f55`)
Found while auditing: link-assistant/formal-ai#1637 (shared defect class: toolchain
pinning / false-positive CI legs).

Duplicate check (2026-11-25, `gh issue list --state all`): no existing issue about the
MSRV leg, `rust-toolchain.toml` override or toolchain drift. Related but different:
#147 (hash-pinned dtolnay must restate `toolchain:` — closed), #202 (closed).

## Problem

`.github/workflows/release.yml` (Test job, ~L565–585):

```yaml
matrix:
  os: [ubuntu-24.04, macos-26, windows-2025]
  toolchain: ['1.100.0']
  include:
    - os: ubuntu-24.04
      toolchain: '1.85.0'          # MSRV, matches Cargo.toml rust-version = "1.85"
steps:
  - uses: dtolnay/rust-toolchain@683b588c487a69d01f6c6ddf40f036ff682cc2c8
    with:
      toolchain: ${{ matrix.toolchain }}
```

`dtolnay/rust-toolchain` only installs the toolchain and runs `rustup default`. The
repository root contains `rust-toolchain.toml` (`channel = "1.100.0"`), and a
toolchain file **takes precedence over `rustup default`**. Every subsequent `cargo`
call therefore uses 1.100.0. The job titled "Test (ubuntu-24.04, Rust 1.85.0)" never
compiles with 1.85.0, so the declared MSRV is untested — a **false positive** green
check.

## Evidence

Run 45090928431, job 137434276998 "Test (ubuntu-24.04, Rust 1.85.0)":

```
  1.85.0-x86_64-unknown-linux-gnu installed - rustc 1.85.0 (4d91de4e4 2025-02-17)
info: note that the toolchain '1.100.0-x86_64-unknown-linux-gnu' is currently in use (overridden by '.../rust-toolchain.toml')
warn: the missing active toolchain `1.100.0-x86_64-unknown-linux-gnu` has been auto-installed
```

Every other Rust job also logs `default toolchain implicitly overridden with 1.100.0
by rustup toolchain file`, i.e. the toolchain is installed twice.

Secondary drift risk: the toolchain version is spelled in 18 places — 17
`toolchain: 1.100.0` workflow inputs plus `rust-toolchain.toml` — and
`tests/unit/ci-cd/issue_147.rs:125-136` hardcodes `1.100.0`/`1.85.0`. Dependabot's
`rust-toolchain` ecosystem bumps only `rust-toolchain.toml`, so after a bump the
workflow inputs silently install a stale toolchain that is then overridden (wasted
download, misleading logs), and the test does not notice the mismatch.

## Reproduction

```bash
git clone --depth 1 https://github.com/link-foundation/rust-ai-driven-development-pipeline-template t && cd t
rustup toolchain install 1.85.0 --profile minimal
rustup default 1.85.0          # what dtolnay/rust-toolchain does
cargo --version                # -> cargo 1.100.0 (toolchain file wins)
rustup show active-toolchain   # -> 1.100.0 (overridden by rust-toolchain.toml)
RUSTUP_TOOLCHAIN=1.85.0 cargo --version   # -> cargo 1.85.0
```

## Workaround (downstream)

Set `RUSTUP_TOOLCHAIN` on the MSRV job (env var beats the toolchain file):

```yaml
    env:
      RUSTUP_TOOLCHAIN: ${{ matrix.toolchain }}
```

## Suggested fix

1. In the Test job add job-level `env: RUSTUP_TOOLCHAIN: ${{ matrix.toolchain }}`
   (or invoke `cargo +${{ matrix.toolchain }} …`). Add a step that asserts the
   compiler actually in use:
   ```yaml
   - name: Assert active toolchain
     shell: bash
     run: |
       active="$(rustc --version | awk '{print $2}')"
       [ "$active" = "${{ matrix.toolchain }}" ] || { echo "::error::expected ${{ matrix.toolchain }}, got $active"; exit 1; }
   ```
2. Single source of truth for the pinned version: either drop the `toolchain:` input
   for non-matrix jobs and rely on `rust-toolchain.toml` (use `dtolnay/rust-toolchain`
   with `toolchain: ${{ steps.toolchain.outputs.channel }}` read from the toml), or
   keep the inputs and extend `issue_147.rs` with a test that parses
   `rust-toolchain.toml` and asserts every workflow `toolchain:` literal equals its
   `channel`, and that the MSRV include equals `Cargo.toml` `rust-version` (+ `.0`).
3. Optionally derive the MSRV matrix entry from `Cargo.toml` `rust-version` in a
   small prep step so bumping MSRV is a one-line change.

## Impact on formal-ai

formal-ai has the opposite half of the same problem (floating `@stable`, no toolchain
file, `rust-version = "1.99"` never tested). Adopting the template's pinning as-is
would import this MSRV false positive; formal-ai's fix should use `RUSTUP_TOOLCHAIN`
for any MSRV leg.
