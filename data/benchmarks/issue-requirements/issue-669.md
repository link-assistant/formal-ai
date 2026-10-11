Parent: #651. Full planning context with audits: `docs/case-studies/issue-651/` (this issue is E50 in `proposed-issues.md` there; added by PR #652).


**Problem**

`VISION.md` (Growable Memory) names "future cloud sync" as a persistence
target beyond disk and IndexedDB; F3 describes the journey — the same
`formal_ai_bundle` follows the user between machines automatically. Today
migration is manual export/import only.

**Approach**

1. Keep it associative and vendor-neutral: sync the append-only event log
   through any user-supplied backend, starting with the simplest ones — a
   user-owned git repository and a WebDAV/S3-compatible endpoint. No
   Formal-AI-hosted service.
2. Append-only makes sync tractable: push = append local events since the
   last synced id; pull = append remote events not present locally; the
   projected prefix never shrinks (the existing concurrency guarantee).
   True conflicts are impossible at the log level; duplicate-suppression by
   content-addressed event id.
3. `formal-ai memory sync --remote <url>` plus a `SolverConfig`/env knob;
   off by default, never syncing without explicit opt-in (privacy is the
   product's promise).
4. Browser side: manual "Sync now" against the same remote via fetch;
   background sync deferred.

**Existing components**

- Content-addressed event ids; `memory::{export,import}_full_memory`;
  `/v1/memory/since` incremental endpoint (the same delta shape sync needs).

**Acceptance criteria**

- `cargo test memory_sync` — two stores syncing through a temp-dir remote
  converge to the same projected state from interleaved appends; re-running
  sync is idempotent.
- Sync refuses to run without the explicit opt-in flag/env.
- e2e: CLI ↔ web round-trip through a local WebDAV fixture.



