# Issue #541 — Requirement Inventory

This document itemises every functional requirement extracted from
[`raw-data/issue-541.json`](raw-data/issue-541.json). The verbatim quote from
the issue body is preserved for each entry so the PR review can verify nothing
in the spec was dropped or paraphrased away.

Status legend used below:

| Status | Meaning |
|---|---|
| ✅ Implemented | The fix lives on the `issue-541-bf330e9c749a` branch and is covered by a test. |
| 🟡 Partial | Some sub-points covered, others deferred — never used in this iteration. |
| ⏳ Deferred | Followed up in [`proposed-issues.md`](proposed-issues.md). |

Priorities:

- **P0** — visible incorrectness or data loss for users (R1, R2, R3, R4, R9).
- **P1** — perceived-quality / accessibility regressions (R5, R6, R7, R8).
- **P2** — process / meta (R10).

---

## Theme A — CSS theming

### R1 — Every primary widget honours the active theme

> *"Not all UI elements has correctly applied theme, may be we should migrate to chakra-ui.com, so we do everything right, and it will also look better, more polished."*

- **Priority:** P0
- **Status:** ✅ Implemented (`151865c7`)
- **Why a chakra migration is rejected:** the web app runs without a build step
  via hyperscript React (`h(…)` straight from `src/web/app.js`); Chakra requires
  JSX + Emotion + framer-motion bundling — see
  [`raw-data/background-research.md`](raw-data/background-research.md) §2. The
  three-tier CSS-custom-property pattern already in `src/web/styles.css`
  delivers the same "define once, override per theme" capability; the bug is
  *missing overrides*, not a wrong system.
- **Acceptance:** with `theme "dark"` (or auto + dark OS) set in
  `formal-ai.preferences.v1`, every primary widget — the topbar mode-status
  badge, the collapsed-sidebar toggle, the mobile drawer section headings, and
  the per-step `tool`/`agent` mode badges — renders against the dark palette,
  not the light hex from the base rule. The Playwright spec
  [`tests/e2e/tests/issue-541-theme.spec.js`](../../tests/e2e/tests/issue-541-theme.spec.js)
  reads `getComputedStyle()` and asserts the colour is in the dark band.

---

## Theme B — Desktop reliability

### R2 — Docker availability is detected correctly

> *"`Docker unavailable` is wrong, because it is actually available."*

- **Priority:** P0
- **Status:** ✅ Implemented (`6454154c`)
- **Sub-points implied by the bug class:**
  - GUI launches must find Docker even when the shell's interactive PATH is not
    inherited (`/usr/local/bin`, `/opt/homebrew/bin`, `/Applications/Docker.app/...`,
    Windows `Program Files`, NixOS).
  - A `docker` that comes up *after* the app opened must be detected on the
    next probe (no permanent "missing" memoisation).
  - Diagnostics are opt-in via `FORMAL_AI_DESKTOP_DEBUG`; binary path can be
    overridden via `FORMAL_AI_DOCKER_BIN`.
- **Acceptance:** `dockerIsAvailable()` returns `true` on a real GUI launch
  with Docker Desktop running; the in-process unit suite at
  `desktop/scripts/docker-detect.test.mjs` exercises every well-known path,
  the TTL re-probe, and the env-var override.

### R3 — Conversations survive app upgrades + brand renames

> *"Previous desktop conversation were deleted, we need to make sure we use some kind of application folder in each OS, where all the memory security kept, and when new version arrives, we need to make sure we migrate our data to new form, so all previous users will be able to continue using their data (that is critical feature, and must be done right)."*

- **Priority:** P0
- **Status:** ✅ Implemented (`52cdf3f7`)
- **Sub-points:**
  - The userData directory is pinned by `app.setName('formal-ai')` *before*
    the `ready` event, so the pinned name is the one Electron derives
    `userData` from.
  - On first launch the data-migration module non-destructively copies the
    Chromium-managed subtrees (`IndexedDB`, `Local Storage`, `Session Storage`)
    from any known legacy profile (`formal-ai Desktop`, `formal-ai-desktop`,
    `Formal AI`, `formal_ai`, and the electron-default name) into the pinned
    profile.
  - A version stamp (`formal-ai-data-version.json` carrying `DATA_VERSION=1`)
    prevents re-running the migration on every boot and seeds future schema
    migrations.
- **Acceptance:** `desktop/scripts/data-migration.test.mjs` exercises:
  "no legacy profile" (no-op), "legacy profile present and pinned profile
  empty" (subtrees copied), "both profiles non-empty" (skip, legacy left in
  place), "version stamp already present" (skip), and the
  `pinAppName()` ordering. The migration never deletes a legacy file.

---

## Theme C — Demo mode safety

### R4 — Demo mode never touches the user's conversations

> *"Demo mode should now delete or overwrite any user conversations, if we are in the existing conversation we should create new conversation for demo mode, that is overridden when active, so when user switching to any non-demo mode conversation we should automatically disable demo mode, and keep last example in the newly created demo conversation."*

- **Priority:** P0
- **Status:** ✅ Implemented (`c53ed080`)
- **Note on the verbatim quote:** the original sentence reads
  *"Demo mode should now delete or overwrite"* — context (and the rest of the
  sentence) make it clear the intent is *"should NOT delete or overwrite"*.
  All implementation and tests treat this as the user's intent.
- **Sub-points:**
  - Turning Demo mode **on** from inside a real conversation does **not**
    write into that thread; it spawns (or rejoins) a dedicated, sidebar-hidden
    demo conversation that holds the scripted turns.
  - Turning Demo mode **off** restores the user's previous conversation
    exactly as they left it.
  - Clicking any real conversation in the sidebar **auto-disables** demo
    mode.
  - The dedicated demo conversation is reused within a session, so the last
    example survives an on/off toggle without ever surfacing in the sidebar.
- **Acceptance:** `tests/e2e/tests/issue-541-demo-mode.spec.js` drives the
  full toggle cycle, asserts the sidebar conversation count, replays demo
  on→off→on, and reads the IndexedDB events to confirm `isDemo: true` is
  persisted on every demo event.

---

## Theme D — Reasoning / answer reveal UX

### R5 — Minimum animation budget honoured by every reveal

> *"By default reasoning steps animation is too fast, and answers are too fast, we need to make sure user will feel something is happening, and thinking animation is scrolling, even if it is immediate, also we should have setting for minimum message animation time where 0 is immediate display, and by default it should be 2 seconds for full animation play out."*

- **Priority:** P1
- **Status:** ✅ Implemented (`62a1e7ae`)
- **Sub-points:**
  - New preference `minMessageAnimationMs` (default 2000, clamp 0–8000;
    `normalizeAnimationBudgetMs`).
  - Settings UI exposes "Immediate (0 s)" through "Up to 8 s" choices.
  - `usePrefersReducedMotion` short-circuits the budget to 0 when the OS asks
    for reduced motion.
  - Existing AI/UX research justifies the default: see
    [`raw-data/background-research.md`](raw-data/background-research.md) §6
    (Nielsen's 1 s / 10 s thresholds, OpenAI streaming pattern).
- **Acceptance:** logic test
  `experiments/reveal-budget-logic.test.mjs` covers the budget math; the
  Playwright demo-mode spec asserts the freshly produced assistant message
  carries `data-animation="revealing"` for at least a tick when the budget is
  non-zero.

### R6 — Reasoning reveals first; answer body fades in after

> *"First we display reasoning steps, animate scrolling, and after that only when we scrolled to last thinking step we can show message itself. All these animations + showing message also with animation should be under the selected budget."*

- **Priority:** P1
- **Status:** ✅ Implemented (`62a1e7ae`)
- **Sub-points:**
  - The `useMessageReveal` hook divides the budget: 72 % to step reveal, 28 %
    to body fade-in. The split is deterministic so reduced-motion users always
    skip to body immediately.
  - The reveal is **per-message**, not per-page, so reloading history does
    not re-animate.
- **Acceptance:** logic test `experiments/reveal-budget-logic.test.mjs`
  asserts the 0.72 / 0.28 split, the early-return for `prefersReducedMotion`,
  and the per-step pacing under non-trivial step counts.

### R7 — Collapsed reasoning preview shows ≥ 1 full step

> *"Also at the moment reasoning steps in collapsed more are too small, so even not a single line of thinking is displayed, we need to make sure we actually display at least one paragraph/reasoning step fully."*

- **Priority:** P1
- **Status:** ✅ Implemented (`62a1e7ae`)
- **Sub-points:**
  - `.thinking-preview-current` gains `min-height: 1.55em` and loses the
    `nowrap` / single-line ellipsis.
  - `.thinking-preview-previous` retains the rotated-scroll fade but at a
    height (1.05em) that fits one full *previous* line without cropping the
    current step.
- **Acceptance:** visual inspection plus the test inside the demo-mode spec
  that asserts the collapsed preview's measured height is ≥ 1.55em.

### R8 — Reasoning steps are human-readable, default to 50 % detail

> *"Reasoning steps should be more human like, we don't expect user to understand too much detail on how it works by default. We need to make sure that by default at 50% setting of reasoning detalization we show only high level thinking steps (not sub steps) - if there is no steps hierarchy it should be added. We also need to make sure in maximum detail thinking/reasoning steps are still fully human readable, they can have injected actual data, terms and so on in markdown, but they should not be too much technical, meaning no special syntax and so on. Reasoning steps should be short, by concise, expressive, so it is really possible to understand how the thinking was done."*

- **Priority:** P1
- **Status:** ✅ Implemented (`70b37a0d`)
- **Sub-points:**
  - `PREFERENCE_DEFAULTS.thinkingDetailLevel = "standard"` (the 50 % midpoint).
  - At standard, `filterThinkingEntriesForDetail` returns only `level === "high"`
    steps plus the final reflection — exactly the high-level hierarchy.
  - At "detailed", the full hierarchy renders but **no raw Links tuples** —
    `naturalizeThinkingStep()` drops the `(@USER OP:…)` interpolation.
  - The formalization step is projected to a plain task noun via nine new
    `formalizeOp*` keys (`Greet`, `Farewell`, `Express`, `Compute`, `Define`,
    `Lookup`, `Search`, `Procedure`, `Identify`) in all four locales.
  - The raw symbolic tuple stays available in **Diagnostics** mode for
    maintainers (no information lost — just relocated).
- **Acceptance:** the i18n catalog audit
  (`tests/e2e/scripts/check-i18n-catalog.mjs`) pins the new keys; visual
  inspection during the demo-mode spec confirms the default mode renders only
  high-level steps and reaches the final reflection without ever showing the
  tuple.

---

## Theme E — Permissions UX

### R9 — One-click grant + replay deferred terminal task

> *"After permissions are granted nothing happens, the message for granting permissions should also include button to grant all permissions and switch to agent mode, which when clicked should actually evaluate pending task for execution. So everything will be maximum user friendly by default."*

- **Priority:** P0
- **Status:** ✅ Implemented (`1722d764`)
- **Sub-points:**
  - The `DesktopPermissionPanel` now renders a single primary CTA *above* the
    per-tool rows.
  - With no pending task: copy is **"Grant all and switch to Agent mode"**.
  - With a pending task: copy upgrades to **"Grant all, switch to Agent mode,
    and run pending task"**.
  - Click handler `grantAllAndRunPending`: mirrors all six
    `DESKTOP_TOOL_OPTIONS` to `true` via refs *before* setting React state,
    flips `modeRef.current` to `"agent"`, and calls
    `executeTerminalCommand(task.command, "agent")` if a task was pending.
  - Default-deny is preserved — the grant flows through the existing
    `isPermitted` gate; the CTA grants every tool individually rather than
    bypassing checks.
- **Acceptance:**
  [`tests/e2e/tests/issue-541-permissions.spec.js`](../../tests/e2e/tests/issue-541-permissions.spec.js)
  contains two cases: (a) no pending task — grant flips mode to Agent and
  every gate now reports `granted: true`; (b) pending task `ls ~` from chat
  mode — clicking the CTA flips mode, grants, and the mocked `runAgentProvider`
  observes a replay call with the exact original command.

---

## Theme F — Process / meta

### R10 — Compile a deep case study under `docs/case-studies/issue-541`

> *"We need to download all logs and data related about the issue to this repository, make sure we compile that data to `./docs/case-studies/issue-{id}` folder, and use it to do deep case study analysis (also make sure to search online for additional facts and data), in which we will reconstruct timeline/sequence of events, list of each and all requirements from the issue, find root causes of the each problem, and propose possible solutions and solution plans for each requirement (we should also check known existing components/libraries, that solve similar problem or can help in solutions). … If there is not enough data to find actual root cause, add debug output and verbose mode if not present, that will allow us to find root cause on next iteration. … Please plan and execute everything in this single pull request."*

- **Priority:** P2 (deliverable, not user-facing)
- **Status:** ✅ Implemented
- **Sub-points:**
  - [`raw-data/issue-541.json`](raw-data/issue-541.json) is the original
    issue body verbatim.
  - [`raw-data/background-research.md`](raw-data/background-research.md)
    surveys Electron storage paths, design-token theming, AI-chat timing
    studies, and the Chakra-migration trade-off.
  - [`raw-data/desktop-storage-investigation.md`](raw-data/desktop-storage-investigation.md)
    and [`raw-data/reasoning-step-emission.md`](raw-data/reasoning-step-emission.md)
    document the read-only root-cause traces for R3 and R8.
  - This file and [`solution-plans.md`](solution-plans.md) cover the
    per-requirement analysis.
  - [`best-practices.md`](best-practices.md) extracts the reusable patterns.
  - [`proposed-issues.md`](proposed-issues.md) discusses why this issue
    lands as a single PR and lists any follow-ups worth filing.
  - Verbose mode is delivered for the highest-failure subsystem
    (`FORMAL_AI_DESKTOP_DEBUG` opt-in tracing in `desktop/main.cjs`) so the
    Docker / userData diagnostics that historically went into stderr now
    surface labelled lines on demand.

---

## Traceability matrix

| Req | Test | Source files touched | Commit |
|---|---|---|---|
| R1 | `tests/e2e/tests/issue-541-theme.spec.js` | `src/web/styles.css` | `151865c7` |
| R2 | `desktop/scripts/docker-detect.test.mjs` | `desktop/lib/docker-detect.cjs`, `desktop/main.cjs` | `6454154c` |
| R3 | `desktop/scripts/data-migration.test.mjs` | `desktop/lib/data-migration.cjs`, `desktop/main.cjs` | `52cdf3f7` |
| R4 | `tests/e2e/tests/issue-541-demo-mode.spec.js` | `src/web/app.js`, `src/web/memory.js` | `c53ed080` |
| R5 | `experiments/reveal-budget-logic.test.mjs` | `src/web/app.js`, `src/web/preferences.js`, `src/web/i18n-catalog.lino` | `62a1e7ae` |
| R6 | `experiments/reveal-budget-logic.test.mjs` | `src/web/app.js`, `src/web/styles.css` | `62a1e7ae` |
| R7 | manual + measured in demo-mode spec | `src/web/styles.css` | `62a1e7ae` |
| R8 | i18n audit (`check-i18n-catalog.mjs`) + manual | `src/web/app.js`, `src/web/i18n-catalog.lino` | `70b37a0d` |
| R9 | `tests/e2e/tests/issue-541-permissions.spec.js` | `src/web/app.js`, `src/web/i18n-catalog.lino`, `src/web/i18n-catalog-permissions.lino` | `1722d764` |
| R10 | case-study deliverables in this folder | `docs/case-studies/issue-541/**` | (this commit) |
