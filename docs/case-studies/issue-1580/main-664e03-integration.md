# Reduced embeddings main integration

PR [#1581](https://github.com/link-assistant/formal-ai/pull/1581) integrates
main `664e03feb0d337d7e891313cfc9c705bc3943136`, release 0.356.8 and PR #1625.
The complete 1,200-line incoming source patch and all conversation, inline and
review records were read. This retains reduced, unit-normalized embeddings,
matching JSON/base64 values, and rejection of a whole batch at its first
unmatched input. The native Codex replay also retains `--disable plugins`.

Six merge conflicts are resolved without dropping either branch's occupation
concepts: the canonical and embedded seeds now contain the nine disjoint
records. Maintained generators produce the seed and worker inventories and
runtime feature witnesses; the TypeScript registry matches its generated
JavaScript source. Regeneration also removes the duplicate seed bundle function
reintroduced by the automatic merge.

The baseline reader is unchanged. Complete object comparison preserves all 223
supported cases and 967 unsupported records from the previous branch head.
The only four changed leaves are Cargo manifest hashes for release 0.356.8;
172 physical source identities are verified. Candidate and canonical V8
serializations are retained separately and compared as complete objects.

All six embeddings HTTP tests, the default-branch policy file, 34 identifier
controls, two fixture-integrity and two seed-headroom controls pass. Canonical
generator checks, changed Rust formatting, translation currency and regenerated
status projections pass. No local Rust build was performed. An initial formatter
scope also included unchanged generated `embedded_files.rs`, whose canonical
layout differs from default rustfmt; its failed log is retained and its bytes
are verified identical to the committed parent. The corrected command checks
the three changed Rust sources.

Before resolution, conflict markers in the generated worker prevented Formal
AI's command runner from loading. A documented bootstrap exception restored
only that file's stage-2 bytes so the required runner could restart. All subsequent
repository commands used the runner again; this step earns zero verification or
independent authorship credit. The initial malformed launch command is retained.

The evidence archive and every payload are round-trip hash verified. Full local
gates and hosted verification of the final combined head remain required, as do
all wider criteria recorded in the issue work plan. The PR remains draft.
