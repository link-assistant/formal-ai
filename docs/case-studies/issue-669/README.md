# Append-only memory synchronization (#669)

Manual export/import has no incremental convergence protocol. This slice introduces a `Transport` contract and a user-owned directory backend. Call `sync_memory` with `SyncConfig { enabled: true }`, the local `MemoryStore`, a `DirectoryTransport`, and a retained `SyncCursor`. Disabled sync returns before creating the remote directory.

Each canonical event is exported with the existing memory codec and addressed by SHA-256. Local recall/write counters are excluded because they are machine-local bookkeeping. Events require stable nonempty ids. Reusing one event id for different content returns IdentityConflict; explicit substitutions of old events therefore require a new append-only event id before sync. Additional events arrive by append, preserving the current local prefix. Both stores converge as sets of events; their physical append ordering remains local. This does not impose a distributed total order on order-sensitive projections.

The directory transport publishes complete fsynced objects using a hard link, then removes the temporary name. Concurrent publishers never overwrite an object. Remote data is digest-checked and canonical-round-trip-checked before local mutation. Failed transport writes can leave valid remote objects, which the next pass safely recognizes. A caller must durably save its local store after successful sync. Cursor loss is safe and remote restoration is handled by checking actual remote presence rather than trusting cursor history.

Tests draft interleaved convergence, repeated-sync idempotence, consent refusal without remote I/O, conflicting identities and path traversal. They were not run under the bulk-work instruction.

Integration requires `pub mod cloud_sync;` in `rust/src/lib.rs`, `mod issue_669_memory_sync;` in the unit registry, and registration of `cloud-sync-policy.lino`. CLI `memory sync`, SolverConfig/env opt-in, browser Sync now, authenticated git/WebDAV/S3 adapters and CLI/web fixture tests remain pending. The directory backend works with local or user-mounted storage; it does not claim to authenticate to a cloud URL.
