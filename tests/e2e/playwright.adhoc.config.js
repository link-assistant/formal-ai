// @ts-check
const { defineConfig, devices } = require('@playwright/test');

const PORT = process.env.E2E_PORT || 3499;
const ORIGIN = `http://localhost:${PORT}`;
// The web app lives under /app/ (issue #479); see playwright.local.config.js.
const BASE_URL = `${ORIGIN}/app/`;

module.exports = defineConfig({
  testDir: './tests',
  testMatch: ['**/demo.spec.js', '**/multilingual.spec.js'],
  timeout: 30_000,
  retries: 0,
  reporter: [['list']],
  use: {
    baseURL: BASE_URL,
    trace: 'on-first-retry',
  },
  webServer: {
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
