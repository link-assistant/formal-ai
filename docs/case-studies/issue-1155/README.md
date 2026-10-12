# Issue #1155 — An unauthenticated `gh` banner was accepted as the issue body

2026-09-27, Hive Mind Scala run, `--tool agent`, backend 0.352.1.
Full log: https://gist.github.com/konard/8a196a1d179ecb105304232a46e09ede

## What happened

Formal AI planned the work-item read through the Agent CLI's `bash`:

```
gh issue view '<issue-url>' --json title --jq .title && echo && gh issue view '<issue-url>' --json body --jq .body
```

The environment had no `gh` credential (link-assistant/hive-mind#2314), so
the command printed its how-to-authenticate banner:

```
To get started with GitHub CLI, please run:  gh auth login
Alternatively, populate the GH_TOKEN environment variable …
```

and exited 4. The Agent CLI's own record held `"metadata": {"exit": 4}`, but
the tool message it echoed to Formal AI was only the banner text — no status,
no error flag (link-assistant/agent#317). Formal AI took the banner as a
successful read: it stored the text as the issue body, found no artifact in
it, wrote `.formal-ai/general-change-plan.lino`, and answered
`planned_not_executed`. The `webfetch` tool the client had offered was never
tried; the PR stayed draft.

## Root cause

1. For `Run` results `Progress::scan` never inferred failure from text —
   `failure_message` read only an explicit error flag or a parsed exit code,
   and the echo carried neither.
2. `fetched_pages.push((url, text))` accepted any non-empty output of a
   recognized read command; nothing checked that the output looked like an
   issue.
3. With a (bogus) objective in hand, `plan_work_item_read` and its fetch
   fallback were never reached — the fallback chain existed only behind a
   read that never recorded itself as attempted-and-failed.

## The fix

1. **The read reports its own status** (`data/meta/work-item-steps.lino`):
   every planned read ends with `printf '\n__formal_ai_exit=%s\n' $?`, so a
   client that drops the status field cannot make a failed read look
   successful. `exit_sentinel` reads the last sentinel line; non-zero is a
   failed read whatever the echo says.
2. **The shape is validated** (`rust/src/agentic_coding/progress.rs`): the
   `gh` reads must print a title line, a blank separator, then a body; the
   raw-body `curl` read must not answer the API's JSON error blob. A failed
   read is recorded with its reason and never becomes page evidence.
3. **The fallbacks walk in the issue's order**
   (`rust/src/agentic_coding/general_execution.rs`): after the failed `gh`
   view — the client's fetch tool on the issue URL, then
   `curl -fsSL -H 'Accept: application/vnd.github.raw+json'
   https://api.github.com/repos/<owner>/<repo>/<issues|pulls>/<n>`
   (credential-free, exactly the case an unauthenticated `gh` used to end a
   session for), then two `gh api` calls printing title and body, and last
   the prepared pull request's own page, whose title and body restate the
   issue it resolves.
4. **The close is honest**: when every route is exhausted the run answers
   with `read_failed_report` — every read tried, its reason and its first
   answer line — and the only write this mode produces is no longer spent
   on a plan record for an issue the session never read.

## Verification

`rust/tests/unit/issue_1155_read_validation.rs` replays the incident
transcript shape with the Agent tool list: the status-less banner and the
`__formal_ai_exit=4` sentinel both yield the `webfetch` of the issue URL as
the next plan; the REST routes are pinned in order (issue and `pulls/`
endpoints); the prepared PR is the last fallback; an exhausted retrieval
closes with the full read report and no plan-record write; a successful
sentinel-carrying read drives execution exactly once and keeps the sentinel
line out of the issue text.
