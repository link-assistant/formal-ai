// @ts-check
const { defineConfig, devices } = require('@playwright/test');

const PORT = process.env.E2E_PORT || 3456;
const ORIGIN = `http://localhost:${PORT}`;
// The web app moved from / to /app/ (issue #479); the site root is now the
// landing page. Pointing baseURL at /app/ keeps every relative goto('./') in
// the app specs aimed at the app, while absolute paths like /download/ and
// relative ../tests/ continue to reach their siblings unchanged.
const BASE_URL = `${ORIGIN}/app/`;

module.exports = defineConfig({
  testDir: './tests',
  testMatch: [
    '**/demo.spec.js',
    '**/multilingual.spec.js',
    '**/connectivity.spec.js',
    '**/issue-135.spec.js',
    '**/issue-157.spec.js',
    '**/issue-153.spec.js',
    '**/issue-193.spec.js',
    '**/issue-205.spec.js',
    '**/issue-209.spec.js',
    '**/issue-210.spec.js',
    '**/issue-180.spec.js',
    '**/issue-218.spec.js',
    '**/issue-221.spec.js',
    '**/issue-223.spec.js',
    '**/issue-224.spec.js',
    '**/issue-228.spec.js',
    '**/issue-230.spec.js',
    '**/issue-242.spec.js',
    '**/issue-280.spec.js',
    '**/issue-282.spec.js',
    '**/issue-327.spec.js',
    '**/issue-286.spec.js',
    '**/issue-288.spec.js',
    '**/issue-330.spec.js',
    '**/issue-334.spec.js',
    '**/issue-335.spec.js',
    '**/issue-336.spec.js',
    '**/issue-337.spec.js',
    '**/issue-338.spec.js',
    '**/issue-339.spec.js',
    '**/issue-343.spec.js',
    '**/issue-347.spec.js',
    '**/issue-353.spec.js',
    '**/issue-360.spec.js',
    '**/issue-363.spec.js',
    '**/issue-386.spec.js',
    '**/issue-388.spec.js',
    '**/issue-392.spec.js',
    '**/issue-402.spec.js',
    '**/issue-404.spec.js',
    '**/issue-409.spec.js',
    '**/issue-435.spec.js',
    '**/issue-438.spec.js',
    '**/issue-440.spec.js',
    '**/issue-479.spec.js',
    '**/issue-479-site.spec.js',
    '**/issue-488.spec.js',
    '**/issue-511-cold-start.spec.js',
    '**/issue-513.spec.js',
    '**/issue-514.spec.js',
    '**/issue-518.spec.js',
    '**/issue-541-demo-mode.spec.js',
    '**/issue-541-permissions.spec.js',
    '**/issue-541-theme.spec.js',
    '**/issue-548.spec.js',
    '**/issue-550-chakra-migration.spec.js',
    '**/issue-1963.spec.js',
  ],
  // Per-test cap. A single app spec navigates, waits for the worker to boot,
  // and asserts on one answer — comfortably under 30s even on a cold worker.
  timeout: 30_000,
  // Whole-suite cap so a hung worker or server can never wedge CI indefinitely;
  // it aborts the run instead of waiting for the job-level kill. Sized for the
  // full local matrix (~50 specs, retries:1) with headroom for the build step.
  globalTimeout: 15 * 60_000,
  // Fail individual web-first assertions fast (default is 5s) so flakes surface
  // quickly rather than each burning the full per-test budget.
  expect: { timeout: 10_000 },
  retries: 1,
  reporter: [['html', { open: 'never' }], ['list']],
  use: {
    baseURL: BASE_URL,
    trace: 'on-first-retry',
    // Bound navigation/action waits so a stuck page errors promptly.
    navigationTimeout: 15_000,
    actionTimeout: 10_000,
    // Issue #541 (R5/R6): freshly produced assistant messages stage a reasoning-
    // then-body reveal that hides the answer body via `.is-revealing { display:
    // none }` for the configured animation budget (default 2 s). Headless tests
    // read `innerText()` immediately, which would return an empty string during
    // that window and flake. Emulating prefers-reduced-motion makes
    // `usePrefersReducedMotion()` return true, which short-circuits
    // `useMessageReveal` to "show everything at once" — matching what users with
    // reduced-motion preferences see, and giving tests deterministic text.
    reducedMotion: 'reduce',
  },
  webServer: {
    // The seed mirror under src/web/seed/ is generated from the canonical
    // data/seed/ tree on every server start so we never serve stale data.
    command:
      `bun --cwd ../.. run build:web && ../../scripts/sync-seed.sh && npx serve ../../src/web --listen ${PORT} --no-clipboard`,
    url: ORIGIN,
    reuseExistingServer: false,
    timeout: 15_000,
  },
  projects: [
    {
      name: 'chromium',
      use: { ...devices['Desktop Chrome'] },
    },
  ],
});
