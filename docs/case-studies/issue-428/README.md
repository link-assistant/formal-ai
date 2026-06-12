# Case study - Issue #428: update to latest meta-language

> Source issue: <https://github.com/link-assistant/formal-ai/issues/428>
> Branch: `issue-428-7778bb0172a9` - PR: #429
> Raw data: [`raw-data/`](./raw-data)

## Summary

Issue #428 asks Formal AI to use the latest
[`link-foundation/meta-language`](https://github.com/link-foundation/meta-language)
across the codebase and to examine how the newer link-network engine should
support natural-language, source-code, and meaning-formalization workflows.

The safe immediate change is the dependency update from `meta-language` 0.39 to
0.40.0. The current Formal AI integration already routes program concrete syntax
through `meta_language::LinkNetwork`, so the existing bridge compiled without an
API rewrite and the source/CST regression suite still passes. This case study
archives the issue, PR, upstream release, release comparison, and repository
search data, then records the follow-up integration plan for the new 0.40
capabilities.

No upstream blocker was found. Upstream issues that previously tracked the
missing pieces relevant here, including source generation/unparsing,
translation-rule registries, query/transform algebra, incremental reparsing, and
additional grammars, are closed in `link-foundation/meta-language`. No new
upstream issue was filed from this pass.

## Archived data

- `raw-data/issue-428.json` - issue metadata and body captured with
  `gh issue view`.
- `raw-data/issue-428-comments.json` - paginated issue comments. No comments
  were present when archived.
- `raw-data/pr-429.json` - draft PR metadata captured before implementation.
- `raw-data/pr-429-conversation-comments.json` - PR conversation comments.
- `raw-data/pr-429-review-comments.json` - PR inline review comments.
- `raw-data/pr-429-reviews.json` - PR review records.
- `raw-data/meta-language-repo.json` - upstream repository metadata.
- `raw-data/meta-language-tags.json` - upstream tag data, including `v0.40.0`
  and `v0.39.0`.
- `raw-data/meta-language-releases.tsv` - upstream release list.
- `raw-data/meta-language-release-v0.40.0.json` and
  `raw-data/meta-language-release-v0.40.0-body.md` - latest release metadata.
- `raw-data/meta-language-compare-v0.39.0-v0.40.0.json` - GitHub compare data
  for the dependency bump.
- `raw-data/meta-language-issues.json` - upstream issue list used to check
  feature status.
- `raw-data/meta-language-README.md` - upstream README snapshot.
- `raw-data/cargo-search-meta-language.txt` and
  `raw-data/cargo-info-meta-language-0.40.0.txt` - Cargo registry evidence for
  the latest crate version.
- `raw-data/cargo-info-meta-language-0.39.0.txt` - previous-version Cargo
  metadata used for the Rust-version compatibility comparison.
- `raw-data/link-assistant-meta-language-code-search.json` - `gh search code`
  results for the existing in-organization integration points.
- `raw-data/online-research.md` - external source survey and interpretation.

## Timeline

| Time (UTC) | Event |
| --- | --- |
| 2026-06-12T21:51:57Z | Upstream `link-foundation/meta-language` release `v0.40.0` was published. |
| 2026-06-12T22:23:09Z | Issue #428 was opened requesting a codebase-wide update to the latest upstream meta-language. |
| 2026-06-12T22:41:02Z | Draft PR #429 was opened from `issue-428-7778bb0172a9`. |
| 2026-06-12 | Formal AI issue, PR, upstream release, upstream issue, and code-search data were archived under this case-study directory. |
| 2026-06-12 | `Cargo.toml` and `Cargo.lock` were updated to `meta-language = "0.40.0"`. |

## Current implementation

The current codebase already has one concrete `meta-language` integration:

- `src/coding/cst.rs` imports `LinkNetwork`, `NetworkProjection`,
  `ParseConfiguration`, and `LinkType` from `meta_language`.
- `LinkNetwork::parse` is the CST parser used by Formal AI's coding layer.
- `verify_full_match`, `projected_links(NetworkProjection::ConcreteSyntax)`,
  and `reconstruct_text` are used to validate parsed source and prove the text
  round trip.
- `data/seed/program-cst-grammars.lino` records that the configured program
  languages use the `meta_language` CST engine.
- `tests/source/source_tests/coding/cst/tests.rs` verifies the CST metadata,
  successful parsing, failed parsing for invalid input, and round-trip coverage
  for every currently catalogued Formal AI program language.

This means the dependency update immediately affects the source-code
formalization path, but it does not automatically expand Formal AI's public
language catalog. Upstream 0.40 adds more parser coverage than the Formal AI
catalog currently exposes. New public languages still need catalog aliases,
idioms, examples, execution profiles, and user-facing tests before they should be
declared in `program-cst-grammars.lino`.

## Requirements and solution plan

| ID | Requirement from issue #428 | Existing components checked | Status in this PR | Follow-up plan |
| --- | --- | --- | --- | --- |
| R428-01 | Use the latest `link-foundation/meta-language` all around the codebase. | `Cargo.toml`, `Cargo.lock`, `src/coding/cst.rs`, CST source tests. | Done for the current crate dependency: Formal AI now pins `meta-language` 0.40.0. Existing Rust call sites needed no API changes. | Use future PRs for broader architectural adoption of 0.40 APIs instead of folding unrelated rewrites into the version bump. |
| R428-02 | Preserve issue data, PR data, upstream data, and online research under `docs/case-studies/issue-428`. | Prior case-study directories and GitHub/Cargo source data. | Done. Raw data and research notes are archived here. | Append new upstream or CI evidence here if PR review asks for deeper investigation. |
| R428-03 | Convert natural language and source code into meaning formalization through trusted APIs. | Existing solver formalization traces, seed meanings, translation caches, and `meta_language::LinkNetwork` source parsing. | Partially existing. Source code already goes through `meta-language` CST validation; natural-language meaning formalization remains Formal AI's local seed/cache pipeline. | Evaluate upstream 0.40 concept interning, natural-language fixtures, and translation rules as a backing representation for Formal AI meaning records. |
| R428-04 | Manipulate links networks for reasoning, editing, code, and text. | Formal AI seed `.lino` files, CST bridge, solver traces, and upstream 0.40 README/API notes. | Partially existing. Formal AI manipulates seed links and validates source CSTs; it does not yet expose upstream 0.40 query/edit APIs in solver workflows. | Prototype `LinkQuery`, replacement, `apply_edit`, and `NetworkSnapshot` usage behind focused tests before replacing local ad hoc transformations. |
| R428-05 | Translate back to target natural/formal languages. | Existing answer rendering, translation cache, and upstream 0.40 `reconstruct_text_as`/source rendering capabilities. | Partially existing. Formal AI can render known responses and source snippets; it does not yet delegate meaning-to-language rendering to upstream translation rules. | Add a small reversible meaning-to-text experiment using upstream syntax mappings, then compare it against existing Formal AI answer rendering. |
| R428-06 | Report upstream issues when features are missing. | Upstream issue list and closed feature issues in `link-foundation/meta-language`. | Not needed in this pass. The relevant upstream feature gaps were already closed before this PR. | File an upstream issue only after a local reproducer demonstrates a missing or broken 0.40 API. |
| R428-07 | Check existing components and libraries before implementing from scratch. | `meta-language`, Tree-sitter parsers, `links-notation`, and the optional Doublets backend were reviewed. | Done. Existing components are documented in `raw-data/online-research.md`. | Prefer upstream 0.40 APIs for link-network editing/source generation once Formal AI has focused acceptance tests for those workflows. |
| R428-08 | Keep the change reviewable and testable. | Existing CST and source test suites. | Done. This PR keeps the code change to the dependency update and documents the larger plan. | Open child PRs for catalog expansion, source generation, natural-language mapping, and persistent link-store work. |

## Latest upstream findings

Cargo and GitHub both report `meta-language` 0.40.0 as the latest available
version for this pass. The crate still requires Rust 1.77 according to Cargo
metadata, which was already true for the previous 0.39 dependency used by this
repository.

The upstream 0.40 README and closed issues show several capabilities that matter
for Formal AI's longer-term direction:

- Mutable link networks with point links, relation links, source spans, and
  metadata.
- Concrete-syntax parsing and reconstruction through Tree-sitter-backed
  language support.
- Source rendering from programmatically constructed syntax networks via
  `render_source` and lower-level insertion helpers.
- Network projections, snapshots, storage abstractions, and an optional
  Doublets-backed store.
- Query, replacement, substitution-rule, and incremental edit APIs.
- Concept-to-language syntax mappings and translation-rule infrastructure.
- Natural-language grammar fixtures and mixed-region parsing support.

These are relevant follow-up opportunities, but the existing Formal AI API
surface does not need an immediate breaking rewrite to consume 0.40.0.

## Implementation

- Updated `Cargo.toml` from `meta-language = "0.39"` to
  `meta-language = "0.40.0"`.
- Refreshed `Cargo.lock` with `cargo update -p meta-language --precise 0.40.0`.
- Archived the issue, PR, upstream release, upstream issue list, Cargo metadata,
  and code-search data under this case-study directory.
- Added this case study and online-research note to document the current
  integration state and the larger adoption plan.

## Validation

The existing source-code/CST tests are the regression guard for the current
integration because they exercise the `meta_language::LinkNetwork` path directly.

- `cargo test cst -- --nocapture` - passed. This covers the focused CST source
  tests, the issue #395 integration path that validates code through CST, and
  the unit-level CST filter.
- `cargo test --test source -- --nocapture` - passed with 389 tests.

Additional final checks were run after the documentation update and are recorded
in the PR description.
