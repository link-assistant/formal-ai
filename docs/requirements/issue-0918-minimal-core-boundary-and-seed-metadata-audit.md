## Issue #918 Minimal-Core Boundary And Seed-Metadata Audit

Issue [#918](https://github.com/link-assistant/formal-ai/issues/918) turns E71's
architectural goal into audited data and shrink-only gates. See PR #986 and
`docs/case-studies/issue-918/`.

| ID | Requirement | Status |
| --- | --- | --- |
| R918-1 | Define the smallest acceptable compiled core and classify everything else as data or migration debt. | Implemented in `docs/design/minimal-core-boundary.md`: meta algorithm, link store, generic interpreters, and host surfaces are the only four core categories. |
| R918-2 | Inventory every handler source recursively and record migrate, promote, or delete with a reason. | `data/meta/core-boundary-ledger.lino` covers all 88 recursively counted handler sources: 37 are promoted generic interpreters that read their vocabulary from seed data and 51 remain migration debt (`disposition migrate`, each with its data target and reason). `scripts/check-minimal-core-boundary.rs` ratchets the 20,507 outside-core lines two ways (growth needs a recorded note; shrinkage lowers the baseline). Per-handler progress lives in `data/meta/handler-migration-ledger.lino` (31 migrated, 40 pending as of the 2026-10-08 R914-6 batch), so the row is partial until the pending rows reach zero. |
| R918-3 | Define role, precondition, effect, unit, and example metadata using established semantic-data shapes. | `data/meta/seed-metadata-schema.lino` defines the five-field contract and records the FrameNet and Wikidata provenance used to choose it. |
| R918-4 | Make concepts on the coding path satisfy the complete schema. | All 37 direct concepts in `meanings-coding-catalog.lino` and `meanings-coding-tasks.lino` carry all five fields; the auditor rejects any new gap. |
| R918-5 | Represent all other metadata gaps as reviewable data, not an informal document. | Sixteen deterministic `seed-metadata-gaps-*.lino` shards record the exact missing fields for the other 3,447 concepts, and the auditor rejects stale, omitted, or invented rows. |
| R918-6 | Preserve the existing regression floor and reproducible self-hosting evidence. | Focused Rust/script tests, default CI gates, raw issue/PR evidence, and one of five reviewed leaves produced by the real Formal AI/Agent CLI loop are preserved in the case study. |
