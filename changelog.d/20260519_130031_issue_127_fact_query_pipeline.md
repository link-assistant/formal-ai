---
bump: minor
---

### Added
- Issue #127: structured fact-query reasoning pipeline. Multilingual prompts
  about a country's capital, population, currency, official language,
  continent, area, head of state, and head of government are parsed into
  `(relation, subject, language)` triples, routed against a 1-week TTL cache
  pre-warmed from `data/seed/facts.lino`, and resolved live against Wikidata
  (`wbsearchentities` + `wbgetentities`) for any uncovered country. Each step
  is recorded in the append-only memory log as a `fact_query:*` event so the
  reasoning trace is fully inspectable.
- Pre-warmed capital cache for Russia, Japan, France, Germany, China, India,
  the United States, the United Kingdom, and Brazil — every entry carries
  multilingual `subject_aliases`, localized labels, and Wikidata Q-IDs so the
  same prompt resolves consistently across English, Russian, Hindi, and
  Chinese.
- Force-fresh markers in every supported language (e.g. "refresh", "не из
  кэша", "ताज़ा", "刷新") let users bypass the cache and force a live
  Wikidata fetch when they explicitly ask for fresh data.

### Changed
- The Rust solver now emits structured `fact_query:request`,
  `fact_query:relation`, `fact_query:subject`, `fact_query:cache:hit`,
  `fact_query:subject_qid`, and `fact_query:value_qid` evidence links
  alongside the legacy `fact_lookup:*` events so the Rust and browser stacks
  agree on the reasoning shape and on Q-ID anchoring.
