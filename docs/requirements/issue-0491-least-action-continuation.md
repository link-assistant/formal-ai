## Issue #491 Least Action Continuation

The issue updated on 2026-09-15 extends the earlier R491-1 audit with explicit
resource and outcome-quality dimensions. Optimize only among solutions that
still satisfy the task; incomplete work is not a cheaper solution.

| ID | Requirement | Status / evidence |
| --- | --- | --- |
| R491-C1 | Recursively decompose into two children where useful, preserve already-atomic tasks, and measure elementary work rather than padding the structure. | Partial: task-decomposition and failure-driven recursive-execution tests cover binary structure and atomic leaves. Complete decomposition of arbitrary natural-language obligations remains open. |
| R491-C2 | Shorter reasoning or code must retain the full required behavior and input range. | Partial: verified-candidate selection, conjunctive instruction checks and bound recipe evidence reject several incomplete outcomes. Finite benchmark cases do not prove arbitrary input-range equivalence or whole-task completion. |
| R491-C3 | Evaluate elapsed time, computational work and memory alongside path/code size; choose lower-cost solutions only after correctness. | Partial: existing candidate selection uses size/step costs. A shared measured-resource optimizer across all reasoning and execution paths remains open. |
| R491-C4 | Include user satisfaction and requirement completeness when comparing candidate solutions and learning general procedures. | Open as a universal capability: read-only refactor and missing-runtime probes show that completing a supported intermediate step can leave the requested outcome unimplemented. Plan 07 retains these counterexamples. |
