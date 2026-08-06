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
    '**/issue-426.spec.js',
    '**/issue-435.spec.js',
    '**/issue-438.spec.js',
    '**/issue-439.spec.js',
    '**/issue-440.spec.js',
    '**/issue-441.spec.js',
    '**/issue-460.spec.js',
    '**/issue-464.spec.js',
    '**/issue-466.spec.js',
    '**/issue-476.spec.js',
    '**/issue-478.spec.js',
    '**/issue-479.spec.js',
    '**/issue-479-site.spec.js',
    '**/issue-481.spec.js',
    '**/issue-485.spec.js',
    '**/issue-488.spec.js',
    '**/issue-493.spec.js',
    '**/issue-497.spec.js',
    '**/issue-500.spec.js',
    '**/issue-501.spec.js',
    '**/issue-511-cold-start.spec.js',
    '**/issue-513.spec.js',
    '**/issue-514.spec.js',
    '**/issue-518.spec.js',
    '**/issue-535.spec.js',
    '**/issue-541-demo-mode.spec.js',
    '**/issue-541-permissions.spec.js',
    '**/issue-541-theme.spec.js',
    '**/issue-548.spec.js',
    '**/issue-550-chakra-migration.spec.js',
    '**/issue-554-site.spec.js',
    '**/issue-556.spec.js',
    '**/issue-672-theme-snapshots.spec.js',
    '**/issue-672-animation-override.spec.js',
    '**/issue-672-reasoning-hierarchy.spec.js',
    '**/issue-672-migration-replay.spec.js',
    '**/issue-541-permissions-cold-start.spec.js',
    '**/issue-676-thinking-narrative.spec.js',
    '**/issue-687.spec.js',
    '**/issue-707.spec.js',
    '**/issue-709.spec.js',
    '**/issue-708.spec.js',
    '**/issue-747.spec.js',
    '**/issue-759.spec.js',
    '**/issue-776.spec.js',
    '**/issue-845.spec.js',
    '**/issue-864.spec.js',
    '**/issue-870.spec.js',
    '**/issue-890.spec.js',
    '**/issue-896.spec.js',
    '**/issue-1963.spec.js',
  ],
  // Per-test cap. A single app spec navigates, waits for the worker to boot,
  // and asserts on one answer — comfortably under 30s even on a cold worker.
  timeout: 30_000,
  // Whole-suite cap so a hung worker or server can never wedge CI indefinitely;
  // it aborts the run instead of waiting for the job-level kill.
  //
  // Issue #977: this was 15 minutes -- exactly the `timeout-minutes: 15` of the
  // `E2E Tests (local web app)` job, which also has to pay for checkout, bun
  // install, the web bundle build, `npm ci` and the browser install. The job
  // clock therefore always ran out first, and a job killed by `timeout-minutes`
  // is reported as **cancelled**, not failed: run 31073507682 died at test
  // 159/468 and the pipeline showed a green-ish "cancelled" instead of a red
  // failure. `if: failure()` never fired either, so no Playwright report was
  // uploaded. The job cap is now 40 minutes, and this one is deliberately kept
  // well below the remaining budget so *Playwright* aborts first, exits
  // non-zero, and leaves a report behind.
  globalTimeout: 25 * 60_000,
  // Issue #977: the suite is 468 tests. Playwright's default is half the
  // available cores (2 on a 4-vCPU ubuntu-latest runner), which left the suite
  // unable to finish in any reasonable budget. These specs are I/O-bound
  // (navigate, wait for the wasm worker, assert), so one worker per vCPU is the
  // right trade. Locally the default is kept so a dev machine is not saturated.
  workers: process.env.CI ? 4 : undefined,
  // Fail individual web-first assertions fast (default is 5s) so flakes surface
  // quickly rather than each burning the full per-test budget.
  expect: { timeout: 10_000 },
  // Issue #672 (F1): snapshot baselines live in a single reviewable directory
  // next to the specs. The default template appends `{-projectName}` and a
  // platform suffix, which would fork one baseline per OS — pointless for the
  // computed-colour tables this suite snapshots (CSS colours do not vary by
  // platform) and a trap for contributors on macOS whose run would silently
  // write a second baseline instead of failing against the committed one.
  snapshotPathTemplate: '{testDir}/__snapshots__/{testFileName}/{arg}{ext}',
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
      `bun run --cwd ../.. build:web && ../../scripts/sync-seed.sh && npx serve ../../src/web --listen ${PORT} --no-clipboard`,
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
