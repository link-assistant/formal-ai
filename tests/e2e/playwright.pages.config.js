// @ts-check
const { defineConfig, devices } = require('@playwright/test');

function normalizeBaseUrl(url) {
  return url.endsWith('/') ? url : `${url}/`;
}

// PAGES_URL is the deployed site root (the landing page). The web app moved to
// /app/ (issue #479), so the specs that exercise the app run against /app/.
// connectivity.spec.js reaches the sibling /tests/ harness via a relative
// ../tests/, which resolves correctly under the Pages path prefix.
const PAGES_ROOT = normalizeBaseUrl(
  process.env.PAGES_URL || 'https://link-assistant.github.io/formal-ai/',
);
const PAGES_URL = `${PAGES_ROOT}app/`;

module.exports = defineConfig({
  testDir: './tests',
  testMatch: [
    '**/demo.spec.js',
    '**/multilingual.spec.js',
    '**/connectivity.spec.js',
    '**/issue-157.spec.js',
    '**/issue-193.spec.js',
    '**/issue-205.spec.js',
    '**/issue-209.spec.js',
    '**/issue-335.spec.js',
    // The landing + docs chooser pages navigate relative to the /app/ baseURL
    // (../, ../docs/), so the same spec verifies them on the live Pages site.
    '**/issue-479-site.spec.js',
  ],
  // Per-test cap. The live Pages site adds network latency over the local
  // server, so the budget is wider than the local config's 30s.
  timeout: 60_000,
  // Whole-suite cap so a slow or unreachable deployment aborts the run instead
  // of hanging CI. Sized for the smaller Pages matrix with retries:2.
  globalTimeout: 12 * 60_000,
  // Fail individual assertions faster than the per-test budget while still
  // tolerating real network round-trips against the deployed site.
  expect: { timeout: 15_000 },
  retries: 2,
  reporter: [['html', { open: 'never' }], ['list']],
  use: {
    baseURL: PAGES_URL,
    trace: 'on-first-retry',
    navigationTimeout: 30_000,
    actionTimeout: 15_000,
  },
  projects: [
    {
      name: 'chromium',
      use: { ...devices['Desktop Chrome'] },
    },
  ],
});
