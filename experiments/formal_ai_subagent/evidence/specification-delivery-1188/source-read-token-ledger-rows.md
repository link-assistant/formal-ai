Finalized current-batch rows through T1379. Five physical non-plan write/edit tool calls in T1369–T1377 (five source/test calls; T1377 reads only), plus the T1378 and T1379 documentation writes: seven physical calls through T1379. The ledger artifact written at T1380 is an additional documentation write, reported separately after completion. No source or requirement implementation was applied manually.

Validation: two focused JavaScript tests pass; native formatting, planner twins and the hardcoded-language gate pass. Exact T1370 retry T1377 does not implement the reader; G112 remains open.

| # | Task (prompt summary; full tools/results in transcript) | Before | After |
| --- | --- | --- | --- |
| T1369 | Copy only the specification reader and closest test into the sandbox through an explicit Node command | Pass: exactly the two selected files copied. | No repository source changes. |
| T1370 | Ask Formal AI to implement assertion-preserving multi-turn and fixture specification translation | Fail: ran a top-level listing and returned no files; both targets unchanged. | Standalone ls-token bug repaired under G123; broad synthesis remains open under G112. |
| T1371 | Replace the JS ls substring check with a standalone token check | Pass: expected source bytes. | General request classification repaired. |
| T1372 | Mirror the standalone ls-token check in the native planner | Pass: expected source bytes. | Same behavior in the Rust twin. |
| T1373 | Create plural-noun and real ls-command planner regressions | Pass: exact authored source. | Correction of result-kind metadata follows at T1374, before execution. |
| T1374 | Correct regression result-kind metadata to the declared tool_calls variant | Pass: expected source bytes. | Both semantic routing assertions remain; two JS tests pass. |
| T1375 | Append the equivalent native source-read and ls-command regressions | Pass: exact authored source. | Native execution remains delegated to CI. |
| T1376 | Run rustfmt on the native read planner and closest regression | Pass: formatter executed through Formal AI. | Standalone formatting check passes. |
| T1377 | Retry the exact original T1370 implementation request | Fail as a requirement: reads the named source and returns its contents; implements nothing. | G123 routing succeeds; G112 synthesis remains open; both sandbox targets unchanged. |
| T1378 | Create finalized T1300–T1368 ledger rows | Pass: exact authored Markdown. | 61 invoked IDs, 44 physical non-plan write/edit calls, zero manual edits. |
| T1379 | Create a changelog for the standalone shell-token boundary repair | Pass: exact authored Markdown. | Records the repair and honestly retains the synthesis gap. |
