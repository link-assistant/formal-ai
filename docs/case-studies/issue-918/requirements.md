# Issue 918 Requirements

| ID | Requirement | Verification |
| --- | --- | --- |
| R918-1 | Define the minimal compiled core before deciding what belongs in code. | `docs/design/minimal-core-boundary.md` defines four exhaustive categories and a promotion test. |
| R918-2 | Ledger every recursive handler source as migrate, promote, or delete, with a reason and shrink ratchet. | `data/meta/core-boundary-ledger.lino`, `scripts/check-minimal-core-boundary.rs`, and `minimal_core_ledger_covers_every_recursive_handler_source`. |
| R918-3 | Define role, precondition, effect, unit, and example using semantic-data precedent. | `data/meta/seed-metadata-schema.lino` records the FrameNet and Wikidata shapes and primary sources. |
| R918-4 | Require full metadata for concepts on the coding path. | `coding_path_has_complete_metadata_and_every_other_gap_is_data` requires all five fields on all 37 coding records. |
| R918-5 | Record every remaining gap as deterministic data and prevent regressions. | `scripts/audit-seed-metadata.rs` checks 3,447 gap records across 16 stable shards exactly. |
| R918-6 | Preserve traceability, CI, and self-hosting evidence. | Root docs, raw GitHub captures, changelog, script tests, and the Agent CLI invariant test preserve the review trail. |

## Reviewed Leaf Accounting

The reviewed leaves are: (1) boundary definition and handler ledger, (2)
metadata schema and complete coding records, (3) gap shards and auditor, (4)
regressions, CI, and traceability, and (5) the minimal-core invariant document.
The real Formal AI/Agent CLI session authors leaf 5 without manual byte edits.
That is one of five leaves, meeting the repository's 20% floor without
attributing the other manually implemented work to the agent.
