### Changed

- PR #1188 round 19:
  - **No job over 30 minutes (R1188-U9).** Two of the listed exceptions are gone:
    - the agent CLI e2e workflow runs its computer-use record and generalization phases as their own matrix legs, each under a 30-minute cap;
    - the GitHub Pages build (web bundle, `cargo doc`, stamp, upload) moved to the reusable `.github/workflows/pages-artifact.yml` under 30 minutes. The release workflow's `deploy-pages` job now only deploys that artifact, capped at 15 minutes.
  - **Meaningful file names (R1188-U5).** The 25 numbered browser worker modules are named by subject (`formal_ai_worker_00.js` → `formal_ai_worker_seed_responses_and_language.js`, …), together with their `ts/` twins and line-budget shards. The renames are recorded in `data/meta/rename-map.lino` and applied by rule (`experiments/formal_ai_subagent/rename-by-rule.mjs`, gate `check-rename-map`). The modules now load in any order.
  - **Readable links notation (R1188-U6, R1188-U7).** `scripts/measure-notation.mjs` measures abbreviated names, `_` against `-`, and duplicated lines across the `.lino` files. The counts are held by `data/meta/notation-ratchet.lino` (gate `check-notation`), the rules live in `data/meta/notation-rules.lino`, and the style is described in `docs/links-notation-style.md`.
  - **A valid, testable release (R1188-U27)** is recorded as a requirement of its own.
