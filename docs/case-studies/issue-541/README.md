# Issue 541 Case Study — *Missing UI/UX improvements*

> **Issue:** <https://github.com/link-assistant/formal-ai/issues/541> (`bug`, opened 2026-06-20 by konard)
> **Pull request (this work):** <https://github.com/link-assistant/formal-ai/pull/542> (branch `issue-541-bf330e9c749a`)
> **Case study date:** 2026-06-20
> **Type:** Multi-symptom UX bug report + deep case study, requirements decomposition, and a single-PR implementation.
> **Status:** All nine requirements (R1–R9) implemented and tested on this branch; case study + analysis delivered in the same PR.

All raw, third-party captures referenced below live under [`raw-data/`](raw-data/).

| Artifact | Path |
|---|---|
| The issue, as filed (JSON) | [`raw-data/issue-541.json`](raw-data/issue-541.json) |
| Online research (Electron storage, design tokens, AI-chat timing) | [`raw-data/background-research.md`](raw-data/background-research.md) |
| Read-only investigation of desktop persistence (R3 root cause) | [`raw-data/desktop-storage-investigation.md`](raw-data/desktop-storage-investigation.md) |
| Read-only investigation of reasoning-step emission (R8 root cause) | [`raw-data/reasoning-step-emission.md`](raw-data/reasoning-step-emission.md) |
| Issue screenshots (overview, permissions panel, reasoning trace) | [`assets/screenshot-0-overview.png`](assets/screenshot-0-overview.png), [`assets/screenshot-1-permissions.png`](assets/screenshot-1-permissions.png), [`assets/screenshot-2-reasoning.png`](assets/screenshot-2-reasoning.png) |
| **Full requirement inventory (R1–R9)** | [`requirements.md`](requirements.md) |
| **Per-requirement root cause + fix + test** | [`solution-plans.md`](solution-plans.md) |
| **Patterns and best practices applied** | [`best-practices.md`](best-practices.md) |
| **Possible follow-up issues + scope discussion** | [`proposed-issues.md`](proposed-issues.md) |

---

## 1. Summary

A maintainer filed nine related complaints after a manual smoke pass of the
desktop app. The complaints span four distinct subsystems — CSS theming,
Electron persistence, the desktop Docker probe, and the chat / reasoning UI —
but they share a common shape: **the user-visible behaviour drifted from the
designed behaviour without anyone noticing, because nothing in the test suite
pinned it**. Concretely, the screenshots and bullet list cover:

- mixed light/dark surfaces after switching the theme (R1);
- `Docker unavailable` displayed when Docker Desktop is installed and running (R2);
- previous desktop conversations lost across app upgrades (R3);
- Demo mode overwriting the user's real conversation (R4);
- reasoning + answer reveal that fires instantly so the user cannot tell anything
  was thought about (R5/R6);
- collapsed reasoning preview that clips below one full step (R7);
- reasoning trace that exposes raw symbolic Links tuples and defaults to the
  noisiest detail level (R8);
- a permission panel that, after the user grants `shell`, just sits there — the
  pending terminal task is dropped and the user has to re-type the prompt (R9).

The issue text is explicit:

> *"We need to download all logs and data related about the issue to this
> repository, … compile that data to `./docs/case-studies/issue-{id}` folder, and
> use it to do deep case study analysis … reconstruct timeline/sequence of
> events, list of each and all requirements from the issue, find root causes of
> the each problem, and propose possible solutions and solution plans for each
> requirement … Please plan and execute everything in this single pull
> request."*

So unlike #511 (which was decomposed into a multi-PR epic), the instruction
here is to land **everything in PR #542**. This case study is the analysis
half of that instruction; [`requirements.md`](requirements.md),
[`solution-plans.md`](solution-plans.md), and the implementation commits on the
branch are the execution half.

---

## 2. What actually happened (root-cause map)

The nine complaints split cleanly along subsystem lines, which is also how the
fixes are organised. Each root cause is summarised below; the full chain — bug,
quoted code, fix, test — lives in [`solution-plans.md`](solution-plans.md).

| Req | Subsystem | Root cause (one-line) | Where it lives |
|---|---|---|---|
| R1 | CSS theme | Five widgets (`mode-status`, `sidebar-toggle.is-collapsed`, `drawer-menu-section h2`, `tool-mode`, `tool-mode-agent .tool-mode`) had hardcoded light hex values with NO `:root[data-theme="dark"]` or `@media (prefers-color-scheme: dark)` counterpart. | `src/web/styles.css` |
| R2 | Desktop / Docker | Bare `spawn("docker")` plus a result that was memoised on the first call. A GUI-launched `.app` does not inherit the user's interactive shell PATH, so `/usr/local/bin/docker` was unreachable; even after Docker Desktop started, the cached "missing" verdict never expired. | `desktop/main.cjs` (now via `desktop/lib/docker-detect.cjs`) |
| R3 | Desktop / Electron | `userData` directory was derived from `productName`. Any rebrand or spacing/casing fix orphans the Chromium IndexedDB / Local Storage subtree. There was no `app.setName`, no `app.setPath`, no migration step. | `desktop/main.cjs` (now via `desktop/lib/data-migration.cjs`) |
| R4 | Web / persistence | Demo mode wrote into whatever conversation was current. Toggling it off left the demo turns mixed into the user's thread; the sidebar surfaced demo conversations as if they were navigable; clicking a real conversation while demo was on appeared to "overwrite" it. | `src/web/app.js`, `src/web/memory.js` |
| R5/R6 | Web / animation | Once the deterministic engine had an answer, the renderer set `messages` and the React tree rendered the reasoning + body in the same paint. There was no animation budget, no per-step pacing, no reveal gate on the answer body. | `src/web/app.js` |
| R7 | Web / CSS | `.thinking-preview-current` was `white-space:nowrap; text-overflow:ellipsis`, so a wrapped step was clipped to a single line; `.thinking-preview-previous` had `max-height:0.75em`, half a line. | `src/web/styles.css` |
| R8 | Web / app + i18n | `naturalizeThinkingStep()` interpolated the raw Links tuple (`(@USER OP:… ?term)`) into the human text, `thinkingDetailLevel` defaulted to `"detailed"`, and the `OP:*` verb was never projected to a plain task noun. | `src/web/app.js`, `src/web/i18n-catalog.lino` |
| R9 | Web / permissions | When a terminal command was issued in Chat mode the worker emitted an `agent_suggestion`; the renderer accepted that answer but threw the original command away. After the user granted `shell` and flipped to Agent, the queue was empty so nothing ran. There was no `pendingAgentTask` slot, and the permission panel had no aggregate CTA. | `src/web/app.js` |

The **deeper** root cause is shared: the desktop app shipped *visible* surface
area (theme switcher, mode toggle, permission panel, animation budget) that had
no automated coverage proving the surface matched the user-facing contract. Every
fix below is paired with a test (Playwright e2e, Node unit, or a recorded
fixture) so the same drift cannot recur silently.

---

## 3. Current-state inventory (what already exists — reuse, don't rebuild)

The single most important finding of this study, mirroring #511: **most of the
primitives the issue needs already exist.** Issue #541 is a *finishing* and
*coverage* problem on top of shipped UI infrastructure, not a green-field build.

| Capability the issue needs | Already in repo? | Where | Gap for #541 |
|---|---|---|---|
| CSS theming with three layers (light base, `[data-theme="dark"]` override, `@media (prefers-color-scheme: dark)`) | ✅ | `src/web/styles.css` (theme system at lines 3168, 3176, 3226, 3493, 3498) | Five primary widgets had no dark/auto counterpart — patch into the existing pattern, no chakra migration needed |
| Electron sandboxed user-data directory + IPC bridge | ✅ | `desktop/main.cjs`, `desktop/preload.cjs` | No `app.setName`, no `app.setPath`, no migration; introduce `data-migration.cjs` + pin to `formal-ai` |
| Docker probe with sandbox dispatch (`runInSandbox`) | ✅ | `desktop/main.cjs` (`runInSandbox`, `runDocker`) using `konard/box-dind` | Probe used bare `spawn("docker")` and memoised; replace with `docker-detect.cjs` (well-known paths + TTL) |
| Demo-mode preference + scripted turns | ✅ | `src/web/app.js` `demoPreferences`, demo greeting variations | Demo turns persisted into whatever conversation was current; needs a dedicated, sidebar-invisible demo conversation |
| Reasoning-step model with `level` + `summary` + `parent_id` | ✅ | `src/thinking.rs`, `src/event_log.rs` (curate_thinking_event), `src/web/formal_ai_worker.js` (`HIGH_LEVEL_THINKING_STEPS`, `withThinkingLevels`) | Web `naturalizeThinkingStep()` used the raw tuple; default detail level was `"detailed"`; needs i18n keys for the operation noun |
| Per-tool permission panel + grant gates | ✅ | `src/web/app.js` `DesktopPermissionPanel`, `desktop/lib/tool-router.cjs` `isPermitted` | No "Grant all" affordance; no `pendingAgentTask` slot; no queue replay |
| `ThinkingPreview` component (rotated-scroll one-step display) | ✅ | `src/web/app.js` `ThinkingPreview`, `src/web/styles.css` `.thinking-preview-*` | Clipped to a single line and 0.75em previous; needs height to fit one full step |
| `useMessageReveal` + `usePrefersReducedMotion` | ❌ (new) | n/a | New hooks added in `src/web/app.js`; reuse the existing `prefers-reduced-motion` media query pattern from the CSS |
| Preference store (Links Notation in `localStorage`) | ✅ | `src/web/preferences.js`, `PREFERENCE_DEFAULTS` | Add `minMessageAnimationMs` and change `thinkingDetailLevel` default to `"standard"` |
| i18n catalog (`lino` + `check-i18n-catalog.mjs` allowlist) | ✅ | `src/web/i18n-catalog.lino`, `src/web/i18n-catalog-permissions.lino`, `tests/e2e/scripts/check-i18n-catalog.mjs` | Add 13 keys for operation nouns, animation labels, and grant-all CTA |
| Playwright e2e harness (`playwright.local.config.js`, `testMatch`) | ✅ | `tests/e2e/playwright.local.config.js` | Add three new specs (`issue-541-demo-mode`, `issue-541-permissions`, `issue-541-theme`) to `testMatch` |

The maintainer's first instinct on R1 — *"may be we should migrate to chakra
ui.com, so we do everything right"* — would be a costly rebuild given the no-build
hyperscript React app (see [`raw-data/background-research.md`](raw-data/background-research.md) §2:
Chakra requires JSX + Emotion + framer-motion bundling). The existing CSS
custom-property tokens already give us "define once, override per theme"; the
fix is to finish the override coverage, not replace the system.

---

## 4. Requirements (summary — full inventory in [`requirements.md`](requirements.md))

The issue body yields **nine functional requirements** plus a meta-requirement
to author this case study. The full inventory, with verbatim source quotes and
acceptance criteria, is in [`requirements.md`](requirements.md). In brief:

- **A. Theming (R1):** every primary widget honours the active theme; no
  hardcoded light values bleed through in dark or auto mode.
- **B. Desktop reliability (R2, R3):** Docker availability is detected correctly
  across GUI-launched OS paths and re-probed on a TTL; user conversations
  survive app upgrades and rebrand-style userData renames via a pinned name +
  non-destructive migration.
- **C. Demo mode safety (R4):** demo turns never touch the user's real
  conversation; demo conversations are hidden from the sidebar; demo mode
  auto-disables when the user navigates to a real thread.
- **D. Reasoning/answer UX (R5, R6, R7, R8):** there is a minimum animation
  budget (default 2 s, 0 = immediate) honoured by `prefers-reduced-motion`; the
  reasoning steps reveal first, then the answer body fades in; collapsed
  reasoning shows at least one full step; reasoning steps read as plain
  language at every detail level, with the diagnostics tuple available only
  in Diagnostics mode; the default detail level is the 50% midpoint
  (`"standard"`).
- **E. Permissions UX (R9):** the permission panel has a single primary CTA
  that grants every desktop tool, switches to Agent mode, and (if a task was
  deferred while in Chat / under-granted mode) replays it through
  `executeTerminalCommand` in one click.
- **F. Process (R10):** compile this case study folder; produce a per-requirement
  root-cause + fix + test analysis; verify all in CI; ship in PR #542.

---

## 5. Recommended solution shape (detail in [`solution-plans.md`](solution-plans.md))

Every fix lands inside an existing subsystem rather than introducing a parallel
one. The guiding principle: **add the smallest seam that makes the failure
mode impossible**, paired with a test that reproduces the *specific* bug rather
than locking the entire surface in place.

1. **CSS theme finish, not framework swap (R1).** Add five `:root[data-theme="dark"]`
   rules + a matching `@media (prefers-color-scheme: dark) { :root:not([data-theme="light"]) … }`
   block at the end of `src/web/styles.css`. Test reads `getComputedStyle()` on
   each widget under `theme "dark"` and asserts it is in the dark band, not the
   light hex.
2. **Inject side-effects, test in isolation (R2, R3).** Move `docker` resolution
   and the legacy-profile migration into `desktop/lib/docker-detect.cjs` and
   `desktop/lib/data-migration.cjs` respectively. Both accept injected
   `fs` / `spawnSync` / `now` / `app` so Node unit tests can exercise every
   branch without a real Electron profile or Docker daemon. Pin the userData
   name to `formal-ai` via `app.setName` *before* the `ready` event so future
   renames cannot break the path. Migration is non-destructive (legacy copy
   stays put) and stamped via `formal-ai-data-version.json` so future schema
   migrations are deterministic.
3. **Dedicated demo conversation (R4).** Maintain a sidebar-invisible demo
   conversation per session; route all demo turns there; auto-disable demo when
   the user clicks a real conversation; tag every persisted memory event with
   `isDemo: true` so the IndexedDB roundtrip preserves the flag.
4. **Reveal hook + reduced-motion guard (R5, R6).** Introduce `useMessageReveal`
   (steps unveil over 72 % of the budget; body fades in for the remaining 28 %)
   and `usePrefersReducedMotion`. New preference `minMessageAnimationMs`
   (default 2000, clamp 0–8000). Mark freshly produced messages with
   `animateReveal: true`; hydrated history never gets the flag so reloads are
   instant.
5. **Collapsed-preview height (R7).** Drop the `nowrap`/`ellipsis` clip on the
   current step, give `.thinking-preview-current` `min-height: 1.55em`, and
   move the rotated-scroll fade to the previous-step element.
6. **Naturalise the trace + new default (R8).** Map `OP:*` verbs to plain task
   nouns via 9 i18n catalog keys (`formalizeOpGreet`, `…OpCompute`, etc.) per
   locale; drop the raw tuple from the public view (still available in
   Diagnostics); set `PREFERENCE_DEFAULTS.thinkingDetailLevel = "standard"`.
7. **`pendingAgentTask` + Grant-all CTA (R9).** Stash a deferred shell command
   in `pendingAgentTaskRef`; render a primary CTA on the permission panel that
   mirrors all six `DESKTOP_TOOL_OPTIONS` to `true`, flips mode to `agent`, and
   calls `executeTerminalCommand(task.command, "agent")`. Sync refs *before*
   triggering React state so the replay does not race the next render.
8. **Prove it.** Three new Playwright e2e specs (`issue-541-theme`,
   `issue-541-permissions`, `issue-541-demo-mode`) and two new Node unit-test
   suites (`docker-detect.test.mjs`, `data-migration.test.mjs`) plus a logic
   experiment (`experiments/reveal-budget-logic.test.mjs`) cover the nine
   requirements end-to-end.

---

## 6. Constraints & non-negotiables

- **No build step.** The web app is hyperscript React via `h()` directly out of
  `src/web/app.js`. Any fix that requires JSX, Emotion, or a bundler is
  inadmissible (R1 chakra discussion in
  [`raw-data/background-research.md`](raw-data/background-research.md) §2).
- **i18n parity.** Every new user-facing string must land in all four locales
  (`en`/`ru`/`zh`/`hi`) inside `src/web/i18n-catalog.lino` and pass the
  `check-i18n-catalog.mjs` `REQUIRED_KEYS` allowlist. The no-hardcoded-UI-strings
  CI guard (introduced in #511's E2 follow-up) rejects any prose literal passed
  to `h(...)` as a child.
- **Non-destructive migration.** The legacy profile copy must never be deleted.
  Worst-case, the user keeps two profiles instead of zero.
- **Reduced-motion respected.** R5/R6 must short-circuit to "show everything
  immediately" when `prefers-reduced-motion: reduce` is in effect.
- **Default-deny still rules permissions (R9).** The grant-all CTA records
  grants through the existing `isPermitted` gate — it does not bypass it.
- **Hermetic tests.** Every new Playwright spec runs against the in-process
  worker with `localStorage` + IndexedDB mocked; the Docker / Electron unit
  tests inject `fs` and `spawnSync` so they never touch the host environment.

---

## 7. How this PR delivers the whole feature

Unlike #511 (an agent-execution epic spanning three repos and a Docker image),
issue #541 is a finishing pass: every fix is *inside* this repo, *inside* a
subsystem that already exists. Per the maintainer's instruction (*"Please plan
and execute everything in this single pull request"*), all nine requirements
land on the `issue-541-bf330e9c749a` branch in seven commits, each tagged with
the requirement IDs it satisfies:

| Commit | Tag | Delivers |
|---|---|---|
| `6454154c` | (R2, #541) | `desktop/lib/docker-detect.cjs` + `desktop/scripts/docker-detect.test.mjs` + main.cjs rewire; opt-in `FORMAL_AI_DESKTOP_DEBUG`, `FORMAL_AI_DOCKER_BIN`. |
| `62a1e7ae` | (R5/R6/R7, #541) | `useMessageReveal`, `usePrefersReducedMotion`, `minMessageAnimationMs` preference, collapsed-preview height fix, `experiments/reveal-budget-logic.test.mjs`. |
| `70b37a0d` | (R8, #541) | `naturalizeThinkingStep` rewrite (no raw tuple), 9 `formalizeOp*` i18n keys × 4 locales, `thinkingDetailLevel` default → `"standard"`. |
| `52cdf3f7` | (R3, #541) | `desktop/lib/data-migration.cjs` + `desktop/scripts/data-migration.test.mjs`; pinned `app.setName("formal-ai")`; legacy profile names migrated non-destructively; `formal-ai-data-version.json` stamp. |
| `c53ed080` | (R4, #541) | Dedicated demo conversation, `isDemo` event flag, sidebar hides demo turns, auto-disable demo on real-conversation click, `tests/e2e/tests/issue-541-demo-mode.spec.js`. |
| `1722d764` | (R9, #541) | `pendingAgentTaskRef`, `grantAllAndRunPending`, permission-panel `onGrantAll` CTA, two new i18n keys, `tests/e2e/tests/issue-541-permissions.spec.js`. |
| `151865c7` | (R1, #541) | Five dark + auto overrides in `styles.css`, `tests/e2e/tests/issue-541-theme.spec.js`. |

The case-study folder (`docs/case-studies/issue-541/`) and changelog
(`changelog.d/20260620_ui_ux_improvements_issue_541.md`) carry the analysis
deliverables.

---

## 8. Acceptance criteria for "issue #541 fully done" — all met

The issue is complete when, on a cold launch of the desktop app on the
`issue-541-bf330e9c749a` branch:

1. ✅ Switching theme to dark (or letting auto track a dark OS) leaves no
   primary widget on a light surface — `tests/e2e/tests/issue-541-theme.spec.js`
   reads `getComputedStyle()` on the topbar status badge and the collapsed
   sidebar toggle and asserts the dark band. (R1)
2. ✅ With Docker Desktop installed and running, `dockerIsAvailable()` returns
   `true` even from a GUI launch, and a Docker daemon started *after* the app
   opens is detected on the next probe (TTL re-probe). The fix and every code
   path are covered by `desktop/scripts/docker-detect.test.mjs`. (R2)
3. ✅ Upgrading from any historical `<productName>` profile to the pinned
   `formal-ai` profile copies IndexedDB + Local Storage forward without
   deleting the legacy copy; `desktop/scripts/data-migration.test.mjs`
   exercises every branch with injected `fs`. (R3)
4. ✅ Enabling Demo mode from inside a real conversation spawns a dedicated,
   sidebar-invisible demo conversation; switching demo off restores the
   user's thread exactly; clicking any conversation in the sidebar
   auto-disables demo — verified by `tests/e2e/tests/issue-541-demo-mode.spec.js`. (R4)
5. ✅ A freshly produced assistant message reveals its reasoning steps first,
   then fades in its body, across `minMessageAnimationMs` wall-clock (default
   2 s, `0` = immediate, capped at 8 s); `prefers-reduced-motion: reduce`
   short-circuits to instant display. Pure-logic tests in
   `experiments/reveal-budget-logic.test.mjs`. (R5, R6)
6. ✅ Collapsed reasoning preview shows the current step in full — at least one
   wrapped paragraph — even when the step is long. (R7)
7. ✅ The reasoning trace defaults to the `"standard"` (50 % midpoint) detail
   level and shows plain human language at every level; the raw symbolic
   tuple stays in Diagnostics mode only. (R8)
8. ✅ The permission panel has a single primary CTA — *"Grant all and switch
   to Agent mode"* (or *"… and run pending task"* when a command is queued) —
   that grants all six desktop tools, flips mode to Agent, and replays the
   pending shell command through `executeTerminalCommand` in one click —
   verified by `tests/e2e/tests/issue-541-permissions.spec.js`. (R9)
9. ✅ This case-study folder exists with the raw inputs, the inventory, the
   per-requirement root-cause + fix + test analysis, the patterns learned,
   and a note on follow-up issues. (Meta)

All nine hold, so the feature ships in PR #542.
