---
bump: minor
---

### Added
- Generated, split Mermaid system diagrams (overview, solver handlers, CLI subcommands, HTTP routes) from live data, with a drift gate (R382).
- `serve --debug-session`: a loopback-only, token-guarded protocol that pauses a turn between solver stages and advances it step by step from the VS Code debugger view, in both servers (R383, partial).
- The proof engine, number-constraint answers and coding guidance read their prose from seed data in every registered language (R379: 250 allowlist rows retired).
- Two more handlers migrated to seed rule sets (`algorithm`, `document_generation_plan`), with `operation`, `table` and `response` interpreter primitives in both runtimes (R918-2, R914-6, R1085-2).

### Fixed
- Formal AI (the repository's own agent) learned the edit shapes it failed while doing this work: a positional insert widens a fragment anchor to its whole line, reads position cues and target cues outside quoted payloads, and takes a colon-introduced block as the inserted lines; "set K to V" assigns a setting instead of overwriting its key; a quoted literal led by a description ("the heading '# Title'") stands for the literal; a sentence's closing mark is not file content; a path inside a quoted payload never outranks the file the request names; functions named by identifier can be removed whole.
- Fourteen native and eight browser routing probes now answer as intended (R1173-3): seeded English contractions, lock-picking legality, HTTP `GET`, `find` calculations, one-item change problems.
