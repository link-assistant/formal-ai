TASK (tag CI-SPEED): make CI fast to iterate on, and enforce that automatically.

The user's rule:
- No CI job or step runs longer than 15–30 minutes.
- Long-running jobs and tests start first, so smaller ones fill the remaining capacity once the big ones are running.
- This holds at two levels:
  - jobs on parallel machines;
  - tests inside a job, run in parallel.
- It is enforced automatically.

State to measure first, read-only:
- `gh run list --branch qa-reasoning-coding-bulk-fixes --limit 30` and `gh run view <id> --json jobs`: per-job durations of the last full runs, e.g. 37753751698 (CI/CD Pipeline), 37753750834 (Coverage) and 37753750847 (Layered CI).
- Write the measured durations to `data/meta/ci-durations.lino`, one row per job: workflow, job, median and maximum minutes, measured date.
- Make the generator reusable: `experiments/formal_ai_subagent/ci-durations.mjs --write`, using `gh`. Later it can run in CI on a schedule.
- Known numbers: "Test suite took 656s of its 2400s execution budget"; specification shards about 276s. Find the jobs over 30 minutes and the critical path.

Do:
1. **A gate in data, plus a JS checker,** e.g. `scripts/check-ci-speed.mjs` (no Rust twin needed unless the gate runner requires one). It fails when:
   - a workflow job lacks `timeout-minutes` or sets it above 30;
   - a job's measured maximum in `ci-durations.lino` is above 30 minutes;
   - in a job with test shards, the shard assignment does not put the longest tests first (see 3).

   Register it under `data/meta/ci-gates/` with a justification citing #1188.
2. **Split every job over the limit** into parallel shards or jobs of 15 minutes or less where possible. Keep "small parallel CI jobs". Order the `needs:` graph so the long jobs start first: they get no `needs` on fast lint jobs unless they truly depend on them.
3. **Test-level scheduling, longest first:**
   - Rust: where the workflow shards `cargo test` / nextest, order the shard lists by recorded test duration. Record durations from CI output, e.g. a `--report-time`-like listing, into `data/meta/test-durations.lino`.
   - Node: run `node --test` with `--test-concurrency` set to the core count, and list long files first.
   - E2E: Playwright `fullyParallel` with spec files ordered longest-first.

   Make the shard assignment a deterministic longest-processing-time greedy split computed from the duration data, not a hand list.
4. **Do not lose coverage:** every test that ran before still runs. Add a check that the union of shards equals the full test list.
5. **Document it** in `docs/ci.md`, or wherever CI is documented: the 15–30 minute rule, longest first, and how to refresh durations.

Rules:
- **No cargo and no rust-script locally.**
- You may read CI logs with `gh`.
- Workflows can only be checked by pushing, so make the YAML edits careful. Validate them with `node experiments/formal_ai_subagent/local-gates.mjs` and any workflow lint the repository has (`grep -rn actionlint`).
- Claim `.github/workflows/*` and `data/meta/ci-gates/*` in `claims.md`. SCRIPTS-A and SCRIPTS-B added twin gates there, so keep those.
- Use Formal AI for the small edits; ledger rows T340–T359.
- Do not commit.
- Report: the before and after critical path, the jobs split, the gate, and what still exceeds the limit and why.
