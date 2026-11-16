---
bump: patch
---

### Fixed
- A repeated or resumed delivery is a success (#1157): when the verified files already match the last commit, the answer says so in a sentence ("Nothing to commit: … a repeated or resumed run is a success, not a failure") and the run makes no new commit and no pull request edit. The commit stages only the verified files by literal pathspec. `rust/tests/unit/issue_1157_idempotent_commit.rs` drives the Rust planner through two sessions against a real temporary repository.
