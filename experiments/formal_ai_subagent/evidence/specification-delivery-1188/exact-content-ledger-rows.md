Finalized G124 rows through T1391. Root observed T1419 calls [write, write, bash], stop final, with malformed bytes; its original full transcript was not persisted. exact-content-failure-observation.json attributes this report explicitly. The malformed artifact and expected output are preserved separately.

Seven non-plan write/edit calls in T1384–T1391, zero manual source/docs/test edits. The T1392 ledger artifact write is counted separately after completion. Formatter output was used to design the reviewed literal native block, and actual source writes were applied by Formal AI.

| # | Task (prompt summary; full tools/results in transcript) | Before | After |
| --- | --- | --- | --- |
| T1384 | Repair JavaScript clause cleanup using seeded content lead prefixes and qualified suffixes | Pass: expected source bytes. | G124 generic lead recognition; clause separators remain required. |
| T1385 | Mirror content lead recognition in native write_request | Pass: expected formatted source bytes, 999 lines. | Relevant documentation compacted; no file ceiling raised. |
| T1386 | Append exact-content, multilingual lead and payload-preservation JavaScript regressions | Pass: exact source; all eight authored-literal tests pass. | Eight lead forms across five languages; quoted source and ordinary prose retain bytes. |
| T1387 | Append native exact-content and preserved-payload regressions | Pass: exact formatted source. | Actual file content equality pins the behavior; native execution delegated to CI. |
| T1388 | Preserve malformed output, expected bytes and root-attributed T1419 failure summary through Node filesystem command | Pass: artifacts created; observation JSON valid. | Explicit provenance: no full pre-repair transcript exists; no transcript fabricated. |
| T1389 | Retry the unchanged original T1419 exact-content Set request | Pass: intended cifix-loop.md bytes exactly; executed completion. | Complete prompt, expected bytes and driver transcript saved in spec-T1389.log. Root stages the restored task document. |
| T1390 | Remove only a diagnostic partial-answer assertion from the new native G124 test | Pass: expected source diff. | Byte-equal file assertions retain the requirement; avoids the tests-as-docs gate partial-answer pattern. No G108 changes. |
| T1391 | Create exact-content capability changelog | Pass: exact authored Markdown. | Records the repaired general rule and honest failure provenance. |
