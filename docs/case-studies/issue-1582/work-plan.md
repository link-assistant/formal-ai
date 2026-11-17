# Issue #1582 work plan

Objective: implement and verify every requirement of #1582 in PR #1583, retain
the complete evidence and feedback, keep the PR unmerged, and mark it ready
only when the acceptance criteria and required checks pass.

- [x] Read the complete issue, comments, PR discussion, inline comments and reviews; check sub-issues (none).
- [x] Download and inspect both screenshots, prior draft logs and available artifacts.
- [x] Archive all 33 available historical branch runs and log-fetch attempts; explain inaccessible original Mac logs. Current repair-head CI remains below.
- [x] Enable read-only network retrieval by default, respecting explicit offline/deny controls.
- [x] Answer internet access and permission questions with actionable harness controls in the supported languages.
- [x] Send routine startup/exit diagnostics to files; opt-in terminal logging and nested formatting.
- [ ] Configure actual resource-derived context limits for every supported client. Claude and configurable integrations are implemented; **Gemini input context is blocked upstream** (google-gemini/gemini-cli#30333). Native adapter tests still need execution on an adequately provisioned runner.
- [x] Add regression tests, reproduce failures before the changes, and run focused checks afterward.
- [x] Complete timeline, root causes, solution research, upstream reporting where applicable, evidence index and limitations.
- [x] Update mirrors, generated translations and release notes. Complete local gate run: 138/138 passed or explicitly skipped; rounds three through five have no failures; round six caught lower debt measurements and a stale comment projection; round seven verifies the native-route repairs and round eight verifies the subsequent CI fixes: each has 138/138 passed or explicitly skipped, no failures.
- [x] Review the full PR diff (paginated API when the combined diff exceeds GitHub limits), confirm current main is already included, commit, check the push gate and resolve all current repair-head CI: 22/22 workflows pass.
- [x] Update PR title/description with reproduction and test evidence; retain `Closes #1582`.
- [ ] Mark ready only after the Gemini criterion and native regression execution are complete. Keep draft/unmerged while blocked. Verify the evidence-commit checks and clean status before reporting.

Coordination, supplied shell commands, patches and experiment recipes in this
investigation are externally authored by Codex. Formal AI execution receipts
are recorded as execution evidence with **zero independent authorship credit**.
Failed planner attempts are retained. The 100% autonomous authorship and 5%
reasoning targets are not claimed achieved by supplied command execution.
