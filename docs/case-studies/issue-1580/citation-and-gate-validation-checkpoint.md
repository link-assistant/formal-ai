# Citation repair and complete local gate observations

On local source through 1fb210cd, the complete 141-entry JavaScript gate
inventory reports 87 passes, 53 explicit skips and one failure. The failure is
check-twin-citations: five JavaScript comments still cite the capability router
parent after its functions were extracted into tool_classification.rs.

Extend the maintained headroom regression to resolve each of those five real
citations against the actual extracted Rust source. The regression fails before
the comment correction and passes afterward, with all 16 headroom cases passing.
The focused citation gate then resolves all 1,318 citations and 15 prose paths.
The other 140 gate results are retained separately from that corrected replay;
the original complete attempt remains recorded as a failure. The official
TypeScript writer and its currency check pass for all 903 module projections;
the comments require no generated TypeScript delta. Runtime code is unchanged.

The complete maintained observation suite passes 186 cases. The concurrency,
runner-recovery, native cache-binding and deployed release-projection checks
also pass, and changed Rust files pass formatting. These results do not qualify
native compilation or execution of the unpublished changes.

The first component commit attempt failed the unchanged abbreviation budget
because two archived filenames used an abbreviated suffix. The resumed attempt
renames those archive files to full words and preserves their original paths,
contents and hashes. The threshold remains 90. Both failed and resumed command
streams are retained here; no rejected attempt is credited as passing.

Published fbac28e5 still has an active Pipeline run. The completed Coverage and
Layered runs fail their large Git inventory, and the Pipeline's completed native
workflow-contract job exposes the helper-coverage observer mismatch. Local
commits repair those causes; hosted qualification requires the next published
source. No next push is permitted while the current Pipeline is active.

Evidence: experiments/issue-1580/evidence/checks/citation-and-gate-validation/.
Archives preserve complete streams with SHA256, bytes and original line counts.
Supplied implementation and reasoning earn zero independent credit. All eight
template practices, original live/budget criteria and full issue acceptance
remain unfinished. PR stays draft and unmerged.

PR: https://github.com/link-assistant/formal-ai/pull/1581
