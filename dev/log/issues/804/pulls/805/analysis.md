# Issue #804 — deep analysis (iteration 1)

All facts below are quoted from the logs stored next to this file, or from the
repository at commit `8b5acee` (the SHA both failing runs were built from).

## 1. Timeline of events

| Time (UTC) | Event |
| --- | --- |
| 2026-07-19 16:38 | `46aa1dd7` "fix: preserve bundled browser framework seals" adds the `browser-runtime` ignore rule to `desktop/scripts/adhoc-sign-mac.cjs` (issue #798 / PR #799). |
| 2026-07-19 23:46 | `CI/CD Pipeline` run 29708451325 fails on `bcda460`. |
| 2026-07-20 00:04 → 01:53 | Four `Desktop Release` runs fail on `bcda460` / `ac9d925`. |
| 2026-07-20 05:38 | PR #803 merged as `8b5acee`; `CI/CD Pipeline` run **29719602956** fails in `Auto Release → Collect changelog and bump version`. |
| 2026-07-20 05:56 | `Desktop Release` run **29720321919** starts; `macos-x64` and `macos-arm64` packaging jobs fail; the aggregation job then fails with `No artifacts from: macos-x64 macos-arm64`. |
| 2026-07-20 06:20 | Logs collected into this folder. |

Note that `46aa1dd7` **is an ancestor of `8b5acee`** (`git merge-base --is-ancestor`
returns true), so the #798 fix was already present in the failing run — it did not work.

## 2. Requirements extracted from the issue

| # | Requirement | Source |
| --- | --- | --- |
| R1 | Find and fix every failure in the latest default-branch CI/CD runs (`Desktop Release`, `CI/CD Pipeline`). | issue table |
| R2 | Find and fix **false positives** (CI red while the code is fine). | title |
| R3 | Find and fix **false negatives** (CI green while something is broken). | title |
| R4 | Eliminate warnings, not only errors. | title |
| R5 | Compare *every* workflow/CI script against the three `link-foundation/*-ai-driven-development-pipeline-template` repos and adopt their best practices. | issue body |
| R6 | If a defect also exists in a template, file an issue against that template repo. | issue body |
| R7 | Follow `link-assistant/hive-mind/docs/CI-CD-BEST-PRACTICES.md`. | issue body |
| R8 | Apply each fix everywhere it applies (whole codebase, not one call site). | solver instructions |
| R9 | Where data is insufficient, add opt-in debug/verbose output (default off) so the next iteration can find the root cause. | solver instructions |
| R10 | Do all of it in the single PR #805. | issue body |

## 3. Failures, root causes, proposed solutions

### F1 — `CI/CD Pipeline` / Auto Release: self-hosting ratchet blocks every release (**false positive**)

Log (`ci-logs/failed-29719602956.log`):

```
Max published version on crates.io: 0.300.0
Initial bump (minor) from 0.300.0: 0.301.0
Error recording self-hosting release metric: self-hosting ratchet would fall from 32.68% to 19.19% for v0.301.0
##[error]Process completed with exit code 1.
```

Root cause — `scripts/self-hosting-metric.rs:245-256`:

```rust
if let Some(previous) = rows.last() {
    if row.trailing_percentage_basis_points < previous.trailing_percentage_basis_points {
        return Err(format!("self-hosting ratchet would fall from {} to {} for {tag}", ...));
    }
}
```

The ratchet requires the **trailing 3-release weighted percentage to be
monotonically non-decreasing**. The ledger (`data/meta/self-hosting-ledger.lino`)
shows why that is unsatisfiable in practice:

| tag | per-release % | trailing % |
| --- | --- | --- |
| v0.298.1 | 0.00 | 0.00 |
| v0.299.0 | 5.95 | 5.74 |
| v0.300.0 | 69.56 | 32.68 |

`v0.300.0` contributed 141 380 self-authored of 203 258 changed lines. Once that
outlier enters the window, *any* subsequent release with a normal ratio lowers the
trailing average — and when the outlier eventually leaves the 3-release window the
average drops further still. So the check hard-fails the release job on every
release regardless of the actual quality of the work. It is a false positive and it
is also a **release-blocking** one: no crate/tag has been published since v0.300.0.

Additional defect: the metric is *recorded* (a ledger write, i.e. a side effect)
inside the version-bump step, so a metric-policy failure aborts the release even
when versioning succeeded.

Proposed solutions (in preference order):

1. **Make the ratchet tolerant and advisory-by-default.** Compare against the
   *minimum* trailing value of the window, or allow a configurable slack
   (e.g. fail only if the trailing value falls below `max_seen - slack`, default
   slack large enough to absorb outliers), and emit `::warning::` instead of a hard
   error unless `SELF_HOSTING_RATCHET=enforce`. Keeps the signal, removes the false
   positive.
2. **Ratchet on a floor, not on the last value.** Persist an explicit
   `ratchet_floor_basis_points` in the ledger that is raised deliberately (a human
   decision, like a coverage floor in `cargo-llvm-cov` or `codecov.yml`'s
   `threshold:`), not automatically from the last measurement.
3. **Decouple**: record the metric in a separate, non-release-blocking job/step
   (`continue-on-error: true` + job summary), so a policy metric can never block a
   publish.

Recommended: (2) as the model, implemented with (1)'s escape hatch, plus (3).
This mirrors how coverage ratchets are done in the ecosystem (Codecov
`threshold`, `cargo-llvm-cov --fail-under-lines`, `danger-js` warnings).

### F2 — `Desktop Release` / macOS: codesign breaks the bundled Chrome for Testing framework seal

Log (`ci-logs/failed-29720321919.log`, both `macos-x64` and `macos-arm64`):

```
electron-osx-sign Executing... codesign --sign - --force --timestamp=none --options runtime \
  --entitlements build/entitlements.mac.plist \
  .../Contents/Resources/browser-runtime/Frameworks/Google Chrome for Testing Framework.framework
 > Stderr: ...: replacing existing signature
 ...: unsealed contents present in the root directory of an embedded framework
  ⨯ Command failed: codesign ...
##[error]Process completed with exit code 1.
```

Facts established:

- Our custom hook *is* invoked: `• executing custom sign file=release/mac/formal-ai Desktop.app ... identityName=-`.
- `@electron/osx-sign@1.3.3` is the signer (`electron-osx-sign electron-osx-sign@1.3.3`).
- In 1.3.3, `shouldIgnoreFilePath` **does** support function entries
  (`if (typeof ignore === 'function') return ignore(filePath)`), and it is applied
  to the walked child paths — so the ignore predicate added in `46aa1dd7` is the
  right mechanism in principle.
- Yet the very first `Signing...` line in the run is a `browser-runtime` path, and
  the log contains **zero** `Skipped...` lines, i.e. the predicate never returned
  true for any path.
- The log also contains **no** `codesign --verify --deep --strict` invocation from
  our own `runCodesign` re-seal step, because the build dies before reaching it.

Root cause: **not yet provable from the available logs.** The predicate compares
`path.relative(path.resolve(appPath)/Contents/Resources/browser-runtime, path.resolve(filePath))`.
Both sides look correct on paper (electron-builder passes a *relative* app path —
`file=release/mac/formal-ai Desktop.app` — and the step runs with
`working-directory: desktop`, so `path.resolve` should reconstruct the same absolute
prefix `@electron/osx-sign` walks). The surviving hypotheses are:

- H1: `findAppPath` picks a different key than the one `@electron/osx-sign` walks
  (`opts.app`), so the ignore root points at a different bundle.
- H2: `process.cwd()` at hook time is the repo root rather than `desktop/`,
  making the ignore root `<repo>/release/mac/...` — which matches nothing — while
  `@electron/osx-sign` walks the absolute path it resolved itself.
- H3: the `ignore` array we pass is dropped/overwritten by `validateOptsIgnore`
  normalisation for function entries in this exact version.

Because the evidence cannot discriminate between H1–H3, this iteration adds
**opt-in debug output** (see §5) that prints, for each candidate path, the resolved
app path, the ignore root, the CWD, and the predicate's verdict. One further CI run
with `FORMAL_AI_MACOS_SIGN_DEBUG=1` (already set by
`.github/workflows/desktop-release.yml:232`) will decide it.

Structural solutions to consider regardless of which hypothesis wins — relying on a
signer's `ignore` list to protect a *nested, independently signed framework* is
fragile, and the ecosystem's usual answer is to not nest it during signing:

1. **Move-out/move-back**: relocate `Contents/Resources/browser-runtime` outside the
   `.app` before signing (`afterPack`/`afterSign` hooks) and restore it afterwards,
   then re-seal the outer bundle once. Deterministic; no dependence on ignore
   semantics.
2. **Ship the browser runtime as an opaque archive** (`browser-runtime.zip` in
   `extraResources`) extracted on first run — codesign never sees an embedded
   framework. This is what Playwright/Puppeteer-based Electron apps commonly do,
   and it also removes the "unsealed contents in the root directory of an embedded
   framework" class of error permanently.
3. **Do not ship Chrome for Testing at all**; download it at runtime into
   `app.getPath('userData')` (Playwright's own default). Smallest artifact, no
   signing interaction.

Recommended: (2), with (1) as the interim fix if the artifact must stay bundled.

### F3 — aggregation job reports a **partial** release as a failure only at the end

```
##[error]No artifacts from: macos-x64 macos-arm64. SHA256SUMS.txt was published but covers only the targets that built.
```

The guard itself is correct and is a genuine improvement over silence (it prevents a
false negative). The defect is ordering: `SHA256SUMS.txt` **is published** before the
completeness check, so a partially-complete checksum manifest reaches the release
page and only afterwards the job turns red. Fix: verify target completeness *before*
publishing `SHA256SUMS.txt`/provenance, or publish it to a staging asset name and
promote it only when all targets are present.

## 4. Systematic work still open (R2–R8)

These require a file-by-file comparison and are tracked as follow-up work in this
PR; they are listed here so nothing is lost between iterations:

- Compare `.github/workflows/release.yml` (1625 lines) and
  `desktop-release.yml` (650 lines) against the three
  `link-foundation/*-ai-driven-development-pipeline-template` repos, item by item
  (permissions blocks, `concurrency` groups, action SHA pinning, `timeout-minutes`
  on every job, `--locked` cargo invocations, cache keys, `continue-on-error`
  audits, `if: always()` on report steps).
- Audit every `continue-on-error: true` and every `|| true` in workflows and in
  `scripts/*` — each one is a candidate **false negative**.
- Audit every step that greps logs for success markers — a changed message silently
  turns the check into a no-op.
- Grep the workflows for missing `timeout-minutes` (hangs surface as 6-hour runs).
- Cross-check the same defects in the templates and open issues there (R6).

## 5. Debug instrumentation added in this iteration (R9)

`desktop/scripts/adhoc-sign-mac.cjs` now prints, when
`FORMAL_AI_MACOS_SIGN_DEBUG=1` (default off; the desktop workflow already sets it
for the ad-hoc packaging step), lines prefixed `[adhoc-sign-mac]`:

- the raw signing-option keys received from electron-builder and the value of each
  candidate app-path field (`app`, `appPath`, `path`);
- `process.cwd()`, the chosen app path and its resolved form;
- the computed `browser-runtime` ignore root;
- one `ignore <decision> <path>` line per file the predicate is asked about.

This is enough to discriminate H1/H2/H3 on the next `Desktop Release` run without
guessing.
