# Append capability batch ledger payload

Each row is one physical Formal AI invocation, including retries. No manual repository source, test or documentation edits were used. Temporary drafts and audit files were operational inputs; all repository mutations were executed by Formal AI. Unavailable original logs are explicitly observational or reconstructed. Native compilation and tests were deferred to CI.

| Task | Outcome | Observed tools | Evidence and scope |
| --- | --- | --- | --- |
| T1637 | Fail | bash | Open G112: broad implementation request read through bash and echoed source |
| T1638 | Pass | write (observed) | Initial reviewed JS append draft byte-equal; original raw log later overwritten by accidental helper repeat. Exact prompt reconstructed from retained draft; no invented transcript |
| T1639 | Fail | none | Driver bootstrap blocked by duplicate sourceFromReadResult import after accidental helper repeat; exact prompt retained, original complete stderr not retained; observational provenance only |
| T1640 | Applied; draft defect | read → write → read → write → bash | Operational duplicate helper reapplied malformed draft; exact bytes applied but driver boot failed. FA loader recovery T1641 |
| T1641 | Pass | write | FA bootstrap recovery used temporary loader serving committed execution module; restored reviewed append draft |
| T1642 | Pass | read → write → read → write → bash | Reviewed general capability source/data applied through Formal AI; requested bytes verified |
| T1643 | Fail | read → read | Open authored-target scope: quoted full metadata selected planner-precedence.lino; two failed reads, no mutation |
| T1644 | Fail | none (boot failure) | Blocked before tools by CI-owned invalid mutating_action regex; exact retry T1647 after CI repair |
| T1647 | Pass | read → write → read → write → bash | Reviewed general capability source/data applied through Formal AI; requested bytes verified |
| T1648 | Pass | read → write → read → write → bash | Reviewed general capability source/data applied through Formal AI; requested bytes verified |
| T1649 | Pass | read → edit → bash | Exact canonical action edit applied and observed |
| T1650 | Pass | read → write → read → write → bash | Reviewed general capability source/data applied through Formal AI; requested bytes verified |
| T1651 | Pass | read → write → read → write → bash | Reviewed general capability source/data applied through Formal AI; requested bytes verified |
| T1652 | Pass | read → edit → bash | Exact native action edit applied and observed |
| T1653-a | Pass | write (observed) | Initial new closest test created, byte-equal; raw transcript later overwritten by second idempotent read. Exact original prompt reconstructed; complete original transcript unavailable |
| T1653-b | Pass | read | Second identical source request only read existing already-correct test; idempotent state, zero new write |
| T1654-a | Fail | none (quote fault) | Initial guillemet-wrapped full source declined due nested quotes; retry uses variable fence. Exact original prompt reconstructed; complete original transcript unavailable |
| T1654-b | Pass | read → write → read → write → bash | Reviewed general capability source/data applied through Formal AI; requested bytes verified |
| T1655-a | Fail | none (quote fault) | Initial guillemet-wrapped full native source declined due nested quotes; retry uses variable fence. Exact original prompt reconstructed; complete original transcript unavailable |
| T1655-b | Pass | read → write → read → write → bash | Reviewed general capability source/data applied through Formal AI; requested bytes verified |
| T1656 | Pass | read → write → read → write → bash | Reviewed general capability source/data applied through Formal AI; requested bytes verified |
| T1657 | Pass | read → write → read → write → bash | Reviewed general capability source/data applied through Formal AI; requested bytes verified |
| T1658 | Pass | read → write → read → write → bash | Reviewed general capability source/data applied through Formal AI; requested bytes verified |
| T1659 | Pass | read → write → read → write → bash | Reviewed general capability source/data applied through Formal AI; requested bytes verified |
| T1662 | Pass | read → write → bash | Reviewed general capability source/data applied through Formal AI; requested bytes verified |
| T1663 | Pass | read → write → bash | Reviewed general capability source/data applied through Formal AI; requested bytes verified |
| T1664 | Fail | read → bash → bash | Failed authored native fixture Set: misparsed payload as Rename contents to formal_ai; unintended test edit restored by T1668 |
| T1665 | Pass | read → write → read → write → bash | Reviewed general capability source/data applied through Formal AI; requested bytes verified |
| T1666 | Pass | read → write → read → write → bash | Reviewed general capability source/data applied through Formal AI; requested bytes verified |
| T1667 | Fail | bash | Failed explicit shell form: exactly and guillemets entered command; exit127, no file edit |
| T1668 | Pass | bash | Plain Run executed reviewed fixture copier: three real file writes |
| T1669 | Pass | read → write → read → write → bash | Reviewed general capability source/data applied through Formal AI; requested bytes verified |
| T1670 | Pass | read → write → read → write → bash | Reviewed general capability source/data applied through Formal AI; requested bytes verified |
| T1671 | Pass | read → write → read → write → bash | Reviewed general capability source/data applied through Formal AI; requested bytes verified |
| T1672 | Pass | read → write → read → write → bash | Reviewed general capability source/data applied through Formal AI; requested bytes verified |
| T1673 | Pass | read → write → bash | Reviewed general capability source/data applied through Formal AI; requested bytes verified |
| T1674 | Pass | read → write → read → write → bash | Reviewed general capability source/data applied through Formal AI; requested bytes verified |
| T1675 | Pass | read → write → read → write → bash | Reviewed general capability source/data applied through Formal AI; requested bytes verified |
| T1676 | Pass | read → write → read → write → bash | Reviewed general capability source/data applied through Formal AI; requested bytes verified |
| T1677 | Pass | read → write → read → write → bash | Reviewed general capability source/data applied through Formal AI; requested bytes verified |
| T1678 | Fail | bash | Open G112: unchanged T1637 retry read and echoed fixed source; synthesis did not implement code |
| T1679 | Pass | read → write → read → write → bash | Reviewed general capability source/data applied through Formal AI; requested bytes verified |
| T1680 | Pass | bash | Plain Run copied formatted native G15 fixture: one file write |
| T1681 | Pass | read → write → read → write → bash | Reviewed general capability source/data applied through Formal AI; requested bytes verified |
| T1682 | Pass | read → write → read → write → bash | Reviewed general capability source/data applied through Formal AI; requested bytes verified |
| T1683 | Applied; draft defect | bash → write → bash | Draft applied byte-equal; later source review found misplaced native atomic-shell guard; corrected T1689 |
| T1684 | Pass | bash → write → bash | Reviewed general capability source/data applied through Formal AI; requested bytes verified |
| T1685 | Pass | bash | Plain Run copied honest shell harness and sequence contracts: three file writes |
| T1686 | Applied; draft defect | bash | Reviewed mapper/data files copied; initial awk quoting failed closest execution checks; repaired T1687 |
| T1687 | Pass | bash | Plain Run copied corrected awk command template: one file write |
| T1688 | Pass | bash → write → bash | Created closest concurrent-lock and sandbox-mapping regressions |
| T1689 | Pass | bash → write → bash | Reviewed general capability source/data applied through Formal AI; requested bytes verified |
| T1690 | Pass | bash → write → bash | Created append capability changelog |

Through T1690: 53 invocations; 9 failed task outcomes; three additional supplied-draft defects documented separately. 38 non-plan Write/Edit calls and six Bash calls that changed capability source/test files (11 file writes across those six Bash calls). These are 44 physical non-plan mutation calls, excluding plan-event persistence and read/verification calls. Auxiliary plan-event append calls are not counted as requirement edits. G15 behavior is repaired; G112 synthesis remains open. Actual model tokens and costs were not measured. Raw transcript byte lengths are only an operational proxy.

Final checks:18/18 append+authored-content tests;2/2 selected G15 tests; standalone Rust formatter and owned diff checks pass. Current notation measurements retain9420 underscore names and80 repeated metadata fields. Remaining whole-tree hardcoded-language findings at this cutoff belong to CI mutating_action plus the stale migrated native action allowlist row; root owns final projections and allowlist pruning.

T1691 writes these rows and the audit through one actual Bash copier; T1692 copies bounded evidence without source changes. Their logs and final audit deltas are reported separately so this cutoff remains reproducible.

| T1691 | Pass | bash | Wrote ledger payload and audit via actual neutral filesystem copier; two evidence files. |
| T1692 | Pass | bash | Copied retained raw prompts/results and explicit reconstruction/observation records; no capability source edit. |
| T1693 | Pass | bash | Final evidence audit update and last neutral-copy transcript retained; no capability source edit. Full operational output retained in /tmp/spec-T1693.log for root integration. |

Final audit cutoff T1693:56 invocations,9 failed task outcomes,44 capability source/doc/test mutation calls (38 Write/Edit and6 source-mutating Bash),3 additional neutral evidence-copy Bash calls,0 manual repository edits. The three additional copier calls are recorded separately; they do not represent requirements synthesized by Formal AI.

| T1694 | Pass | bash | Updated native driver surface pin to six real capabilities, explicitly read_file; core recipe still pins exactly four executed tools. Two actual closest-test file writes. |
| T1695 | Pass | bash | Updated final audit and preserved late retained transcripts; neutral evidence copier. Operational prompt/result retained in /tmp/spec-T1695-* for root integration. |

Final extended cutoff T1695:58 invocations,9 failed task outcomes,45 capability source/doc/test mutation calls (38 Write/Edit plus7 source-mutating Bash calls;13 actual file writes inside the seven Bash calls),4 neutral evidence-copy Bash calls,0 manual repository edits. No owned hardcoded-language strings remain. The migrated native action stale allowlist row is scripts/hardcoded-language-allowlist.txt:58 and is left for root to prune.
