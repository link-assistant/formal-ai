## Issue #1168 Generated Code Uses Versions Resolved at Generation Time

Issue [#1168](https://github.com/link-assistant/formal-ai/issues/1168) (E133)
makes every third-party version a generated artifact pins — action refs,
toolchain versions, the Java LTS — a *resolved* value with provenance, never a
memorized string. Resolution happens at generation time from the publisher's
own API, is recorded in the derivation, and degrades loudly (cache, then
shipped baseline, each said out loud) rather than silently going stale.

| ID | Requirement | Status / Evidence |
| --- | --- | --- |
| R1168-1 | The latest release tag and its commit SHA for every third-party GitHub Action in a generated CI workflow are fetched from `https://api.github.com/repos/{publisher}/{repo}/releases/latest` (then `commits/{tag}` for the SHA). | Implemented: `resolve_github` in `rust/src/version_resolution.rs`, covering `actions/checkout`, `actions/setup-java`, `fwilhe2/setup-kotlin`, `actions/setup-python`; pinned by mocked-transport tests (`resolves_every_pin_from_live_fixtures`). |
| R1168-2 | The Kotlin version is the latest `JetBrains/kotlin` release tag. | Implemented in the same module (`v2.4.20` as shipped baseline); the tag feeds `fwilhe2/setup-kotlin`'s `version:` argument through `fill_workflow_versions`. |
| R1168-3 | The Java version is Adoptium's own `most_recent_lts`, never a literal `'21'`. | Implemented: `resolve_java_lts` reads `https://api.adoptium.net/v3/info/available_releases` (25 as shipped baseline). |
| R1168-4 | Every resolved version is recorded in the derivation as a `SourceCapture` (source_url, fetched_at, sha256, cached, bytes) via the existing `CachedSourceClient`. | Implemented: `VersionSet::record` appends each capture's provenance plus one `version_resolution` line per pin; pinned by `records_the_resolution_in_the_derivation`. |
| R1168-5 | When the live API is unreachable, the most recent cached capture is used and the answer is annotated `(version resolved from cache; fetched_at=<timestamp>)`. | Implemented: `CachedSourceClient` replay supplies `Origin::Cache`, and `provenance_note` emits exactly that wording — extended to the shipped baseline, which annotates `(version resolved from the shipped baseline measured <date>)`; `ci_workflow::render` prepends the notes as workflow comments. Pinned by `offline_replays_the_cached_capture_and_says_so`. |
| R1168-6 | `workflow_template` and `ci_setup` values carry placeholders, never literal SHAs or bare versions. | Code half implemented: `fill_workflow_versions` fills `{checkout_ref}` … `{java_lts}` **and** still rewrites any stale `@<sha>` / `version: '…'` literal that slips through, scoped per action block. The two `data/meta` template files themselves (and their embedded mirrors) are rewired by the integrating commit; until then the R1168-8 gate is red exactly there. |
| R1168-7 | A CI gate fails on any `@<sha>` or bare version literal for a third-party action or toolchain in `data/`. | Implemented: `scripts/check-generated-version-literals.rs` (scans `data/` minus `data/cache/`; the toolchains baseline is exempt by shape, not by name), with embedded tests plus `rust/tests/unit/issue_1168_latest_versions.rs` pinning that findings are confined to the two templates awaiting the R6 rewrite. CI-stage wiring rides with the R6 rewrite in the integrating commit. |
| R1168-8 | Three-roots parity: `version_resolution.rs` translated to js/ts and parity cases added. | Open follow-up: the `formal-ai translate` pass and the `data/parity/cross-runtime-synthesis.json` cases need a build to generate and verify; the Rust contract is pinned here so the translation has a fixed target. |

The shipped baseline in `data/seed/toolchains.lino` (`generated_version` records)
was measured 2026-09-30 against the live APIs and tags: checkout `v7.0.1`
(`3d3c42e5…`), setup-java `v6.0.1` (`de7274f0…`), setup-kotlin `v2.0`
(`ee96925…`), setup-python `v7.0.0` (`5fda3b95…`), CPython `3.14.7`, Kotlin
`v2.4.20`, Java LTS `25`.
