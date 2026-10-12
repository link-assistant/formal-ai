Parent: #244
Audit source: PR #245 post-implementation audit on 2026-05-26, after issues #246-#259 and #262 were merged.

## Problem

Issue #246 introduced the `LinkStore` boundary and doublet projections, but the native durable store is still the reviewable `.lino` projection unless a feature-backed implementation is selected. `ARCHITECTURE.md` still lists the native physical-store migration as a remaining question, and `src/link_store.rs` explicitly keeps the `.lino` implementation as the current default.

The vision says the associative network is the AI. That requires the native runtime to use `doublets-rs` as the default physical store, while keeping Links Notation as the auditable export/import projection.

## Scope

- Make `doublets-rs` the default native physical backend for persisted link records.
- Preserve `.lino` export/import as the human-reviewable projection and migration format.
- Keep browser storage compatible with `doublets-web`/IndexedDB expectations.
- Add migration coverage from existing `.lino` bundles into the default native store.
- Document how to recover, inspect, and export the store.

## Acceptance criteria

- A default native build persists link records through `doublets-rs` without requiring an opt-in feature for the primary store path.
- Existing `.lino` memory bundles import into the native store and export back to deterministic Links Notation.
- CLI, HTTP, library, and Telegram surfaces continue to share the same store semantics.
- Tests cover migration, stable IDs, append-only history, malformed import rejection, and feature/fallback behavior.
- `ARCHITECTURE.md`, `ROADMAP.md`, and `REQUIREMENTS.md` no longer describe the native doublets store as future work.

## Requirement links

- `REQUIREMENTS.md` R60
- `ARCHITECTURE.md` section 16.1
- Follow-up from #246

