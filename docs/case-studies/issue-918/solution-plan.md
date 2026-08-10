# Issue 918 Solution Plan

## R918-1 and R918-2: Recursive Boundary Census

Define the only four admissible compiled-core categories and a strict promotion
test. Enumerate `src/solver_handlers/**/*.rs` recursively, give each source one
decision and reason, and fail on unledgered, missing, resurrected, enlarged, or
stale-baseline entries. Preserve #699's method ledger as history while closing
its source-census gap.

## R918-3 and R918-4: Metadata Contract And Coding Floor

Represent role, precondition, effect, unit, and example as direct concept
fields. Record why the shape follows FrameNet and Wikidata. Require every
coding-catalog and coding-task concept to carry every field so the first
problem-solving path cannot regress.

## R918-5: Gap Data

Audit direct concepts in every `data/seed` meanings root. Emit each absent
field as a deterministic Links Notation record sharded by stable concept ID.
Check generated and committed maps for exact equality so missing, invented,
duplicated, and stale gaps all fail.

## R918-6: Traceability And Self-Hosting

Preserve raw issue, PR, parent, prior-art, and CI data; update root requirements,
architecture, roadmap, and release metadata; and wire both scripts into CI.
Run one exact invariant leaf through `formal-ai serve` and the installed Agent
CLI, preserve the raw evidence, and byte-check it in the unit suite.
