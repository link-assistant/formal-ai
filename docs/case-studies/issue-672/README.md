# Issue 672: landing the deferred issue-541 UI follow-ups (F1–F5)

Issue [#672](https://github.com/link-assistant/formal-ai/issues/672) is about a
specific failure mode of the process, not about a bug in the product. PR #542
(issue #541) finished its work by writing five concrete follow-ups into
[`docs/case-studies/issue-541/proposed-issues.md`](../issue-541/proposed-issues.md)
— and none of them was ever filed, so all five sat in a markdown file for
months. #672 treats that file as the specification and requires each item to
either land with its own tests or be explicitly reconciled against newer work,
with no silent drops.

All five landed. Two of them landed differently from the original sketch; both
divergences are argued below rather than left for a reader to discover in a
diff.

## Outcome

| Item | Sketch in proposed-issues.md | What landed | Divergence |
| --- | --- | --- | --- |
| F1 | `toHaveScreenshot()` at 5 % on five widgets | computed-colour tables snapshotted as text for 5 widgets × 4 theme states × 2 surfaces, plus unasserted review PNGs | yes — see below |
| F2 | widen `STORAGE_SUBTREES`, bump `DATA_VERSION` | done, plus the IPC channels and the renderer notice the issue title asks for ("replay memory migration notices") | partial — `Cache`/`Code Cache` deliberately excluded |
| F3 | `data-skip-animation` + `useMessageReveal` clears its timers | exactly that | no |
| F4 | right-click menu wired to a renderer-only override map | right-click menu wired to an append-only event log with a projection | no (the issue body's stricter wording won) |
| F5 | derive from `issue-511-cold-start.spec.js`, provider in the main process | exactly that | no |

Test counts, all green in one run
(`raw-data/playwright-issue-672.log`, `raw-data/desktop-node-tests.log`):

```text
  44 passed (1.7m)
# tests 121
# pass 121
# fail 0
```

## F1 — dark-theme regression coverage

`tests/e2e/tests/issue-672-theme-snapshots.spec.js`, 10 tests, snapshots under
`tests/e2e/tests/__snapshots__/issue-672-theme-snapshots.spec.js/`.

Issue #541's R1 fixed five widgets that kept their light palette in dark mode,
but its spec asserts colour bands on two of them (`mode-status`,
`sidebar-toggle.is-collapsed`). The other three were left to visual inspection.

**Divergence: computed colours, not pixels.** F1 itself recorded why the
snapshots were not shipped in #542: "snapshot baselines add ~100 KB of PNGs to
the repo and a flaky-screenshot blast radius into CI". That trade-off has not
changed — pixel baselines captured on a contributor's machine disagree with CI
over font hinting and subpixel AA, which is exactly the flake class
`playwright.local.config.js` is tuned against. So the spec snapshots the
*computed* colour of each widget instead:

```text
drawer-menu-section-heading: color=rgb(201, 193, 182)
sidebar-toggle: color=rgb(236, 231, 223) backgroundColor=rgb(40, 51, 47)
tool-mode-agent: color=rgb(242, 192, 132) backgroundColor=rgba(255, 170, 90, 0.22)
tool-mode-thinking: color=rgb(174, 188, 255) backgroundColor=rgba(120, 140, 255, 0.22)
topbar-mode-status: color=rgb(201, 193, 182)
```

This catches the exact bug F1 cares about — a widget rendering in the wrong
theme's palette — is byte-stable across machines, and diffs in review as text.
It also covers more than the sketch asked for: `light`, `dark`, and `auto` under
both OS colour schemes, on both the web surface and the desktop surface (the
same bundle behind a `FormalAiDesktop` bridge, which unlocks the desktop-only
sidebar). Every one of those runs in CI, because `playwright.local.config.js` is
what the `test-e2e-local` job executes.

The pixel view is not lost: the last test still writes full-page dark-theme PNGs
to `docs/screenshots/issue-672/` for human review. They are simply not asserted
on.

## F2 — migration replay

`desktop/lib/data-migration.cjs`, `desktop/main.cjs`, `desktop/preload.cjs`,
`desktop/scripts/data-migration.test.mjs`,
`tests/e2e/tests/issue-672-migration-replay.spec.js` (8 tests).

The v1 migration copied IndexedDB / Local Storage / Session Storage, so a user
whose profile moved arrived logged out. v2 adds `Cookies`, `Service Worker`,
`WebStorage`, and `WebSocketStorage`, and `subtreesAddedAfter()` tops up an
existing v1 profile without re-copying what it already has.

**Divergence: `Cache` and `Code Cache` are excluded.** F2 listed them with the
auth subtrees. They are pure derived caches: Chromium regenerates them on
demand, they are the largest directories in a profile, and carrying a cache
built by a *different* Chromium build into a fresh profile is a documented way
to produce hard-to-diagnose corruption. Copying them cannot fix the reported
symptom — being logged out — so the risk buys nothing. They are named in
`EXCLUDED_SUBTREES` with that reasoning inline, so the exclusion is a decision
in the code rather than an omission.

**Addition beyond the sketch.** F2's scope section only widened the subtree
list, but #672's own summary of the item is "a UI affordance to replay memory
migration notices". `migrate()`'s result was being discarded, so there was
nothing to surface. Two channels (`dataMigrationStatus`, `replayDataMigration`)
now carry it to the renderer, which shows a notice naming the profile the data
came from and what moved, with a replay button. The spec covers the awkward
states as well as the happy one: a failed transfer surfaces the error instead of
claiming success, a clean install is never interrupted, the web build (no
bridge) renders nothing, and an older desktop build without the channels
degrades quietly.

## F3 — per-message animation override

`src/web/app/main.jsx`, `tests/e2e/tests/issue-672-animation-override.spec.js`
(11 tests: 6 behavioural, one per supported language, and one that writes the
review screenshots).

The reveal budget was one global preference, so a user who likes the paced
reveal but wants *this* answer now had to change it for every future message.
The message container now carries `data-skip-animation="available"` while the
body is being withheld, and the button inside it settles that message only.

**Mutation check.** A spec that clicks a button and then waits can pass whether
or not the button does anything. Commenting out the one line that does the work
(`setSkipped(true)` in `useMessageReveal`) and re-running the spec fails exactly
the two behavioural tests and leaves the rest green
(`raw-data/f3-mutation-probe.log`, recorded before the per-language and
screenshot tests were appended, hence six tests rather than eleven):

```text
  ✘  1 … › the override shows the withheld answer immediately
  ✘  5 … › skipping one message leaves the others alone
  2 failed
  4 passed (2.0m)
```

The mutation was reverted before the commit; the current build has
`setSkipped(true)` intact and every test green.

## F4 — reasoning-step hierarchy editing

`src/web/app/main.jsx`, `src/web/styles.css`, `src/web/i18n-catalog.lino`,
`tests/e2e/tests/issue-672-reasoning-hierarchy.spec.js` (12 tests: 7
behavioural, one per supported language, and one that writes the review
screenshots).

Right-clicking a step in Diagnostics mode opens a menu with *Bump to high
level* / *Demote to sub-step* / *Restore the original level*, in all four
locales — asserted per language on the rendered labels, not just on the catalog,
and each of those tests performs the edit too so a locale cannot be labelled
correctly and broken behaviourally.

F4's sketch said "a renderer-only override map keyed by step id". #672's body
asks for something stricter — "edits append events, never mutate" — and that is
what landed: the state is the event log (`stepLevelEvents`), the map the UI
reads is `projectStepLevels()` over it, and a reset is an appended empty-level
event rather than a deletion. The map is still keyed by step id, so an edit
applies to that step in later answers too; the spec asserts that as intended
behaviour rather than leaving it as an accident.

The message record is never touched. Each rendered step carries the solver's own
label on `data-solver-level` beside the user's `data-level-override`, and one
test compares the full ordered `data-step` list before and after two edits in
opposite directions to prove the audit surface is unchanged.

## F5 — mode flip over raw IPC

`tests/e2e/tests/issue-541-permissions-cold-start.spec.js` (3 tests).

The R9 spec mocks `window.FormalAiDesktop` inside the page: every captured
request is a closure in the renderer's own realm, so a regression that broke
serialization of the replayed task, or mutated the grants after the boundary
call, would still pass. This spec derives from `issue-511-cold-start.spec.js`
instead — `runAgentProvider` and `setToolGrants` are `page.exposeFunction`
bindings, so the payloads really leave the browser context and are answered from
the test's Node process by a provider listing a hermetic temp home. The chat →
agent flip is asserted on both sides (the renderer's radiogroup and the `mode`
field Node received), and what the user reads back is a directory listing this
spec wrote to disk.

**Reconciliation of "server ⇄ embedded".** #672's summary line calls this
"IPC mode-flip tests (server ⇄ embedded)". The item text it summarizes is about
the Chat → Agent flip across the bridge, which is what landed. The
server-vs-embedded axis — a bridge reporting `apiReady: true` with an `apiBase`
— is already covered by `issue-511-cold-start.spec.js`, whose second test runs
the same journey against a container-gated commander provider on a real
`apiBase`. Duplicating it here with a fake `apiBase` would send the renderer's
memory sync and answer path at a URL that answers nothing, which tests the
fallback rather than the flip.

## CI

The four #672 specs and the #541 cold-start spec are registered in
`tests/e2e/playwright.local.config.js`, which is the config the `test-e2e-local`
job runs — so the dark-theme tables for both surfaces are gated on every push.

One gap turned up while doing F2: `desktop/scripts/*.test.mjs` was written but
never executed by any workflow, so the profile-migration code F2 extends had no
gate at all. The Lint job now runs them. `web-tools.test.mjs` is excluded
because it imports `@link-assistant/web-search` through
`desktop/lib/web-tools.cjs`, which only exists after `npm ci` in `desktop/`;
that exclusion is written into the step comment rather than hidden in a skip.

## Review screenshots

`docs/screenshots/issue-672/` is written by the specs themselves, so the images
in the pull request cannot drift from the code: the dark-theme surfaces by F1,
`f3-before-skip.png` / `f3-after-skip.png` by F3 (the withheld answer with the
*Show answer now* control, then the same message settled), and
`f4-before-edit.png` / `f4-menu-open.png` / `f4-after-edit.png` by F4 (the
thinking preview without `formalize`, the right-click menu, then the same
preview with the promoted step — above an unchanged diagnostics trace).

## Reproduction

```bash
cd tests/e2e
npx playwright test --config playwright.local.config.js issue-672 issue-541-permissions-cold-start
cd ..
node --test $(ls desktop/scripts/*.test.mjs | grep -v '/web-tools.test.mjs$')
```

Run Playwright from `tests/e2e/`. From the repository root a second
`@playwright/test` copy resolves first and the run fails with "Playwright Test
did not expect test.describe() to be called here".

## Raw data

- `raw-data/playwright-issue-672.log` — the 44-test run covering F1–F5.
- `raw-data/desktop-node-tests.log` — the 121 desktop library tests now wired
  into CI.
- `raw-data/f3-mutation-probe.log` — the F3 spec against a deliberately broken
  build, showing which tests are load-bearing.
