// @ts-check
const { defineConfig, devices } = require('@playwright/test');

const PAGES_URL = process.env.PAGES_URL || 'https://link-assistant.github.io/formal-ai';

module.exports = defineConfig({
  testDir: './tests',
  testMatch: ['**/demo.spec.js', '**/multilingual.spec.js'],
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
