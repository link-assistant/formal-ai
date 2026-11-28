# Clean local verification of native admission manifest repairs

The complete local inventory executed on clean source bdbac26520862f995f39913b2ce42a91239344bc from 04:35:37 through 04:44:19 UTC on 2026-11-23. All 143 entries have observations: 89 passed, zero failed and 54 explicitly require hosted execution. Source identity and a clean worktree were preserved throughout. No native compilation or execution took place locally.

The [evidence manifest](../../../experiments/issue-1580/evidence/checks/native-admission-clean-gates-20261123/manifest.json) retains the runner stream, all 89 executed gate streams, their command/outcome metadata, and verified archive member hashes. The preceding foreground command was killed at the Formal AI tool execution deadline. Its partial inventory and original error remain archived and receive no complete verification credit. The completed run used a recorded background worker.

The same archive preserves the fully reviewed repair diff and a bounded aggregate-review inventory. Expanding the rename-map extraction restores the exact base bytes. Concatenating both self-hosting ledger archive parts, with the duplicate root removed, also restores the exact base bytes. These mechanical observations establish conservation of those moves; they do not establish complete literal review of the aggregate PR.

The current hosted 8682 source is still being collected. Its two admission assertions have local repairs and reproducing controls, but actual native tests require the next hosted head. Full and consumer-only artifact reruns, remaining workflow and template practices, production delivery and other original acceptance criteria remain unfinished. The PR stays draft and unmerged. Supplied controllers receive zero independent-authorship credit.

PR: https://github.com/link-assistant/formal-ai/pull/1581.
