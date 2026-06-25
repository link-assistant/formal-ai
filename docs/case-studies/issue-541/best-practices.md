# Issue #541 — Best Practices & Patterns

The fixes for R1–R9 reach across CSS, Electron main-process JS, the
hyperscript React renderer, the IndexedDB memory store, the i18n catalog, and
the Playwright harness. Six patterns recur often enough to call out as the
"house style" they enforce.

---

## 1. The DI factory pattern for desktop modules

Both new desktop modules (`desktop/lib/docker-detect.cjs`,
`desktop/lib/data-migration.cjs`) follow the same shape, first introduced by
`desktop/lib/tool-router.cjs` in #511's E1 work:

```js
function createDockerDetector({
  env = process.env,
  platform = process.platform,
  spawnSync = require('node:child_process').spawnSync,
  existsSync = require('node:fs').existsSync,
  now = Date.now,         // <-- *must* be injected; never call Date.now directly
  log = () => {},
  okTtlMs = 30_000,
  failTtlMs = 3_000,
  probeTimeoutMs = 2_000,
} = {}) {
  // ... module state captured in closure ...
  return { dockerIsAvailable, resolveDockerBinary };
}
module.exports = { createDockerDetector };
```

**Why it matters for #541:**

- Every probe / migration step is unit-testable without a real Electron
  profile, a real Docker daemon, or a real filesystem.
- `desktop/scripts/docker-detect.test.mjs` and
  `desktop/scripts/data-migration.test.mjs` are plain Node test scripts —
  no Playwright, no Electron, no sandbox.
- Calling `now: () => 1` lets the test time-travel the TTL cache
  deterministically.

**Rule of thumb:** any new desktop main-process module that touches `fs`,
`child_process`, or `app` ships with a `createXxx` factory and an injected
clock. Calls to `Date.now()` directly inside the module body fail
review.

---

## 2. The ref-mirror pattern for state used in async callbacks

The React tree in `src/web/app.js` uses functional `useState` plus a
parallel `xxxRef` mirrored via `useEffect`:

```js
const [mode, setMode] = useState('chat');
const modeRef = useRef(mode);
useEffect(() => { modeRef.current = mode; }, [mode]);
```

`grantAllAndRunPending` (R9) is the canonical place this matters. The
click handler needs to:

1. Grant six permissions.
2. Flip mode to Agent.
3. Execute the queued terminal command.

If we read `mode` inside the handler, we read the value from the closure
captured at the previous render — *not* the value we just set. The ref
fixes that: the handler writes to `modeRef.current` and *also* calls
`setMode`, then `executeTerminalCommand` reads `modeRef.current` and gets
the just-set value.

**Pattern usage in #541:**

- `permissionsRef` mirrors `permissions`; the CTA writes to both before
  calling `executeTerminalCommand`.
- `modeRef` mirrors `mode`; same shape.
- `pendingAgentTaskRef` mirrors `pendingAgentTask`; capture writes to both
  so the worker callback (which runs async) can read it without waiting on
  React's render schedule.

**Rule of thumb:** any state that is read inside an `await` chain, a
worker callback, or a synchronous click handler that immediately calls
into another effect-using subroutine should be ref-mirrored. Reads inside
pure render code stay on the state value.

---

## 3. Reveal under a budget — 72 % / 28 % split

`useMessageReveal` (R5/R6) splits an animation budget deterministically:

```js
const stepShareMs = budgetMs * 0.72;
const bodyShareMs = budgetMs * 0.28;
const perStepMs   = stepShareMs / Math.max(1, stepCount);
```

The 72/28 split is not arbitrary — it gives the body fade-in enough time
to perceptually settle (≈ 560 ms at a 2 s budget) while leaving most of the
wall-clock to the actual *reasoning* (the user-facing point of the
animation). See [`raw-data/background-research.md`](raw-data/background-research.md)
§6 for the underlying timing research (Nielsen, OpenAI streaming).

**Rule of thumb:** when an animation has multiple visible phases, allocate
the budget per-phase as ratios, not absolute milliseconds. Absolute values
do not scale across `0` (immediate), the default, and the maximum
(`prefers-reduced-motion: reduce` ⇒ 0).

---

## 4. Theme coverage audit — measure the seam, not the value

R1's test
([`tests/e2e/tests/issue-541-theme.spec.js`](../../tests/e2e/tests/issue-541-theme.spec.js))
does NOT assert specific rgb values. It asserts the *band*:

```js
function isMutedDarkText(rgb)  { return rgb.r >= 170 && rgb.g >= 170 && rgb.b >= 170; }
function isDarkSurface(rgb)    { return rgb.r < 70  && rgb.g < 70  && rgb.b < 70;  }
```

The light bug values (`#40515f` text, `#eef2f1` surface) fail both bands;
the dark palette values (`#c9c1b6` text, `#222624` surface) pass them.
Palette tweaks remain free; the *seam* — "dark surface, light text bleeds
through" — stays locked.

**Rule of thumb:** UI assertions on color, size, or motion should pin the
behaviour, not the specific value. Tests that fail every time a designer
tweaks a hex are tests nobody runs.

---

## 5. Non-destructive migrations + version stamp

`data-migration.cjs` (R3) copies subtrees forward and *never* deletes the
source. The stamp file `formal-ai-data-version.json` carries `DATA_VERSION`
+ `migratedAt`, and the migration is short-circuited when the stamp is
already present.

**Why this matters:**

- If the copy fails midway, the user keeps their data in the legacy
  location.
- If the new schema is broken, the user can roll back the app and
  resume against the legacy profile.
- Future migrations bump `DATA_VERSION` and migrate from the highest
  stamp forward — no "did we run version 1?" guesswork.

**Rule of thumb:** every step that *might* lose user data must be
non-destructive *and* idempotent. If a step needs to delete (truly,
e.g. unused cache), it goes in a separate, opt-in "vacuum" routine, not
the migration path.

---

## 6. i18n parity is enforced, not hoped for

Every new user-facing string in this PR landed in all four locales
(`en`/`ru`/`zh`/`hi`) inside `src/web/i18n-catalog.lino` *and* was added to
`tests/e2e/scripts/check-i18n-catalog.mjs`'s `REQUIRED_KEYS`. The audit
script fails if a required key is missing from any locale.

The complementary guard, `check-web-hardcoded-ui-strings.mjs` (introduced in
#511's E2 follow-up), rejects any string literal passed to `h(...)` as a
child. Together they guarantee:

- No new user-visible string ships untranslated.
- No new user-visible string ships outside the catalog.

**Rule of thumb:** if a fix introduces new UI prose, the same commit
updates the catalog *and* the audit allowlist. Tests are not a
formality — they are the *only* mechanism that prevents one-locale
regressions.

---

## Verification checklist (the same one used during PR review)

When reviewing this PR (or any future PR that touches the same surface
area), run through this list:

- [ ] Every preference touched has a default in `PREFERENCE_DEFAULTS`?
- [ ] Every new state with an async/click reader has a ref mirror?
- [ ] Every new user-facing string is in all four locales *and* on the
      REQUIRED_KEYS allowlist?
- [ ] Every new desktop main-process module is a DI factory with an
      injected clock?
- [ ] Every animation honours `prefers-reduced-motion: reduce`?
- [ ] Every CSS rule for a primary widget has a dark counterpart *and*
      an auto-mode counterpart?
- [ ] Every migration is non-destructive, stamped, and idempotent?
- [ ] Every UI bug fix has a Playwright spec that fails on the bug and
      passes on the fix?
- [ ] No `Date.now()` / `Math.random()` calls inside testable module
      bodies (must be injected)?
- [ ] No hardcoded `'docker'` PATH lookup — go through
      `dockerDetector.resolveDockerBinary()` instead?

If any answer is "no", the patch is not done.
