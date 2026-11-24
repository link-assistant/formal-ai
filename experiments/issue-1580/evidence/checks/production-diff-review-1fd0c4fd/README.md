# Current production diff review

At commit `1fd0c4fd131e318593299cfe05d0167f6efd35fc`, all 154 changed paths
under `.github/`, `scripts/`, `js/`, `ts/` and `rust/src/` have a complete
literal diff review. The base is recorded in `assessment.json`.

Seventy-nine earlier records retain matching source and diff hashes. The
remaining 75 diffs (6,267 lines) were read in bounded chunks. Their complete
bytes are retained in the archive; `reviewed-diffs.jsonl` records current
source and diff identities. This is a diff review, not a claim that every
unchanged line of these files was read.

The review followed source binding and host boundaries, failure propagation,
release identity, original timeout limits, generated runtime parity and
preservation of selected test inventories. In particular:

- Process deadlines retain monitoring after the command root exits while a
  live process remains in its group; zombies do not extend that lifetime.
- Release registry and filesystem uncertainty propagates as failure. Cargo
  identity is parsed from the package table; version updates preserve the
  surrounding source bytes. Fragment cleanup uses the collected snapshot.
- Typed composition retains rejected IR diagnostics and does not substitute
  generic prose composition for a concrete root output contract.
- Generated worker diffs carry the source changes for quote cues, source-read
  evidence, language fallback, calendar quantities and seed list syntax.
- Native admission includes the selected fixture/helper dependencies. Shard
  changes retain the selected inventory and existing execution budgets.
- CodeQL uses original sources, declared dependency contexts, a compatible
  compiler ABI and a strict gate for findings and extraction diagnostics.

No additional established defect was found in this diff pass. Hosted CodeQL
verification is pending. Tests, data, documentation, experiments and retained
archives outside these prefixes still require the rest of the whole-PR
review; coverage of this production subset does not close that criterion or
prove preservation of every pre-existing feature. The PR remains a draft.

PR: https://github.com/link-assistant/formal-ai/pull/1581
