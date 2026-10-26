# Released client repeat and Hive's no-change exit

[PR 1418](https://github.com/link-assistant/formal-ai/pull/1418) merged and
[v0.355.12](https://github.com/link-assistant/formal-ai/releases/tag/v0.355.12)
passed main CI, Docker publication, native and desktop packaging, public-byte
verification, both signed Mac package/renderer checks, and the browser Worker
check. These publication results do not establish coding acceptance.

In [the fresh three-client run](https://github.com/link-assistant/formal-ai/actions/runs/40377859689),
Claude, Agent and Codex each passed the original first-run oracle. Their ready
pull requests contained the requested program, verifier and workflow. The
programs printed the required line and the original task workflows passed.
Each repeat then failed only the original `solve completed` assertion: Hive
returned exit 5 after reporting that it made no new commit, file change or
comment. The unchanged source and ready PR were still valid. The original
failed observations remain failures and are never rewritten or rerun.

The pinned Hive revision documents exit 5 as a finished session with no changes,
and provides the supported
[`HIVE_MIND_SOLVE_NO_CHANGES_EXIT_CODE` compatibility setting](https://github.com/link-assistant/hive-mind/blob/8f22b87c515adece0427cb59e429531f35d7760a/src/solve.exit-codes.lib.mjs).
Use that setting at the invocation boundary:

- The first delivery explicitly uses 5, including when its ambient environment
  contains an override. An empty first delivery cannot become successful.
- Only the repeat after a qualified first pass uses 0. Forward this variable
  through the original Docker argument builder to the actual Hive process.
- Require the original harness result to exit zero and every original assertion
  to pass. Do not normalize a failed receipt or accept arbitrary nonzero exits.
- Still independently require the same checkout, branch, commit, clean tree,
  file membership and file identities, and no source-mutation tool calls.

Verification-only work can usefully leave no code diff. Do not manufacture a
commit, file rewrite or comment to satisfy a change counter or an output-size
target. Hive retains its no-change warning; only this supported repeat-process
exit policy changes. Its original oracle files and task definitions stay intact.

The captured regression retains all three clients' successful first-run and
failed-repeat assertion rows, artifact/result hashes, and the pinned upstream
exit function and Docker constructor. It exercises that original code and the
adapted environment boundary without GitHub, Docker, a compiler or a product
binary. Failed assertions, first-pass no-ops and changed repeat state still
refuse. Fresh hosted three-client delivery and trusted cleanup remain required
before this fix receives released coding-acceptance credit.

## Cleanup after a released-client owner

The same run also exposed a missing cleanup trigger. The authenticated lineage
was release 40343663381 (push), desktop 40369287716, downloads 40376949703, then
original delivery 40377859689 (three consecutive `workflow_run` levels). No
cleanup run or outcome appeared after that owner completed. GitHub documents
that a fourth `workflow_run` level does not execute:
[workflow chaining limit](https://docs.github.com/en/actions/reference/workflows-and-actions/events-that-trigger-workflows#workflow_run).
Scheduled discovery remains a fallback, not a timely-cleanup guarantee.

The final actual-owner job now requests cleanup through `workflow_dispatch`
using only the default token's `actions: write`. It sends the exact owner run,
attempt and protocol commit to trusted main. The controller waits at most 13
read-only observations, ten seconds apart, for that owner to become terminal;
then it authenticates only that immutable owner. It uses the existing source
custody, pre-effect claim, queued recheck and protected-reference rules. No
uncertain dispatch mutation is retried, and no previously claimed cleanup is
replayed. Dispatch itself grants no cleanup or delivery credit. Read-only
`checks` dispatches and PR audits retain their separate nonmutating path.
