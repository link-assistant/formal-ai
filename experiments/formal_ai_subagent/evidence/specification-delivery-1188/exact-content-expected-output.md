TASK (tag CIFIX): continuously monitor CI/CD for PR #1188, draft failures immediately and repair their general causes through Formal AI. Follow docs/case-studies/pull-request-1188/multi-agent-workflow.md (owner, 2026-10-09, R1188-U26). This supersedes the older immediate-push arrangement.

1. Monitor the remote PR head and all checks while the coordinator works. Collect failed logs promptly and group duplicate failures by general cause.
2. Ask Formal AI through its JavaScript planner to repair each cause. Fix JavaScript first and the Rust twin generally; repair Formal AI after a failure and retry. Never run cargo, rustc or rust-script locally. Run only closest JavaScript tests and standalone rustfmt checks.
3. Commit completed fixes in small coherent pieces. Acquire /private/tmp/formal-ai-pr1188-commit.lock before reading HEAD, initialize an isolated index from fresh HEAD, stage owned paths only, confirm the base is unchanged, commit, refresh the default index and release the lock. Never stash or reset --hard.
4. The coordinator pushes only when all contributing agents finish their current bulk batch or every fix in the current CI failure batch is fully committed. Report readiness; never push independently. Monitor the new exact head and cancel superseded runs after the coordinated push.
5. Import self-AST census evidence only for the final source head through the checked apply-census rule. Apply routing measurements honestly and lower ceilings only when measured.
6. Supply finalized dogfood rows, original requests, failures, successful retries, regression pins and observed mutation counts. Leave unresolved capability gaps open.

Validate Docker runtime and release artifacts alongside tests. Keep post-merge-only observations pending until observed. Respect ownership claims, delete completed scratch copies and never copy the whole repository.
