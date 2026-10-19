## Issue #673 Workspace Source Representation

Issue [#673](https://github.com/link-assistant/formal-ai/issues/673) (E54) asks
the self-representation to grow from the single pinned module of R381 to a
census of the whole workspace, so a self-coding planner can introspect more than
one file. PR [#807](https://github.com/link-assistant/formal-ai/pull/807) adds
per-module census documents under `data/meta/self-ast/`, an in-memory workspace index, a
drift guard, and census-backed edit-target resolution in the general planner.

| ID | Requirement | Status |
| --- | --- | --- |
| R480 | Represent every owned source module in an addressable workspace model. The 2026-10-18 doctrine requires complete readable recursive CSTs across JavaScript, TypeScript and Rust. | Partial legacy inventory: the current self-AST census addresses Rust modules but does not preserve complete source content. Migrate its consumers to the source model; this row is not full CST acceptance. |
| R481 | Preserve every module completely. The former full-AST/signature split is superseded by the 2026-10-18 source-model doctrine; fidelity labels cannot waive comments, whitespace, syntax or formalized documentation. | Required migration: signature and histogram documents do not satisfy the full recursive content contract. |
| R482 | Regenerate deterministically from complete source content and fail on changed, missing or orphaned structural content. Validate source-free reconstruction and rule-based edits without manually maintained expected counters. | Partial legacy drift detection; the existing census comparison must be replaced by full CST/content checks and derived inventories. |
| R483 | Resolve edit targets through the census index instead of hardcoded paths, for any module the method registry knows. | `resolve_census_target` in `rust/src/agentic_coding/general_planner.rs` routes `compose_edit_request` through `WorkspaceCensus::resolve`; covered by `the_planner_resolves_an_edit_target_outside_planner_rs_via_the_census` and `the_index_resolves_every_path_symbol_the_method_registry_knows`. |

Current architectural authority: [readable recursive source and comment meta-language](doctrine-readable-recursive-source-and-comment-meta-language-2026-10-18.md). Historical census results remain scoped to their actual narrower behavior.
