---
bump: patch
---

### Fixed

- Survive the transient package-mirror stalls that fail the agentic CLI matrix. Issue #1017 gave the matrix's Xvfb install a 300s budget so a hung mirror would report `failure` instead of a benign-looking `cancelled`; in run 32272689026 that deadline fired for real and turned a green pipeline red, while the sibling GUI legs of the same run installed the same package in 52s. `scripts/apt-install-with-retry.sh` now bounds each *attempt* as well: a stalled attempt is killed while the budget still has room for another, the wrapper refuses to start when its attempts cannot fit the budget above it, and a test checks that arithmetic for every budgeted retry a workflow composes.
