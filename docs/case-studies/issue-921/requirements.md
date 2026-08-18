# Issue 921 Requirements

| ID | Requirement | Verification |
| --- | --- | --- |
| R921-1 | Prove Hive Mind accepts `--tool agent --model formal-ai` and its production executor drives the actual Agent CLI through Formal AI. | `run.sh` uses public `solve` command preparation under a prepare-only permission shim, imports shipped `executeAgentCommand` without that shim, byte-checks `hive-mind-to-formal-ai/result.txt`, and records the resulting Agent session. |
| R921-2 | Prove Formal AI dispatches an external Agent CLI on a hive-mind-shaped issue task. | The committed `task.md` is reduced to its explicit acceptance payload and passed to public `formal-ai agent run`; `result.txt` and `workspace-effect.patch` prove the effect. |
| R921-3 | Make both effects reviewable and the reverse session replayable. | Both fixture effects are real Git commits; `orchestration-session.json` is canonical JSON with a validated hash chain and recorded `workspace_effect` event. |
| R921-4 | Propagate failures honestly in both directions. | Injected Agent exit 23 returns 23 from the Hive Mind executor with no commit, while Formal AI returns nonzero and saves `status=failed` with `exit_code=23`. |
| R921-5 | Preserve the gate, traceability, and CI diagnostics. | The issue/PR/upstream snapshots, focused regression, version record, release workflow step, and failure-only raw-log upload are committed. |
