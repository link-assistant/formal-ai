# Issue 686 — Holistic Requirements

This list combines the issue body with the maintainer's follow-up on PR #689
([comment](https://github.com/link-assistant/formal-ai/pull/689#issuecomment-4973111296)).
The issue thread itself has no comments, but the PR feedback materially expands
the acceptance criteria: implement the full vision through auto-learning, execute
the same task through Formal AI via Agent CLI, follow the repository's
self-hosting/testing guidance, and generalize touched logic.

## Conceptual feature requirements

| ID | Requirement | Status and evidence |
|---|---|---|
| **R686-01** | Apply **all transferable Wikontic practices**, not merely deduplication and degree. | Done — candidate event ingestion preserves qualifiers; evidence relations are validated; prefixed aliases normalize to canonical ids; duplicate ids merge; warnings are retained; bounded multi-hop recall is executable in `AssociativeMemory::from_memory_events` and `recall_related`. |
| **R686-02** | Persist meta-language expressions in associative links networks. | Done — `AssociativeMemory`, Links Notation, and the durable `MemoryEvent` adapter. |
| **R686-03** | Count reads. | Done — `access_count` round-trips durably and becomes `PersistedExpression::reads`; multi-hop recall increments reads. |
| **R686-04** | Count writes/changes. | Done — durable `MemoryEvent::write_count`/`writeCount`, stable-id updates, sync merge, Rust substitution, and browser substitution. |
| **R686-05** | Retain frequently read or changed data longer. | Done — dreaming now uses `AssociativeMemory::retention_score`, so the live eviction plan consumes the four-signal score. |
| **R686-06** | Derive usage from both incoming and outgoing links. | Done — evidence and legacy id references become directed associations; both degree halves contribute to live dreaming usage. |
| **R686-07** | Keep everything as links / links networks, never a separate vertex/edge model. | Done — associations use `SubstitutionLink`; persistence/report artifacts use Links Notation. |

## Paper-pipeline and runtime requirements

| ID | Requirement | Status and evidence |
|---|---|---|
| **R686-08** | Preserve context qualifiers during candidate extraction. | Done — event kind, role, intent, tool, time, and conversation qualifiers are retained per expression and serialized as links. |
| **R686-09** | Apply ontology-aware alignment without hiding failures. | Done — evidence relation endpoints are checked; unresolved candidates remain persisted with `validation_issues`. |
| **R686-10** | Normalize aliases and deduplicate incrementally. | Done — stable ids deduplicate expressions; namespaced evidence aliases resolve to canonical event ids; stable-id rewrites replace stale text and increment writes. |
| **R686-11** | Support iterative, bounded multi-hop retrieval. | Done — deterministic bidirectional breadth-first `recall_related(id, max_hops)` with read accounting. |
| **R686-12** | Integrate the policy into real persistent memory and auto-learning/dreaming. | Done — `MemoryEvent` serialization/sync/substitution and `dreaming::usage_counts` use the associative adapter; the browser mirror persists the same counters. |
| **R686-13** | Execute the same learning task through Formal AI via Agent CLI. | Done — `agentic_coding::associative_learning`, the generalized document recipe, derived input fixture, driver tests, and committed external-CLI evidence. |

## Case-study and delivery requirements

| ID | Requirement | Status and evidence |
|---|---|---|
| **R686-14** | Collect issue and PR data in this case-study directory. | Done — refreshed `raw-data/` includes the maintainer feedback and all three PR comment/review APIs. |
| **R686-15** | Perform deep online research and distinguish paper claims from project inferences. | Done — `raw-data/online-research.md` covers all six Wikontic stages and labels degree-weighted retention as issue #686's inference. |
| **R686-16** | Propose solutions/plans and survey reusable components. | Done — `solution-plans.md` and `persistence-mapping.md`. |
| **R686-17** | Add reproducing automated tests before fixing logic defects. | Done — stable-id stale-text, durable write round-trip, bidirectional retention, qualifiers/warnings, multi-hop, and Agent CLI regressions. |
| **R686-18** | Plan and execute all work in PR #689, including merging current `main`. | Done — the branch contains merge commit `978a0164` and all implementation/docs/tests. |

## Relationship to the global matrix

The global [`REQUIREMENTS.md`](../../../REQUIREMENTS.md) retains R445–R458 for
the original issue-body deliverables and adds R453–R458 for the maintainer's
full-pipeline, durable-runtime, browser-mirror, and Agent CLI acceptance criteria.
