---
bump: minor
---

### Added
- Repository history (#1180 R11): the git subprocess, events store, cursor and incremental import have a JavaScript twin, `js/agentic/crate/history_store.mjs`, over an injected io.
- Obligation gaps (#1166 R4): every executor reports the obligations it cannot discharge, including output literals the request demands but the program does not print. This covers the catalog `write_program` arm in Rust, the JS agentic root and the browser worker.
- Generated versions (#1168 R8): `ci_workflow::render_with` / `renderWith` take an injected version set. A shared fixture pins the same rendered workflow in both runtimes.
- Code examples (#1164 R11): a supplied page whose query asks for the parts of its code examples is decomposed by the solver in both runtimes (parity case `e1164_supplied_page_code_example_parts`).
- Derivations (#1184 R9): the JS server persists each answer's derivation and gains `explain <id>`, mirroring `finalize_answer` and `formal-ai explain`.
- Prompt-text questions (#1172 R7): the answer projects the covering sentence into subject, predicate, object, time and place slots, and answers the asked slot first.
- Explanation research (#1172 R8) has a browser twin. The JS server reads the procedure cache and calls `cachedWriteProgram` (#1165 R10).
- Capability routing (#1175 R3): six more claim rows (supplied page, JavaScript program, incompatible units, fetch URL, navigation URL, calendar date signal).
