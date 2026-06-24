# Issue #541 — Solution Plans (per requirement)

For each of the nine functional requirements (R1–R9) plus the meta
deliverable (R10) we document: (a) the *reuse* — which existing components,
hooks, modules, or CI checks we can lean on; (b) the *plan* — the smallest
seam that makes the failure mode impossible; (c) the *test* — how the fix is
proven and how a regression would be caught.

Every file:line reference uses the state of the branch at the
[`151865c7`](https://github.com/link-assistant/formal-ai/commit/151865c7) commit.
Coordinates may drift over time; treat them as breadcrumbs, not contracts.

---

## R1 — CSS dark/auto theme coverage

### Root cause

`src/web/styles.css` already implements a three-tier theme system: light values
defined on `:root`, dark overrides under `:root[data-theme="dark"]`, and an
auto override behind `@media (prefers-color-scheme: dark) { :root:not([data-theme="light"]) … }`.
Most widgets honour the system, but five primary classes had **only** the light
rule, so their light hex (`#40515f` for text, `#eef2f1` for surface) bled
through in dark/auto mode:

| Selector | Bug colour | Where rendered |
|---|---|---|
| `.mode-status` | `color: #40515f` | Topbar Chat/Agent badge |
| `.sidebar-toggle.is-collapsed` | `background: #eef2f1` | Sidebar handle while collapsed |
| `.drawer-menu-section h2` | `color: #40515f` | Mobile drawer section headings |
| `.tool-mode` | `color: #40515f` | Per-step "tool"/"agent" badge inside reasoning trace |
| `.tool-mode-agent .tool-mode` | `color: #2c8f71` | Agent variant of same badge |

### Reuse

- The existing dark palette tokens (`--surface-overlay-dark`, `--text-muted-dark`,
  `--accent-dark`, …) defined elsewhere in `styles.css`.
- The existing `:root[data-theme="dark"]` block at line 3176 and the auto block
  at line 3493 — we add to them, not replace them.

### Plan

Append a labelled block (`/* Issue #541 — dark + auto coverage */`) near the
end of `src/web/styles.css` (around line 4171) that:

1. Adds `:root[data-theme="dark"]` overrides for each of the five selectors,
   using the dark palette tokens that are already in scope.
2. Adds the matching `@media (prefers-color-scheme: dark) { :root:not([data-theme="light"]) … }`
   block so users on a dark OS with `theme "auto"` get the same coverage
   without having to flip the preference.

Why this is the right size: the bug is "missing rules," not "the architecture
is wrong." Five lines of CSS per selector × five selectors × two contexts
(`[data-theme="dark"]` and the auto media query) is the floor.

### Test

[`tests/e2e/tests/issue-541-theme.spec.js`](../../tests/e2e/tests/issue-541-theme.spec.js)
boots the app with `theme "dark"` seeded into `localStorage`, waits for
`<html data-theme="dark">`, then for each widget reads
`getComputedStyle(node).color` (or `.backgroundColor`), parses the rgb, and
asserts:

- `isMutedDarkText(rgb)` — `r,g,b ≥ 170` (e.g. `#c9c1b6 = rgb(201,193,182)`).
- `isDarkSurface(rgb)` — `r,g,b < 70` (e.g. `#222624 = rgb(34,38,36)`).

The light bug values (`#40515f = rgb(64,81,95)`, `#eef2f1 = rgb(238,242,241)`)
fail both checks, so the test pins the *specific* regression without locking
in exact palette values.

---

## R2 — Docker detection

### Root cause

`desktop/main.cjs` historically did:

```js
const result = spawnSync('docker', ['--version']);
const dockerAvailable = result.status === 0;
```

Two failure modes:

1. **PATH-from-GUI.** A `.app` bundle launched from Finder does not inherit
   the interactive shell's PATH; `/usr/local/bin/docker` is on the user's
   `~/.zshrc` but Electron resolves `'docker'` against `/usr/bin:/bin` only.
   `spawnSync` returns `ENOENT`, `dockerAvailable` is `false`.
2. **Permanent memoisation.** The result was assigned to a module-level constant.
   Starting Docker Desktop *after* the app launched left the verdict stale —
   the only fix was a restart.

### Reuse

- The `runInSandbox` / `runDocker` helpers in `desktop/main.cjs` already
  manage the `konard/box-dind` flow once a binary is resolved.
- The `dependency-injection factory` pattern used by the unit-testable
  desktop modules in #511's E1 work (see `desktop/lib/tool-router.cjs`).

### Plan

Introduce `desktop/lib/docker-detect.cjs` exporting
`createDockerDetector({env, platform, spawnSync, existsSync, now, log,
okTtlMs, failTtlMs, probeTimeoutMs})`. Behaviour:

1. **Override:** if `env.FORMAL_AI_DOCKER_BIN` is set and exists, use it.
2. **Well-known paths** (in order):
   - macOS / Linux: `/usr/local/bin/docker`, `/opt/homebrew/bin/docker`,
     `/usr/bin/docker`, `/Applications/Docker.app/Contents/Resources/bin/docker`,
     `/run/current-system/sw/bin/docker` (NixOS).
   - Windows: `%ProgramFiles%\Docker\Docker\resources\bin\docker.exe`.
3. **PATH fallback:** bare `'docker'` (the historical behaviour, used last so
   the override + well-known paths take precedence on GUI launches).
4. **TTL cache:** ok verdicts cached for 30 s, fail verdicts for 3 s so a
   Docker Desktop that comes up post-launch is detected within seconds.
5. **Diagnostics:** when `FORMAL_AI_DESKTOP_DEBUG` is set, `log()` traces every
   probe: which paths were tried, which one matched, the cached verdict, and
   the TTL countdown.

Wire into `desktop/main.cjs`: replace the inline `spawn` with
`dockerDetector.dockerIsAvailable()` and `dockerDetector.resolveDockerBinary()`
inside both `runInSandbox` and `runDocker`.

### Test

`desktop/scripts/docker-detect.test.mjs` injects:

- a `spawnSync` stub that succeeds for `/usr/local/bin/docker` only;
- an `existsSync` stub that returns true for the same path;
- a `now` stub for time-travel testing the TTL.

Cases covered: env-var override, well-known path success on macOS, well-known
path success on Linux, well-known path success on NixOS, well-known path
success on Windows, PATH fallback success, total miss returns `false`, cache
returns ok within TTL, cache invalidates and re-probes after TTL, fail TTL
shorter than ok TTL so daemon-coming-online is detected quickly. Each case is
a pure-Node assertion, no real Docker daemon or filesystem.

---

## R3 — Desktop conversation persistence

### Root cause

Electron derives the userData directory from `app.getName()` (which itself
defaults to `productName` in `package.json`). Any rebrand or spacing/casing
fix moves userData to a fresh directory, orphaning IndexedDB + Local Storage.
[`raw-data/desktop-storage-investigation.md`](raw-data/desktop-storage-investigation.md)
walks the chain. No `app.setName`, no `app.setPath`, no migration step
existed in `desktop/main.cjs`.

### Reuse

- Electron's `app.setName()` + `app.getPath('userData')` for stable path
  derivation.
- Node's `fs.cp` (`recursive: true`) for the subtree copy.
- Same DI factory pattern as R2.

### Plan

Introduce `desktop/lib/data-migration.cjs` exporting
`createDataMigration({app, fs, path, log, now, pinnedName})`. Two public
methods:

1. `pinAppName()` — calls `app.setName('formal-ai')`. Must run **before**
   `app.whenReady()` so that Electron's `getPath('userData')` derives the
   pinned name. Wired in `desktop/main.cjs` at module load.
2. `migrate()` — invoked from `app.whenReady().then(...)`. Steps:
   - Check `formal-ai-data-version.json` under the pinned userData. If
     `DATA_VERSION` matches, no-op.
   - For each `legacyName` in `KNOWN_LEGACY_NAMES = ["formal-ai Desktop",
     "formal-ai-desktop", "Formal AI", "formal_ai", electronDefaultName]`:
     resolve the legacy userData path. For each `subtree` in
     `STORAGE_SUBTREES = ["IndexedDB", "Local Storage", "Session Storage"]`,
     `fs.cp(legacy/subtree, pinned/subtree, { recursive: true })` **only**
     when the destination is missing or empty. Never delete the legacy.
   - Stamp `formal-ai-data-version.json` with `DATA_VERSION: 1` and a
     timestamp.
   - Log every step under `FORMAL_AI_DESKTOP_DEBUG`.

### Test

`desktop/scripts/data-migration.test.mjs` injects an in-memory `fs` and a fake
`app`. Cases:

- No legacy directories ⇒ no-op, stamp written.
- One legacy directory present, pinned empty ⇒ subtree copied, legacy untouched.
- One legacy + pinned already has content ⇒ skip, do not overwrite.
- Stamp already present ⇒ idempotent no-op.
- `pinAppName()` called twice ⇒ second call is a no-op.
- `pinAppName()` after `whenReady()` is rejected (logs the misuse).

---

## R4 — Demo mode isolation

### Root cause

Demo mode wrote turns into `currentConversation`. With the user in a real
conversation, toggling demo on appended scripted turns to their history;
toggling off left the demo turns mixed in; the sidebar surfaced demo
conversations as if navigable; clicking a real conversation while demo was on
appeared to "overwrite" it.

### Reuse

- `src/web/app.js` `demoPreferences` already drives the toggle UI.
- `src/web/memory.js` already round-trips events through IndexedDB and accepts
  arbitrary metadata on each event.
- `src/web/app.js` `groupConversations` already filters conversations by
  predicates for the sidebar.

### Plan

1. Stash a per-session reference: `demoConversationIdRef`. When demo turns
   on while a real conversation is current, pick (or create) a dedicated
   `conversation:demo:<sessionId>` and set `currentConversationIdRef` to
   point at it for the duration of demo mode.
2. Tag every memory event written by the demo path with `isDemo: true`.
   Update `src/web/memory.js`'s save path and the IndexedDB round-trip to
   preserve the flag.
3. Filter `groupConversations` to drop any conversation whose first event
   has `isDemo: true` from the sidebar.
4. Subscribe a click handler on the sidebar so that clicking any non-demo
   conversation while demo is on flips `demoPreferences.demoMode = "off"`
   automatically.
5. The dedicated demo conversation persists within the session — toggling
   demo off, then on again, rejoins the same one (so the "last example"
   survives toggling).

### Test

[`tests/e2e/tests/issue-541-demo-mode.spec.js`](../../tests/e2e/tests/issue-541-demo-mode.spec.js)
drives:

- seed a real conversation with two user turns;
- flip demo on, send a scripted turn, assert the sidebar shows one item
  (the real conversation) and the chat shows three turns (real + scripted);
- flip demo off, assert the chat shows the user's original two turns
  with no scripted ones spliced in;
- click the real conversation (already current — should be a no-op), then
  flip demo on, then click the real conversation again ⇒ demo
  auto-disables;
- inspect IndexedDB via `page.evaluate` ⇒ every demo event has
  `isDemo: true`, no real event does.

---

## R5/R6 — Animation budget + reasoning-first reveal

### Root cause

Once the deterministic engine produced an answer, the React tree mounted both
the reasoning trace and the answer body in the same paint. No budget, no
pacing, no per-step pause. R5 asks for a configurable minimum; R6 asks for
ordered reveal under the same budget.

### Reuse

- The existing `prefers-reduced-motion` media query pattern in `styles.css`.
- The `preferences` flush + load round-trip in `src/web/preferences.js`
  (Links-notation localStorage).
- The existing `i18n` catalog at `src/web/i18n-catalog.lino`.

### Plan

1. Add `minMessageAnimationMs: 2000` to `PREFERENCE_DEFAULTS` (clamp 0–8000
   via `normalizeAnimationBudgetMs`).
2. Add the setting to Settings UI; four labelled choices:
   "Immediate (0 s)", "Brief (1 s)", "Standard (2 s)", "Relaxed (4 s)".
3. New hook `usePrefersReducedMotion()` — watches the media query, returns
   `boolean`. When `true`, every consumer should short-circuit to "show
   instantly."
4. New hook `useMessageReveal(message, budgetMs)`:
   - If `prefersReducedMotion` or `budgetMs === 0` ⇒ return `revealed: true`,
     `revealedStepCount: steps.length` immediately.
   - Otherwise: split the budget 72 % to steps, 28 % to body. For each step,
     after `budgetMs * 0.72 / stepCount` ms, increment `revealedStepCount`.
     After the last step, after `budgetMs * 0.28` ms, set `revealed: true`.
5. Mark freshly produced messages with `animateReveal: true` in the message
   object; hydrated history never gets the flag, so reloading the app does
   not re-animate.

### Test

`experiments/reveal-budget-logic.test.mjs` exercises the pure-logic
`normalizeAnimationBudgetMs` + step-by-step pacing math:

- `0` ⇒ 0; `2000` ⇒ 2000; `-5` ⇒ 0; `99999` ⇒ 8000.
- 5 steps + budget 1000 ⇒ 144 ms per step + 280 ms body fade.
- `prefersReducedMotion === true` short-circuits to instant.

The demo-mode spec asserts `[data-animation="revealing"]` appears for at
least one tick on a freshly produced assistant message when the budget is
non-zero, and never appears under `prefers-reduced-motion: reduce`.

---

## R7 — Collapsed reasoning preview height

### Root cause

`.thinking-preview-current` was `white-space: nowrap; text-overflow: ellipsis;
overflow: hidden;`, clipping any wrapped reasoning step to a single line.
`.thinking-preview-previous` had `max-height: 0.75em`, i.e. half a line, which
hid the rotated-scroll fade behind the current step.

### Reuse

The selectors already exist; the CSS is the only seam.

### Plan

1. Remove `white-space: nowrap`, `text-overflow: ellipsis`, and `overflow:
   hidden` from `.thinking-preview-current`.
2. Add `min-height: 1.55em` so a single short step still has the same visual
   footprint as a two-line wrapped one.
3. Bump `.thinking-preview-previous` `max-height` to `1.05em` so it covers
   one previous line's height plus a hairline of fade.

### Test

Manual + the demo-mode spec measures the collapsed preview's bounding box
and asserts `height ≥ 24px` (≈ 1.55em at 16px font).

---

## R8 — Human-readable reasoning trace

### Root cause

See [`raw-data/reasoning-step-emission.md`](raw-data/reasoning-step-emission.md):

- `naturalizeThinkingStep()` interpolated the raw `(@USER OP:Compute ?term)`
  Links tuple into the human text.
- `PREFERENCE_DEFAULTS.thinkingDetailLevel` was `"detailed"` (the noisiest).
- The `OP:*` verb was never projected to a plain task noun.
- The server-side model already has `level` and `parent_id` — the renderer
  just was not using them.

### Reuse

- Server-side `level`, `parent_id`, and `summary` fields on `ThinkingStep`
  in `src/thinking.rs`, `src/event_log.rs`.
- The i18n catalog and the `check-i18n-catalog.mjs` REQUIRED_KEYS audit.
- `filterThinkingEntriesForDetail` already exists; it just needed the right
  default mode and the right input.

### Plan

1. Change `PREFERENCE_DEFAULTS.thinkingDetailLevel` to `"standard"`.
2. Rewrite `naturalizeThinkingStep` to:
   - Look up the `OP:*` verb in a 9-entry table mapping to i18n keys
     (`formalizeOpGreet`, `…Compute`, etc.).
   - Use the resolved plain noun in the human text — never the raw tuple.
   - Reuse `formalizePlain` / `pendingFormalizing` keys for the connecting
     prose.
3. Add the 9 keys to `src/web/i18n-catalog.lino` in all four locales
   (`en`/`ru`/`zh`/`hi`).
4. Update `tests/e2e/scripts/check-i18n-catalog.mjs`'s `REQUIRED_KEYS`
   allowlist so the new keys are protected from accidental removal.
5. The raw tuple stays available in Diagnostics mode — the existing diagnostics
   panel reads from a different path.

### Test

`check-i18n-catalog.mjs` itself is the regression guard (`pnpm test:i18n` or
the equivalent CI step). Visual inspection: with a default cold start, send
a greeting and verify the trace reads in plain prose with no `OP:`,
`@USER`, `?term`, or angle-bracket syntax.

---

## R9 — Permission CTA + replay deferred terminal task

### Root cause

When a terminal command was issued in Chat mode the worker emitted an
`agent_suggestion` (the renderer accepted the answer text but discarded the
deferred command). After the user granted `shell` and switched to Agent, the
queue was empty, so nothing ran. There was no place to stash the deferred
command; there was no aggregate CTA on the permission panel.

### Reuse

- `executeTerminalCommand` already encapsulates the dispatch-via-provider
  path (`requestDesktopAgentProvider` first, fallback to
  `requestDesktopToolCall(invokeTool)`).
- `DesktopPermissionPanel` already iterates per-tool rows for grant gates.
- The mode toggle is a `role="radio"` group with `aria-checked`.

### Plan

1. Add `pendingAgentTaskRef` + `pendingAgentTask` state. The worker captures
   into this ref when it emits `agent_suggestion` while the user is in Chat
   mode or shell is not granted.
2. `capturePendingAgentTask({command, source})` writes; `clearPendingAgentTask()`
   reads + resets.
3. Render the CTA inside `DesktopPermissionPanel`:
   - With no pending task ⇒ `permissions.action.grantAll` ("Grant all and
     switch to Agent mode").
   - With pending task ⇒ `permissions.action.grantAllAndRun` ("Grant all,
     switch to Agent mode, and run pending task").
4. Click handler `grantAllAndRunPending`:
   - For every entry in `DESKTOP_TOOL_OPTIONS` (six tools), mirror its
     permission to `true` via both refs and state.
   - Flip `modeRef.current = "agent"`, then `setMode("agent")`.
   - If there is a pending task, await one tick to let refs settle, then
     `executeTerminalCommand(task.command, "agent")` and
     `clearPendingAgentTask()`.
5. Default-deny posture is preserved — every grant goes through the same
   `isPermitted` write path the per-tool toggles use.

### Test

[`tests/e2e/tests/issue-541-permissions.spec.js`](../../tests/e2e/tests/issue-541-permissions.spec.js)
mocks the FormalAiDesktop bridge and runs two cases:

- *no pending task*: click CTA ⇒ mode flips to Agent, the gate registry now
  reports `granted: true` for every tool.
- *pending task*: seed the renderer's worker to suggest `ls ~` as
  `agent_suggestion` from Chat mode ⇒ capture stores it. Click CTA ⇒ mode
  flips, all gates grant, mocked `runAgentProvider` observes a call with
  `command = "ls ~"`.

The aria correctness (`role="radio"` + `aria-checked="true"`) is verified by
expectation on the Agent button after the click.

---

## R10 — Case study

The case study is the artefact you are reading. The information-gathering
budget is upstream of execution: the four `raw-data/` files were authored
*before* a single fix landed, so each commit message could reference the
already-documented root cause. The five top-level documents are:

| File | Purpose |
|---|---|
| `README.md` | Executive summary, sub-system map, current-state inventory, acceptance criteria. |
| `requirements.md` | Quote-by-quote requirements extraction. |
| `solution-plans.md` | This file — per-requirement reuse / plan / test. |
| `best-practices.md` | Patterns we leaned on or invented. |
| `proposed-issues.md` | Follow-ups not in scope of this PR. |

---

## Build vs. reuse summary

| Capability | Reused | New |
|---|---|---|
| CSS theming | Three-tier override pattern, dark palette tokens | Five overrides + auto media block |
| Docker dispatch | `runInSandbox`, `runDocker`, sandbox image | `desktop/lib/docker-detect.cjs` (DI factory) |
| Storage subtrees | Electron Chromium-managed IndexedDB / Local Storage | `desktop/lib/data-migration.cjs` (DI factory) + `app.setName('formal-ai')` |
| Demo mode | `demoPreferences` UI, `groupConversations` filter | Demo-conversation ref, `isDemo` flag, sidebar exclusion, auto-disable on click |
| Animation | `prefers-reduced-motion` CSS, preferences | `useMessageReveal`, `usePrefersReducedMotion`, `minMessageAnimationMs` |
| Reasoning trace | Server-side `level`/`parent_id`, i18n catalog | 9 `formalizeOp*` keys × 4 locales, no-tuple naturaliser, new default detail level |
| Permissions UI | `DesktopPermissionPanel`, `executeTerminalCommand`, mode toggle | `pendingAgentTaskRef`, `grantAllAndRunPending`, primary CTA above per-tool rows |
| Tests | Playwright local config, i18n audit script | 3 new e2e specs, 2 new node unit suites, 1 logic experiment |

The single biggest pattern: **every visible UI seam now has a test that fails
on the original bug and passes on the fix.** No surface fix landed without
its regression guard.
