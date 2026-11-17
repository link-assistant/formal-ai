# Pull request #1583 review record

PR: https://github.com/link-assistant/formal-ai/pull/1583
Issue: https://github.com/link-assistant/formal-ai/issues/1582

The issue's reproductions, product requirements, root causes and hashed evidence
belong in [the issue case study](../issue-1582/README.md). This record tracks the
review conversation and validation decisions for the pull request.

On 2026-11-17 at 14:55Z the prior session explicitly left the PR draft with
Gemini input-context support and native regression execution unfinished. A
15:06Z conversation comment said ready to merge; that statement does not
resolve either acceptance criterion and the actual PR remains draft.

At 17:29:18Z the reviewer requested a complete check of supported tools and
natural/programming languages. All conversation comments, inline review
comments and review decisions were read. The latter two inventories are empty.
The continuation enumerates 12 adapters, 25 declared modes, five natural
languages, 16 code-generation targets and the JavaScript, TypeScript and Rust source roots. Source/config
inspection is distinguished from an installed third-party client execution.

Main at `512ce428029a92687628b4180309ee5ab660fe08` was merged forward in
`d9d3f299b`; both regression inventories and complete governed cases survive.
The review found paraphrases missed permission guidance and quoted task input
could take over its routing. A shared fixture reproduced 12 failures out of 18
checks. Seed aliases, anchored ownership and quote exclusion repair the cause
in all three roots; 35 final JS/TS checks pass. Native regressions are drafted,
not executed, because the repository excludes pre-merge native test builds.

Local verification passes 63 focused JS checks, 34 complete-witness checks and
62 checks for the merged main changes. The full 138-entry gate inventory was
run. Eight of its nine initial failures have passing focused reruns; the last
is a newly reviewed picomatch advisory with no patched release. The advisory
and upstream response are preserved in the issue archive. No audit waiver or
unqualified dependency replacement was used to manufacture a green result.

Current-head hosted runs will be recorded after the complete repair batch is
pushed. The PR stays draft and unmerged while Gemini input context, native
runtime validation or dependency qualification is unfinished. The positive
closing reference required by the current task remains in the PR description;
it does not claim that pending acceptance is complete.
