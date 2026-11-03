# Source-client creation failures — 2026-11-03

These are distinct stage0 diagnostics against the source JavaScript server through installed Codex and Agent clients. They do not replace the unchanged read/report/write compound benchmark. The literal payload is supplied externally and earns zero independent synthesis credit.

Both clients received the same task: create `candidate.json` with exactly a supplied 188-byte declarative JSON payload, including its final newline. The payload describes a false declared scalar, its contradicted claim and an unconditional following create. No historical implementation patch was supplied.

| Observation | Codex | Agent |
| --- | --- | --- |
| Local API transport | HTTP 200 | Five HTTP 200 responses |
| Client process | Exit zero | Exit zero |
| Observed workspace calls | None | One Write and three Reads |
| Physical result | File absent | File present with instruction framing around the intended bytes |
| Exact task accepted | No | No |

Codex returned `Created candidate.json through the workspace file tools without read-back verification.` Its source route used the unverified-write wording when no Write capability was selected, even though no effect had occurred. The client exposes a scoped patch tool; correct support requires its actual contract and evidence, not simply classifying every Edit as Write or changing the wording.

Agent wrote text beginning `this content: ` and included the enclosing quotation and sentence framing. Its process success and subsequent reads do not make those wrong bytes correct. Both failures require small offline regressions with the original task plus independent variants.

The private diagnostic observer records calls and physical bytes separately. It does not retain a matched successful Write receipt, so it cannot award qualified effect acceptance from calls alone. The original compound task remains unsolved; no overall coding or reasoning success percentage follows from these trials.

Both diagnostics used the JavaScript-only host policy, bounded process deadlines and 384 MiB Node heaps; child cleanup was confirmed. No local Rust compilation, native or product WASM execution occurred. Main production qualification of merged PR #1518 is a separate workstream.

## Focused repairs and unchanged replay

The shared grammar lacked the complete `containing exactly this content` marker. Adding that and its `containing this content` counterpart makes all five focused JavaScript groups pass, preserving exact payloads and existing ambiguity guards. An unchanged actual Agent replay now writes the exact original 188 bytes, including the final newline. This proves physical payload correctness; the observer still lacks the full matched receipts required for qualified completion.

Creation handling now distinguishes direct Write from a scoped canonical AddFile operation and requires actual qualified preflight and mutation/readback evidence. Sixteen focused JavaScript groups pass, including failed, pending, duplicate, foreign and stale evidence, existing-target protection, unsupported encodings and Unicode line separators. The unchanged Codex zero-effect task has an honest gap when its safe preflight capability is unavailable. No real Codex completion is claimed. Rust counterparts and controls are authored for CI only.

Main after PR #1518 also exposed a stale generated progressive plan in both its first lint lane and JavaScript shard 6. The shared generator owns that repair; the small existing regression is added to the fast PR inventory so this failure is caught before another merge. These are generated requirements projections, not evidence of newly delivered runtime capability.

Two native contract failures exposed the source-generation mode addition: the default-token dispatcher still admitted only an exact singleton checks list, while a layered-tier test compared against an unwrapped historical condition. Repair the production checks admission and its independent recording-client test, then assert the exact isolated-mode exclusion in the native contracts. The dispatcher continues to select only checks; an extra declared mode never authorizes automatic source generation or publication.

The first PR #1519 run completed all 168 selected files in 310 seconds and reported eleven assertions in three files, not a suite timeout. Two causes were repaired without changing those assertions: failed Write receipts must retain their original bytes for exact matching, and known disjoint file creations from the same standing request may preserve independent artifact proofs. Current request declarations are scanner-private and parsed lazily through existing owned clause contracts. Foreign, ambiguous, pending and same-target effects remain barriers; generic source-read freshness is not relaxed. The unchanged transaction file and both original failure cases pass locally.
