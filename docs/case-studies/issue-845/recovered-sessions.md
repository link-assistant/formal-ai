# Killed-session recovery record

Two maintainer comments required recovery from:

- [the original 14.5 MB session](https://gist.githubusercontent.com/konard/475718ddace837f5a29ff4f579fbc09d/raw/e78372c37122e87a8ac03c25bd3a839606282a91/tmp-start-command-logs-isolation-docker-702c9ee1-6ad7-4791-b9b2-ac26742f1e03.log.txt);
- [the failed 160 KB resume](https://gist.githubusercontent.com/konard/74c64ad9f7d21397e7d18ca48f0aeb50/raw/7922953d476063828554962edfbf3484af8b9452/tmp-solution-draft-log-pr-1784988775750.txt.log.txt).

The first session had created two local commits:

```text
eae7e61b Implement context-relative fact checking
6703fa7d Document issue 845 verification evidence
```

Neither commit was pushed before the source container was killed. The source
commit was recovered patch-by-patch and committed on the prepared branch as
`e37eb0bb`. Its recovered contents include the formal-system type, fact-checker,
world-model permission integration, generated self-AST data, unit/integration
tests, and changelog fragment.

The evidence commit's object was no longer available in any repository or
snapshot store. Its manifest was nevertheless recovered from the transcript:
31 files, 1,674 insertions, including raw issue/PR data, three Agent CLI
attempts, dialogue/server logs, the red compiler reproduction, focused
unit/integration logs, formatting, Clippy, file-size, hardcoded-language, and
self-AST logs. The transcript also preserves their byte counts and command
outputs.

This case study reconstructs the requirements, root causes, Agent CLI failure
account, and test index. Large raw Agent CLI streams are linked to the immutable
recovery transcript instead of being presented as newly captured output. New
red/green runtime and browser regressions were then captured in the continued
session.

The second Gist contains no additional successful edit: it records the failed
attempt to resume after the first container was gone.
