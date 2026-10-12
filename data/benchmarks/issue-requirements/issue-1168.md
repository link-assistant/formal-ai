Part of the E127 umbrella (#1183).

## Gap

Versions in generated code are memorized strings, and they are stale:

| Pin in generated code | Where | Latest (2026-09-29) |
|---|---|---|
| `actions/checkout@11d5960…` = **v4.4.0** | `data/meta/work-item-steps.lino` `workflow_template` | v7.0.1 |
| `actions/setup-java@b6effb0…` = **v5.7.0** | `data/meta/stdout-program-contracts.lino` kotlin `ci_setup` | v6.0.1 |
| `fwilhe2/setup-kotlin@51a059f…`, Kotlin **2.3.10** | same | Kotlin 2.4.20 |
| `java-version: '21'` | same | LTS 25 (`https://api.adoptium.net/v3/info/available_releases` → `most_recent_lts: 25`) |
| `fwilhe2/setup-kotlin@51a059f…` | same | tag `v2.0` (resolve its SHA at generation time) |

This repository's own CI uses `actions/checkout@v7`; the pull requests Formal AI opens for others use v4.

## What to do

1. Resolve every version a generated artifact needs at generation time from its publisher: GitHub releases/tags API for actions (pin the commit SHA of the latest release, with the tag in a comment), official release feeds for toolchains (kotlinlang GitHub releases, `static.rust-lang.org` channel manifests, go.dev/dl JSON, …), package registries for libraries. These are E128 (#1163) sources like any other.
2. Record the resolution (URL, response hash, chosen version) in the answer's derivation.
3. Offline: use the most recent cached resolution and say so in the answer.

## How to test

- Generated Kotlin workflow pins the SHA of the current latest `actions/checkout` and `actions/setup-java` releases and the latest Kotlin (mocked transport in unit tests; one online test).
- A gate fails when any `@<sha>` or version literal for a third-party action or toolchain appears in `data/`.

---

## Requirements

**R1.** At generation time the system MUST fetch the latest release tag and its commit SHA for every third-party GitHub Action referenced in a generated CI workflow by querying `https://api.github.com/repos/{publisher}/{repo}/releases/latest`.

**R2.** At generation time the system MUST fetch the latest Kotlin release from `https://api.github.com/repos/JetBrains/kotlin/releases/latest` and use its tag as the `version` argument to `fwilhe2/setup-kotlin`.

**R3.** At generation time the system MUST take the Java version from the publisher's own statement of the current LTS: `GET https://api.adoptium.net/v3/info/available_releases` → `most_recent_lts` (25 on 2026-09-29), never a literal `'21'`.

**R4.** Every resolved version MUST be recorded in the answer's derivation as a `SourceCapture` (fields: `source_url`, `fetched_at`, `sha256`, `cached`, `bytes`) via the existing `CachedSourceClient` in `rust/src/source_fetch.rs`.

**R5.** When the live GitHub API is unreachable, the system MUST fall back to the most recent cached `SourceCapture` and MUST annotate the generated answer with `(version resolved from cache; fetched_at=<timestamp>)`.

**R6.** The `workflow_template` value in `data/meta/work-item-steps.lino` and the `ci_setup` values in `data/meta/stdout-program-contracts.lino` MUST NOT contain literal 40-hex-character SHAs or bare toolchain version strings; all such values MUST be placeholders filled at render time from resolved data.

**R7.** A CI gate MUST scan every file under `data/` and fail if any literal 40-hex-character SHA following `@` or any bare semantic version following `version:` or `kotlin-version:` for a third-party action or toolchain is found.

**R8.** Three-roots parity: `version_resolution.rs` is translated with `formal-ai translate --from rust --to js|ts --input rust/src/version_resolution.rs --write`, and `data/parity/cross-runtime-synthesis.json` gains resolution cases (fixture responses in, pinned tag/SHA out) answered identically by all three roots.

---

## Design

### Scope distinction: E133 vs E134

E134 (#1169) governs Formal AI's own `Cargo.lock`, `bun.lock`, and `Dockerfile` — the project's runtime dependencies. E133 (#1168) governs version strings embedded inside generated CI workflows destined for **other** repositories. The gate introduced here scans `data/` for hard-coded third-party action SHAs and toolchain versions; `scripts/check-dependencies-latest.rs` from E134 scans manifest and lockfiles. There is no overlap.

### New module: `rust/src/version_resolution.rs`

```rust
/// A resolved version and its provenance.
pub struct ResolvedVersion {
    pub tag: String,       // e.g. "v7.0.1"
    pub sha: String,       // 40-hex commit SHA for the tag
    pub source_url: String,
    pub fetched_at: String,
    pub response_sha256: String,
}

/// Fetch the latest release of a GitHub-hosted action or tool and return
/// its tag and the commit SHA that tag points to, via two API calls:
///   1. GET /repos/{publisher}/{repo}/releases/latest  → tag_name
///   2. GET /repos/{publisher}/{repo}/git/ref/tags/{tag_name} → object.sha
///      (follows peeled tag objects one level)
pub fn resolve_latest_release(
    publisher: &str,
    repo: &str,
    client: &CachedSourceClient<impl SourceTransport>,
) -> Result<ResolvedVersion, FetchError>
```

The two-call approach is required because GitHub's releases API returns the tag name but not the underlying commit SHA; a peeled tag dereference is needed to produce a pinnable SHA.

### Schema changes in `data/meta/`

**`data/meta/work-item-steps.lino`** — replace the literal SHA in `workflow_template` with a placeholder and add a source field:

```
checkout_sha_source "https://api.github.com/repos/actions/checkout/releases/latest"
workflow_template "# Runs {path} on every push and pull request with the commands the\n# change was verified with.\nname: Run {path}\n\non:\n  push:\n  pull_request:\n\npermissions:\n  contents: read\n\njobs:\n  run:\n    runs-on: ubuntu-latest\n    timeout-minutes: 15\n    steps:\n      - uses: actions/checkout@{checkout_sha} # {checkout_tag}\n"
```

**`data/meta/stdout-program-contracts.lino`** — replace the literal SHAs and version in the kotlin `ci_setup` with placeholders and add source fields beside the language block:

```
  language kotlin
    ...
    setup_java_source "https://api.github.com/repos/actions/setup-java/releases/latest"
    setup_kotlin_source "https://api.github.com/repos/fwilhe2/setup-kotlin/releases/latest"
    kotlin_version_source "https://api.github.com/repos/JetBrains/kotlin/releases/latest"
    ci_setup "      - uses: actions/setup-java@{setup_java_sha} # {setup_java_tag}\n        with:\n          distribution: temurin\n          java-version: '{java_version}'\n      - uses: fwilhe2/setup-kotlin@{setup_kotlin_sha} # {setup_kotlin_tag}\n        with:\n          version: '{kotlin_version}'\n"
```

### New gate script: `scripts/check-generated-version-pins.rs`

Walks every file under `data/` and fails on any match of `@[0-9a-f]{40}` or standalone semantic-version patterns (`\d+\.\d+\.\d+`) in values that follow `ci_setup`, `workflow_template`, or equivalent keys. Passes when those fields contain only `{placeholder}` tokens. Runs in the `rust` CI stage via `scripts/run-ci-gates.rs`.

### JS/TS parity

Per R8.

---

## Tests

**Module `rust/tests/unit/issue_1168_version_resolution.rs`** — register in `rust/tests/unit/mod.rs` as `mod issue_1168_version_resolution;`

Fixture directory: `rust/tests/fixtures/issue-1168/` containing canned GitHub API JSON responses (releases/latest for checkout, setup-java, setup-kotlin, kotlin).

```rust
// Pattern copied from issue_991_how_to_synthesis.rs
fn live_fetch_requested() -> bool {
    matches!(
        std::env::var("FORMAL_AI_LIVE_FETCH")
            .unwrap_or_default().trim().to_ascii_lowercase().as_str(),
        "1" | "true" | "yes" | "on"
    )
}

fn offline_client() -> CachedSourceClient<CurlSourceTransport> {
    let fixture = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent().unwrap()
        .join("rust/tests/fixtures/issue-1168");
    CachedSourceClient::new(&fixture, CurlSourceTransport).with_online(false)
}

#[test]
fn resolves_checkout_sha_from_fixture() { /* asserts tag == "v7.0.1", sha is 40-hex */ }

#[test]
fn resolves_setup_java_from_fixture() { /* asserts tag, sha */ }

#[test]
fn resolves_kotlin_version_from_fixture() { /* asserts version string */ }

#[test]
fn offline_cache_miss_returns_err() { /* OfflineCacheMiss on unknown action */ }

#[test]
fn live_checkout_sha_matches_latest_release() {
    if !live_fetch_requested() { return; }
    // uses real CurlSourceTransport; asserts the resolved SHA is 40-hex
}
```

**Module `rust/tests/unit/issue_1168_gate.rs`** — register as `mod issue_1168_gate;`

```rust
#[test]
fn gate_rejects_literal_sha_in_workflow_template() { /* synthetic lino with bare SHA → fail */ }

#[test]
fn gate_accepts_placeholder_in_workflow_template() { /* {checkout_sha} token → pass */ }

#[test]
fn gate_rejects_bare_version_in_ci_setup() { /* "2.3.10" literal → fail */ }
```

Run command: `RUSTUP_TOOLCHAIN=1.98.1 cargo test --test unit issue_1168_`

---

## Definition of done

- [ ] Requirement shard `docs/requirements/issue-1168-generated-version-pins.md` written in the format of `docs/requirements/issue-1138-prerequisite-discovery.md` (ID column `R1168-*`, Requirement column, Status/evidence column).
- [ ] `RUSTUP_TOOLCHAIN=1.98.1 rust-script scripts/assemble-requirements.rs --write` regenerates `REQUIREMENTS.md` without error.
- [ ] Traceability rows for R1168-1 through R1168-8 added to `docs/requirements-traceability.md`.
- [ ] Case-study data under `docs/case-studies/issue-1168/` (at minimum: before/after snippets from both `.lino` files showing SHA removal, and a sample derivation log showing `SourceCapture` provenance).
- [ ] Changelog fragment in `changelog.d/` (format per `changelog.d/README.md`, `bump: minor`, category `### Changed`).
- [ ] `RUSTUP_TOOLCHAIN=1.98.1 rust-script scripts/run-ci-gates.rs --stage rust` exits 0, including `scripts/check-generated-version-pins.rs` passing on the updated `data/` tree.
- [ ] `RUSTUP_TOOLCHAIN=1.98.1 cargo test --test unit issue_1168_` all pass offline; live test passes under `FORMAL_AI_LIVE_FETCH=1`.
- [ ] No literal 40-hex SHA or bare toolchain version in `data/meta/work-item-steps.lino` or `data/meta/stdout-program-contracts.lino`.

---

## Depends on / blocks

- **Depends on #1163 (E128):** Version resolution hits `api.github.com` and the kotlinlang releases feed as external sources; it reuses `CachedSourceClient` / `SourceCapture` from `rust/src/source_fetch.rs`, the same boundary E128 formalizes for all fetched evidence. E133 cannot land before E128 establishes that a fetched JSON API response is a first-class formalized source.
- **Interacts with #1165 (E130):** Both edit `data/meta/stdout-program-contracts.lino` (this issue removes version literals, #1165 removes program strings); whichever lands second rebases.
- **Interacts with #1166 (E131):** E131 formalizes requests into obligations rather than phrase matching; a CI-workflow obligation MUST carry resolved version bindings as derived obligations, not ad-hoc string substitution.
- **Interacts with #1167 (E132):** This issue lands first with placeholder substitution; #1167 then moves the substitution to its single render site as part of its own delivery.
- **Distinct from #1169 (E134):** E134 updates Formal AI's own `Cargo.lock`, npm lockfiles, and `.github/workflows/` action pins — the project's internal dependencies. E133 updates version literals Formal AI writes into generated files for other people's repositories. The `scripts/check-generated-version-pins.rs` gate (this issue) scans `data/`; the `scripts/check-dependencies-latest.rs` gate (E134) scans manifests and lockfiles. Neither subsumes the other.

