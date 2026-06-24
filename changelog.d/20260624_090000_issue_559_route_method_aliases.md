---
bump: patch
---

### Added
- Added route→method aliases as first-class link data (`data/meta/route-method-aliases.lino`, `src/route_method_alias.rs`) so the meta core can resolve a meta-language intent slug that is coarser or finer than the handler serving it — for example `write_program` → `write_script` — to a catalogued method (issue #559, R336).

### Changed
- The solution evidence join now resolves each need's route through `MethodRegistry::method_for_route` (direct match, then alias), recording `method_via_alias` provenance, so the program-writing need in a request like "translate apple to Russian and write a hello world program in Python" reports a resolving method instead of appearing unaddressed.
