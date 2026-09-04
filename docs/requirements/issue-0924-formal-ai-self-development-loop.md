## Issue #924 Formal AI Self-Development Loop

Issue [#924](https://github.com/link-assistant/formal-ai/issues/924) closes E77
by making one real, reviewable Formal AI contribution a release-cycle
condition. The contract, replay evidence, and requirement map live in
[`../case-studies/issue-924/`](../case-studies/issue-924/).

| ID | Requirement | Status / Evidence |
| --- | --- | --- |
| R924-1 | Require at least one merged Formal AI-authored pull request in each release cycle. | `ensure_self_development_release` scans first-parent GitHub merge commits and rejects a release range with no matching session-backed PR. |
| R924-2 | Preserve replayable evidence and prove the authored commit reached review unchanged. | The required session/evidence trailers are extended by `Formal-AI-Pull-Request`; the gate matches its number to the merge subject and requires the same commit object in the merged second-parent history. |
| R924-3 | Record each contribution and continue reporting per-release and trailing self-hosting share. | New ledger rows carry repeated `self_authored_pull_request` values plus the existing changed-line measurements; release notes include the reviewed PR URLs and target. |
| R924-4 | Make the release target a non-decreasing ratchet, lowerable only in the open. | Each row records its target. The next target is the greater of the prior target and comparable trailing share; the pre-version release gate rejects a projection below it while the range can still be repaired. Because a ratchet that only climbs can strand a cycle it cannot answer (issue #1069), a row may carry a reviewed `target_override_basis_points` that replaces it and carries forward until another commit changes it. That is the only lever: no flag, environment variable, or workflow input moves the target, so every change to the level is a diff and is named in the release notes. |
| R924-5 | Keep E69/E74 dependencies and the ordinary review, CI, and promotion gates intact. | E69 supplies the write-effect floor, E74 supplies the two-way Hive Mind gate, and #924 grants no bypass. The real Agent CLI-authored leaf and raw session evidence are committed in the case study. |
| R924-6 | Count a pull request only when every non-merge commit it introduces is validly attributed to Formal AI. | `merged_self_authored_pull_requests` checks the complete second-parent contribution set; the mixed human/AI fixture is rejected even though one commit carries valid evidence. |
| R924-7 | Run the same compound task through Formal AI via Agent CLI, attempt it whole first, and split recursively only after observed failure. | The issue-924 self-authoring experiment uses incremental dispatch; its committed report retains the failed root, productive three-child split, and passing child sessions. |
| R924-8 | Compose only verified passing effects and avoid regressing a valid composition with a redundant parent mutation. | `TaskExecutor::retry_after_children` lets incremental orchestration verify the parent task directly; the composed-verifier replay and a destructive-parent integration regression pin the behavior. |
| R924-9 | Use those execution sessions for proposal-only auto-learning while leaving promotion human-gated. | `learning.lino` observes the four real Agent sessions, excludes the non-client composed verifier, records `human_gated`, and contains no approved decision. |
| R924-10 | Have Formal AI Agent CLI author at least 20% of the smallest task leaves with exact replay evidence. | Two of six leaves (33%) are the Agent-authored execution and pull-request contracts; canonical and captured bytes match and native session/resume evidence is retained. |
| R924-11 | Preserve exact reproductions and generalize only from demonstrated failures. | Focused before/after tests cover mixed PR authorship, cross-platform Python startup, and the redundant parent retry found by the first real incremental run; the case study retains the successful replay. |
