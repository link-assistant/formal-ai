---
bump: minor
---

### Added
- A non-decreasing coverage ratchet. `scripts/check-coverage-ratchet.rs` reads the
  LCOV report CI already produced, publishes a human-readable
  `coverage/summary-<name>.md` (per-metric table, deltas in percentage points, the
  ten least-covered files) into the run summary alongside a machine-readable
  `coverage/summary-<name>.json`, and fails the build when a percentage drops below
  the reviewed floor in `coverage/baseline.json`. Raising a floor is
  `--update-baseline`; lowering one is refused unless `--justification "<reviewed
  reason>"` records the decision in the file, so a decrease reaches review as a
  sentence in the diff rather than two digits changing (issue #895, epic #710).
- Coverage of the browser production sources, which had none. `tests/web/` loads the
  unbundled `src/web/` page scripts and the 24-module worker mirror into a `node:vm`
  sandbox under their real repository paths — which is what makes V8 attribute
  coverage to the files the browser downloads — and boots the worker through its real
  entry point with the canonical `data/seed/*.lino` corpus behind `fetch`. 48 new
  tests; `npm run test:web` and `npm run coverage:web` run them, and a new
  `browser-coverage` job in `.github/workflows/coverage.yml` enforces the browser
  denominator.
- `coverage/browser-unmeasured.txt`, a committed `path<TAB>reason` inventory of every
  `src/web/**` file the browser denominator does not measure. A file that is neither
  measured nor declared fails the build, as does a stale, redundant or unexplained
  row, so the denominator cannot be quietly narrowed to flatter the number. The list
  can shrink; it cannot grow silently.

### Changed
- The Rust and browser denominators are ratcheted separately and never averaged into
  a single figure: a large Rust suite would otherwise mask an untested website while
  the combined number went up. `docs/design/coverage-ratchet.md` documents the
  measurement, the publication format, and the baseline-update policy.
- Coverage now runs from `.github/workflows/coverage.yml` instead of the release
  pipeline. Nothing in the release graph depended on the job, so the move changed no
  ordering; it puts one job per denominator in the checks list, and it returns
  `release.yml` to 1930 lines, back under the 2000-line ceiling
  `scripts/check-file-size.rs` documents as debt that must not grow.

### Fixed
- The coverage job's timeout, raised from 25 to 40 minutes. Issue #812 set 25 against
  a worst case of 14.1 minutes, but the instrumented suite has since grown to
  17.2–19.6 minutes across the last eight green runs on `main` — 78% of the budget,
  the same one-slow-run margin #812 was filed about — and it hit the cap outright.
- `npm run test:web` and `npm run coverage:web`, which passed `tests/web/` to
  `node --test`. Node 20 recurses into that directory, but the Node 22 the workflow
  pins resolves it as a module path and fails with "Cannot find module". Both scripts
  now use `tests/web/*.test.mjs`, which the shell expands identically on either
  version.
