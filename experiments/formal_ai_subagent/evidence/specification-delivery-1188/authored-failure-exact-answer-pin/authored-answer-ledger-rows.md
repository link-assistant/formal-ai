# Authored verification exact-answer pin ledger

| Task | Outcome | Observed tools | Evidence and scope |
| --- | --- | --- | --- |
| T2040 | Pass | bash → write → bash | scratch initial previous bytes applied through FA |
| T2041 | Pass | bash → write → bash | exact unchanged native Error diagnostic request; bytes preserved; complete successful final answer observed |
| T2042 | Pass | bash → write → bash | second scratch initial previous bytes applied through FA |
| T2043 | Pass | bash → write → bash | exact unchanged native compact JSON request; bytes preserved; complete successful final answer observed |
| T2044 | Fail | write → bash | full12-backtick authored Rust Set request misrouted and wrote unrelated modulo-by-zero test stub; byte mismatch caught; original incorrect bytes preserved |
| T2045 | Pass | bash | closest JS literal verification test passed1/1;10 other tests explicitly filtered out |
| T2046 | Pass | bash | changed explicit FA Node copier restored complete desired native source; recovery is not original literal retry |
| T2047 | Pass | bash → write → bash | exact unchanged T2044 source request passed byte-equal after original agent general fence-header repairT1871/T1872 |

CutoffT2047:8 actual invocations,1 failed request with an actual incorrect Write effect,4 scratch Writes,2 repository Writes (incorrect original and successful unchanged retry),1 repository source-restoring Bash. Seven actual capability mutations exclude the5 separate plan-history append Bash calls. Packaging invocation observed afterward and reported separately. No manual repository mutation or local native build.
