## Issue #673 Workspace Self-AST Census

Issue [#673](https://github.com/link-assistant/formal-ai/issues/673) (E54) asks
the self-representation to grow from the single pinned module of R381 to a
census of the whole workspace, so a self-coding planner can introspect more than
one file. PR [#807](https://github.com/link-assistant/formal-ai/pull/807) adds
per-module census documents under `data/meta/self-ast/`, an in-memory workspace index, a
drift guard, and census-backed edit-target resolution in the general planner.

| ID | Requirement | Status |
| --- | --- | --- |
| R480 | Census every owned `src/` module, not one pinned file, and address each module through a workspace index. | `src/self_ast_census.rs` compiles a `WorkspaceCensus` from the compile-time `OWNED_SOURCE_FILES` manifest and renders one `.lino` document per module. The deterministic aggregate remains available through `index_notation`/`dump_self_ast_census` but is not tracked, preventing every parallel source branch from editing the same generated file; covered by `every_owned_module_has_a_committed_census_with_its_fidelity_marker` and `committed_documents_exclude_the_redundant_workspace_aggregate`. |
| R481 | Scale honestly with a documented fidelity marker per module: full AST for `src/agentic_coding/`, signature-level census elsewhere. | `CensusFidelity::{FullAst, Signature}` is chosen by `fidelity_for` and written as a `fidelity` line in every document; `the_workspace_census_is_addressable_without_a_multi_megabyte_seed` pins the size discipline. |
| R482 | Regenerate deterministically and incrementally, and fail a drift check when a committed census diverges from its source. | `WorkspaceCensus::documents` is a pure, path-sorted function of the sources; `drift_report` reports `Missing`/`Stale`/`Orphan`. Covered by `census_regenerates_deterministically_and_incrementally`, `drift_check_fails_on_a_fixture_with_a_stale_census`, and the disk guard `committed_census_documents_match_what_the_sources_render`. |
| R483 | Resolve edit targets through the census index instead of hardcoded paths, for any module the method registry knows. | `resolve_census_target` in `src/agentic_coding/general_planner.rs` routes `compose_edit_request` through `WorkspaceCensus::resolve`; covered by `the_planner_resolves_an_edit_target_outside_planner_rs_via_the_census` and `the_index_resolves_every_path_symbol_the_method_registry_knows`. |
