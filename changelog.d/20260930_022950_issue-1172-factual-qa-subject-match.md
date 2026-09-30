---
bump: patch
---

### Fixed

- Factual Q&A no longer answers for the wrong subject: "What is the capital
  of Australia?" answered "The capital of the United States is Washington,
  D.C." because the United States alias "us" matched as a raw substring
  inside "a**us**tralia" (issue #1172). `FactRecord::matches_normalized`
  (`rust/src/seed/facts.rs`) now matches subject aliases and question
  keywords only at word boundaries through the new
  `FactRecord::contains_word_sequence` — the phrase is tokenized with the
  same normalizer the prompt was and must appear as a consecutive token run,
  while CJK surfaces (no inter-word spaces) keep substring matching, the
  contract `seed::meanings::surface_present` established (issue #386). The
  browser worker's `tryFactLookup`, the coreference keyword prefilter, and
  the antecedent fact-alias match (`js/worker/formal_ai_worker_05.js`) get
  the same boundary treatment via the shared `surfacePresent` helper.
  `fact_store_resolves` therefore returns `false` for subjects with no
  seeded record, unblocking the open-web planner's fallback instead of
  discarding it for a false hit. Possessive aliases ("japan's") match again
  through the phrase-side normalization, and the contradicted
  "Implemented" rows R173/R174/R177 of the issue-0127 requirements shard now
  record the defect and its correction. Pinned by
  `rust/tests/unit/issue_1172_factual_qa_subject_match.rs`.
