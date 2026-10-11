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
| R1169-2 | "Latest" comes from the publisher's own registry — crates.io `max_stable_version`, npm `dist-tags.latest`, GitHub `releases/latest`, Docker Hub tags, the rust stable channel — never a memorized list. | Implemented (re-checked 2026-10-07): the `latest_*` fetchers (curl + `USER_AGENT`, 30 s cap) hand each body to a pure reader in `scripts/check-dependencies-latest-parsers.rs` — `crates_io_latest` (`max_stable_version`), `npm_latest` (`dist-tags.latest`), `github_latest` (`tag_name`), `docker_hub_latest` (the highest dotted-numeric tag, so `latest`, suffixed variants and a rebuilt old tag listed first do not win) and `rust_channel_latest` (`[pkg.rust]` `version`, first word). This pass fixed three readers that could not have answered: the JSON reader required `"key":"value"` with no space, which api.github.com never sends (it pretty-prints `"tag_name": "v7.0.1"`); the rust channel reader looked for a `[rust]` section the channel file does not have; and a sub-path action (`github/codeql-action/init`) was asked for releases of a repository that does not exist (`action_repository` now asks `github/codeql-action`). The readers were also compiled out of every test build; they are now pure functions pinned against real-shaped answers by `registry_answers_parse_in_their_publishers_shapes` in `scripts/check-dependencies-latest-tests.rs`, which `rust/tests/unit/issue_1169_dependency_currency.rs` compiles into the unit suite. No live registry run was recorded locally. |
| R1169-3 | A dependency may be held back only with a same-line annotation naming the tracking issue; the gate verifies the annotation, and drift without one fails. | Implemented (re-checked 2026-10-07): `blocked_annotation` (Cargo.toml, same line) and the `"<name>//"` note of a package.json both go through `tracking_reference`, which accepts the first token of the reason, bare or `<…>`-bracketed, only when it is an `https://` issue URL (`…/issues/<n>`) or a published security advisory (`…/advisories/GHSA-…`, the record that tracks an unpatched vulnerability). Before this pass a package.json note was accepted with any text after `blocked:` and the documented `<https://…>` bracket form was rejected in Cargo.toml. Blocked findings print `(blocked)` and `exit_code` returns 0 for them, unblocked drift returns 1. `links-notation = "0.16.1" # blocked: …` was the first resident and was lifted on 2026-10-07 when codec 0.8.0 moved to links-notation 0.23 (the repository-tree test that still asserted it now asserts the current residents); the residents are the desktop `command-stream` note (GHSA-vfj7-8cjw-p6xm) and the desktop/VS Code `browser-commander` and `@kreuzberg/html-to-markdown-node` override notes. Documented in `CONTRIBUTING.md`. Pinned by `blocked_notes_must_carry_a_tracking_reference` and `exit_codes_separate_drift_from_an_unverifiable_run` (embedded suite) and `the_repository_tree_collects_every_ecosystem` in `rust/tests/unit/issue_1169_dependency_currency.rs`. |
| R1169-4 | The tree itself is moved to latest: `cargo update` resolution (already applied to `rust/Cargo.lock` — command-stream 1.2.0, webrtc 0.21.0, toml_edit 0.25.15, clap 4.6.7 …), npm manifests hand-bumped with real registry integrity (react/react-dom 19.3.0, dompurify 3.4.16, marked 18.0.14, command-stream 1.2.0, electron 44.4.5, electron-builder 26.17.0 with its app-builder-lib/dmg-builder/builder-util/electron-publish/squirrel-windows ring, playwright 1.63.0 across desktop + e2e + vscode), and the pin edits (`konrad/box-dind` 2.10.2, `actions/cache` v6 in both composite actions, `zizmor-action` v0.6.4). | Implemented (re-checked 2026-10-07, this pull request). The listed moves are in the tree: `rust/Cargo.lock` resolves command-stream 1.2.0, webrtc 0.21.0, toml_edit 0.25.15 and clap 4.6.7; `package.json` pins react/react-dom 19.3.0, dompurify 3.4.16 and marked 18.0.14; `desktop/package.json` pins command-stream 1.2.0, electron ^44.4.5, electron-builder ^26.17.0 and playwright ^1.63.0; vscode and `rust/tests/e2e` pin playwright ^1.63.0 with their locks; `Dockerfile` runs `FROM konard/box-dind:2.10.2`; all three composite actions use `actions/cache@v6`; `zizmor-action@v0.6.4`. The desktop/e2e lock edits were verified closed by a before/after range-resolution diff over the `packages` map. The stale `konard/box-dind:2.1.1` comments in `Dockerfile.slim` and `container-images.yml` now name 2.10.2. The desktop and VS Code sandbox image now name `konard/box-dind:2.10.2` too (verified on Docker Hub as the current tag, published for linux/amd64 and linux/arm64): `desktop/lib/tool-router.cjs` `SANDBOX_IMAGE`, `desktop/scripts/smoke.mjs`, `vscode/src/lib/config.cjs` `DEFAULT_IMAGE`, the `vscode/package.json` setting default, the `docker_microservice` environment of `data/seed/environments.lino` (embedded mirror too), README, `vscode/README.md` and the architecture/desktop/vscode docs, with `docker_runtime.rs`, `docs_requirements.rs` and `formal_ai_cli.rs` pinning 2.10.2. The one remaining 2.1.1 pin is the historical issue #195 case-study research note, which records what was true then. No desktop or VS Code smoke run against the new image was recorded locally; the pinning tests run in CI. |
| R1169-5 | A daily job refreshes the registry answers, applies every unblocked bump, re-resolves lockfiles, re-runs the CI gates, and opens one PR with whatever moved. | Implemented (2026-10-07): `.github/workflows/dependencies-latest.yml` (cron `23 5 * * *`, off the fleet's :00 stampede per issue #1021, plus `workflow_dispatch`) installs what it runs — the release workflow's Rust toolchain, Bun from `.bun-version`, Node 22 and `rust-script` through `scripts/install-rust-script.sh` (the earlier draft never installed `rust-script` and could not have run) — then `--refresh-snapshot`, `--apply --offline` failing on any exit other than 0 or 1, `cargo update --workspace`, one lock-only re-resolution per committed lockfile by the tool that owns it (`bun install --lockfile-only` for `bun.lock`, `npm install --package-lock-only` for `package-lock.json`; the draft ran `bun install --mode=save || true` in npm directories and swallowed every failure), the three `run-ci-gates` stages with the WASM worker and web bundle builds they need, and `peter-evans/create-pull-request@v8`. Pinned by `rust/tests/web/issue-1169-dependencies-latest-workflow.test.mjs` (5 tests, passing locally). The first scheduled run happens on the default branch after merge; no run of it was recorded from this branch. |
| R1169-6 | Registry flakiness never reddens CI: `--offline` answers from `data/meta/dependency-registry-snapshot.lino`, written by `--refresh-snapshot`; an unreachable registry exits 3, distinct from findings. | Implemented (re-checked 2026-10-07): `read_snapshot`/`write_snapshot` (flat lino, one `registry_entry <ecosystem>/<name>` record per answer) and the exit-code ladder 0 clean / 1 findings / 2 usage / 3 unverifiable. `offline_snapshot` makes an absent snapshot an error naming `--refresh-snapshot`, and `main` exits `EXIT_UNVERIFIABLE` (3) on it, so a missing or unreachable registry answer never reads as "no findings" nor as drift. The repository commits no `data/meta/dependency-registry-snapshot.lino`: the first `--refresh-snapshot` run (the daily job of R1169-5) writes it from live answers, and none was fabricated here. Pinned by `offline_without_a_snapshot_is_unverifiable_not_clean` and `offline_gate_flags_drift_and_honors_blocked_annotations` in `rust/tests/unit/issue_1169_dependency_currency.rs`, plus `snapshot_round_trip` and `exit_codes_separate_drift_from_an_unverifiable_run` in the embedded suite. |
| R1169-7 | `--apply` moves manifests (not lockfiles) and leaves annotated pins alone; lockfiles are re-derived by the daily job's resolvers. | Implemented (re-checked 2026-10-07): `apply_latest` + the per-ecosystem line rewriters move manifests and `uses:`/`FROM`/`rust-version` lines, never lockfiles, and skip every finding that carries a blocked annotation. Workflow refs are compared and rewritten at their own precision: a moving branch or tool selector (`dtolnay/rust-toolchain@stable`, `taiki-e/install-action@nextest`) is not a release pin and is not collected; a floating major (`@v7`) is current while the latest release is a `7.x.y` (`selects_latest`) and moves to `@v8`, not `@v8.0.1` (`same_precision`); `rewrite_uses_line` matches the declared ref with its `v`, which the earlier draft dropped, so a `@v6` line was never rewritten; a 40-hex commit SHA is reported unverifiable. Pinned by `apply_moves_unblocked_manifests_and_leaves_blocked_alone` in `rust/tests/unit/issue_1169_dependency_currency.rs` and `action_refs_float_and_rewrite_at_their_own_precision`, `apply_rewrites_manifests_not_locks` and `apply_leaves_blocked_lines_alone` in the embedded suite. |
| R1169-8 | The gate's parsers and rules are pinned by tests that compile the script into the unit suite. | Implemented: sixteen embedded tests in `scripts/check-dependencies-latest-tests.rs` (the script's `#[cfg(test)]` module; eleven before the 2026-10-07 pass) plus four fixture-tree tests in `rust/tests/unit/issue_1169_dependency_currency.rs` (`#[path]` include, the `ci_gates.rs` idiom). |

The unit suite registers `issue_1169_dependency_currency` in
`rust/tests/unit/mod.rs`. The first live `--refresh-snapshot` run (2026-10-07,
79 entries) is committed as `data/meta/dependency-registry-snapshot.lino`, so
the gate now runs on every pull request through
`data/meta/ci-gates/check-dependencies-latest.lino` (`--offline`, after the
embedded tests). That run exposed and fixed five collector bugs: multi-stage
aliases (`FROM builder`) and official images without `library/` looked up on
Docker Hub, anonymous GitHub API calls exhausting the hourly limit (a
`GH_TOKEN` now reaches api.github.com only, via curl's stdin config), string
rather than numeric drift (a pin past a lagging dist-tag read as drift), and a
`rust:1.98-slim` rewrite that dropped the `-slim` variant. Every manifest was
then moved to the snapshot's latest. Floating major `uses:` refs (`@v7`) are no
longer findings while their major is the latest, and moving refs (`@stable`)
are not collected, so the remaining first-run findings are real drift.

VS Code packaging: `@vscode/vsce` is `^4.0.0` (resolved 4.0.0 in
`vscode/package-lock.json`, full npm lockfile resolution, 2026-10-07). The bump
closes the former 3.9.2 currency gap and drops the 3.x chain through
`secretlint` > `globby` > `fast-glob` > `micromatch` > `braces`, whose
GHSA-vfj7-8cjw-p6xm has no patched release. 4.0.0 requires Node >=22, which
every packaging workflow already uses (22 or 24.x). The same pass overrides
`global-agent` to ^4.1.3 in `desktop/` (removing `roarr` > `sprintf-js`,
GHSA-hp3w-g68c-fv3c, no patched release), `proxy-addr` to ^2.0.8 in the root
`bun.lock` and `compression` to ^1.8.2 in `rust/tests/e2e/`; all five lockfiles
audit clean at `--audit-level=moderate`.
Playwright and playwright-core are updated together to 1.63.0 in that lockfile,
using the same registry integrity records as the e2e lockfile. No builds or tests
were run for these integration changes.

Latest-release pass (2026-10-07, lockfile-only; nothing was built locally, so
the Rust adaptations compile and run first in CI):

- Rust (`rust/Cargo.toml` + `cargo update`): links-notation 0.16.1 → 0.23.0
  (pin lifted), lino-objects-codec 0.7.0 → 0.8.0, lino-arguments =0.3.0 →
  0.4.0 (the exact pin had no recorded reason; 0.4.0 only moves `ctor` to 1.x,
  and link-cli 1.0.0 still pulls 0.3.0 beside it), link-calculator 0.20.3 →
  0.22.0, link-cli 0.2.11 → 1.0.0, web-search 0.5.0 → 0.6.0 with the
  recommended `merge` feature, command-stream 1.2.0 → 1.5.3. Only `cc` 1.2.67
  (tree-sitter grammars) and `generic-array` 0.14.7 (RustCrypto 0.14 line) stay
  behind, held by transitive requirements; meta-language 0.58.2 still resolves
  its own links-notation 0.13.0.
- npm: root `@link-assistant/web-search` 0.11.1, `lino-i18n` 0.3.0, `marked`
  18.1.0 (bun.lock); e2e `lino-i18n` 0.3.0; desktop electron ^44.6.0,
  web-search ^0.11.1, puppeteer/puppeteer-core overrides ^25.12.0,
  `browser-commander` override 0.16.1 → 0.20.0, command-stream 1.2.0 → 1.5.0;
  VS Code the same web-search and override moves.
- Held back, with `"<name>//"` notes: desktop `command-stream` at 1.5.0 (1.6.0+
  and 2.0.0 depend on shelljs > fast-glob > micromatch > braces,
  GHSA-vfj7-8cjw-p6xm, no patched release); the `browser-commander` override at
  0.20.0 (0.21+ launches the installed Chrome over CDP by default, which
  web-capture 1.11.2 does not opt out of; web-capture#160); the
  `@kreuzberg/html-to-markdown-node` override at 3.5.5 (every later release,
  through 3.7.2, still declares linux musl packages npm does not have).
- Security overrides kept, still required: `proxy-addr` (express 5.2.1 allows
  ^2.0.7), `compression` (serve 14.2.6 pins 1.8.1), `global-agent`
  (`@electron/get` 3.1.0 wants ^3.0.0) and the web-capture `qs` override.
  bun.lock and all three package-locks audit clean at `--audit-level=moderate`.

