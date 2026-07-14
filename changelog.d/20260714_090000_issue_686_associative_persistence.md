---
bump: minor
---

### Added
- Usage-weighted associative persistence for issue #686
  (`src/associative_persistence.rs`): an `AssociativeMemory` that keeps a
  persistent version of meta-language expressions saved in an associative links
  network. Each expression is a content-addressed node (`stable_id`, so one meaning
  is one node) in an embedded `SubstitutionGraph`; the store counts usages (reads)
  and changes (writes) per expression and derives an independent usage signal from
  each node's incoming and outgoing link degree. A single `retention_score` (reads
  + writes + in-degree + out-degree, under configurable `RetentionWeights`) drives
  an LFU-style policy so the most used, most changed, and most connected knowledge
  persists longest; `eviction_order` / `evict_least_used` / `retain_most_used`
  forget the lowest-scored first, and `forget` removes an expression together with
  its incident links. Everything serializes to Links Notation, `from_context`
  ingests an issue #649 world-model `Context` preserving statement ids, and the
  whole policy is deterministic (no clocks, no randomness). Durable
  `MemoryEvent::write_count` now round-trips through native serialization, sync,
  substitutions, link projection, and the browser mirror; automatic dreaming
  rebuilds this associative view and uses the complete score for real eviction.
  Event ingestion also preserves qualifiers and validation warnings, normalizes
  evidence aliases, and supports bounded multi-hop recall. A derived persisted
  memory scenario executes through Formal AI and the real external Agent CLI.
  Covered by the issue-686 persistence, dreaming, and agentic regression suites.
- Design case study for issue #686 under `docs/case-studies/issue-686/`: a deep
  analysis mapping persistence, read/write counting, incoming/outgoing-link-degree
  usage, and links-only retention onto the associative stack, with cited online
  research (the Wikontic paper's full transferable symbolic pipeline,
  AriGraph, LFU/LRU cache replacement, reference counting, degree centrality), a
  per-requirement solution plan and prior-art survey, requirement rows R445–R458 in
  `REQUIREMENTS.md`, and the `tests/unit/docs_requirements_issue_686.rs`
  traceability test.
