# Source witness validation and template metadata, 2026-11-21

The maintained complete JavaScript gate inventory passes after the generated
native source witness repair: 88 executed gates pass and 53 entries are
explicitly skipped, with zero failures. The sequential run takes 577.591
seconds under a 256 MiB Node heap setting; measured child peak RSS is
382,472 KiB. All 88 executed gate streams verify by raw/compressed SHA256,
byte count and line count. No local Rust compilation or native execution
is performed. The run starts at local commit `396a4af7`; later evidence-only
changes do not acquire additional native runtime verification from it.

The pinned Rust template's complete npm 12.2.2 metadata is now literally
read: all 4,513 lines across ten contiguous ranges reassemble the original
96,962 bytes exactly. Literal source review reaches 356/373 files and
57,336 lines, with 74 practice decisions. Node engine patch floors,
registry integrity/signatures, dependencies and bundled payload identity
must be checked before a package-manager major upgrade. This historical
metadata does not prove live compatibility or publication. Complete template
comparison and applicable behavioral verification remain unfinished.

Fresh registry observations still show `command-stream 1.13.0` and
`link-calculator 0.24.0` requiring Chrono. The command-stream migration
[issue 238](https://github.com/link-foundation/command-stream/issues/238)
is closed, but that registry version still has the dependency. Calculator
[issue 265](https://github.com/link-assistant/calculator/issues/265) remains
open. Both complete comment endpoints are empty. `meta-language 1.0.0`
has none of the selected direct maintenance dependencies; this does not
qualify a change to the intentionally historical 0.58.1 reproducer or its
transitive graph. No dependency or lockfile changes are made. Maintenance
migrations remain unresolved.

The published head `27e895c1` is still collecting its final Pipeline jobs.
Its known Layered CI source-witness failure is repaired locally, with the
existing five-case regression passing. Do not interpret the JavaScript gate
result as final-head hosted acceptance. All source warnings, wider issue
requirements and final review remain open; PR #1581 stays draft and unmerged.

Evidence: `experiments/issue-1580/evidence/checks/current-source-witness-final-validation/`
and `experiments/issue-1580/evidence/checks/rust-source-review/`.


The post-archive abbreviation check rejects one prefixed copy of the existing
`check-tests-as-docs` gate log: its new evidence filename falls outside the
maintained command's fixed-name spelling. Rename that evidence file using
`documentation`, preserve its original bytes and update the manifest identity.
The failed check and exact original finding are retained separately. Gate
limits and production commands remain unchanged; the targeted retry must pass
before committing this evidence batch.
