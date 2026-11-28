# Aggregate data review, 23 November 2026

The data subset of [PR #1581](https://github.com/link-assistant/formal-ai/pull/1581)
is reviewed against main `03a636334975cd7ca5cedff119e2dd0efbc8579f` at
source `bc08222b5a5195dc9992c0459bde2b6369377b9f`. This covers the 51 data
paths left outside the earlier production and maintained-source reviews.
The complete scoped diff has 14,518 lines and 841,707 bytes; its SHA-256 is
`be0b8e92f1e35c2c82264a5bcfb9d62c0ba05d68f7a5a456238ad7055bbb1b82`.

All 18 compact chunks, totaling 3,445 lines, were read. The compact form hides
only sections backed by the following complete checks:

- 646 unique cross-file seed records preserve their exact original bytes and
  indentation-delimited parent. They account for 7,338 changed diff lines.
  A second readback verifies every old/new interval directly from Git and
  reconstructs every omission and compact chunk.
- One uniform representation conversion reconstructs all five CNL language
  files byte for byte. It splits authored pipe lists into native arrays and
  separates prime/template names from their operands without changing their
  values or ordering.
- Duplicate-key-rejecting JSON parsing confirms the complete canonical structure
  of `data/parity/issue-1584-cnl-infeasible.json` is unchanged.

The literal remainder covers retained workflow witnesses, the focused Codex
process test mapping, the dictionary cache, read-envelope and request-context
fixtures, calendar precedence and spoken markers, native handler operands,
response-intent moves, quoted command/response values, occupation lexemes,
request prefixes and quotation policy. The review includes both sides of the
algorithm-task and Spanish composition-response moves. Existing
`seed-and-workflow-headroom.test.mjs` checks actual lexicon/response reachability
and package/browser registrations; `issue-1580-native-seed-data.test.mjs`
checks canonical command values and actual grouped-operand consumers. This
review did not change source data or rerun those unchanged tests locally.

The [receipt](../../../experiments/issue-1580/evidence/checks/aggregate-data-review-20261123/manifest.json)
retains the complete diff, original path inventory, all compact chunks, whole
proofs and reconstruction code. Every archived member was read back and checked
against its byte count and SHA-256. The earlier `inventory.json` `allRead`
field remains false because it describes literal reading of all original raw
chunks; the new coverage ledger records the completed compact review and its
conservation checks instead.

This is data-subset review evidence. It does not establish runtime behavior,
complete the remaining aggregate documentation/fixture/evidence review, or
satisfy the unfinished original issue criteria. Supplied scripts and conclusions
earn zero independent authorship or reasoning credit.
