# Structured-output main integration

PR [#1581](https://github.com/link-assistant/formal-ai/pull/1581) also integrates
main `aa93fe9caeb222b906904b70be5408a2c8e61b0d`, release 0.356.7. This retains
PR #1623's matching across property naming styles, acronym boundaries,
description fallback and ambiguous-label handling in JavaScript, TypeScript and
Rust. The complete source delta and PR conversation are retained with the checks.

The only merge conflict was the binary specification baseline. Full object
comparison found four source-identity changes against our previous reviewed
baseline, no supported-case semantic changes and no unsupported-case changes.
All 223 supported programs remain semantically identical to main's supported
inventory. The maintained guarded refresh installs an object identical to the complete reviewed candidate;
physical hashes for 172 source files were checked. The reader was unchanged.
The initial byte-equality probe failed: the two V8 serializations have distinct
SHA256 values, while their complete deserialized objects are deeply equal. Both
serializations, the failed probe and full comparison are retained; no test
assertion or reader was changed to accept this difference.

The structured-output HTTP suite, identifier controls, fixture-integrity checks,
Rust formatting and translation-currency checks pass locally. Status projections
were regenerated from the merged ledgers. No native compilation ran locally.
The following full local gate run and exact-head hosted checks still determine
the next push round; this integration does not claim any unfinished original
heavy, publication, template-review, serving-identity or cache-reliability criterion.
Supplied procedures and edits earn zero independent authorship credit.

Evidence: `experiments/issue-1580/evidence/checks/main-aa93fe9c-integration/`.


The unfiltered staged whitespace check reports trailing spaces in the incoming
solver transcript. Its bytes are identical to main, including terminal progress
records. They are preserved as captured evidence. The authored-source staged
check passes with retained upstream logs excluded; the original diagnostic and
its exact byte-identity proof are included in the archive.
