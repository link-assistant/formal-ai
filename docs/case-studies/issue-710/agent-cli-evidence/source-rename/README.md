# Formal AI grounded identifier rename

Producer: `formal-ai/0.350.0`, driven by Agent CLI 0.26.0 with the
`formalai/formal-ai` model. Session: `ses_f59bba59dffejPDpr0cYH2PYEf`,
2026-09-16. The local binary included Plan 07's parsing and retry changes;
version alone does not identify the remote PR head.

Task: rename the identifier `digest` to `source_digest` in
`examples/generate_recurrence_source_cache.rs` and verify the changed file.
The actual current file was provided as isolated workspace context, not as
replacement bytes. Four model rounds performed a read, a grounded word-scoped
rewrite, digest verification, and a final report. The Agent CLI and corrected
authoring helper exited zero.

Independent diff inspection found exactly two changed identifier references.
The root agent transferred those bytes without modification; the resulting
repository file is byte-identical to the produced artifact.

- Input SHA-256: `92e1e42fca911fc67612e90779eddbb6da83aea192c0958610c458bd97804f75`.
- Output SHA-256: `e61f63636226bb3c375c682ac71405600e9ebe974e0408456f5c89812205156c`.
- Private artifacts/traces: `/private/tmp/formal-ai-888-rename.DyDJ1S`.

This is a successful bounded rewrite, not success at the earlier open-ended
refactoring request. That earlier session only read its input and is recorded
as a failure in [Plan 07](../../plans/07-prerequisite-discovery-bridge.md).
The root agent's preceding digest-helper/lint fixes are separate, unattributed
changes. This evidence summary was written by the root agent; raw traces are
not published.
