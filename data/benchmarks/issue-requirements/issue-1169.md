Part of the E127 umbrella (#1183). The architect's words: "we should also demand to update all our dependencies to the latest versions."

## Requirement

Every dependency of Formal AI is at its latest release: direct and transitive Rust crates, the Rust toolchain, npm/bun packages of every `package.json` (root, `vscode/`, `desktop/`, `rust/tests/e2e/`), GitHub Actions, Docker base images. A dependency may stay behind only while an **open issue linked beside the pin** names what blocks the update (extending the existing CONTRIBUTING "Self-maintained dependencies and patches" policy, which today covers only `[patch]` entries). No grace period, no allowlist without an issue.

## State on `main` (d209aac64), measured 2026-09-29

Rust, direct:

| Crate | Declared | Locked | Latest | Note |
|---|---|---|---|---|
| `links-notation` | `0.16.1` | 0.16.1 | **0.22.0** | self-maintained; also 0.13.0 pulled in by `meta-language` 0.58.2 (upstream: https://github.com/link-foundation/meta-language/issues/197) |
| `command-stream` | `=0.16.0` | 0.16.0 | **1.1.1** | self-maintained, exact pin; also pulls `which` 7.0.3 |
| `link-cli` | `0.2.10` | 0.2.10 | 0.2.11 | self-maintained |
| `webrtc` | `0.20` | 0.20.3 | 0.21.0 | |
| `clap` | `4.6` | 4.6.6 | 4.6.7 | |
| `toml_edit` | `0.25.13` | 0.25.13 | 0.25.15 | |
| `which` | `8` | 8.0.5 | 8.0.6 | |

Rust, transitive: `cargo update --dry-run` would move **86** packages; `--verbose` lists 4 held back by the manifest (`cc`, `command-stream`, `generic-array`, `links-notation`). Older duplicates in the lock: `nom` 7.1.3, `sha2` 0.10.9, `base64` 0.22.1 (via `abnf`, `x509-parser`, `rtc*`, `reqwest`, `hyper-util`). Toolchain `rust-version = "1.98"` and `rust:1.98-slim` match stable 1.98.1.

npm:

| Package | Declared | Latest | Where |
|---|---|---|---|
| `command-stream` | 0.19.0 | **1.2.0** | desktop (self-maintained) |
| `@vscode/vsce` | ^3.9.2 | **4.0.0** | vscode |
| `react`, `react-dom` | 19.2.8 | 19.3.0 | root |
| `dompurify` | 3.4.14 | 3.4.16 | root |
| `marked` | 18.0.11 | 18.0.14 | root |
| `electron` | ^44.1.0 | 44.4.5 | desktop |
| `electron-builder` | ^26.15.7 | 26.17.0 (`v26` dist-tag; the `latest` tag lags at 26.15.3) | desktop |
| `playwright`, `@playwright/test` | ^1.62.1 | 1.63.0 | vscode, desktop, e2e |

GitHub Actions: `actions/cache@v4` in `.github/actions/green-ledger/action.yml:61` and `.github/actions/formal-ai-binary/action.yml:40` (v6 elsewhere, latest v6.1.0); `zizmorcore/zizmor-action@v0.6.2` (latest v0.6.4); actions are pinned by major tag rather than SHA of the latest release.

Docker: `FROM konard/box-dind:2.1.1` in `Dockerfile:73` (latest 2.10.2).

No `dependabot.yml`/Renovate configuration exists; nothing reports staleness.

## What to do

1. Update everything above to latest in one pull request, including the major versions (`links-notation` 0.22, `command-stream` 1.x in Rust and JS, `webrtc` 0.21, `@vscode/vsce` 4); adapt the code to the new APIs. Where an update is blocked by a dependency's own defect, file the issue in that dependency (written generally, per CONTRIBUTING) and link it beside the pin.
2. Add `scripts/check-dependencies-latest.rs` and run it in the CI rust stage and pre-commit: compares every manifest and lockfile (Cargo, bun/npm, Actions, Dockerfile `FROM`, toolchain) with the registries' latest releases; fails on anything behind without a linked open issue on the same line. Offline mode uses a cached registry snapshot and says so.
3. Add a scheduled workflow (daily) that runs the updates, builds, tests, and opens a pull request; red updates open an issue instead.
4. Record the requirement as a shard in `docs/requirements/` (regenerate `REQUIREMENTS.md` with `rust-script scripts/assemble-requirements.rs --write`), a traceability row, and a line in CONTRIBUTING next to the self-maintained dependency policy.

## How to test

- `rust-script scripts/check-dependencies-latest.rs` passes on the updated tree and fails when any manifest pin is lowered by one version.
- `cargo update --dry-run` reports no updates; `bun outdated` in each package directory reports none.

## Requirements

- **R1** Every direct dependency in `rust/Cargo.toml`, every `package.json` (root, `vscode/`, `desktop/`, `rust/tests/e2e/`; `experiments/**` excluded because experiments are frozen records), every `uses:` in `.github/workflows/*.yml` and `.github/actions/*/action.yml`, every `FROM` in `Dockerfile*`, and `rust-version` / the CI toolchain are at their latest release.
- **R2** `cargo update --dry-run` prints no `Updating` line; `bun outdated` (or `npm outdated`) prints nothing in each package directory.
- **R3** A pin may stay behind only with an open issue URL in a comment on the same line (`# blocked: https://github.com/<owner>/<repo>/issues/<n>` in TOML/YAML/Dockerfile; a `"//"`-keyed sibling note in `package.json`), and the gate verifies the issue is open.
- **R4** `scripts/check-dependencies-latest.rs` implements R1–R3 and runs in the CI rust stage and in pre-commit.
- **R5** A daily workflow updates everything, builds, tests, and opens (or refreshes) one pull request; a red build opens or updates one issue with the failing log instead.
- **R6** The major bumps are completed in the same change: `command-stream` 0.16 → 1.1 (Rust) and 0.19 → 1.2 (desktop), `webrtc` 0.20 → 0.21, `@vscode/vsce` 3 → 4, `links-notation` 0.16 → 0.22 (together with #1182 R11: the crate is declared but unused today), `electron-builder` to the `v26` dist-tag 26.17.0.
- **R7** The same rule holds in every repository Formal AI depends on or works with; the freshness issues filed there are tracked here (list below).

## Design

### `scripts/check-dependencies-latest.rs` (new, rust-script)

- **Inputs:** `rust/Cargo.toml` + `rust/Cargo.lock`; each `package.json` in R1 with its lockfile (`bun.lock` / `package-lock.json`); `.github/workflows/*.yml`, `.github/actions/*/action.yml`; `Dockerfile*`; `rust/Cargo.toml` `rust-version` and every `dtolnay/rust-toolchain` / `rust:` image tag.
- **Registries:** crates.io `GET /api/v1/crates/<name>` → `max_stable_version`; npm `GET https://registry.npmjs.org/<name>` → `dist-tags.latest`, and for a pin whose declared major is behind `latest` but a `v<major>` dist-tag exists (electron-builder), the newest tag of the latest major; GitHub `GET /repos/<owner>/<repo>/releases/latest` for actions (compare the major tag or the pinned SHA's tag); Docker Hub `GET /v2/repositories/<ns>/<name>/tags?ordering=last_updated` and GHCR via `gh api /orgs/<org>/packages/container/<name>/versions` for images; `https://static.rust-lang.org/dist/channel-rust-stable.toml` for the toolchain.
- **Transitive Rust dependencies:** `cargo update --dry-run --verbose`; every `Updating` line is a finding; every `Unchanged … (available: …)` line needs the R3 annotation on the direct dependency that holds it back.
- **Offline:** `--offline` reads `data/meta/dependency-registry-snapshot.lino` (written by `--refresh-snapshot`) and prints the snapshot date on every line.
- **Output:** one line per finding, `<file>:<line> <name> <declared> -> <latest> [blocked by <url>]`, then a summary; `--format lino` writes the same as Links Notation for the case study.
- **Exit codes:** 0 all latest or blocked-with-open-issue; 1 anything behind without an annotation; 2 an annotation points to a closed or missing issue; 3 a registry could not be reached (never treated as success; offline mode exists for that).

### Daily workflow `.github/workflows/dependencies-latest.yml` (new)

```yaml
on:
  schedule: [{ cron: "23 5 * * *" }]
  workflow_dispatch: {}
permissions: { contents: write, pull-requests: write, issues: write }
jobs:
  update:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v7
      - uses: dtolnay/rust-toolchain@stable
      - uses: oven-sh/setup-bun@v2
      - run: cargo update --manifest-path rust/Cargo.toml
      - run: rust-script scripts/check-dependencies-latest.rs --apply   # rewrites manifests, action refs, FROM lines to latest
      - run: for d in . vscode desktop rust/tests/e2e; do (cd "$d" && bun update --latest); done
      - run: RUSTUP_TOOLCHAIN=stable rust-script scripts/run-ci-gates.rs --stage rust
      - uses: peter-evans/create-pull-request@v8   # already used in this repository
        with: { branch: deps/latest, title: "chore(deps): update every dependency to its latest release", commit-message: "chore(deps): latest releases" }
```

### Migration notes for the major bumps (call sites on `main` d209aac64)

- `command-stream` 0.16 → 1.1 (Rust): the only call site is `rust/src/orchestration/runner.rs` (`StreamingRunner::from_argv`, `OutputChunk::{Stdout, Stderr, Exit}`); 1.1.1 still has `OutputChunk` and `StreamingRunner::from_argv` (`src/stream.rs:80-121`), so the change is the version and any renamed fields reported by the compiler. The `=` exact pin goes.
- `command-stream` 0.19 → 1.2 (desktop JS): follow its 1.0 release notes for the `$` options (`signal`, `killSignal`, `killGrace`).
- `links-notation` 0.16 → 0.22: no Rust call site today (declared, unused); the real work is #1182 R11 (the seed loaders switch to it).
- `webrtc` 0.20 → 0.21: one call site, `rust/src/local_transport.rs`.
- `@vscode/vsce` 3 → 4: `vscode/package.json` scripts `"package": "vsce package"`; check the 4.0 changelog for changed flags.
- GitHub Actions: move `actions/cache@v4` (`.github/actions/green-ledger/action.yml:61`, `.github/actions/formal-ai-binary/action.yml:40`) to `@v6`, `zizmor-action` to `v0.6.4`.
- `Dockerfile:73` `FROM konard/box-dind:2.1.1` → `2.10.2` (or `ghcr.io/link-foundation/box-dind:2.10.2`, which Hive Mind already uses).

### The same rule in related repositories (R7)

Filed 2026-09-29, each with its measured list: link-assistant/hive-mind#2324 (agent pin 0.26.5 → 0.26.6, start-command, npm, freshness gate), link-assistant/agent#319 (59 packages behind, including `ai` 6 → 7 and own `command-stream` 0.7 → 1.2), link-foundation/meta-language#197 (9 behind, including `links-notation` 0.13 → 0.22), link-foundation/command-stream#204, link-foundation/link-cli#104, link-foundation/lino-arguments#37, link-foundation/lino-objects-codec#60, link-assistant/calculator#223 (also: 0.21.0 on `main` never published), link-assistant/web-capture#157, link-assistant/web-search#26. `links-notation` itself is current (0 behind).

## Tests

- `rust/tests/unit/issue_1169_dependency_gate.rs`: on fixture manifests, the checker reports a pin lowered by one version, accepts a pin annotated with an open issue, rejects one annotated with a closed issue (mocked GitHub transport), and parses every `uses:` form (`owner/repo@vN`, `@sha # vN`, local `./.github/actions/*`).
- Commands: `RUSTUP_TOOLCHAIN=1.98.1 cargo test --test unit issue_1169_dependency_gate`; `rust-script scripts/check-dependencies-latest.rs` (exit 0 on the updated tree).

## Definition of done

- [ ] Every table row above is at its latest release or carries an R3 annotation to an open issue.
- [ ] Requirement shard `docs/requirements/issue-1169-dependencies-latest.md`; `rust-script scripts/assemble-requirements.rs --write`; traceability rows; CONTRIBUTING gains the rule next to "Self-maintained dependencies and patches".
- [ ] Case study `docs/case-studies/issue-1169/` with the measurements of 2026-09-29 and the first daily run.
- [ ] Changelog fragment in `changelog.d/`; `RUSTUP_TOOLCHAIN=1.98.1 rust-script scripts/run-ci-gates.rs --stage rust` green with the gate.

## Depends on / blocks

- Depends on nothing to start. The upstream freshness issues above land in their own repositories; any pin they hold back carries the R3 annotation meanwhile.
- Coordinates with #1182 (E147): dependency releases that let Formal AI delete local modules are adopted in the same version-bump pull request.
- Separate from #1168 (E133), which covers versions inside the code Formal AI generates for others.



- Credentials: this workflow takes its GitHub token from the shared resolver of #1187 (E151): GitHub App or one `AUTOMATION_TOKEN` when configured, otherwise the default `GITHUB_TOKEN` with dispatched checks and orphan-branch isolation; it never requires a token.

