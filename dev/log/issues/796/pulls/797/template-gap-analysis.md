# CI/CD gap analysis vs. the link-foundation pipeline templates

Compared `.github/workflows/release.yml` (1565 L), `.github/workflows/desktop-release.yml`
(649 L), `scripts/` and `.pre-commit-config.yaml` against
`rust-`, `js-` and `python-ai-driven-development-pipeline-template`, plus
[hive-mind CI-CD-BEST-PRACTICES.md](https://github.com/link-assistant/hive-mind/blob/main/docs/CI-CD-BEST-PRACTICES.md).

**Headline.** `release.yml` is a direct descendant of the Rust template
(identical `detect-changes` job, byte-identical `.pre-commit-config.yaml`) and
has diverged *ahead* of it in several places. The remaining gaps are narrow but
real; three are release-safety gaps.

## 1. Present in templates, missing here

| # | Gap | Template evidence |
| --- | --- | --- |
| 1.1 | **`Cargo.lock` guard job** — no `cargo-lock` job, no `scripts/check-cargo-lock.rs`. This repo ships binaries, so an unsynced lockfile is a live risk. | rust `release.yml:137-163` + `scripts/check-cargo-lock.rs` |
| 1.2 | **Post-publish smoke test** — no verification of the published crates.io artifact on either release path. A broken publish is found by users first. | rust `release.yml:451-457,619-624`; js `:463,533` |
| 1.3 | **Secrets scanning** — zero hits for secretlint/trufflehog/gitleaks across `.github/` and pre-commit. Mandated by best-practices §11. | js `release.yml:233-234` |
| 1.4 | **Fresh-merge simulation** — lint/test validate a possibly stale merge preview rather than the real merge result. Best-practices §7. | js `release.yml:114-118,210,284` |
| 1.5 | **Resilient Buildx setup** — this repo calls the bare action at `release.yml:606,828`, so a Docker Hub blip fails an otherwise-good publish. No `.github/actions/` directory exists here. | rust `.github/actions/setup-buildx-resilient/action.yml` |
| 1.6 | **File-size check excludes `.md`/`.yml`** — `scripts/check-file-size.rs:18-37` covers `.rs`/`.lino`/worker `.js` only, so nothing catches that `release.yml` is itself 1565 lines. | js `release.yml:120-124` |

## 2. Anti-patterns in this repo

Most usual suspects are **absent** — this repo is in good shape. `timeout-minutes`
is on every job in both workflows; concurrency uses a correct
`cancel-in-progress: ${{ github.ref != 'refs/heads/main' }}`; top-level
`permissions: contents: read` with least-privilege per-job escalation;
`!cancelled()` used per best-practices §10.

Remaining findings:

| # | Finding | Location |
| --- | --- | --- |
| A | `release.yml` is 1565 lines, above the 1000–1500 principle this repo enforces on its own source, and is not self-checked | `.github/workflows/release.yml` |
| B | Actions float on major tags rather than SHA pins (`actions/checkout@v6`, `docker/build-push-action@v7`, `peter-evans/create-pull-request@v8`, `codecov/codecov-action@v7`), and `dtolnay/rust-toolchain@stable` is a **branch** — weakest of all. Relevant for jobs holding `id-token: write` / `contents: write`. | release.yml:74,79,630,978,444; desktop-release.yml:141 |
| C | No `defaults.run.shell: bash` in `release.yml`, despite a `matrix.os` job at `:324-325` that will silently use PowerShell on Windows. `desktop-release.yml:132-134` does this correctly. | `release.yml` |
| D | No multi-line `run:` block in `release.yml` sets `set -euo pipefail`; `desktop-release.yml` does (177, 245, 335, 381, 431). Piped steps can mask failures. | `release.yml` |
| E | Bare `if: always()` at three sites; the repo uses the preferred `!cancelled()` elsewhere | release.yml:727,939,1066 |

## 3. Fixes here that the templates still lack — reported upstream

All items below have been filed; links and verification commands are in
`timeline-and-root-cause.md` §7.

1. **Rust template `release.yml` has no top-level `permissions:` block at all**
   (goes `on:` → `concurrency:`). Jobs inherit the repository default
   `GITHUB_TOKEN` scope, read/write-all on many orgs. This repo fixes it at
   `release.yml:32-33`. The js/python templates set it in secondary workflows
   but **not** in their `release.yml` either. File against all three; Rust is
   the worst case.
2. **`cargo install rust-script` is unretried and unlocked** at `release.yml:76,101,124`
   in the **Rust template only** — verified that the js/python templates never
   invoke it, correcting an earlier draft of this document that said all three.
   This repo replaced it with `scripts/install-rust-script.sh` (short-circuits
   if present, `--locked`, 3 retries with backoff) — consistent with the
   templates already setting `CARGO_NET_RETRY: '10'`.
3. **Outdated codecov action**: rust template `@v5`, python template `@v4`
   (deprecated — last major on the retired bash uploader); current major is
   `v7.0.0` (2026-06-07) and this repo is on `@v7`.
4. **No rustdoc validation in the Rust template.** This repo gates releases on
   `cargo doc` with `RUSTDOCFLAGS: -D warnings` plus a `DOCS_RS=1` profile
   (`release.yml:200-215`). The template deploys docs but never lints them, so
   a broken rustdoc link first surfaces after publish.
5. **No build-provenance attestation in any template**, though all publish
   artifacts. This repo uses `actions/attest@v4` (`desktop-release.yml:414,512`).

## 4. Recommended follow-up order

These are **out of scope for the #796 failures** (which are fixed in this PR)
and are best landed as separate, individually reviewable changes:

1. `Cargo.lock` guard job (§1.1) — highest impact.
2. Post-publish crate smoke test (§1.2).
3. `setup-buildx-resilient` composite (§1.5) — pure reliability, zero behaviour change.
4. `defaults.run.shell: bash` + `set -euo pipefail` in `release.yml` (§2 C/D).
5. Secrets scanning (§1.3) and fresh-merge simulation (§1.4).
6. Split `release.yml` and extend `check-file-size.rs` to `.yml`/`.md` (§1.6, §2 A).
