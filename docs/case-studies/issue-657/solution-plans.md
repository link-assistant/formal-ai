# Issue 657 solution plans

| Requirements | Implemented plan |
| --- | --- |
| I657-01, I657-11 | Use two explicit Git trailers. Resolve evidence with `git show <commit>:<path>` so later working-tree changes cannot forge historical attribution. Require the evidence content to bind the session id and Formal AI marker. |
| I657-02–I657-04 | Enumerate `since..until` with `rev-list --no-merges`; total additions and deletions from `show --numstat --no-renames`; ignore binary `-` fields; calculate rounded integer basis points. |
| I657-05 | Build a temporary Git repository in the unit test, tag a baseline, commit one evidenced two-file change and one human change, and assert 3/4 exactly. |
| I657-06 | Reuse the existing release-body builder. Select the ledger row by the final tag, append a `## Self-hosting` section before the API body-size guard, and fail the release if the requested row is absent. |
| I657-07 | Reuse `version-and-commit.rs`, because it alone knows the final release tag. Measure before generated version changes, append idempotently, stage the ledger, then commit/tag through the existing path. |
| I657-08–I657-10 | Seed an honest historical row, parse ledger rows with a small std-only parser, compute a changed-line-weighted three-release trailing rate, and reject a lower new rate. Pin the ledger and both workflow integrations in the spec suite. |
| I657-12 | Preserve compact source captures, all four feedback channels, primary-source research, and an actual in-repo Formal AI agent replay. |

## Rejected alternatives

- Git author/email is easy to spoof and does not bind a Formal AI session.
- Counting commits weights a one-line change like a large implementation.
- Git blame measures surviving ownership, while the issue asks for changed work
  in a release; `numstat` includes both additions and deletions.
- A permanent hard percentage floor would make an honest 0% baseline fail;
  the requested trailing ratchet instead starts at observed history.
