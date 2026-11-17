# Issue 1580 work plan

Objective: fully implement and verify issue #1580 in PR #1581 on issue-1580-8d61a32a61e1.

- [x] Read the issue and all issue/PR conversation comments; inline comments and reviews are empty.
- [x] Recover the interrupted session gist and five preserved commits; merge origin/main.
- [x] Preserve/read all cited and latest logs: 71 run records with hashes and original line numbers; expired graph log explicitly unavailable. Successful scope gates are not heavy-path verification.
- [ ] Fix main CI/CD and Layered CI: worker budget, factual QA network ownership, browser runner, download retries, Pages flow-YAML timeout, solver size, synthesis/spec regressions, described-file ownership, shard timing.
- [ ] Fix or verify Hive Mind draft, dependencies-latest, issue 1028 ladder, Learning cycle, Job Headroom, External Benchmarks and work-item qualification.
- [ ] Compare complete trees and every workflow/CI script in eight current language templates; report reproduced shared defects upstream.
- [ ] Apply current Hive Mind CI-CD-BEST-PRACTICES.md.
- [ ] Add behavioral regression tests with negative controls, default-off tracing where needed, and fast pre-merge replays of production failures.
- [ ] Record requirement/implementation/verification mapping, template comparison, provenance and limitations in case study; add changelog/release trigger.
- [ ] Run focused JS tests sequentially, touched gates, generation checks and rustfmt; no local Rust build.
- [ ] Review diff and PR text; commit; push only after all actionable failures are collected and active runs finish.
- [ ] Verify final-head CI, update PR with Fixes #1580, mark ready, confirm clean tree. External blockers remain explicit and unmerged.

## Development provenance

Initial exploration/recovery used external tools. The first Formal AI request, `Run the shell command "git status --short".`, returned unknown. `Run git status --short` executed successfully. A supplied multi-file Python creation command was refused by the declared-create guard with no effects. The orchestration script is an external bootstrap; subsequent shell operations use JavaScript Formal AI. Supplied commands/edits earn zero autonomous-authorship credit. Recovered commits retain their authorship, without new attribution trailers.

## Frozen substantive reasoning units

Frozen before new implementation: CI freshness/taxonomy; worker budget; passage network ownership; flow-YAML parsing; retry coverage; child-result consumption; compound decomposition; constraint precedence; described-file authority; solver boundaries; native timing/shard balance; collector failure propagation; draft custody; transfer/deadline accounting; dependency verification; external benchmark diagnosis; qualification preconditions; full-tree template comparison; negative regression controls; final delivery evidence (20 units). Formal AI attempted and independently accepted shares are reported separately. Supplied conclusions receive zero credit.

## Current verification checkpoint

At 22ffaa73d1b2b0dc800ed2f9093059ac80ddef56, local gates: 86 passed, 52 explicit skips, zero failures. Focused JS tests and native source formatting passed. All 3,007 template blobs and 48 workflows are inventoried; semantic CI closure review remains open. See docs/case-studies/issue-1580/README.md for each unresolved criterion. No native execution or final-head hosted CI is claimed.
