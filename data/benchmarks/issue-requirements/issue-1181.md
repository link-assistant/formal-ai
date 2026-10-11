Part of the E127 umbrella (#1183): people cannot run the current release to check any of this.

## Evidence

- Release v0.352.1 assets: desktop apps (Electron; no Rust CLI inside the macOS archive), a VSIX, provenance files. **No standalone `formal-ai` CLI binary for Linux, macOS or Windows.** The only other channel is `cargo install` (a full Rust release build) or `ghcr.io/link-assistant/formal-ai` (24.3 GB, #1153).
- `FORMAL_AI_SILENT=1 formal-ai chat …` fails with `invalid value '1' for '--silent' [possible values: true, false]`; environment switches conventionally accept `1`/`0`.

## What to do

- Publish `formal-ai-cli-<target>.tar.gz` for linux x64/arm64 (musl), macOS x64/arm64 and windows x64 on every release, with checksums, plus an install script and `cargo binstall` metadata.
- Accept `1/0/yes/no/on/off` for every boolean environment variable (a single clap value parser used everywhere).

## How to test

Release job asserts the five CLI archives exist and `formal-ai --version` runs from each on its runner; unit test for the boolean parser.

## Verification of the evidence above (2026-09-29, `origin/main` @ `d209aac64`)

Confirmed exactly as stated:
- `gh release view v0.352.1 -R link-assistant/formal-ai --json assets --jq '.assets[].name'` returns 24 assets: `BUILD-PROVENANCE.txt`, 6 `formal-ai-desktop-linux-*` (AppImage/deb/tar.gz × x64/arm64), 8 `formal-ai-desktop-macos-*` (dmg/dmg.blockmap/zip/zip.blockmap × x64/arm64), 6 `formal-ai-desktop-windows-*` (installer/portable × x64/arm64), `formal-ai-vscode-0.352.1.vsix`, `latest-linux.yml`, `latest-mac.yml`, `latest.yml`, `SHA256SUMS.txt`. No CLI binary. The existing `SHA256SUMS.txt` covers only these desktop/VSIX assets and is produced by `.github/workflows/desktop-release.yml`'s `finalize` job (see Design) — the new CLI archives must merge into the *same* file, not a second one.
- `gh issue view 1153` is real, `OPEN`, title "the ghcr.io image is a 24.3 GB desktop/DinD box, not a CLI runtime — please publish a slim tag"; body states "24.3 GB on disk (6.2 GB compressed content, 59 layers)", matching the evidence line verbatim.
- `#1183` is real, `OPEN`, the E127 umbrella.

**Corrections to the "What to do" plan** (nothing in the original body is wrong, but four premises need refining before implementation):
1. `scripts/install.sh` **already exists** (issue #554, plus `scripts/install.ps1`) with a `cli` target — but it runs `cargo install formal-ai`, not a binary download. R5 below *changes* `install_cli()` to prefer a prebuilt archive, not a new script.
2. `cargo binstall` metadata is genuinely new — no `[package.metadata.binstall]` table exists in `rust/Cargo.toml` today.
3. No existing cross-compilation or musl setup anywhere in the repo (`git grep -l musl` over `.github`, `Dockerfile`, `scripts`, `rust/Cargo.toml` returns nothing). `desktop-release.yml`'s `build` job runs `cargo build --release --bin formal-ai` per-OS natively (glibc) purely to embed the binary in the Electron app — not evidence of a musl pipeline.
4. The real per-OS matrix/checksum/attestation/finalize machinery this issue must extend lives in **`.github/workflows/desktop-release.yml`**, not `.github/workflows/release.yml` (which only orchestrates `build-artifacts` → `build` → `auto-release`/`manual-release` for the crate and Docker image, not per-OS binaries).

## Requirements

- **R1** `desktop-release.yml` gains a `cli` job that cross-compiles `formal-ai` for `x86_64-unknown-linux-musl`, `aarch64-unknown-linux-musl`, `x86_64-apple-darwin`, `aarch64-apple-darwin`, `x86_64-pc-windows-msvc` and uploads one archive per target to the same GitHub Release the `build`/`vscode` jobs publish to.
- **R2** Each archive is named `formal-ai-cli-<target-triple>.tar.gz` (`.zip` for the Windows target), contains the `formal-ai` binary plus `LICENSE` and `README.md`, and is reproducible from `cargo build --release --target <triple> --manifest-path rust/Cargo.toml --bin formal-ai --locked`.
- **R3** The `finalize` job's consolidated `SHA256SUMS.txt` and `BUILD-PROVENANCE.txt` cover the five new CLI archives alongside the existing desktop/VSIX assets, in the same file, with the same "INCOMPLETE" honesty reporting `desktop-release.yml` already has for partial matrices.
- **R4** `rust/Cargo.toml` gains a `[package.metadata.binstall]` table so `cargo binstall formal-ai` resolves one of the R2 archives without a source build.
- **R5** `scripts/install.sh`'s `install_cli()` (and `install.ps1`'s equivalent) downloads and verifies the matching `formal-ai-cli-<target>` archive first, falling back to `cargo install formal-ai` only when no matching archive exists for the host's OS/arch (e.g. windows-arm64, which R1 does not build).
- **R6** A single boolean-env parser function accepts `1/0/true/false/yes/no/on/off` case-insensitively, is used as the `value_parser` for every clap arg with an `env = "FORMAL_AI_*"` attribute on a `bool` field, and is used by every ad-hoc `std::env::var("FORMAL_AI_*")` boolean read in `rust/src` (listed in Design) so the whole family of switches is consistent, not just the one that currently errors.
- **R7** `FORMAL_AI_SILENT=1 formal-ai chat --prompt …` (and `=true`, `=yes`, `=on`, case-insensitive, and their false-y counterparts) no longer errors.

## Design

### Release workflow (`.github/workflows/desktop-release.yml`)

New `cli` job, modeled directly on the existing `build` job's matrix/checksum/attest/upload steps (lines ~181-658) and `finalize` job's consolidation (lines ~774-901):

```yaml
  cli:
    name: Build CLI ${{ matrix.label }}
    needs: [resolve, base]
    concurrency:
      group: desktop-release-${{ github.event.pull_request.number || needs.resolve.outputs.tag }}-cli-${{ matrix.label }}
      queue: max
    if: ${{ !cancelled() && (github.event_name == 'pull_request' || needs.resolve.outputs.should_build == 'true') }}
    strategy:
      fail-fast: false
      matrix:
        include:
          - { os: ubuntu-latest,    target: x86_64-unknown-linux-musl,  label: cli-linux-x64,    ext: tar.gz }
          - { os: ubuntu-24.04-arm, target: aarch64-unknown-linux-musl, label: cli-linux-arm64,  ext: tar.gz }
          - { os: macos-15-intel,   target: x86_64-apple-darwin,        label: cli-macos-x64,    ext: tar.gz }
          - { os: macos-14,         target: aarch64-apple-darwin,       label: cli-macos-arm64,  ext: tar.gz }
          - { os: windows-latest,   target: x86_64-pc-windows-msvc,     label: cli-windows-x64,  ext: zip }
    runs-on: ${{ matrix.os }}
    timeout-minutes: 30
    permissions:
      contents: write
      id-token: write
      attestations: write
    defaults:
      run:
        shell: bash
    steps:
      - uses: actions/checkout@v7
        with:
          persist-credentials: false
          ref: ${{ github.event_name != 'pull_request' && needs.resolve.outputs.tag || '' }}
          fetch-depth: ${{ github.event_name == 'pull_request' && '0' || '1' }}
      - name: Simulate fresh merge with base branch (PR only)
        if: github.event_name == 'pull_request'
        env:
          BASE_REF: ${{ github.base_ref }}
          BASE_COMMIT: ${{ needs.base.outputs.commit }}
        run: bash scripts/simulate-fresh-merge.sh
      - name: Install Rust toolchain
        uses: dtolnay/rust-toolchain@stable
        with:
          targets: ${{ matrix.target }}
      - name: Install musl cross tools
        if: contains(matrix.target, 'musl')
        run: sudo apt-get update && sudo apt-get install -y musl-tools
      - name: Build formal-ai binary (release, ${{ matrix.target }})
        run: cargo build --manifest-path rust/Cargo.toml --release --locked --target ${{ matrix.target }} --bin formal-ai
      - name: Package CLI archive
        id: pkg
        run: bash scripts/package-cli-archive.sh "${{ matrix.target }}" "${{ matrix.ext }}"
      - name: Attest build provenance
        if: github.event_name != 'pull_request'
        uses: actions/attest@v4
        with:
          subject-path: release-cli/formal-ai-cli-${{ matrix.target }}.${{ matrix.ext }}
      - name: Upload asset to release
        if: github.event_name != 'pull_request'
        env:
          GH_TOKEN: ${{ github.token }}
          REPO: ${{ github.repository }}
          TAG: ${{ needs.resolve.outputs.tag }}
        run: gh release upload "$TAG" release-cli/formal-ai-cli-${{ matrix.target }}.${{ matrix.ext }} --repo "$REPO" --clobber
      - name: Checksum fragment
        run: |
          set -euo pipefail
          cd release-cli
          sha256sum "formal-ai-cli-${{ matrix.target }}.${{ matrix.ext }}" > "SHA256SUMS-${{ matrix.label }}.partial"
      - name: Upload checksum fragment
        uses: actions/upload-artifact@v7
        with:
          name: checksums-${{ matrix.label }}
          path: release-cli/SHA256SUMS-${{ matrix.label }}.partial
          retention-days: 7
```

Fragment names use the `cli-` prefix (`checksums-cli-linux-x64`, file `SHA256SUMS-cli-linux-x64.partial`) so they cannot collide with the desktop `build` job's identically-shaped `linux-x64` label when `finalize` downloads every `checksums-*` artifact with `merge-multiple: true` into one flat `fragments/` directory.

`finalize` job changes (same file, ~line 776 and ~822):
- `needs: [resolve, build, vscode]` → `needs: [resolve, build, vscode, cli]`
- `builder_of` map and the `for label in ...` loop gain the five `cli-*` labels mapped to their runner (`cli-linux-x64` → `ubuntu-latest`, etc.), so `BUILD-PROVENANCE.txt`'s "Builders" / "INCOMPLETE" reporting covers them the same honest way it covers desktop targets.

New file **`scripts/package-cli-archive.sh`** [NEW]: takes `<target-triple> <ext>`, copies `rust/target/<triple>/release/formal-ai` (`.exe` on Windows) plus `LICENSE` and `README.md` into `release-cli/`, and `tar czf`/`zip`s them as `formal-ai-cli-<target-triple>.<ext>` (R2). Modeled on the existing `scripts/collect-build-artifacts.sh` (single-purpose, `set -euo pipefail`, errors loudly on a missing binary rather than silently skipping it).

### `cargo binstall` metadata (R4)

`rust/Cargo.toml`, appended near the existing `[[bin]]` block (line 36):
```toml
[package.metadata.binstall]
pkg-url = "{ repo }/releases/download/v{ version }/formal-ai-cli-{ target }.{ archive-suffix }"
bin-dir = "formal-ai{ binary-ext }"
pkg-fmt = "tgz"

[package.metadata.binstall.overrides.x86_64-pc-windows-msvc]
pkg-fmt = "zip"
```

### `scripts/install.sh` (R5, modifying the existing file, not adding one)

`install_cli()` (currently just `cargo install formal-ai`) gains an archive-first path reusing the file's existing `detect_os`/`detect_arch`/`asset_url_matching`/`download`/`verify_checksum` helpers, mapping `(os, arch)` to the R2 target-triple naming (e.g. `linux`+`x64` → `x86_64-unknown-linux-musl`), and falling back to today's `cargo install formal-ai` body when no archive matches (e.g. windows-arm64, or the target-triple lookup itself fails). `install.ps1` gets the mirrored change for Windows.

### Boolean env parser (R6, R7)

New shared function, placed in `rust/src/main.rs` near the `Args` struct (it must be visible to `#[arg(value_parser = ...)]`, which clap derive resolves in the defining crate) and re-exported for non-clap call sites via a small new module:

```rust
// rust/src/env_bool.rs [NEW]

/// Parse a boolean CLI argument or environment-variable value. Accepts
/// `1/0`, `true/false`, `yes/no`, `on/off`, case-insensitively, matching the
/// conventional set of truthy/falsy environment-switch spellings.
pub fn parse_bool_env(s: &str) -> Result<bool, String> {
    match s.trim().to_ascii_lowercase().as_str() {
        "1" | "true" | "yes" | "on" => Ok(true),
        "0" | "false" | "no" | "off" => Ok(false),
        other => Err(format!(
            "invalid value '{other}': expected one of 1/0, true/false, yes/no, on/off"
        )),
    }
}

/// Read a `FORMAL_AI_*` boolean environment variable directly (non-clap call
/// sites), defaulting to `false` when unset or unparseable.
#[must_use]
pub fn env_flag(name: &str) -> bool {
    std::env::var(name)
        .ok()
        .and_then(|v| parse_bool_env(&v).ok())
        .unwrap_or(false)
}
```
Registered in `rust/src/lib.rs` alongside the other `pub mod` declarations (line 3 region, alphabetically near `engine`/`entity_resolution`).

Call sites to update:
- **Clap-declared, currently broken** — `rust/src/main.rs:81`: `#[arg(long, global = true, env = "FORMAL_AI_SILENT", default_value_t = false)]` → add `value_parser = env_bool::parse_bool_env`. This is the **only** clap arg in the whole crate with both `env = "FORMAL_AI_*"` and a `bool` field (verified: `git grep -n 'env = "FORMAL_AI'` over `rust/src` lists 25 `env=` args; cross-referencing every `bool`-typed field in `main.rs`/`cli_coding.rs`/`cli_local_transport.rs`/`cli_memory.rs` shows none of the others — `verbose`, `agent_mode`, `ws`, `webrtc`, `transport_trace`, `confirm`, `write`, `list`, `body`, `commit`, `promote`, `apply`, `open_draft_pr`, `events_only`, `disable_daydreaming`, `transcript`, `thinking` — carry an `env` attribute).
- **Ad-hoc, bare `"1"`-only today** (these don't error, but silently reject `true`/`yes`/`on` and are inconsistent with the two below that already accept more spellings) — replace `std::env::var("FORMAL_AI_X").as_deref() == Ok("1")` with `env_bool::env_flag("FORMAL_AI_X")`:
  - `rust/src/agent.rs:745` — `FORMAL_AI_TRACE_COMMANDS`
  - `rust/src/agentic_coding/planner/continuation.rs:156` — `FORMAL_AI_TRACE_REQUESTS`
  - `rust/src/agentic_coding/self_ast.rs:55` — `FORMAL_AI_TRACE_SLOW_INIT`
  - `rust/src/dialog_log.rs:55` — `FORMAL_AI_SILENT` (a *second*, non-clap read of the same variable; `!= Ok("1")` → `!env_bool::env_flag(...)`)
  - `rust/src/dialog_log.rs:109` — `FORMAL_AI_TRACE_REQUESTS`
  - `rust/src/dreaming_runtime.rs:76` — `FORMAL_AI_DREAMING_DEBUG`
  - `rust/src/link_store/staging.rs:87` — `FORMAL_AI_LINK_CLI_DEBUG`
  - `rust/src/memory_sync.rs:233,242,257` — `FORMAL_AI_MEMORY_DEBUG` (three call sites)
  - `rust/src/protocol.rs:533` — `FORMAL_AI_TRACE_REQUESTS`
  - `rust/src/protocol_responses.rs:137` — `FORMAL_AI_TRACE_REQUESTS`
- **Already flexible, left alone (proof the target shape is right)** — `rust/src/memory_sync.rs:165` (`FORMAL_AI_RECORD_CHAT`, matches `Ok("0"|"false"|"off")`) and `rust/src/seed_links.rs:247` (`FORMAL_AI_SEED_LINKS_MIRROR`, matches `Ok("1"|"true"|"on")`) could switch to `env_bool::env_flag`/`!env_bool::env_flag` too for one canonical implementation, but are not broken.

## Tests

- `rust/tests/unit/issue_1181_env_bool.rs` [NEW]: table-driven cases for `parse_bool_env` covering every accepted spelling both cases (`"1"`, `"TRUE"`, `"yes"`, `"ON"`, `"0"`, `"False"`, `"no"`, `"off"`) plus rejected input (`"2"`, `""`, `"maybe"`) asserting the exact error message. Registered as `mod issue_1181_env_bool;` in `rust/tests/unit/mod.rs`. Run with `RUSTUP_TOOLCHAIN=1.98.1 cargo test --manifest-path rust/Cargo.toml --test unit issue_1181_env_bool`.
- `rust/tests/unit/main_cli.rs` (or nearest existing CLI-arg test module): a case that sets `FORMAL_AI_SILENT=1` via `env` on a spawned `formal-ai chat` process (or exercises the `Args::parse_from` path directly with `env` injected) and asserts no `invalid value` error — the regression test for R7.
- Release job assertion: a "Smoke test CLI archive" step inside each `cli` matrix leg, immediately after "Package CLI archive" (mirroring `desktop-release.yml`'s existing "Smoke test macOS/Linux/Windows release artifacts" steps), extracts the just-built archive and runs `./formal-ai --version` on that leg's own OS/arch — the only runner that can actually execute it. `finalize` (single `ubuntu-latest` runner) can only assert "all five archives exist," not execute the non-Linux-x64 ones.
- `RUSTUP_TOOLCHAIN=1.98.1 rust-script scripts/run-ci-gates.rs --stage rust` must stay green (the `rust` stage is where clippy/fmt/unit gates run per `scripts/run-ci-gates.rs`'s `STAGES` list).

## Definition of done

- Requirement shard `docs/requirements/issue-1181-cli-binaries-and-bool-env.md` with R1-R7; `rust-script scripts/assemble-requirements.rs --write` regenerates `REQUIREMENTS.md`.
- `docs/requirements-traceability.md` gains one row per requirement (R1-R7), each citing the delivering commit/PR, the pinning automated test, and honestly marking manual confirmation `not yet confirmed` unless actually run by hand, per the file's stated 2026-08-04 honesty rules.
- `docs/case-studies/issue-1181/` holds the raw evidence this body already cites plus anything gathered during implementation (release asset diffs, CI run logs for the new `cli` matrix legs).
- A changelog fragment lands in `changelog.d/` following the existing `YYYYMMDD_HHMMSS_description.md` + frontmatter format documented in `changelog.d/README.md`.
- `RUSTUP_TOOLCHAIN=1.98.1 rust-script scripts/run-ci-gates.rs --stage rust` is green.
- The next tagged release's `gh release view <tag> --json assets` lists five `formal-ai-cli-*` archives and a `SHA256SUMS.txt` that covers them alongside the desktop/VSIX assets (R3) — the concrete, observable proof this issue is actually closed, not merely merged (per the release-pipeline silent-deferral risk: merged on main is not released).

## Depends on / blocks

- Depends on nothing; lands first, because every other issue in #1183 is verified by running the released CLI.
- Blocks the release-time probe run of #1171 (E136), which needs a downloadable CLI for each release.
- **#1153** (ghcr.io image is 24.3 GB) — related but independent distribution-channel problem for the same underlying complaint ("people cannot run the current release"); fixing #1181 does not fix #1153 and vice versa, but both close out the same umbrella evidence about there being no lightweight way to run `formal-ai`.

