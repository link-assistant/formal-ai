---
bump: minor
---

### Added
- Statement-level summarization of many sources (issue #844): `formal_ai::summarization` gains `dedup` (one `MergedStatement` per fact, a retractable `MergeLink` per absorbed sentence, `DedupReport::split` to undo a conflating merge), `importance` (the kind prior blended with observed frequency, source authority, and stance, with zero ranking evidence from unoriginal mirrors), `gathering` (a recursive unmet-difference loop bounded by depth and terminating at a fixpoint, with a content-addressed `SourceCache` that replays byte-identically), `recheck` (a verdict per fact, so an unsupported statement is withheld from the summary but kept), and `context` (`merge_into_context` builds a `world_model::Context`, not a list: a probability per statement and mutual `Contradicts` edges for every disagreement).
- An identifier rung below the topic rung: `SummarizationMode::Identifier` renders a label through `summarization::identifier::to_identifier`, which honours a `NamingConvention`, an `IdentifierBudget`, and the seed's reserved words.
- `SummarizationConfig::keeping_boilerplate()` keeps `install`/`example` sentences in the output, for a merged context where the install command is the answer rather than boilerplate.
- `data/seed/multilingual-responses-summarization.lino` holds the merge's reader-facing wording — the evidence summary, the denial clause and the disputed wrapper — in English, Russian, Hindi and Chinese, read back through `summarization::vocabulary::rendered_response`, so no summary sentence is hardcoded in Rust (R379).
- `cargo run --example issue_844_statement_merge` walks the issue's Stack Overflow case end to end — recursive gathering over a citation cycle, warm-cache replay, evidence-ranked merge, reported disagreement, recheck, and the ladder down to a single identifier. Documented in `docs/case-studies/issue-844/`.

### Fixed
- `world_model::Context::recalculate` no longer reports a claim and its denial as both probable. A contradiction whose two sides each carry saturating support makes the JTMS update the exact swap `x ← 1 - x`, which oscillates until the pass bound and returned whichever half the last pass landed on. Repeated states are now detected and the cycle is collapsed to its mean, verified as a fixpoint: two original sources that flatly disagree settle at `0.5`.
- `summarization::formalize` no longer splits a sentence inside a token, so `crates.io`, `docs.rs`, and `1.96` stay one term instead of becoming a spurious extra statement.
- `summarization::SourceCache` keeps provenance per URL alongside content-addressed bodies, so an unoriginal mirror of a first-party page no longer inherits the first party's source tier.
