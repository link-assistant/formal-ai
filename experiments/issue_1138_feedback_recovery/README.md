# Feedback recovery for issue #1138 / PR #1139

Recover genuine user directives from Claude Code session transcripts (.jsonl)
so every stated requirement can be audited against the repository requirement
shards, and new process notes recorded before they are lost to compaction.

Usage:
  python3 collect_user_feedback.py <session.jsonl> [...] > feedback.md

The bundled recovered-2026-09-23.md snapshot was produced from the PR #1139
driving sessions (2026-09-14 through 2026-09-23); 14 genuine directives
survive filtering (harness wrappers, continuation summaries, and teammate
agent reports are excluded; recurring cron checkpoints collapse to one entry).
Re-run the collector on any future session to re-audit requirements coverage.
