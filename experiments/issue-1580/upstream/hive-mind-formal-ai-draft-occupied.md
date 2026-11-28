## Summary

`scripts/formal-ai-draft.mjs` starts a Formal AI draft attempt even when another session already has an open `issue-N-*` PR for the issue. Its post-attempt policy then claims that PR as the draft:

- it labels the PR `formal-ai-draft`;
- it runs `gh pr ready --undo` on it whenever the PR is not a draft.

## Observed (source `5bf337c3eee664cecb26f75104fa58ce386fad7b`)

link-assistant/formal-ai run [43813238887](https://github.com/link-assistant/formal-ai/actions/runs/43813238887):

1. `decideDraft` returned `run` ("link-assistant/formal-ai#1580 was just opened").
2. The image pull and build took about 14 minutes.
3. During that time a separate `solve` session opened link-assistant/formal-ai#1581 for the same issue.
4. The attempt continued link-assistant/formal-ai#1581 (see https://github.com/link-assistant/hive-mind/issues/4071). The policy at [`scripts/formal-ai-draft.mjs` L179-L192](https://github.com/link-assistant/hive-mind/blob/5bf337c3eee664cecb26f75104fa58ce386fad7b/scripts/formal-ai-draft.mjs#L179-L192) then picked it through `selectDraftPullRequest`, the newest open `issue-N-*` PR whoever opened it.
5. It labelled link-assistant/formal-ai#1581 `formal-ai-draft`. link-assistant/formal-ai#1581 is not a Formal AI draft.

Had link-assistant/formal-ai#1581 been ready for review at that moment, `keepPullRequestAsDraftArgs` would have turned it back into a draft.

## Suggested fix

- **Before the attempt** (after the image pull, as close to `docker run` as possible): if `gh pr list --state open` shows an `issue-N-*` head, deny with a new code such as `DRAFT_DECISION_CODES.occupied = 'issue-has-open-pull-request'` and exit 0.
- **In the failure policy:** label and un-ready only a PR created after the attempt started and not present in the pre-attempt census. formal-ai's `selectAttemptPullRequest` (createdAt ≥ start, unique match) is one way to do it.

## Workaround in formal-ai

link-assistant/formal-ai#1581 adds an `occupancy` step (`scripts/hive-mind-draft-result.mjs --occupied`) that skips the attempt, and the issue comment, when such a PR exists.
