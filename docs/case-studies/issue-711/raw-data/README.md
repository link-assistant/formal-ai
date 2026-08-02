# Raw evidence index

This directory intentionally preserves machine-readable and verbatim evidence
used by the issue-711 case study.

- `issue-*` and `pr-*`: GitHub issue/PR payloads and all three PR feedback
  streams (conversation comments, inline review comments, and reviews).
- `release-v*.json`: affected GitHub release payloads. v0.285.0 has no GitHub
  release object even though its release commit and tag exist.
- `first-parent-history.tsv`, `release-commits.tsv`, `changelog-history-*`, and
  fragment lists: Git provenance used by the reconstruction.
- `template-*`, `diff-*`, and `rust-template-*`: full four-template inventory,
  comparison, and upstream issue/PR evidence.
- `agent-cli-*`, `formal-ai-serve-*`, and `solve-*` where present: external
  agent and local Formal AI execution evidence.
- `regression-test-before.log` and `regression-test-after.log`: the red/green
  reproduction.
- build, formatting, file-size, reconstruction, and script-test logs: local
  verification evidence.

The raw files are retained instead of summarized away so later investigators
can reproduce counts, timestamps, content comparisons, and decisions.
