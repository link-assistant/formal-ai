Parent: #651

## Motivation and evidence

Across roughly fifteen closed issues (#481–#508 range and earlier, e.g. #67, #292, #326), the maintainer's requirement "**all languages**" was silently narrowed to the four seed languages (en/ru/hi/zh). The vision (`VISION.md`, issue #526 contract) states that translation goes through the language-neutral meta language, so once meanings are grounded in Wikidata P/Q anchors, **adding a language should reduce to data**: the destination-language labels/lexemes on the same anchors. That property has never been demonstrated for a fifth language.

Open issue #660 (E41) builds the bulk importer for the existing four languages. This issue owns the **any-language protocol**: the repeatable, data-only procedure that takes the system from N to N+1 languages, and the contract that keeps every added language honest.

## Requirements

1. **Language-addition protocol as data + docs**: a documented, scripted procedure (building on #660's importer) that adds a new language by importing its labels/lexemes for the existing grounded meaning set, its detection rules, its multilingual-response entries, and its operation-vocabulary surfaces — with **zero Rust changes** (any needed code change is a defect to fix, per the "adding a language is a seed-data edit" precedent of E33/#326).
2. **Round-trip contract per language** (issue #526): language→meta→same-language survival tests and directed-pair tests through the shared meaning are generated automatically for every registered language, not hand-written per pair.
3. **Prove it with a fifth language**: add one new language end to end (candidate: Spanish or Arabic — decide in the case study from Wikidata lexeme coverage data) and pass the same specification suites the four seed languages pass (greetings, identity, concept lookup, math wrappers, operation vocabulary, translation round-trip).
4. **Honest coverage ledger**: `data/seed/languages.lino` records per-language coverage (meanings with labels, detection confidence, suites passing); partially covered languages answer in-language where covered and state the gap honestly otherwise (`language_gap` event) instead of silently falling back to English.
5. **Scale path**: document (and bound) what full breadth would take — Wikidata supports hundreds of languages; the protocol must be repeatable by a data contributor without Rust knowledge.

## Acceptance criteria

- The fifth language passes the auto-generated round-trip matrix and ≥ 80% of the multilingual specification suite, with the remainder recorded as `language_gap` entries.
- A dry-run of the protocol on a sixth language produces a coverage report without any code edit.
- CI language-parity checks (the guard konard requested ~11 times across PRs #175–#240) extend automatically to every language registered in the ledger.

## Dependencies

- Blocked by #660 (E41 importer is the ingestion engine).
- Related: #659 (E40 hardcoded-string lint prevents regressions), #674 (E55 same-skill-links-from-any-language), #526/#292 (closed parents).

## Process

Collect data to `docs/case-studies/issue-{id}` (Wikidata lexeme coverage statistics per candidate language, detection-rule research, survey: Universal Dependencies, CLDR); single PR until every requirement is addressed.

