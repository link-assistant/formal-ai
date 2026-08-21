---
bump: patch
---

### Fixed

- Ask the kernel, not the filesystem, whether an agent's timeout terminated a descendant process. `timeout_terminates_descendant_processes` inferred termination from the absence of a file its descendant writes after a delay, which is also what an alive-but-sleeping descendant looks like, so a loaded macOS runner failed it (run 32272689475, job 96137354605) on a branch that touches neither `run_agent` nor its fixture. The fixture now records the descendant's pid, the test polls its process state, and the three ways this can go wrong -- never spawned, still running, terminated only after outliving its own delay -- are reported as three different failures instead of one ambiguous one.
