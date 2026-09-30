## Issue #669 Cloud memory sync

| Requirement | Status | Evidence / remaining work |
|---|---|---|
| Vendor-neutral user-owned backend | Implemented transport library | Transport contract and atomic DirectoryTransport; no hosted service. Git/WebDAV/S3 network adapters pending. |
| Append-only push/pull and content-addressed duplicate suppression | Implemented library | One canonical event per SHA-256 object; immutable publication; existing local prefix preserved. |
| Explicit opt-in, off by default | Implemented library | SyncConfig defaults disabled; refusal occurs before any backend operation. CLI/env/SolverConfig wiring pending. |
| Interleaved stores converge and repeat sync is idempotent | Tests drafted, not run | issue_669_memory_sync.rs exercises a temp-dir remote, four interleaved appends and projection equality. |
| Invalid data and conflicting event identities | Implemented library | Validate digest, canonical round-trip, key syntax and single-event shape before changing local memory. |
| CLI memory sync command | Pending integration | Register module, expose remote option and opt-in flag; caller persists local memory and cursor after success. |
| Browser Sync now and CLI/web WebDAV e2e | Pending integration | Browser adapter and local WebDAV fixture remain. |
| Formal sync policy | Drafted seed | cloud-sync-policy.lino and byte-identical embedded mirror; registry integration pending. Current library invariants are explicit; seed is descriptive schema, not live configuration. |

No local build or tests were run. This is a library slice, not a completed cross-surface cloud service.
