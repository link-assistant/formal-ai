## Issue #982 Persisted-Memory Compatibility Contract

Issue [#982](https://github.com/link-assistant/formal-ai/issues/982) requires a
safe upgrade boundary for long-lived `.lino` memory shared by CLI, server,
desktop, Telegram, and container deployments. The schema contract lives in
`src/memory/upgrade.rs`; operator guidance is in
`docs/configuration/memory.md`; regression and container coverage is in
`tests/integration/issue_982_memory_upgrade.rs` and
`experiments/issue_982_memory_upgrade/run_container_upgrade.sh`.

| ID | Requirement | Status / Evidence |
| --- | --- | --- |
| R982-1 | Preflight compatibility must be machine-readable and must not mutate memory or create migration artifacts. | Implemented by `formal-ai memory upgrade-status --format json` / `preflight_memory_upgrade`; `upgrade_status_detects_released_schema_without_mutating_memory` asserts byte equality and a one-entry directory. |
| R982-2 | Persist and report explicit detected, minimum-readable, maximum-readable, and target schema versions. | Schema 1 names released unversioned `demo_memory`; schema 2 adds the root marker. The CLI and `/health` expose all four values and the migration state. |
| R982-3 | Upgrades must be explicit; startup, health, ordinary reads, and ordinary writes must never silently migrate an existing file. | `MemoryStore` and `SyncStore` retain the detected source schema on save; only `memory migrate` targets schema 2. Guarded by `ordinary_server_write_preserves_released_schema_and_unknown_metadata` and the health test. |
| R982-4 | Migration must coordinate with ordinary writers and reject incompatible/future schemas without modifying them. | `migrate_memory` holds the existing sibling `fs2` writer lock for the whole transaction and fails closed. Lock and schema-99 refusals are asserted as nonzero JSON responses. |
| R982-5 | Create and verify a byte-exact rollback backup before commit. | The migration checks both bytes and SHA-256, preserves source permissions, rejects a conflicting pre-existing backup, and emits its path/digest in the receipt. |
| R982-6 | Commit atomically and make interruption/retry safe and idempotent. | Same-directory create-new staging, `sync_all`, atomic rename, parent sync, cleanup on the pre-commit interruption hook, content-addressed default backup, and target-schema no-op retry; tested by interruption and whole-flow cases. |
| R982-7 | Preserve unknown metadata, stable identifiers, event ordering, and history. | Migration inserts only the additive root marker; event parsing/formatting also round-trips unknown fields. The whole-flow test compares exact bytes apart from the marker and checks ids/order/extensions/query/export. |
| R982-8 | Emit a durable machine-readable migration receipt with rollback instructions. | `MemoryMigrationReceipt` includes binary/schema versions, migration id, paths, before/after hashes, event count, changed flag, and rollback strategy; its exact JSON value is checked against stdout. |
| R982-9 | Provide fixtures for every readable/released schema and reject an intentionally incompatible fixture. | `tests/fixtures/memory/schema-{1,2}.lino` are enumerated by `fixtures_cover_every_readable_schema`; the schema-99 test verifies status and migration refusals. |
| R982-10 | Prove a previous released container and candidate container can use the same named volume through upgrade, verification, rollback, and old-version reopen. | `run_container_upgrade.sh`, wired after the candidate Docker build in CI, writes with `0.335.0`, preflights/migrates, checks candidate server health/query/export, restores the verified backup, and reopens with `0.335.0`. |
| R982-11 | Keep implementation provenance and self-hosting evidence reproducible. | The issue/PR evidence collector outputs are committed with manifests; two differently worded real Agent-CLI runs and deterministic session JSON live under `docs/case-studies/issue-982/self-hosting/` and are replayed byte-for-byte by the integration suite. |
