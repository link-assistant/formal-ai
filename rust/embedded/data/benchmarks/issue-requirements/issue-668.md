Parent: #651. Full planning context with audits: `docs/case-studies/issue-651/` (this issue is E49 in `proposed-issues.md` there; added by PR #652).


**Problem**

`docs/USER-JOURNEYS.md` F6: a user packages datasets, skills, rules, and
handlers with permissions and shares them with another instance —
Deep.Foundation-style packages adapted to doublets. `src/associative_package.rs`
unified the in-repo package/permission model (E18), but there is no
export/import of a *package* as a shareable artifact between two running
instances.

**Approach**

1. Define the package manifest in Links Notation: name, version, declared
   permissions, contained links (meanings, rules, skills, handler
   references), and provenance — a scoped subset of the full
   `formal_ai_bundle`.
2. `formal-ai package export <name>` / `formal-ai package import <file>`:
   import runs the permission review (list declared permissions, require
   explicit confirmation for `agent`-tagged tools), appends
   `package_imported` events, and rejects handler references that are not
   locally available instead of silently degrading.
3. Round-trip across surfaces: CLI-exported package imports in the web demo
   (file picker) and vice versa.
4. Ship one real example package (e.g. a language pack or skill pack) under
   `examples/packages/`.

**Existing components**

- `src/associative_package.rs`, permission gating, bundle
  export/import/migration notices (`memory::export_full_memory`).

**Acceptance criteria**

- `cargo test associative_package_sharing` — export → fresh store → import
  reproduces the package's links and permission gates; importing a package
  demanding an unavailable handler fails loudly.
- Web ↔ CLI round-trip covered by an e2e test.
- Example package documented in `README.md`.



