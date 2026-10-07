## Issue #1169 Every Dependency at Its Latest Release

Issue [#1169](https://github.com/link-assistant/formal-ai/issues/1169) (E134)
makes dependency currency a fact the repository enforces, not a habit it
hopes for: every dependency — crates, npm packages, GitHub Action refs, the
container base image, the rustc floor — sits at its publisher's latest
release, and a gate fails when one does not. The one sanctioned hold-back is
an in-tree annotation naming the issue that tracks it.

| ID | Requirement | Status / Evidence |
| --- | --- | --- |
| R1169-1 | A gate reads every ecosystem the repository declares: `rust/Cargo.toml` against `rust/Cargo.lock`, every `package.json` against its lockfile (manifest-range fallback where a directory ships no lock), `uses:` refs in `.github/workflows` and composite actions, `FROM` bases in the `Dockerfile`, and the `rust-version` floor. | Implemented: `collect` in `scripts/check-dependencies-latest.rs`, walking with `node_modules`/`target`/`.git`/`.claude`/`cache` excluded; pinned by `the_repository_tree_collects_every_ecosystem` in `rust/tests/unit/issue_1169_dependency_currency.rs`. |
| R1169-2 | "Latest" comes from the publisher's own registry — crates.io `max_stable_version`, npm `dist-tags.latest`, GitHub `releases/latest`, Docker Hub tags, the rust stable channel — never a memorized list. | Implemented: the `latest_*` fetchers (curl + `USER_AGENT`, 30 s cap), each parsed from the publisher's own answer. |
| R1169-3 | A dependency may be held back only with a same-line annotation naming the tracking issue; the gate verifies the annotation, and drift without one fails. | Implemented: `blocked_annotation` requires `# blocked: <https://…/issues/…>` in Cargo.toml and a `"<name>//"` sibling key in package.json; blocked findings are reported with `(blocked)` and do not fail, unblocked drift exits 1. Documented in `CONTRIBUTING.md` beside the self-maintained-dependencies policy. `links-notation = "0.16.1" # blocked: …` is the first resident: lino-objects-codec 0.7.0 requires `^0.16`. |
| R1169-4 | The tree itself is moved to latest: `cargo update` resolution (already applied to `rust/Cargo.lock` — command-stream 1.2.0, webrtc 0.21.0, toml_edit 0.25.15, clap 4.6.7 …), npm manifests hand-bumped with real registry integrity (react/react-dom 19.3.0, dompurify 3.4.16, marked 18.0.14, command-stream 1.2.0, electron 44.4.5, electron-builder 26.17.0 with its app-builder-lib/dmg-builder/builder-util/electron-publish/squirrel-windows ring, playwright 1.63.0 across desktop + e2e + vscode), and the pin edits (`konrad/box-dind` 2.10.2, `actions/cache` v6 in both composite actions, `zizmor-action` v0.6.4). | Partial (re-checked 2026-10-07, this pull request). The listed moves are in the tree: `rust/Cargo.lock` resolves command-stream 1.2.0, webrtc 0.21.0, toml_edit 0.25.15 and clap 4.6.7; `package.json` pins react/react-dom 19.3.0, dompurify 3.4.16 and marked 18.0.14; `desktop/package.json` pins command-stream 1.2.0, electron ^44.4.5, electron-builder ^26.17.0 and playwright ^1.63.0; vscode and `rust/tests/e2e` pin playwright ^1.63.0 with their locks; `Dockerfile` runs `FROM konard/box-dind:2.10.2`; all three composite actions use `actions/cache@v6`; `zizmor-action@v0.6.4`. The desktop/e2e lock edits were verified closed by a before/after range-resolution diff over the `packages` map. The stale `konard/box-dind:2.1.1` comments in `Dockerfile.slim` and `container-images.yml` now name 2.10.2. Open: the desktop and VS Code sandbox image (`desktop/lib/tool-router.cjs`, `vscode/src/lib/config.cjs`, `vscode/package.json`), the `docker_microservice` environment seed, README and the architecture/desktop/vscode docs still name `konard/box-dind:2.1.1`, pinned by `docker_runtime.rs`, `docs_requirements.rs` and `formal_ai_cli.rs`. Moving them changes the sandbox the desktop and VS Code surfaces run, which needs those surfaces' smoke runs. |
| R1169-5 | A daily job refreshes the registry answers, applies every unblocked bump, re-resolves lockfiles, re-runs the CI gates, and opens one PR with whatever moved. | Implemented: `.github/workflows/dependencies-latest.yml` (cron `23 5 * * *`, off the fleet's :00 stampede per issue #1021) — `--refresh-snapshot`, `--apply --offline`, `cargo update --workspace`, `bun install` per manifest directory, `run-ci-gates` all three stages, `peter-evans/create-pull-request@v8`. Not yet run (needs CI). |
| R1169-6 | Registry flakiness never reddens CI: `--offline` answers from `data/meta/dependency-registry-snapshot.lino`, written by `--refresh-snapshot`; an unreachable registry exits 3, distinct from findings. | Implemented: `read_snapshot`/`write_snapshot` (flat lino, one `registry_entry <ecosystem>/<name>` record per answer) and the exit-code ladder 0/1/2/3 in `main`. |
| R1169-7 | `--apply` moves manifests (not lockfiles) and leaves annotated pins alone; lockfiles are re-derived by the daily job's resolvers. | Implemented: `apply_latest` + the per-ecosystem line rewriters; pinned by `apply_moves_unblocked_manifests_and_leaves_blocked_alone`. |
| R1169-8 | The gate's parsers and rules are pinned by tests that compile the script into the unit suite. | Implemented: eleven embedded tests in the script plus three fixture-tree tests in `rust/tests/unit/issue_1169_dependency_currency.rs` (`#[path]` include, the `ci_gates.rs` idiom). |

Residuals for the integrating commit: the unit-suite registration line in
`rust/tests/unit/mod.rs` (shared wiring, deliberately untouched here), the
`data/meta/ci-gates/` registration shard, and the first
`--refresh-snapshot` run to seed `data/meta/dependency-registry-snapshot.lino`
(all under the parent's `data/meta`/shared-wiring ownership). The gate is
expected to run red on first CI invocation for legacy pins it newly
surfaces — most `uses:` refs across `.github/workflows` predate this gate
and carry no blocked annotation; the daily job's first PR is the designed
way they move.

VS Code packaging residual: `@vscode/vsce` remains at `^3.9.2`, matching
`vscode/package-lock.json`. This is an unresolved dependency currency gap. Publisher
metadata for 4.0.0 requires Node >=22 and a changed dependency graph, including
`@napi-rs/keyring`, `@vscode/vsce-sign` ^2.1.0, and Secretlint core/types/source
creator packages. Complete this bump in CI with a full npm lockfile resolution;
replacing the top-level package version alone would leave missing dependencies.
Playwright and playwright-core are updated together to 1.63.0 in that lockfile,
using the same registry integrity records as the e2e lockfile. No builds or tests
were run for these integration changes.
