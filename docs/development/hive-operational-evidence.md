# Hosted operational evidence

Pinned Hive 8f22b87c515adece0427cb59e429531f35d7760a requires repository-relative same-commit-tree evidence for its legacy attribution metric. Its supported log-dir option changes console log storage, while development-log explicitly stages and commits development artifacts. Attribution none would discard attribution collection. No supported flag can preserve both that same-tree metric and a task-only pull-request file set.

The reviewed hosted adapter retains the original development-log writer, publication sanitizers, complete session stream and model-identity rejection. It redirects destination writes to mounted /home/box/logs/formal-ai-operational and omits only the staging/trailer hooks and development-log Git publication. Existing task hooks remain active. Original tasks and acceptance assertions are unchanged. Workflow artifacts retain the mounted directory.

External manifests bind the pinned Hive source blobs, task/session identities, observed commit/tree/parent identities, and streamed file hashes and lengths. They do not prove causal edit coverage, signing, or same-tree attribution: that legacy result is explicitly Unknown and receives zero credit. No external evidence is automatically committed to the task repository.

After the task container exits, the hosted workflow transfers ownership of only its exact operational evidence subtree to the artifact runner. It preflights all entries, refuses symlinks and hard links, preserves private file/directory modes, and leaves all sibling logs and task files untouched. It does not make evidence world-readable. The local control uses the current owner; cross-user transfer is exercised only by hosted CI.
