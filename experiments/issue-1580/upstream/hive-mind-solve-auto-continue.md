## Summary

`solve <issue-url>` with auto-continue logs that it **skips** an open PR because another session is still working on it, and then continues **that same PR** through the existing-branch fallback. Two sessions end up writing to one PR.

## Observed (hive-mind `v2.41.0`, source `5bf337c3eee664cecb26f75104fa58ce386fad7b`)

In link-assistant/formal-ai run [43813238887](https://github.com/link-assistant/formal-ai/actions/runs/43813238887) (job `draft`), PR link-assistant/formal-ai#1581 had been opened 17 minutes earlier by a separate `solve` session for issue link-assistant/formal-ai#1580, which was still running.

```text
📋 Found 1 existing branch(es) in main repo matching pattern 'issue-1580-*':
  • issue-1580-8d61a32a61e1
  PR #1581: created 0h ago (OPEN, draft)
  PR #1581: CLAUDE.md/.gitkeep exists, age 0h < 24h - skipping
⏭️  No suitable PRs found (missing CLAUDE.md/.gitkeep or older than 24h) - creating new PR as usual
✅ Using existing branch from main repo: issue-1580-8d61a32a61e1
   Existing open PR found: #1581
...
🔄 Continue mode:            ACTIVE
   Using existing PR:      #1581
```

The second session then:

- posted "AI Work Session Started" and "Finished without changes" comments on link-assistant/formal-ai#1581;
- pushed three `Add development log for issue link-assistant/formal-ai#1580 PR link-assistant/formal-ai#1581` commits to the first session's branch;
- left the first session's local branch diverged from its remote.

## Root cause

In [`src/solve.auto-continue.lib.mjs`](https://github.com/link-assistant/hive-mind/blob/5bf337c3eee664cecb26f75104fa58ce386fad7b/src/solve.auto-continue.lib.mjs#L700-L756):

1. Lines 700–705 skip PR link-assistant/formal-ai#1581 as an active session (`.gitkeep` present, younger than 24 h).
2. The fallback at lines 723–756 then picks the newest `issue-N-*` branch. That is the branch of the PR that was just skipped. It finds that branch's open PR and returns `isContinueMode: true` for it.

The fallback never learns which branches the PR loop rejected.

## Reproduction

1. Start `solve https://github.com/<owner>/<repo>/issues/<N>` and let it open its placeholder PR (`.gitkeep` / `CLAUDE.md` commit).
2. While it runs, start a second `solve` for the same issue URL with auto-continue enabled (the default).
3. The second session logs "...< 24h - skipping", then "Using existing branch ... Existing open PR found: #<first PR>", and works in the first session's PR.

## Suggested fix

Collect the head branches of the PRs that the loop skips as active (the `< 24h - skipping` case). Then either:

- remove them from `existingBranches` before the fallback; or
- in the fallback, return `{ isContinueMode: false }` when the selected branch's open PR is one of the skipped ones.

Either way, the "creating new PR as usual" message becomes true. A unit test can drive `processAutoContinueForIssue` with a mocked `$` that returns one young placeholder PR and its branch, and assert `isContinueMode === false`.

## Workaround in formal-ai

link-assistant/formal-ai#1581 adds a step before its draft attempt that leaves the issue alone when an open `issue-N-*` PR already exists. This avoids the collision only for that workflow; other callers of `solve` remain exposed.
