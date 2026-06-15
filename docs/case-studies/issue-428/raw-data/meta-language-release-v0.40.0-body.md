[![Crates.io](https://img.shields.io/crates/v/meta-language?label=crates.io)](https://crates.io/crates/meta-language/0.40.0) [![Docs.rs](https://docs.rs/meta-language/badge.svg)](https://docs.rs/meta-language/0.40.0)

### Added
- Changeset-style fragment format with frontmatter for specifying version bump type
- New `get-bump-type.mjs` script to automatically determine version bump from fragments
- Automatic version bumping on merge to main based on changelog fragments
- Detailed documentation for the changelog fragment system in `changelog.d/README.md`

### Changed
- Updated `collect-changelog.mjs` to strip frontmatter when collecting fragments
- Updated `version-and-commit.mjs` to handle frontmatter in fragments
- Enhanced release workflow to automatically determine bump type from changesets

### Changed
- Add `detect-changes` job with cross-platform `detect-code-changes.mjs` script
- Make lint job independent of changelog check (runs based on file changes only)
- Allow docs-only PRs without changelog fragment requirement
- Handle changelog check 'skipped' state in dependent jobs
- Exclude `changelog.d/`, `docs/`, `experiments/`, `examples/` folders and markdown files from code changes detection

### Fixed
- Fixed README.md to correctly reference Node.js scripts (`.mjs`) instead of Python scripts (`.py`)
- Updated project structure in README.md to match actual script files in `scripts/` directory
- Fixed example code in README.md that had invalid Rust with two `main` functions

### Added

- Added crates.io publishing support to CI/CD workflow
- Added `release_mode` input with "instant" and "changelog-pr" options for manual releases
- Added `--tag-prefix` and `--crates-io-url` options to create-github-release.mjs script
- Added comprehensive case study documentation for Issue #11 in docs/case-studies/issue-11/

### Changed

- Changed changelog fragment check from warning to error (exit 1) to enforce changelog requirements
- Updated job conditions with `always() && !cancelled()` to fix workflow_dispatch job skipping issue
- Renamed manual-release job to "Instant Release" for clarity

### Fixed

- Fixed deprecated `::set-output` GitHub Actions command in version-and-commit.mjs
- Fixed workflow_dispatch triggering issues where lint/build/release jobs were incorrectly skipped

### Fixed

- Fixed changelog fragment check to validate that a fragment is **added in the PR diff** rather than just checking if any fragments exist in the directory. This prevents the check from incorrectly passing when there are leftover fragments from previous PRs that haven't been released yet.

### Changed

- Converted shell scripts in `release.yml` to cross-platform `.mjs` scripts for improved portability and performance:
  - `check-changelog-fragment.mjs` - validates changelog fragment is added in PR diff
  - `git-config.mjs` - configures git user for CI/CD
  - `check-release-needed.mjs` - checks if release is needed
  - `publish-crate.mjs` - publishes package to crates.io
  - `create-changelog-fragment.mjs` - creates changelog fragments for manual releases
  - `get-version.mjs` - gets current version from Cargo.toml

### Added

- Added `check-version-modification.mjs` script to detect manual version changes in Cargo.toml
- Added `version-check` job to CI/CD workflow that runs on pull requests
- Added skip logic for automated release branches (changelog-manual-release-*, changeset-release/*, release/*, automated-release/*)

### Changed

- Version modifications in Cargo.toml are now blocked in pull requests to enforce automated release pipeline

### Added

- Added support for `CARGO_REGISTRY_TOKEN` as alternative to `CARGO_TOKEN` for crates.io publishing
- Added case study documentation for Issue #17 (yargs reserved word and dual token support)

### Changed

- Updated workflow to use fallback logic: `${{ secrets.CARGO_REGISTRY_TOKEN || secrets.CARGO_TOKEN }}`
- Improved publish-crate.mjs to check both `CARGO_REGISTRY_TOKEN` and `CARGO_TOKEN` environment variables
- Added warning message when neither token is set

### Added
- New `scripts/rust-paths.mjs` utility for automatic Rust package root detection
- Support for both single-language and multi-language repository structures in all CI/CD scripts
- Configuration options via `--rust-root` CLI argument and `RUST_ROOT` environment variable
- Comprehensive case study documentation in `docs/case-studies/issue-19/`

### Changed
- Updated all release scripts to use the new path detection utility:
  - `scripts/bump-version.mjs`
  - `scripts/check-release-needed.mjs`
  - `scripts/collect-changelog.mjs`
  - `scripts/get-bump-type.mjs`
  - `scripts/get-version.mjs`
  - `scripts/publish-crate.mjs`
  - `scripts/version-and-commit.mjs`

### Changed

- **check-release-needed.mjs**: Now checks crates.io API directly instead of git tags to determine if a version is already released. This prevents false positives where git tags exist but the package was never actually published to crates.io.

### Added

- **CI/CD Troubleshooting Guide**: New documentation at `docs/ci-cd/troubleshooting.md` covering common issues like skipped jobs, false positive version checks, publishing failures, and secret configuration.

- **Enhanced Error Handling in publish-crate.mjs**: Added specific detection and helpful error messages for authentication failures, including guidance on secret configuration and workflow setup.

- **Case Study Documentation**: Added comprehensive case study at `docs/case-studies/issue-21/` analyzing CI/CD failures from browser-commander repository (issues #27, #29, #31, #33) with timeline, root causes, and lessons learned.

### Fixed

- **Prevent False Positive Version Checks**: The release workflow now correctly identifies unpublished versions by checking crates.io instead of relying on git tags, which can exist without the package being published.

### Changed

- Translated all CI/CD scripts from JavaScript (.mjs) to Rust (.rs) using rust-script
- Scripts now use native Rust with rust-script for execution in shell
- Removed Node.js dependency from CI/CD pipeline
- Updated GitHub Actions workflow to use rust-script instead of node
- Updated README and CONTRIBUTING documentation with new script references

### Added

- Cache `restore-keys` for partial cache hits across all workflow jobs
- Explicit `token` parameter in checkout for release jobs
- Code coverage job with `cargo-llvm-cov` and Codecov integration
- Codecov badge in README.md
- Pre-release version support (e.g., `0.1.0-beta.1`) in version parsing
- `--release-label` parameter for multi-language release disambiguation
- `ensure_version_exceeds_published()` logic to prevent publishing duplicate versions
- `get_max_published_version()` to query highest non-yanked version from crates.io
- `max_published_version` output from check-release-needed for downstream use
- Version fallback logic in auto-release Create GitHub Release step

### Changed

- Updated `actions/checkout` from v4 to v6
- Updated `actions/cache` from v4 to v5
- Updated `peter-evans/create-pull-request` from v7 to v8
- Made `publish-crate.rs` fail (exit 1) when version already exists on crates.io
- Improved `create-github-release.rs` to check combined stdout+stderr and detect "Validation Failed"

### Fixed

- Fix publish steps overriding workflow-level CARGO_TOKEN fallback, breaking CARGO_REGISTRY_TOKEN-only configurations (#32)
- Fix non-fast-forward push failures in multi-workflow repos by adding fetch/rebase and push retry logic (#31)
- Add mono-repo path support to check-changelog-fragment.rs, check-version-modification.rs, and create-changelog-fragment.rs
- Add `!cancelled()` guard to test job condition to respect workflow cancellation

### Fixed

- Fixed `version-and-commit.rs` to check crates.io instead of git tags for determining if a version is already released
- This prevents the release pipeline from getting stuck when git tags exist without corresponding crates.io publication

### Added

- Added `--tag-prefix` support to `version-and-commit.rs` for multi-language repository compatibility
- Added crates.io and docs.rs badges to README.md
- Added automatic crates.io and docs.rs badge injection in GitHub release notes
- Added documentation deployment job to CI/CD pipeline (deploys to GitHub Pages after release)
- Added case study documentation for issue #25

### Fixed
- Fixed unsupported look-ahead regex in `create-github-release.rs` that caused a panic when parsing CHANGELOG.md. Replaced with a two-step approach using only features supported by Rust's `regex` crate.

### Changed
- Restructured example application as a simple CLI sum calculator using `lino-arguments`
- Renamed default package to `example-sum-package-name` with Unlicense license
- Reorganized test structure: `tests/unit/sum.rs`, `tests/integration/sum.rs`, `tests/unit/ci-cd/`
- Converted experiment scripts into proper unit tests in `tests/unit/ci-cd/changelog_parsing.rs`
- Added CI/CD skip logic for template default package name `example-sum-package-name`
- Updated README.md badges and documentation

### Fixed

- Change detection script now uses per-commit diff instead of full PR diff, so commits touching only non-code files correctly skip CI jobs even when earlier commits in the same PR changed code files

### Fixed
- Make release scripts resolve the publishable crate manifest when the repository root uses a Cargo workspace manifest.

### Fixed
- Decoupled GitHub Pages documentation deployment from package release publication and fixed release-script warning failures under `RUSTFLAGS=-Dwarnings`.

### Changed
- Added explicit GitHub Actions job timeouts and documented Rust test timeout guidance.

### Fixed
- Added a non-blocking warning threshold to the Rust file-size check so near-limit files are surfaced before concurrent PR merges can exceed the hard limit.

### Fixed
- Made `create-github-release.rs` build GitHub release titles as `[Language] X.Y.Z` instead of reusing the tag prefix.

### Added
- Added optional Docker Hub image publishing tied to Rust crate releases, including crates.io visibility waiting, version/latest image tags, and Docker Hub badges in GitHub release notes.

### Changed
- Release completeness checks now self-heal when crates.io exists but configured Docker Hub or GitHub release artifacts are missing.

### Fixed
- Switched documentation deployment to the official GitHub Pages artifact workflow so repositories using GitHub Actions as their Pages source do not get false-positive branch-push deploys.

### Added
- Documented the one-time `Settings → Pages → Source = GitHub Actions` prerequisite for the `deploy-docs` job in `README.md` and as a comment above the `deploy-docs` job in `release.yml`, so downstream template users hit a documented setup step instead of a `Get Pages site failed` error on the first deploy.

### Added
- Tracking case study at `docs/case-studies/issue-52/` registering the `browser-commander` + Playwright preview-regeneration pattern from [`konard/vk-bot-desktop#52`](https://github.com/konard/vk-bot-desktop/pull/52), with an activation checklist for when an example-app surface lands in this template. Documentation only — no workflow, script, or runtime code changes. Primary upstream tracking issue: [`link-foundation/js-ai-driven-development-pipeline-template#62`](https://github.com/link-foundation/js-ai-driven-development-pipeline-template/issues/62).

### Fixed
- Release automation now keeps the workspace package entry in `Cargo.lock` synchronized when `scripts/version-and-commit.rs` bumps `Cargo.toml`, preventing stale lock-file version diffs in later pull requests.

### Fixed
- `scripts/publish-crate.rs` now treats crates.io HTTP 429 throttle responses ("You have published too many versions of this crate in the last 24 hours") as a deferred `publish_result=rate_limited` outcome (it writes the output, prints an explanatory banner and exits successfully) instead of a hard CI failure reported as a generic `failed` ("Failed to publish for unknown reason"). Authentication, already-published and unknown failures still exit non-zero. Failed-publish classification is consolidated through a single `classify_failure` function and `FailureKind` enum (with an `is_deferred` predicate), covered by unit tests runnable via `rust-script --test scripts/publish-crate.rs`.
- The release workflow (`.github/workflows/release.yml`) now gates crate-availability waiting, Docker Hub publishing and GitHub release creation on either an already-published crate or `publish_result=success`, so a deferred (rate-limited) crate upload no longer produces partial downstream release artifacts and the same version is retried automatically on the next push to `main`.

### Fixed
- Fixed reversed `cancel-in-progress` concurrency condition in `release.yml` that cancelled in-flight releases on `main` and never superseded older PR runs. The condition now uses `!=` so `main` releases run to completion while newer PR pushes cancel stale runs.

### Added
- Added a `scripts/check-crate-size.rs` guard that builds the `.crate` archive and fails the release before publishing when it exceeds the crates.io 10 MiB upload limit. The check runs in the build job and before publishing in both the auto-release and manual-release jobs.

### Changed
- Added a narrow `include` allowlist to `Cargo.toml` so docs, case studies, generated CI artifacts, changelog fragments, scripts, and experiments no longer inflate the published release archive.

### Fixed
- Prevented GitHub release creation from treating generic API validation failures as existing releases, and capped oversized release notes with a link to the full tagged changelog.

### Added
- Added the initial `meta-language` links-network core with self-description roots,
  verification, trivia attachment configuration, and a minimal CLI.
- Added the default lossless `parse` entry point, projection views, and a
  tested parity roadmap for competitor features, grammar embedding, and language
  coverage targets.
- Added exact reconstruction, mixed-region detection, query matching,
  substitution rules, concept reconstruction, object identity helpers,
  many-valued truth values, and executable parity fixtures for every tracked
  competitor target.
- Added executable language fixtures and tests for every requested Markdown,
  HTML, top-ten programming-language, and top-ten natural-language target.

### Fixed
- Added a CI guard that rejects Rust test modules and test attributes under `src/`, keeping tests in the `tests/` tree.

## Minor Changes

- Add immutable and mutable network snapshots with provenance and forward
  version commits for roadmap snapshot/versioning coverage.

### Added
- Added `txt` as a first-class markup/container target with a UTF-8 lossless
  fixture and content-sniffing fallback regions.

### Fixed
- Reconciled the natural-language target ordering with the Ethnologue 2025
  total-speaker order cited by the parity roadmap.

### Added
- Added a grammar-backed parser front end for Python, C, Java, C++, C#, JavaScript, and R using official tree-sitter grammar crates.

### Added
- Parse supported mixed-mode embedded regions into the host links network with grammar-backed syntax links.

### Added
- Added natural-language segmentation, identification, normalization, and bidi annotation links over lossless text parses.

### Added
- Added grammar-backed Visual Basic parsing with byte-exact reconstruction and recovery flag coverage.

bump: minor

- Add a grammar-backed `sql-ansi` SQL-family dialect fixture using
  `tree-sitter-sequel`.

### Added
- Added grammar-backed Delphi/Object Pascal parsing through `tree-sitter-pascal`.

### Added
- Enriched link queries with S-expression structural matching, captures, by-type construction, host predicate hooks, and link-cli-style variable substitution bindings.

### Added
- Added persistent snapshot structural sharing with interned metadata text storage.

### Added
- Added a query-transform surface for selecting captured links and replacing
  their source text while preserving unchanged bytes.

### Added
- Materialized self-description roots as controlled links with complete root-definition closure and round-trippable `describe` output.

### Added
- Added common concept ontology seeding from meta-expression's semantic lexicon, with shared concept links, syntax mappings, and structural programming-language concepts.

### Added
- Added semantic cross-language reconstruction for the Hawaii statehood fixture,
  including English/Russian naturalization and configurable formalization levels.

### Added
- Expanded `PARITY_FIXTURES` with multiple provenance-tracked fixtures for tree-sitter, LibCST, Recast, jscodeshift, Rowan, cstree, and Roslyn, including executable recovery and query/transform expectations.

### Added

- Expanded `PARITY_FIXTURES` with upstream-provenanced internal ecosystem corpora for links-notation, link-cli, lino-objects-codec, relative-meta-logic, formal-ai, and meta-expression.

### Added

- Added structural LiNo parsing for links-notation doublets, triplets, named links, indented IDs, and self-references while preserving byte-exact reconstruction.

### Added
- Added fixed-point probabilistic truth values for relative-meta-logic-style
  semantic confidence evaluation.

### Added
- Added grammar-backed Go parsing via the official `tree-sitter-go` grammar
  (accepts `Go`, `go`, and `golang`) so `LinkNetwork::parse` emits real
  `LinkType::Syntax` concrete-syntax links with byte-exact reconstruction.

### Added
- `AccessMode { Mutable, ReadOnly }` setting on `ParseConfiguration`
  (`with_access_mode` / `access_mode`), defaulting to `Mutable` so existing
  callers are unaffected.
- `LinkNetwork::freeze` / `as_read_only` yielding a `ReadOnlyNetwork` view that
  exposes only `&self` operations (query, project, reconstruct, verify,
  serialize); mutators are unreachable at compile time because the view never
  hands out `&mut LinkNetwork`.
- `LinkNetwork::parse_engine`, returning an `EngineNetwork` handle that honours
  the configured access mode: read-only parsing returns the frozen form and
  `EngineNetwork::as_mutable` rejects mutation with a `ReadOnlyViolation`
  diagnostic.
- Snapshot interop: `NetworkSnapshot::as_read_only` / `from_read_only` reuse the
  snapshot's `Arc<LinkNetwork>`, so the frozen form composes with snapshot
  versioning instead of duplicating it.

### Added
- Added grammar-backed Ruby parsing through `tree-sitter-ruby`, so
  `LinkNetwork::parse(source, "Ruby", ...)` (and the `rb` alias) now emits real
  `LinkType::Syntax` links instead of falling back to lossless plain text.

bump: minor

- Add a grammar-backed `TypeScript` front end using `tree-sitter-typescript`,
  wiring the `typescript`/`ts` labels to `LANGUAGE_TYPESCRIPT` and the `tsx`
  label to `LANGUAGE_TSX` so `LinkNetwork::parse` emits real `LinkType::Syntax`
  links for TypeScript and TSX sources.

### Added
- Added grammar-backed parsing for seven data-exchange / interchange formats
  through `src/tree_sitter_adapter.rs`: JSON (`tree-sitter-json`), YAML
  (`tree-sitter-yaml`, accepts `yaml`/`yml`), TOML (`tree-sitter-toml-ng`), XML
  and DTD (`tree-sitter-xml`), INI (`tree-sitter-ini`), Protocol Buffers
  (`tree-sitter-proto`, accepts `protobuf`/`proto`/`Protocol Buffers`), and
  GraphQL (`tree-sitter-graphql`, accepts `graphql`/`gql`). Each parses into
  real `LinkType::Syntax` concrete-syntax links and reconstructs byte-for-byte.
- Added a `DATA_FORMAT_TARGETS` registry in `src/parity.rs` (with the new
  `LanguageFamily::DataFormat`) gated by parity tests, mirroring
  `MARKUP_LANGUAGE_TARGETS`.
- Added per-format UTF-8 `LANGUAGE_FIXTURES` round-trip entries plus
  recovery-case and mixed-region tests (a `json` fence embedded in Markdown
  parses into the host links network).
- Added lossless CSV and JSON5 parsers for the two formats whose published
  tree-sitter crates still pin the incompatible `tree-sitter ~0.20` runtime.
  CSV is validated with the Rust `csv` crate and JSON5 is validated with
  `json5_nodes`; both emit structured syntax links and reconstruct
  byte-for-byte.

### Documentation
- Documented the nine wired data-format parsers (parser, version, license, root
  node) in `docs/parity-roadmap.md`, including the tree-sitter compatibility
  rationale for the in-repo CSV and JSON5 parsers.

### Added
- `LinkNetwork::to_lino` serializes an entire links network to canonical
  links-notation text, keyed by each link's numeric id (doublets-style id
  discipline), covering references, names, types, terms, definitions,
  languages, source spans, parse flags, and term registration.
- `LinkNetwork::from_lino` reconstructs the exact network from that text,
  forming a lossless round-trip (`from_lino(to_lino(n))` is isomorphic to `n`).
- `LinoSerializationError` reports parse and schema failures from `from_lino`.
- A round-trip property test over every language fixture plus synthetic
  networks, and a test that `to_lino` output is accepted by the
  `links-notation` 0.13 crate parser.
- `ParityCapability::LinoSerialization` and an output-side serialization
  parity fixture for the `links-notation` target.

### Changed
- Added the `links-notation` 0.13 crate as a dependency so serialized output
  aligns with the wider links-notation ecosystem.

### Added
- `ParserRegistry` maps language keys to `Arc<dyn LanguageParser>` with the
  built-in parser as a fallback. User registrations shadow the built-in
  dispatch for the same (case-insensitive) key; unregistered keys still route
  through the built-in set.
- `LinkNetwork::parse_with_registry` dispatches a parse through a registry.
- `examples/custom_parser_registry.rs` documents registering a custom parser.

### Added
- Added grammar-backed parsing for five popular programming languages
  immediately below the TIOBE top ten through `src/tree_sitter_adapter.rs`: PHP
  (`tree-sitter-php`, via its `LANGUAGE_PHP` symbol), Swift
  (`tree-sitter-swift`), Kotlin (`tree-sitter-kotlin-ng`, accepts `kotlin`/`kt`),
  Scala (`tree-sitter-scala`), and Lua (`tree-sitter-lua`). Each parses into real
  `LinkType::Syntax` concrete-syntax links and reconstructs byte-for-byte.
- Added a `SECOND_TIER_PROGRAMMING_LANGUAGE_TARGETS` registry in `src/parity.rs`
  gated by parity tests, mirroring `DATA_FORMAT_TARGETS`.
- Added per-language UTF-8 `LANGUAGE_FIXTURES` round-trip entries plus
  case-insensitive alias coverage and a per-language recovery fixture whose
  malformed source still reconstructs while exposing error/missing diagnostics.

### Documentation
- Documented the wired grammars (crate, version, license, root node) in
  `docs/parity-roadmap.md` and kept the temporary Perl follow-up tracked
  explicitly until `ts-parser-perl` was adopted.

### Added
- Add a Rust `ToLinks`/`FromLinks` codec with queryable type-shape links and shared/circular object graph round-trips through LiNo serialization.

### Added

- Added the `LinkStore` storage trait, read-only-aware storage wrappers, and an
  optional file-mapped `doublets` backend with LiNo/binary round-trip coverage.

### Added
- Added exact-match concept interning, language-bound expression links, external-id alias links, and LiNo concept-set import for the shared concept ontology.

### Added
- Added starter natural-language grammaticality parsing with UD-style morphosyntax links, pass/fail fixtures for the ten natural-language targets, and recoverable error links for ungrammatical fixtures.

### Added
- Added configurable `TranslationRuleSet` values, a runtime
  `TranslationRuleRegistry`, LiNo-backed rule-set loading, template
  placeholders, and missing-rule diagnostics for from-meta reconstruction.

### Added
- Added queryable language profiles with JavaScript transform enforcement and rule-set-derived profile domains.

### Added
- Added incremental source edit reparsing, stable outside-edit link IDs, and structural snapshot diff reporting.

### Added
- Added composable `LinkRule` query algebra with relational rules, boolean composition, named sub-rules, ellipsis gap matching, typed metavariables, plain-text token patterns, traversal strategies, and valid/invalid rule snapshot suites.
- Added quasiquote replacement templates with placeholder validation and parenthesization-conservative captured-text replacement.

### Added
- Add the API operation/style parity registry, fluent network pipeline, and link-cli-style substitution text runner.

### Added
- Wire the canonical `ts-parser-perl` tree-sitter grammar as a second-tier
  programming-language target without upgrading the project-wide tree-sitter
  runtime.

### Added
- Added source generation helpers for constructed syntax networks: `insert_source_token`, `insert_syntax_node`, `render_source`, `render_source_from`, and `render_source_from_document`.

### Added
- Added wave-two competitor and ecosystem parity fixtures with executable transform, reconstruction, grammar, and storage gates.
- Added a recorded `cargo llvm-cov` line-coverage floor to CI so coverage cannot silently regress.