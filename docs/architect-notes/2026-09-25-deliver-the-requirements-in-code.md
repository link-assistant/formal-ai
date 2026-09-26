# Deliver the requirements in code, not memories

Source: the architect's 2026-09-25 instructions in
[PR #1139](https://github.com/link-assistant/formal-ai/pull/1139) (work for
[issue #1138](https://github.com/link-assistant/formal-ai/issues/1138)). He
asked that every instruction he has given across the Claude Code sessions be
compiled verbatim into a single auditable file, and that the file be used to
deliver the requirements themselves, as code:

> Please don't update memories too much, only if wrong, even better to delete
> most of them. I need changes in code, and all requirements I asked to be
> delivered, not memories that will never go to code.

> Find our conversation in claude code sessions folder, and make sure you
> compile all my messages into single file, and based on it deliver exactly all
> requirements i asked to deliver.

> That may also go to architect notes, if not already there.

The compiled file is
[`experiments/issue_1138_feedback_recovery/recovered-2026-09-25.md`](../../experiments/issue_1138_feedback_recovery/recovered-2026-09-25.md)
(67 chronological entries, 2026-09-09 through 2026-09-25), regenerable with
[`collect_user_feedback.py`](../../experiments/issue_1138_feedback_recovery/collect_user_feedback.py)
over the session transcripts; the collector also recovers messages that were
typed mid-turn and recorded as queue operations.
