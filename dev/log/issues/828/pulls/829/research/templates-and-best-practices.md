# CI/CD templates & best-practices reference (issue #828)

Collected 2026-07-22 via `gh`/WebFetch from the three pipeline templates and the
hive-mind best-practices doc.

## hive-mind/docs/CI-CD-BEST-PRACTICES.md — key rules

- **Fast-fail ordering:** run fast checks (compile/lint/file-size, ~7–30s) before
  slow checks (full test suites, ~1–10 min).
- **Concurrency control:** group runs and cancel superseded ones;
  `cancel-in-progress: ${{ github.ref == 'refs/heads/main' }}` (templates use the
  inverse — non-main cancels, main runs complete so releases finish).
- **File size limit:** enforce a maximum of 1000–1500 lines per code file.
- **Warnings as errors:** static analysis violations fail the build.
- **Validate the actual merge result (determinism):** "CI must test what will
  actually be merged, not a stale PR snapshot" — fresh-merge the base branch first.
- **Secrets detection:** fail CI immediately if secrets are detected.

> The doc does **not** explicitly cover flaky-test handling, test isolation,
> retries, per-job timeouts, caching, or matrix builds — those appear only as
> concrete implementation choices in the templates.

## Template workflow inventories

- **rust-…-template** `.github/workflows/`: `release.yml` (single combined CI/CD).
- **js-…-template**: `example-app.yml`, `links.yml`, `release.yml`.
- **python-…-template**: `docs.yml`, `release.yml`.

## Rust template `release.yml` — observed settings

- Least-privilege `permissions: contents: read`.
- Global `env: RUSTFLAGS: -Dwarnings`; clippy `--all-targets --all-features`;
  `cargo fmt --all -- --check`; rustdoc gate `RUSTDOCFLAGS: -D warnings`.
- **Determinism enforced via a Cargo.lock Guard job** (same dependency graph every
  run), not via test retries. Tests: `cargo test --all-features --verbose` +
  `cargo test --doc`. No retries, no nextest, no `--test-threads`.
- Network flake hardening: `CARGO_NET_RETRY: '10'`, `CARGO_HTTP_MULTIPLEXING: 'false'`.
- Concurrency: `group: ${{ github.workflow }}-${{ github.ref }}`,
  `cancel-in-progress: ${{ github.ref != 'refs/heads/main' }}`.
- Every job sets `timeout-minutes` (5–60).
- Caching: `actions/cache@v5` on cargo registry/git/target keyed by `Cargo.lock`.
- Matrix: `test` job over `[ubuntu, macos, windows]` with `fail-fast: false`.
- `detect-changes` gates downstream jobs by changed file class.

## Relevance to issue #828

The failing item is a **flaky agent-CLI E2E assertion**, a class the best-practices
doc does not explicitly address and the templates do not carry (the templates have
no shared-state agent-E2E harness). The Rust template's determinism philosophy —
"make CI reproducible; test the real merge; pin inputs" — maps directly onto the
formal-ai fix: **pin/isolate the E2E server's memory input** so each test is
reproducible, rather than loosen the threshold.

Because the templates ship no equivalent non-isolated shared-state E2E harness,
there is **no matching template defect to report upstream** for this specific bug.
