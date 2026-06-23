# Issue 559 Upstream Dependency Audit

Audit date: 2026-06-23.

This audit answers the PR feedback request to check relevant upstream
dependencies before implementation planning continues. The conclusion is:

- No upstream blocker was found for the next behavior-preserving phases
  (`ProblemFrame` trace and recursive `WorkUnit` trace).
- No new upstream issue was created.
- A few existing upstream issues should be tracked as later phase gates.

Raw command output and GitHub API snapshots are stored under
`docs/case-studies/issue-559/raw-data/upstream/`.

## Scope

The audit covered the org-owned or org-adjacent dependencies that are most likely
to affect issue 559:

| Dependency | Current package data | Upstream repository | Issue 559 role | Blocker |
| --- | --- | --- | --- | --- |
| `meta-language` | crate `0.45.0`, latest `0.48.0` | `link-foundation/meta-language` | Lossless source spans, snapshots, structural query/replace, eventual code/data round trip. | No |
| `links-notation` | crate `0.13.0` | `link-foundation/links-notation` | `.lino` records for method registry, work-unit traces, and seed data. | No |
| `lino-arguments` | crate `0.3.0` | `link-foundation/lino-arguments` | Existing Links Notation argument parsing support. | No |
| `lino-objects-codec` | crate `0.2.1` | `link-foundation/lino-objects-codec` | Existing object encoding/decoding support for link data. | No |
| `lino-i18n` | npm `0.1.1` | `link-foundation/lino-i18n` | Existing i18n/link data package. | No |
| `link-calculator` | crate `0.19.0`, latest `0.20.1` | `link-assistant/calculator` | Existing atomic calculation method candidate. | No |
| `agent-commander` | npm `0.8.0` | `link-assistant/agent-commander` | Existing agent command integration package. | No |
| `doublets` | crate `0.4.0` | `linksplatform/doublets-rs` | Optional link-store foundation through upstream crates. | No for current phases |
| `platform-mem` | crate `0.3.0` | `linksplatform/mem-rs` | Optional memory store support through upstream crates. | No |
| `meta-theory` | documentation repository | `link-foundation/meta-theory` | Conceptual source for link-native terminology. | No |

## Existing Upstream Issues To Track

These issues are not blockers for the next implementation phases, but they may
matter later:

- `link-foundation/links-notation#197`, "Add streaming parser for large message
  handling": track if issue-559 trace export becomes too large for current
  parser behavior.
- `link-foundation/meta-language#168`, "Define shared-dialog source-description
  schema": track when `ProblemFrame` or source evidence records need to
  interoperate with shared-dialog source descriptions.
- `link-foundation/meta-language#165`, "Publish @link-foundation/meta-language
  to npm (registry returns 404)": track only if browser-side registry tooling
  requires npm package consumption before another repo-local integration exists.
- `linksplatform/doublets-rs#22` and related older build issues: track only if a
  later phase requires optional doublets features that reproduce those failures.

Other open upstream issues found during the audit are unrelated to the next
issue-559 phases or are documentation/package maintenance tasks that can proceed
independently.

## No-Blocker Rationale

Phase 1A and Phase 1B can be implemented inside this repository:

- `ProblemFrame`, `Need`, and `WorkUnit` can start as Rust data structures with
  trace serialization.
- Existing handlers can remain the source of truth.
- `.lino` registry work can start as local seed data and test fixtures.
- `meta-language` round-trip work is a later phase, not a prerequisite for
  observing frames and recursive work units.
- Links Notation streaming is not required until trace fixtures become large.

This means the first code-bearing phases can move forward without waiting for
upstream changes.

## Upstream Issue Policy

Create a new upstream issue only when implementation reaches a concrete missing
feature or confirmed bug that blocks local progress. The upstream issue should
include:

- the phase that is blocked;
- the exact local test or fixture that fails;
- the upstream package version;
- the smallest reproducer;
- the workaround, if any.

No such blocker exists at this planning stage.
