# R5 — compliance with hive-mind `docs/CI-CD-BEST-PRACTICES.md`

Fifteen numbered principles, audited one at a time against this repository as of
`701d6a4`. The document is archived at
`../references/hive-mind-CI-CD-BEST-PRACTICES.md` so this audit stays legible
against the version that was current when issue #1076 was filed.

Verdicts: **met** — nothing to do. **met, with a gap closed here** — the
principle was partly implemented and this pull request completes it. **gap,
recorded** — genuinely not met, with the reason it is not being closed here.

| # | Principle | Verdict |
| --- | --- | --- |
| 1 | Run checks only on relevant file changes | met |
| 2 | File size limits | met |
| 3 | Automated code formatting | met |
| 4 | Static analysis & linting | met |
| 5 | Fast-fail job ordering | met |
| 6 | Changeset-based versioning | met |
| 7 | Validate the actual merge result | met |
| 8 | Pre-commit hooks | met |
| 9 | Release automation | **gap, recorded** (OIDC) |
| 10 | Concurrency control | met |
| 11 | Secrets detection | met |
| 12 | Documentation validation | met |
| 13 | Container images: native runners per architecture | **gap, recorded** |
| 14 | Lint the workflows themselves | **met, with a gap closed here** |
| 15 | Audit the dependency tree | met |

---

## 1. Run checks only on relevant file changes — met

`detect-changes` jobs exist in `release.yml`, `coverage.yml`,
`summarization-ratchet.yml`, `write-effect-ladder.yml` and `task-ladder.yml`.
`coverage.yml` gates both `coverage` and `browser-coverage` on it, and
`tests/unit/ci-cd/workflow_coverage.rs` pins that contract (issue #846).

## 2. File size limits — met

`scripts/check-file-size.rs`, registered as a CI gate, plus
`scripts/check-worker-line-budget.rs` for the WASM worker.

## 3. Automated code formatting — met

`data/meta/ci-gates/check-formatting.lino` runs `cargo fmt --all -- --check`.
Gates live one-per-file since issue #991, so adding one does not edit a workflow.

## 4. Static analysis & linting — met

`[lints.clippy]` in `Cargo.toml`, and the `lint` job runs
`cargo clippy --lib --bins --tests --all-features -- -D warnings`. The `-D
warnings` is the load-bearing part: issue #812 found clippy printing findings and
exiting 0 because every lint in `[lints.clippy]` is set to `warn`.
`tests/unit/ci-cd/workflow_release.rs` pins the flag.

## 5. Fast-fail job ordering — met

## 6. Changeset-based versioning — met

`changelog.d` fragments with `scripts/check-changelog-fragment.rs`, matching the
rust template's approach for Rust.

## 7. Validate the actual merge result — met

`scripts/simulate-fresh-merge.sh`, shared with three of the four templates
(diffs in `template-diffs/`).

## 8. Pre-commit hooks — met

`.pre-commit-config.yaml`.

## 9. Release automation — gap, recorded

Automatic and manual triggers, a release gate, and publish-then-attach ordering
are all present. The gap is the credential:

```yaml
CARGO_REGISTRY_TOKEN: ${{ secrets.CARGO_REGISTRY_TOKEN || secrets.CARGO_TOKEN }}
```

The principle asks for **OIDC trusted publishing** — "no API tokens needed in CI
(npm, PyPI, crates.io)". crates.io supports trusted publishing, so this is
achievable, but it is **not achievable from a pull request alone**: a trusted
publisher must first be registered on crates.io for this repository and workflow,
which is an account-level action outside the codebase and outside anything CI can
assert. Recorded rather than half-done — a workflow switched to OIDC before the
publisher exists fails every release.

Fix, when the crates.io side is configured: register the trusted publisher for
`link-assistant/formal-ai` + `release.yml`, add `id-token: write` to the publish
jobs (line 1359 already has it for attestations, the publish jobs do not), and
drop the `CARGO_REGISTRY_TOKEN` env bindings at lines 43, 503 and 696.

## 10. Concurrency control — met

The split the principle asks for is exactly the one in place: nine cancellable
`check-*` groups keyed by job identity, and six write jobs sharing the single
repository-scoped `formal-ai-repository-writes` group. `!cancelled()` is used
rather than `always()` throughout, which issue #1017 established.

## 11. Secrets detection — met

`scripts/check-secrets.sh`.

## 12. Documentation validation — met

`links.yml` runs lychee; `scripts/check-web-archive.mjs` adds a Wayback fallback
so a transient 404 is not reported as a broken link.

## 13. Container images: native runners per architecture — gap, recorded

No `docker/build-push-action` step in `release.yml` sets `platforms:`, so every
published image is built on `ubuntu-latest` and is **linux/amd64 only**. In the
principle's words, that "silently excludes Apple Silicon, Graviton, and arm CI
runners".

The repository does the *hard* part of this principle correctly already — there
is no `setup-qemu-action` anywhere, and `desktop-release.yml` already uses the
native `ubuntu-24.04-arm` runner for its arm64 leg, so the pattern and the runner
are both proven here. What is missing is applying it to the image builds.

Not implemented in this pull request, deliberately, for a reason worth stating
plainly: all four image builds are `push: true` steps that run only when a
release is published, so nothing about the change would be exercised by this pull
request's own CI. Shipping an unverifiable rework of the publish path inside a
pull request whose subject is *CI reliability* would risk the exact failure it
sets out to remove. It also is not a false positive, false negative, warning or
error — the images that ship today are correct, just narrower than they should
be.

Solution plan, for a change that can be staged and observed on its own:

1. Convert the two GHCR publish steps to a matrix over
   `{platform: linux/amd64, runner: ubuntu-latest}` and
   `{platform: linux/arm64, runner: ubuntu-24.04-arm}`.
2. Build with `outputs: type=image,push-by-digest=true,name-canonical=true,push=true`
   and export each digest as a job output.
3. Add a `merge-manifest` job running
   `docker buildx imagetools create -t $IMAGE:$VERSION $DIGESTS`.
4. Keep `cache-from`/`cache-to` scoped per platform
   (`scope=docker-image-${{ matrix.platform }}`) so the two legs do not evict
   each other — see D2, where an unscoped `mode=max` export was already taking
   42.9% of the shared 10 GB quota.
5. Assert what shipped: extend `scripts/verify-ghcr-visibility.sh` to check that
   the published manifest lists both platforms. The principle calls this out
   specifically, and it is what turns the change from "probably worked" into a
   gate.
6. The Dockerfile builds with `BINARY_SOURCE=compile` on the publish path, so the
   arm64 leg compiles Rust natively on an arm runner. Budget for it: this is the
   step that decides whether the `timeout-minutes: 25` added here is still right.

## 14. Lint the workflows themselves — met, with a gap closed here

This is the principle this pull request acts on, and both halves of it were
unmet.

**zizmor did not run at all.** Added: `.github/zizmor.yml` and a `zizmor` job in
`.github/workflows/workflows.yml`. The first run found four high-severity
`template-injection` findings in `release.yml`, all fixed.

**actionlint ran in the form the principle names as wrong.** The document is
unusually specific here:

> Run actionlint as the Docker image, not a bare binary. The image bundles
> `shellcheck` and `pyflakes`. A binary without `shellcheck` on `PATH` silently
> skips every shell check and exits 0 — so a green local run means nothing. This
> one detail is the difference between finding fourteen shell bugs and finding
> none.

`release.yml` installed the bare binary. Confirmed on this machine, in both
directions: with no ShellCheck installed, `actionlint` exits 0 across all
workflows; with ShellCheck on PATH it reports SC1072/SC1073. actionlint moved to
`workflows.yml` as `docker://rhysd/actionlint:1.7.12`, and its `-ignore` flag
moved into the existing `.github/actionlint.yaml` — where it stays scoped to the
one message it is about, per the principle's "a blanket `ignore` is
indistinguishable from no gate at all".

The remaining three sub-points are met and now pinned by
`ci_cd::issue_1076::workflows_are_audited_for_security_not_only_syntax`:
annotations rather than SARIF; `--min-confidence medium` and deliberately **no**
`min-severity`; and every suppression scoped with its reasoning recorded.

## 15. Audit the dependency tree — met

`security.yml` satisfies all three sub-points: it runs on `schedule` (Monday
06:00) as well as push and pull_request, `cargo-audit` reads the committed
lockfile, and the JavaScript side fails at an explicit `--audit-level=moderate`
rather than the default.
