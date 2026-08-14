# Issue 924 Requirements

| ID | Requirement | Verification |
| --- | --- | --- |
| R924-1 | Require at least one real, merged Formal AI-authored pull request in every release cycle. | `ensure_self_development_release` rejects a range without a matching first-parent GitHub merge; `release_cycle_requires_a_session_backed_merged_pull_request` proves a direct commit with a claimed PR URL does not count. |
| R924-2 | Keep authorship replayable and bind the reviewed PR to the unchanged session-backed commit. | The three commit trailers bind session, committed evidence, and canonical PR URL. Merge ancestry must contain the same commit object. Session `ses_0020cec63ffe7RIFkQ1qH9YZcY` and its raw client/server evidence are committed under `self-hosting-authorship/`. |
| R924-3 | Record every qualifying contribution and report the self-hosting share per release. | New ledger rows retain repeated `self_authored_pull_request` fields, the per-release share, and the trailing share; release notes render the target and PR URLs. |
| R924-4 | Ratchet the target upward without allowing a silent decrease. | Each row records `target_percentage_basis_points`; the next floor is `max(previous target, previous comparable trailing share)`, and the pre-version release gate rejects a projection below it. |
| R924-5 | Preserve E69/E74 dependencies and the same review, CI, and promotion gates as human work. | E69 is merged; E74 is provided by PR #1004. The gate adds no approval path and counts only an unchanged commit after normal PR merge. Existing PR evidence, CI, and #656 promotion policies remain in force. |
