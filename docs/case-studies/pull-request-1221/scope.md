# PR #1221: one delivery scope

All planned issues and subissues are delivered through [PR #1221](https://github.com/link-assistant/formal-ai/pull/1221). Finished historical release repairs remain part of main history; remaining fixes, requirements, evidence and generated files are combined here. This scope does not assert that every acceptance has passed.

The authoritative umbrella requirements remain [Hive Mind and progressive self-coding](../../requirements/issue-1207-hive-mind-progressive-self-coding.md), [cryptographic provenance](../../requirements/issue-1213-cryptographic-provenance.md), and [structured logging](../../requirements/issue-1220-structured-links-notation-logging.md). Their original 27 full acceptance rows are preserved. Source implementation and focused tests are separate from hosted, released and real-client acceptance. GitHub issue state is not proof of acceptance.

## Included issue tree

| Issue | Required behavior | Umbrella mapping |
| --- | --- | --- |
| [#1203](https://github.com/link-assistant/formal-ai/issues/1203) | Release consistency and actual GitHub assets | R1207-1 |
| [#1204](https://github.com/link-assistant/formal-ai/issues/1204) | Canonical Docker indices and usable runtime | R1207-1 |
| [#1205](https://github.com/link-assistant/formal-ai/issues/1205) | Recover from tool errors without interactive dead ends | R1207-2 |
| [#1206](https://github.com/link-assistant/formal-ai/issues/1206) | Use fetched issue requirements instead of echoing WebFetch | R1207-2 |
| [#1189](https://github.com/link-assistant/formal-ai/issues/1189) | Prevent abbreviation/example text becoming paths | R1207-2 |
| [#1153](https://github.com/link-assistant/formal-ai/issues/1153) | Slim runtime distribution; preserve the completed foundation | R1207-1 |
| [#1201](https://github.com/link-assistant/formal-ai/issues/1201) | Release stage budget; preserve the completed foundation | R1207-1 |
| [#1208](https://github.com/link-assistant/formal-ai/issues/1208) | Keep derivation and source caches out of caller repositories | R1207-2/-7 |
| [#1209](https://github.com/link-assistant/formal-ai/issues/1209) | Execute the actual fetched issue, with honest terminal state | R1207-2 |
| [#1210](https://github.com/link-assistant/formal-ai/issues/1210) | Human-opened issue attempts through Hive Mind with complete logs | R1207-2 |
| [#1211](https://github.com/link-assistant/formal-ai/issues/1211) | Task-owned paths and behavioral verification; no garbage writes | R1207-2 |
| [#1214](https://github.com/link-assistant/formal-ai/issues/1214) | Causal executor edits, commits and anchored trailers | R1213-4 |
| [#1215](https://github.com/link-assistant/formal-ai/issues/1215) | Per-language source Merkle roots and inclusion paths | R1213-5 |
| [#1216](https://github.com/link-assistant/formal-ai/issues/1216) | Initial memory, per-message custody and real replay | R1213-2/-7 |
| [#1217](https://github.com/link-assistant/formal-ai/issues/1217) | Actual thinking and derivation persistence | R1213-3/-8 |
| [#1218](https://github.com/link-assistant/formal-ai/issues/1218) | CPU/GPU digest parity and hosted performance evidence | R1213-9 |
| [#1219](https://github.com/link-assistant/formal-ai/issues/1219) | Signed installation identity and repository/PR verifier | R1213-6/-7/-8/-10 |

The linked client regressions [#1154](https://github.com/link-assistant/formal-ai/issues/1154), [#1156](https://github.com/link-assistant/formal-ai/issues/1156), [#1158](https://github.com/link-assistant/formal-ai/issues/1158), [#1160](https://github.com/link-assistant/formal-ai/issues/1160), [#1212](https://github.com/link-assistant/formal-ai/issues/1212) are also in this scope, including the exact motto and evidence-driven candidate ordering in #1212. Existing #1230/#1232 release repairs are preserved; the newly observed Node 18 dependency incompatibility is repaired here.

Cross-repository acceptance remains tracked here: [Hive Mind #3333](https://github.com/link-assistant/hive-mind/issues/3333) requires real server/dialog log attachment for every client, and [Hive Mind #3334](https://github.com/link-assistant/hive-mind/issues/3334) requires attached provenance and verification before authorship claims. They cannot be marked delivered from a Formal AI fixture or an unexecuted external integration.

## Bulk work arrangement

- Keep one coordinator applying finished work and monitoring release authority. Keep a dedicated CI agent on the exact pushed head; it retrieves bounded failure logs and drafts general repairs immediately.
- Keep at least one delivery agent whose source changes are executed only through the maintained JavaScript Formal AI runtime. Other agents should use it for encountered tasks too. Preserve failed attempts and exact preimages. A supplied implementation/controller earns zero autonomous-authorship or token-savings credit.
- Draft the whole finite requirement batch, qualify independent sparse candidates, resolve overlaps with both proposals retained, and commit completed changes. Push one coherent batch when the delivery agents finish and every identified CI repair is committed. Then observe fresh exact-head CI; earlier-head success is historical.
- Keep small-task capability and cost measurements honest. Coding expansion, small corrections/deletions, yes/no answers and calculations have different useful input/output relations. Never inflate output to improve a ratio, omit failed attempts, or count generated/vendor/duplicate bytes as useful authored code. The 60–80% aspiration remains configurable/report-only while correctness and evidence checks stay enforced.
- Use CI for Rust, native, WASM, Docker and product builds. Keep local work to bounded JavaScript/source checks and small evidence. Source snapshots preflight file/byte budgets and hash one file at a time; oversized input is an explicit refusal, never a silently incomplete root. Preserve private memory/messages and upload only explicitly reviewed nonpersonal evidence.

## Source checkpoint

The first resumed batch integrates 104 qualified source files: Node 22 Docker compatibility; issue ownership and source-cache isolation; the pinned Hive Mind issue workflow; exact motto surfaces; native/JavaScript typed logging and legacy dialog readers; semantic terminal events; bounded source snapshots; and actual JavaScript driver event/edit capture. Each component retains its original failed attempts and focused checks. This is source work, not complete provenance, unassisted self-coding or a production guarantee.

Remaining full-acceptance obligations include fresh native compilation/formatting/Clippy, genuine final-source census and parity, real all-three-client issue-to-green-PR runs, bounded trusted discovery with actual cold-network evidence, measured small-task capability/cost, full engine memory replay, installation signer custody, complete commit/PR attribution, GPU/performance evidence, shared tracing/dedup integration and external log/provenance attachments. The per-issue requirement shards retain these details.

The v0.353.8 crate was published and its registry-byte audit/clean-install smoke passed. Pipeline [38915206976](https://github.com/link-assistant/formal-ai/actions/runs/38915206976) failed at image job 117065327199 because a dependency used the /v regular-expression flag under Node 18.19.1. Cache export was never reached. The source fix supplies the official target-native Node 22.23.3 binary and validates that capability before existing CLI checks. A fresh successful merged-source pipeline and actual configured release outputs are required before publication is accepted. Never replay the failed old-source protocol.
