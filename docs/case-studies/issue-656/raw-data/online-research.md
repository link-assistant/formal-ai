# Online research (2026-07-16)

- GitHub, [About protected branches](https://docs.github.com/en/repositories/configuring-branches-and-merges-in-your-repository/managing-protected-branches/about-protected-branches): required status checks and reviews are branch protection gates. Consequence: local replay cannot substitute for checks on the eventual pushed SHA.
- GitHub, [Troubleshooting required status checks](https://docs.github.com/en/repositories/configuring-branches-and-merges-in-your-repository/managing-protected-branches/troubleshooting-required-status-checks): checks must be associated with the latest commit SHA. Consequence: promotion records local evidence but leaves remote authority to GitHub.
- GitHub, [Managing a merge queue](https://docs.github.com/en/repositories/configuring-branches-and-merges-in-your-repository/configuring-pull-request-merges/managing-a-merge-queue): queued changes are tested with the latest base branch. Consequence: the protocol must not predict post-rebase CI.
- Cargo, [cargo test](https://doc.rust-lang.org/cargo/commands/cargo-test.html): Cargo's stable human output reports test pass/fail totals; JSON output remains unstable. Consequence: the parser prefers explicit suite reports and uses the stable human summary only for the unit gate.

No source above is copied into the implementation. It informs the trust-boundary
and evidence-policy decisions summarized in the case study.
