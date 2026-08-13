---
bump: minor
---

### Added
- Merge-conflict policy: `data/meta/merge-conflict-policy.lino` declares every structural cause of a merge conflict this repository has actually had, the mechanism that removes it, and the verifier that keeps it removed. `python3 scripts/analyze-merge-conflicts.py --ledger` measures the history (884 merges, 1914 conflict events) into `data/meta/merge-conflict-ledger.lino`, and `rust-script scripts/check-merge-conflict-policy.rs` fails the build when a path that has actually been conflicting is neither mechanized nor deferred with a written reason. No `git config` step is needed: every mechanism uses git's built-in `merge=union` driver or a committed generator.
- CI gates are one file each under `data/meta/ci-gates/`, run by `rust-script scripts/run-ci-gates.rs --stage <stage>`. Adding a check no longer edits `.github/workflows/release.yml`, which was the repository's third most conflicted path.
- One seed inventory for both runtimes: `data/meta/seed-registry.lino` names every `data/seed/*.lino` file once, and `rust-script scripts/generate-seed-registry.rs --write` generates `src/seed/embedded_registry.rs` and `src/web/seed-files.js` from it, so the Rust engine and the browser worker cannot disagree about which seed files exist.

### Changed
- `src/seed/embedded.rs` and `src/web/seed_loader.js` no longer carry their own copies of the seed file list; `src/agentic_coding/mod.rs` and `src/web/formal_ai_worker.js` no longer carry their own declaration lists. Each list now lives in a sibling file that contains nothing else and is union merged, with `rust-script scripts/normalize-ordered-lists.rs --write` restoring the canonical order.
- CONTRIBUTING.md documents what to add where so a contribution stops creating an append point, and `docs/case-studies/issue-991/merge-conflict-analysis.md` records the measurement behind every decision.
