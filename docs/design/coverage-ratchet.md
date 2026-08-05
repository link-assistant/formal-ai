# Coverage: published, measured, and non-decreasing

The project's stated goal is to double the tests toward 100% coverage. Until
issue #895 that goal had no mechanism behind it: CI generated an LCOV file,
uploaded it as an artifact, and nothing ever read the numbers. A percentage
nobody checks is not a requirement — it is a hope.

This document describes the mechanism that replaced the hope: what is measured,
how it is published, how a decrease is rejected, and what it takes to lower a
floor on purpose.

## Two denominators, never averaged

The repository ships two independent bodies of production code:

| Denominator | What it counts | How it is measured |
| --- | --- | --- |
| `rust` | The Rust workspace under `src/` | `cargo llvm-cov --all-features --lcov` |
| `browser` | The browser JavaScript under `src/web/` | `node --test --experimental-test-coverage --test-reporter=lcov` |

They are ratcheted separately and are never combined into one figure. A Rust
line and a browser line are not the same unit of risk, and averaging them
produces a number that moves for reasons nobody can act on: a large, well-tested
Rust suite would mask an untested website, and the single number would go *up*
while the shipped product got worse. Two honest numbers beat one dishonest one.

### How browser code is measured honestly

The site's JavaScript is not instrumented as a bundle. `src/web/app.js`,
`src/web/vendor.bundle.js`, `src/web/ocr.bundle.js` and
`src/web/web-search-component.bundle.js` are build output; measuring minified
output would report coverage of a file no human maintains, and would count
vendored dependencies as project code. They are excluded from the denominator
and their sources are measured instead.

The rest of `src/web/` is measured as the browser sees it. The page scripts
(`preferences.js`, `i18n.js`, `syntax-highlight.js`, `memory.js`,
`seed_loader.js`, `site-chrome.js`, the per-page configs) and the 24-module
worker mirror (`src/web/worker/formal_ai_worker_*.js`) are loaded into a
`node:vm` sandbox by `tests/web/support/browser-runtime.mjs`. Passing each
script's real absolute path as the `vm.Script` `filename` makes V8 attribute
coverage to that repository path, so the LCOV report names the same files the
browser downloads. The worker is booted through its real entry point,
`src/web/formal_ai_worker.js`, with the canonical `data/seed/*.lino` corpus
served behind `fetch` — the same files the dev server mirrors into
`src/web/seed/`. The answers those tests assert are therefore the answers the
deployed site gives.

### The unmeasured-file inventory

Excluding a file from the denominator is how a coverage number becomes a lie, so
`src/web/` is checked for completeness. Every `.js`/`.jsx` file under it must be
either measured or listed in `coverage/browser-unmeasured.txt` as a
`path<TAB>reason` row. Modeled on `scripts/hardcoded-language-allowlist.txt`,
the list is a ratchet in its own right — the gate fails when a file is neither
measured nor declared (a new blind spot), when a row names a file that no longer
exists (stale), when a row names a file that *is* measured now (prune it), or
when a row has no reason. It can shrink; it cannot grow silently.

## The gate

```bash
rust-script scripts/check-coverage-ratchet.rs [--only <name>] [--lcov <name>=<path>]
```

The script reads `coverage/baseline.json`, parses each denominator's LCOV
report, and compares the measured line and function percentages against the
reviewed floors. It exits `1` when a percentage falls more than
`tolerance_percent` below its floor, `2` on a configuration or I/O error, and
`0` otherwise. An empty denominator is a hard error rather than `0%`: a glob
that stops matching would otherwise report a perfect regression as no data.

### What it publishes

Both artifacts land in `coverage/` and are uploaded by CI:

- `coverage/summary-<name>.md` — human-readable: a per-metric table of covered
  and total counts, measured percentage, baseline, delta in percentage points,
  and status; the ten least-covered files, so the report says where to write the
  next test; and the inventory result. In CI it is also appended to
  `$GITHUB_STEP_SUMMARY`, so the numbers are visible on the run page without
  downloading anything.
- `coverage/summary-<name>.json` — machine-readable: the same metrics plus
  per-file counts and the inventory breakdown, for any tool that wants to trend
  or diff them.

Regressions are additionally emitted as `::error::` annotations and improvements
as `::notice::`, so a decrease is attached to the failing job rather than buried
in a log.

## Raising and lowering a floor

Raising is meant to be routine:

```bash
rust-script scripts/check-coverage-ratchet.rs --only browser --update-baseline
```

This writes the measured numbers into `coverage/baseline.json`. Commit the
result; the gain is locked in and can never be given back by accident.

Lowering is meant to be deliberate. The same command **refuses** to write a
lower floor:

> Refusing to lower the reviewed baseline for `browser` without
> `--justification "<reviewed reason>"`. A decrease must be an explicit,
> reviewed decision recorded in the baseline file.

Supplying `--justification` records the reason in the denominator's
`lowered_reason` field, so the decrease appears in the diff as a sentence a
reviewer must read and approve, not as two digits changing. A later raise clears
the field, because the decision it records no longer describes the code.

`--reviewed` and `--evidence` update the audit fields alongside the numbers.

## Where it runs

| Job | Workflow | What it does |
| --- | --- | --- |
| `lint` | `release.yml` | `rust-script --test scripts/check-coverage-ratchet.rs` — the gate's own tests, including a case that validates the committed `coverage/baseline.json`, so a hand-edit that breaks the schema fails before the gate is relied on |
| `coverage` | `coverage.yml` | `cargo llvm-cov` → `--only rust` → uploads `coverage-rust` |
| `browser-coverage` | `coverage.yml` | `npm run coverage:web` → `--only browser` → uploads `coverage-browser` |

Both denominators live in `.github/workflows/coverage.yml` rather than in the
release pipeline. Nothing in the release graph `needs:` them, so the move
changed no ordering, and a workflow named *Coverage* with one job per
denominator shows in the checks list that these are two numbers, not one. It
also keeps `release.yml` under the 2000-line ceiling
`scripts/check-file-size.rs` enforces — a file the gate already documents as
debt that must not grow.

## Locally

```bash
npm run coverage:web                       # writes coverage/browser-lcov.info
npm run coverage:ratchet -- --only browser

cargo llvm-cov --all-features --lcov --output-path lcov.info
npm run coverage:ratchet -- --only rust
```

`coverage/baseline.json` and `coverage/browser-unmeasured.txt` are committed;
every other file under `coverage/` is generated output and is ignored.

## Why a ratchet rather than a fixed target

A fixed target such as "90% or fail" has two failure modes. Below the target it
is permanently red, so the team learns to ignore it. At the target it stops
rewarding improvement — 91% and 99% are equally green, and the extra tests are
unrewarded work. A ratchet is green from the first day, rewards every gain by
locking it in, and makes the only way to go backwards a reviewed sentence in a
diff. That is the mechanism the "double the tests toward 100%" requirement
needed.
