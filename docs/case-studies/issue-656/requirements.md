# Complete requirement inventory

| Source requirement | Verification |
| --- | --- |
| proposal → benchmark evidence → decision → applied event chain | `PromotionRun::memory_events`; bundle round-trip test |
| collect open self-improvement proposals | required non-empty `--proposals`; learning-run bridge |
| coding-modification, industry, and unit gates | canonical three-command replay test and real run log |
| accepted `.lino` edits only | seed-path validator and promoted/rejected materialization test |
| dry-run by default | CLI dry-run integration test proves no seed write |
| `--apply` requires `--confirm` | refusal integration test executes before proposal loading/gates |
| rejected proposal persists with evidence | `promotion_rejection` event test |
| bundle export/import | custom event round-trip test |
| branch/draft PR workflow; never direct push | clean-worktree/local-branch test and branch plan; no push execution |
| same task through Formal AI / Agent CLI | exact `write_file` extraction, byte comparison, session id, external e2e artifact |
| real automated benchmark evidence | subprocess replay; injected/malformed/failed evidence tests |
| general rather than one hardcoded learned rule | arbitrary proposal source/path/payload plus alternate wording exact-write tests |
| CI remains an outer gate | no local CI prediction; draft PR plan leaves required checks to GitHub head SHA |
| case study and online research | this directory and traceability test |
