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
  timeout: 60_000,
  retries: 2,
  reporter: [['html', { open: 'never' }], ['list']],
  use: {
    baseURL: PAGES_URL,
    trace: 'on-first-retry',
  },
  projects: [
    {
      name: 'chromium',
      use: { ...devices['Desktop Chrome'] },
    },
  ],
});
