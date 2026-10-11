TASK (tag MIGRATE4): continue R918-2 / R914-6 / R1085-2 / R344. Of the handlers in data/meta/handler-migration-ledger.lino, 20 are still pending. Migrate the smallest pending handlers to seed rule sets (data/seed/handler-rules.lino), smallest first. Follow MIGRATE3's pattern from round 14: table readers in both runtimes, wording in seeded responses, and before/after captures proving answers are byte-identical unless a change is stated and deliberate.

- Skip `learn_from_source` (the browser twin is new) and `proof_request` (its JS twin is a separate hardcoded planner) unless everything smaller is done.
- After each handler:
  - lower `handler_migration_pending` and `literal_predicates` / `hardcoded_language_rows` in data/meta/debt-ratchet.lino as measured, with notes;
  - re-measure data/meta/core-boundary-ledger.lino with reasons;
  - prune stale allowlist rows;
  - update the four requirement rows' counts.
- The JS-literal ratchet (data/meta/js-literal-ratchet.lino) must be lowered when worker literals fall.

Ledger rows T240–T249. Never run cargo. The core-boundary and ratchet gates run through `local-gates.mjs`.
