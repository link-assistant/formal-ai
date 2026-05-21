// @ts-check
const { defineConfig, devices } = require('@playwright/test');

function normalizeBaseUrl(url) {
  return url.endsWith('/') ? url : `${url}/`;
}

const PAGES_URL = normalizeBaseUrl(
  process.env.PAGES_URL || 'https://link-assistant.github.io/formal-ai/',
);

module.exports = defineConfig({
  testDir: './tests',
  testMatch: [
    '**/demo.spec.js',
    '**/multilingual.spec.js',
    '**/connectivity.spec.js',
    '**/issue-193.spec.js',
    '**/issue-205.spec.js',
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
