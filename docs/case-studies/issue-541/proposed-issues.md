# Issue #541 — Proposed Follow-up Issues

The instruction in [`raw-data/issue-541.json`](raw-data/issue-541.json) is
explicit:

> *"Please plan and execute everything in this single pull request, you have
> unlimited time and context …"*

So unlike #511 (which we decomposed into an 8-milestone epic with separate
PRs per theme), **issue #541 ships as a single PR (#542) carrying every
requirement.** That decision is the right one for this issue: the nine
requirements live entirely inside this repo, each has a small blast radius,
and shipping them together avoids any "the migration landed but the CTA did
not" mid-state.

This file therefore documents two kinds of artefacts:

1. **In-scope work** that is delivered in PR #542. These are the same
   themes from the case study, restated as a delivery plan so PR review
   can verify the scope is intact.
2. **Out-of-scope follow-ups** that are reasonable next steps but are not
   in PR #542. None of them block the issue; they are filed here so a
   future contributor does not have to rediscover them.

---

## 1. In-scope: the single-PR delivery plan

PR #542 lands seven commits, each addressing one or more requirements. The
order is deliberate — pre-requisites land first so each commit compiles and
the test suite stays green at every revision.

| Order | Commit | Themes | Risk if shipped alone | Why ship together |
|---|---|---|---|---|
| 1 | `6454154c` | R2 Docker detect | Low — desktop main-process only | Unblocks Agent-mode sandbox dispatch for any subsequent commit's manual testing |
| 2 | `62a1e7ae` | R5 + R6 + R7 animation/preview | Low — pure renderer | Establishes the `usePrefersReducedMotion` + `useMessageReveal` hooks needed by R8's trace render |
| 3 | `70b37a0d` | R8 reasoning trace | Low — renderer + i18n catalog | Re-uses the hooks from commit 2 |
| 4 | `52cdf3f7` | R3 data migration | **High** if shipped without R4 — a user with mixed real+demo content would have demo events migrated forward in a confusing state | R4 (the demo isolation fix) lands next and tags every demo event so the migration carries them across with the `isDemo` flag intact |
| 5 | `c53ed080` | R4 demo isolation | Medium — UX-only but uses the post-migration profile shape | Lands immediately after the migration so any "legacy demo turns" are tagged correctly on first launch |
| 6 | `1722d764` | R9 grant-all CTA | Low — permissions panel only | Depends on Agent-mode dispatch already working (R2 in commit 1) |
| 7 | `151865c7` | R1 dark/auto theme coverage | Low — CSS only | Final commit — affects every screen and is the easiest to visually verify against the other six |

All seven commits are tagged with their R-numbers and the issue number in
the message body so `git log --grep '#541'` returns the full set.

---

## 2. Out-of-scope follow-ups (filed here, not in PR #542)

### F1. Snapshot-based dark-theme regression coverage

R1's test today asserts colour bands on two anchor widgets (mode-status,
sidebar-toggle.is-collapsed). The remaining three covered widgets
(`drawer-menu-section h2`, `tool-mode`, `tool-mode-agent .tool-mode`) are
exercised only by visual inspection of the screenshots in
[`assets/`](assets/). A future follow-up could add a Playwright
visual-regression mode that snapshots each of the primary widget classes
under light + dark + auto and diffs against committed baselines.

**Reproducible scope:** add `test.describe('theme regression — full
widget set')` to `issue-541-theme.spec.js`, iterate over the five
selectors, and call `expect(locator).toHaveScreenshot()` with a 5 %
threshold. **Why not in #542:** snapshot baselines add ~100 KB of PNGs
to the repo and a flaky-screenshot blast radius into CI; the
band-assertion approach was deemed enough for the original bug.

### F2. Migration replay UI for partial profile transfers

`desktop/lib/data-migration.cjs` migrates the canonical three subtrees
(`IndexedDB`, `Local Storage`, `Session Storage`) but does not yet
migrate Chromium-managed *cookies* or the *Service Worker* registration
data. A user who had OAuth-style auth state in the legacy profile would
need to re-login.

**Reproducible scope:** add `Cookies`, `Service Worker`, `Cache`,
`Code Cache`, and `WebSocketStorage` to `STORAGE_SUBTREES`, expand the
unit-test fixtures, and bump `DATA_VERSION` to `2`. **Why not in #542:**
the original issue only flagged conversations being lost, not auth
state; the wider migration set increases the risk of carrying forward
corrupted state.

### F3. Animation budget per-message override

The `minMessageAnimationMs` preference is global. A user with the
budget at 8 s for storytelling might want a one-shot "skip" affordance
on individual messages (e.g. an "instantly show" pill at the top of the
reasoning trace).

**Reproducible scope:** add `data-skip-animation` to the message
container; when clicked, `useMessageReveal` clears its pending timers
and sets `revealed: true`. **Why not in #542:** the original issue is
about the *default* feeling too fast — not about per-message control.

### F4. Reasoning-step hierarchy editing for power users

R8 uses the server-side `level` field to fold sub-steps out of the
"standard" view. The renderer trusts the server's labelling. A future
maintainer tool could let advanced users re-classify a step's level on
the fly to inspect what the "standard" view would have shown.

**Reproducible scope:** add a right-click context menu on each rendered
step in Diagnostics mode with "Bump to high level" / "Demote to
sub-step" entries; wire to a renderer-only override map keyed by step
id. **Why not in #542:** out-of-scope; the issue asks for sensible
defaults, not maintainer tooling.

### F5. Mode-flip-on-grant in tests covering raw IPC

The R9 test mocks the FormalAiDesktop bridge. A higher-confidence
integration test would launch a real Electron `BrowserWindow`, register
the renderer's `preload.cjs` bridge, and exercise the click against the
real IPC channel. The infrastructure for this exists in #511's
cold-start spec.

**Reproducible scope:** add `tests/e2e/tests/issue-541-permissions-cold-start.spec.js`
that derives from `issue-511-cold-start.spec.js`, sets up the mock
provider via `runAgentProvider` in the main process, and clicks the
CTA. **Why not in #542:** the unit-level mock spec already covers the
renderer behaviour; the cold-start variant adds CI time without
catching a new class of regression.

---

## 3. Closing state

PR #542 closes issue #541 in its entirety. The five follow-ups above are
all "nice to have" — none guards a user-visible failure on the supported
configurations, and each can stand on its own as a separate issue when
prioritised by the maintainers.
