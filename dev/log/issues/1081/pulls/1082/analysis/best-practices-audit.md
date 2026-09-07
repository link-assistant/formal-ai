# R4: this repository against all sixteen principles of `CI-CD-BEST-PRACTICES.md`

Issue #1081 requirement R4:

> Follow the CI/CD best practices collected in
> <https://github.com/link-assistant/hive-mind/blob/main/docs/CI-CD-BEST-PRACTICES.md>

The document is snapshotted at `../references/CI-CD-BEST-PRACTICES.md` (529
lines, sixteen numbered principles under "Key CI/CD Principles"). This is one
verdict per principle, with the evidence for it. "Followed" means the control
exists **and** something fails when it stops working; a control nothing can
break is not a control.

Three principles produced work in this pull request: 16 (implemented), 3 (gap
measured, delegated to a follow-up issue) and 13 (gap measured, delegated).
Everything else was already followed, and this file records how it was checked
so the next round does not re-measure it from scratch.

| # | Principle | Verdict | Evidence |
|---|---|---|---|
| 1 | Run checks only on relevant file changes | followed | `detect-changes` job at `.github/workflows/release.yml:48`; 31 `needs.detect-changes.outputs.*` conditions across the file |
| 2 | File size limits | followed, extended | `scripts/check-file-size.rs` caps `.rs` at 1000, `.lino` at 1500, workflow `.yml` at 2000 with a warning band from 1500, worker `.js` at 1500. The template caps source only; the workflow cap was added by issue #812 after `release.yml` reached 1824 lines with CI green |
| 3 | Automated code formatting | **gap (Rust followed, JavaScript not)** | `cargo fmt --all -- --check` is the `check_formatting` gate and a pre-commit hook. **No JavaScript formatter or linter runs anywhere in CI**: 353 of 372 in-scope `.js`/`.mjs` files are non-conforming and nothing reports it. Filed as [#1083](https://github.com/link-assistant/formal-ai/issues/1083); see §2 below |
| 4 | Static analysis & linting | followed | `run_clippy` gate: `cargo clippy --lib --bins --tests --all-features -- -D warnings && cargo check --examples --all-features`. Wider than the template's `--all-targets` in one respect (examples are type-checked separately, so an example cannot silently stop compiling) |
| 5 | Fast-fail job ordering | followed | `build` needs `[lint, test, macos-core-tests, build-artifacts]`; every release job needs the check jobs. Ordering is asserted, not just written: `tests/unit/ci-cd/issue_1055.rs::packaging_runs_after_the_checks_it_gates_on` |
| 6 | Changeset-based versioning | followed | `changelog.d/` fragments, `scripts/check-changelog-fragment.rs`, the `check-fragment-release-map` gate, and `evidence-check` on pull requests. Docs-only changes are exempt through `detect-changes` |
| 7 | Validate the actual merge result | followed | `scripts/simulate-fresh-merge.sh` at seven call sites across `release.yml`, `desktop-release.yml` and `macos-core-tests.yml`, with the base commit pinned once by `pin-base-commit.yml` (issue #1017) so every lane merges the same base |
| 8 | Pre-commit hooks | followed | `.pre-commit-config.yaml`: whitespace, EOF, YAML/TOML parse, large files, merge markers, debug statements, `cargo fmt` |
| 9 | Release automation | followed | `auto-release` (push to `main`) and `manual-release` (`workflow_dispatch`), `version-check` + `scripts/check-version-modification.rs` prohibiting manual bumps, and the rule-blocked-push recovery in `scripts/push-to-shared-branch.sh`. Trusted publishing: see §3 |
| 10 | Concurrency control | followed | 44 concurrency groups. Read-only jobs use `check-…-<job>` with `cancel-in-progress` on non-`main` refs; every writer uses the repository-scoped `formal-ai-repository-writes` with `queue: max`. The rebase-and-retry half is `scripts/push-to-shared-branch.sh`, which classifies a ruleset rejection apart from a lost race instead of retrying both |
| 11 | Secrets detection | followed | `secrets-scan` job over `scripts/check-secrets.sh` (secretlint, version-pinned), scoped to the files a change touches |
| 12 | Documentation validation | followed | `links.yml` runs lychee behind a parser pre-check that fails in seconds on a malformed link; the `check_requirements_document` and `check_tests_as_docs` gates verify required sections. Docs line limits: see §4 |
| 13 | Container images: native runners per architecture | **gap** | The published images are `linux/amd64` only. No `platforms:` input on any of the five `docker/build-push-action@v7` steps, and no `ubuntu-24.04-arm` leg for the image build (the desktop lane does use one). Caching is correct (`type=gha` both ways on the publishing builds) and the release is not gated on the image push. See §5 |
| 14 | Lint the workflows themselves | followed | `.github/workflows/workflows.yml`: actionlint as the Docker image **pinned by digest** (issue #1079 D3) so ShellCheck is always present, plus zizmor with `.github/zizmor.yml`. Added by issue #1076 after the first zizmor run returned four high-severity template-injection findings |
| 15 | Audit the dependency tree | followed, stronger than the template | `security.yml` runs `cargo-audit@0.22.2` over the committed `Cargo.lock` on a weekly `schedule:` as well as on push, and `scripts/check-javascript-dependencies.sh` discovers every committed lockfile with `git ls-files` (so a new workspace cannot be forgotten), audits `bun.lock` too, and fails at `--audit-level=moderate` rather than `high` |
| 16 | Prove you can publish before you build | **implemented in this pull request** | `release-preflight` job + `scripts/preflight-credentials.sh`; every release job now `needs:` it. See §6 |

## 1. How each verdict was reached

Every "followed" row above names a file and a line or a gate. The three rows
that are not "followed" are the ones with a measurement behind them, and those
measurements are below. A principle was never marked followed because a
plausible-looking string exists somewhere -- that mistake is what the #1079
round's `persist-credentials` claim turned out to be, corrected in
`../upstream-reports/README.md`.

## 2. Principle 3: the JavaScript half is unenforced

Measured on this tree: 372 `.js`/`.mjs` files in scope (excluding `target/`,
`node_modules/` and vendored trees), of which **353 do not match
`prettier --check`**. No workflow, gate or pre-commit hook runs Prettier,
ESLint or any other JavaScript formatter or linter. The Rust half of the same
principle is enforced three ways (gate, pre-commit hook, and `cargo fmt` in the
`lint` job).

This is a real gap and it is not fixed here, deliberately: the fix is a
353-file reformat, which would bury every reviewable line of this pull request
in whitespace. It is filed as
[#1083](https://github.com/link-assistant/formal-ai/issues/1083) with the
measurement and a baseline-ratchet design (the same shape as
`scripts/check-coverage-ratchet.rs` + `coverage/baseline.json`, which this
repository already uses to adopt a gate without a flag-day rewrite).

## 3. Principle 9: trusted publishing

The principle asks for OIDC trusted publishing "where supported (npm, PyPI,
crates.io)". This repository publishes one crate and two container images:

- **crates.io** -- publishing uses `CARGO_REGISTRY_TOKEN`. Trusted publishing
  for crates.io exists but requires configuration on the registry side that a
  pull request cannot perform; the preflight added in §6 probes the token's
  ownership of the crate before the build instead, which is the part CI can
  check.
- **Docker Hub / GHCR** -- `DOCKERHUB_TOKEN` and the job's `GITHUB_TOKEN`. The
  preflight probes both with a write (a blob upload session opened and
  cancelled), which is the only check that distinguishes push from pull; a
  login does not, and the measurements proving that are in
  `../upstream-reports/templates-no-release-preflight.md`.

`id-token: write` is granted where it is actually used (`deploy-pages`, and the
two desktop-release jobs), not repository-wide.

## 4. Principle 12: docs line limits

The principle suggests a docs cap "e.g., max 2500 lines". Measured before
adopting one: six committed `.md` files exceed 2500 lines, and **all six are
generated or snapshotted changelogs** -- this repository's own `CHANGELOG.md`
(8119 lines) and five `dev/log/**/references/templates/*/CHANGELOG.md`
snapshots taken by previous issue rounds.

A cap here would therefore fire on nothing a human wrote, and its only
achievable green state is an exclusion list covering exactly the files it
fires on. That is a gate that reports a finding nobody can act on, which §4 of
the issue analysis calls out as its own defect class. Recorded as a considered
decision rather than an unnoticed gap; if a hand-written document ever
approaches the limit, the cap becomes worth adding.

## 5. Principle 13: single-architecture images

Measured: five `docker/build-push-action@v7` steps in `release.yml`, none with
a `platforms:` input, so every published image is `linux/amd64`. There is no
`setup-qemu-action` anywhere (the principle's anti-pattern is absent), caching
is set correctly on the publishing builds, and the release is not gated on the
image push -- so three of the principle's five bullets are already satisfied
and the gap is precisely "one architecture".

`ubuntu-24.04-arm` is already used by `desktop-release.yml`, so the runner is
available to this repository; what is missing is the two-leg build plus a
`docker buildx imagetools create` manifest merge, in the pipeline's most
sensitive job. That is a change whose only real verification is a release run,
which this pull request cannot perform, and it changes what is published rather
than what is reported -- so it is out of the "false positives, false negatives,
warnings and errors" scope this issue is about, and in scope for R4 only as a
finding. Filed rather than attempted here; the finding, its evidence and the
implementation sketch are in `container-image-architectures.md` next to this
file.

## 6. Principle 16: implemented

`release-preflight` runs before anything expensive and every release job
`needs:` it. It probes with a write, reports every failure rather than the
first, distinguishes `unknown` (a timeout or an HTTP 429) from a failure, and
is advisory on pull requests (`PREFLIGHT_MODE=report`) and blocking on a push
to `main` (`PREFLIGHT_MODE=release`) -- the asymmetry the principle opens with.

Behaviour is covered by `tests/unit/ci-cd/issue_1081/release_preflight.rs` and
reproduced end to end, without network access, by the fake-`curl` case runner
in `../../../../../../../experiments/issue_1081_preflight/`.

The same gap exists in all five templates and was filed there: rust#167,
js#181, python#77, php#11, csharp#57.
