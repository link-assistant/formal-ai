# PR 601 Gap Analysis

PR [#601](https://github.com/link-assistant/formal-ai/pull/601) delivered
important slices for issue #538, but not a closed self-learning loop. It proved
that Formal AI can drive bounded Agent CLI recipes, preserve real sessions,
generate recipe diagrams, and write one self-AST census into data. Issue #558 is
about the gap between those slices and an auto-learning system that can notice a
failure, repair itself, rebuild, reattach to the UI, and preserve the accepted
lesson.

## Evidence Reviewed

- Issue #558 and comments: `raw-data/issue-558*.json`
- Issue #538 and comments: `raw-data/issue-538*.json`
- PR #601 metadata, comments, reviews, and full diff:
  `raw-data/pr-601*.json`, `raw-data/pr-601.diff`
- Related GitHub search captures:
  `raw-data/github-code-search-*.txt`,
  `raw-data/recent-related-merged-prs.json`,
  `raw-data/related-issues.json`
- Existing case study for issue #538:
  `docs/case-studies/issue-538/README.md`

PR #601 was merged on 2026-07-02 with 44 commits. The final PR history matters:
an early reading of the branch as "core delivered, advanced parts deferred" is
no longer accurate.

## Gap Inventory

| Gap | Finding | Impact for issue #558 |
| --- | --- | --- |
| G1 | The root `REQUIREMENTS.md` drifted from the final PR #601 state. It still described Agent CLI, diagrams, and self-AST as tracked follow-ups even though the issue #538 case study records delivered slices. | Auto-learning work needs trustworthy status data. Stale requirement rows hide both delivered capability and remaining risk. |
| G2 | The Agent CLI path is recipe-driven. It can route known request families and reproduce committed artifacts, but it does not yet open an arbitrary failure trace and derive a new repair strategy. | This is not yet the general failure-to-repair loop that issue #558 asks for. |
| G3 | The self-AST work stores one module's abstract-syntax census in data. It proves self-inspection, but it is not a whole-repository source graph. | Issue #558 needs every owned source file represented as link-native data with provenance and checksums. |
| G4 | The self-AST census is not a full compiler round-trip. It records parsed structure, but it cannot regenerate the original Rust module, rebuild the binary, or reattach the accepted version to the UI. | Source-to-links without Links-to-source cannot support safe self-programming. |
| G5 | There is no learning-promotion protocol. PR #601 has sessions, tests, and CI, but no single artifact that records failure, hypothesis, patch, validation result, reviewer approval, and promoted lesson. | Without a durable learning record, a fixed failure does not become reusable system knowledge. |
| G6 | The live Agent CLI e2e covers bounded recipes, not hot restart or UI reattachment after an accepted change. | Issue #558 explicitly asks for recompile and reattachment to the UI. |
| G7 | Formal AI cannot yet answer "how do you work?" from a complete source/data/test graph. Documentation and file summaries exist, but not a grounded explanation graph that links requirements to implementation. | Self-explanation remains prose-heavy and incomplete. |

## What PR #601 Should Preserve

- The Agent CLI sessions and clean-copy reproduction workflow are valuable
  evidence that repository edits can be generated and verified through a tool
  loop.
- The live Agent CLI e2e log is the right style of integration proof: boot the
  server, drive a real client, and assert observable protocol-level behavior.
- The permission gates and refusal anti-pattern document are necessary controls
  for a future self-repair loop.
- The generated recipe diagrams prove that the same method can author
  non-lexical artifacts from a different request family.
- The self-AST census proves that source code can begin to enter the data layer,
  but issue #558 must expand it into a source-to-links and Links-to-source
  round trip.

## Root Cause

PR #601 solved a large issue by shipping several concrete, testable slices. That
was the right engineering move, but the final status was not propagated back to
the root requirements table, and the delivered slices were easy to overstate as
"auto-learning." The missing distinction is:

- **Delivered:** bounded Agent CLI recipes, session artifacts, recipe diagrams,
  one self-AST census, and tests proving those artifacts reproduce.
- **Not delivered:** arbitrary failure repair, repository-wide source-to-links,
  Links-to-source regeneration, rebuild/reattach, and a human-gated learning
  promotion protocol.

Issue #558 should therefore treat PR #601 as foundation, not completion.

## How PR #637 Starts To Close These Gaps

PR #637 lands the first implemented, human-gated slice on top of that foundation.
It does not close every gap, but it converts two of them from "not built" to
"first slice built and tested":

- **G4 (no full compiler round-trip):** `SourceRoundTrip::for_pinned_target()`
  now regenerates one real module from its meta-language links network and proves
  `source → links → source` byte-for-byte (`faithful = true`). This is the first
  genuine Links-to-source round-trip — still one module, not the whole repo, and
  not yet driving edits, but no longer merely a census.
- **G5 (no learning-promotion protocol):** `src/self_healing.rs` introduces the
  single durable artifact the gap called for — a `RepairCase` that records the
  failure, the source it maps onto, the benchmark-gated candidate lesson, and a
  terminal human-review outcome (`AwaitingReview`) — emitted as
  `data/meta/self-healing-case.lino`. Promotion itself stays a human decision.

The loop is reachable through the same Agent CLI surface PR #601 built, added as a
fifth recipe. The remaining gaps — G2 (arbitrary failure repair), G3
(whole-repository source graph), G6 (rebuild/UI reattach), and G7 (grounded
self-explanation) — stay open and are the subject of later slices, each
human-gated by design. The self-AST slice PR #601 shipped remains the base the
round-trip builds on; PR #637's contribution is that it is now a verified
round-trip, not a one-way census.
