---
bump: patch
---

### Fixed

- Bring the issue #1028 Agent CLI ladder workflow back under the two policies it
  broke when it landed: it now belongs to a concurrency group, so a superseded
  push releases its runner instead of running the ladder twice (#1017), and it
  no longer caches the `target` tree (#534). Both were pinned by tests that
  failed on the commit that introduced the workflow, but the follow-up push
  touched only a shell script, so path filtering skipped the lane that would
  have caught them.
- Commit `experiments/issue_1028_agent_cli_ladder/run.sh` executable. It shipped
  as mode `100644` while every other script a workflow invokes bare is `100755`,
  so a checkout handed CI a non-executable file and the ladder's only step died
  with `Permission denied` on every run the workflow ever had. A new sweep over
  every workflow pins the executable bit for all thirty such scripts.
