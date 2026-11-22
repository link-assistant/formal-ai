# Generated inventories after the main integration

PR [#1581](https://github.com/link-assistant/formal-ai/pull/1581) ran every local
JavaScript gate on committed `5d68b749`: 85 passed, 54 were CI-only skips and
four failed. The complete logs preserve both distinct failures and their twin
gate failures. Skips provide no native or hosted qualification.

The seed-registry checks found a duplicate `seed_files` body left alongside the
new include in `embedded_registry.rs`. The included `embedded_files.rs` still
named four seed files already retired by main. Running the existing canonical
JavaScript generator removes the duplicate body and those four stale entries;
the one included function retains the complete current registered bundle.
Both generated files were compared byte for byte with the previously
rendered canonical expectations before accepting the change. No generator or assertion
was changed. The existing generator gate reproduces the pre-fix failure.

The worker-budget checks found 1307 lines against a stale ceiling of 1292.
The entire worker is byte-identical to main `aa93fe9c`; its merged live-Wikidata
changes retain capture hashes, reject ambiguous subjects, handle localized
spellings and claim values, and honor offline and service preferences. The one
budget record now names that measured 15-line growth. Every other ceiling and
the 3000-line total target remain intact. No worker logic changed in this repair.

The canonical seed check, worker-budget check and two existing seed-headroom
regressions pass after the repair. All 34 identifier controls also pass against
the unchanged complete specification baseline. The subsequent complete gate
run and hosted native checks remain pending. No Rust build ran locally.
Supplied procedures and edits earn zero independent authorship credit.

Evidence: `experiments/issue-1580/evidence/checks/generated-inventory-merge-repair/`.
