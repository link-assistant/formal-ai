# Terminal aa84dfc2 hosted qualification

[PR #1581](https://github.com/link-assistant/formal-ai/pull/1581) source
aa84dfc247891313dc5dc20847950e91c9528064 has 33 terminal workflows:
32 successes and one explicit Issue 1028 skip. Every complete CLI stream and
paginated job response is retained, totaling 118,342 log lines.
The inventory is not a claim that every line was literally reviewed or that
all original acceptance criteria passed.

Layered CI [run 45271423335](https://github.com/link-assistant/formal-ai/actions/runs/45271423335)
uses tested merge 3c887a7ac57dd0dcbd685cace9b650ca29b8b4ad.
All 922 JavaScript owners finish across ten successful groups. Group nine
warns after 270 seconds at complete log line 22,757; the
[capacity follow-up](javascript-capacity-followup.md) retains and reproduces it.

Pipeline [run 45271423977](https://github.com/link-assistant/formal-ai/actions/runs/45271423977)
passes all six actual Cargo archive-guard tests at complete log lines
67,485 through 67,490. These include classification and formatting controls,
package path derivation, malformed/relative/missing metadata, failed metadata
processes and all five tiny offline Cargo target-directory layouts with
stale-archive controls. This qualifies the previous Cargo repair on hosted
Rust; no local Rust compiler was run. The installer routing and compressed
historical-floor repairs produce no remaining corresponding platform warning.

Four actual platform warnings remain:

- Security run 45271423157, lines 1,028 and 1,029: debug scores 2.8 and
  require-from-string scores 2.3, below the retained threshold of three.
- Layered run 45271423335, line 22,757: the group-nine runtime warning above.
- Pipeline run 45271423977, line 1,980: crates.io publication authority is
  unknown. A token's presence does not prove a successful authorized publish.

No platform error annotation occurs in this cohort. Dependency scores and
publication authority remain unresolved criteria. A passing workflow does
not clear those observations. The later local link gate and eleven-group
changes are absent from this source and require their own hosted qualification.

All original template execution/adoption, strict style debt, CodeQL macro/include
contexts, published dependency migrations, original workflow-family budgets,
main-only delivery, cache reliability, serving identity, trusted epochs,
independent authorship/reasoning and aggregate-review requirements remain
tracked as unfinished. This receipt grants no independent credit and does
not make the PR ready to merge.

Complete outputs and identities: [verified archive receipt](../../../experiments/issue-1580/evidence/checks/terminal-aa84dfc2-cohort-20261123/manifest.json).
