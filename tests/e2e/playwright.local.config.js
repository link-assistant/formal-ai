// @ts-check
const { defineConfig, devices } = require('@playwright/test');

module.exports = defineConfig({
  testDir: './tests',
  testMatch: [
    '**/demo.spec.js',
    '**/multilingual.spec.js',
    '**/connectivity.spec.js',
    '**/issue-153.spec.js',
    '**/issue-193.spec.js',
    '**/issue-205.spec.js',
    '**/issue-180.spec.js',
  ],
  timeout: 30_000,
  retries: 1,
  reporter: [['html', { open: 'never' }], ['list']],
  use: {
    baseURL: 'http://localhost:3456',
    trace: 'on-first-retry',
  },
  webServer: {
    // The seed mirror under src/web/seed/ is generated from the canonical
    // data/seed/ tree on every server start so we never serve stale data.
    command:
      'bun --cwd ../.. run build:web && ../../scripts/sync-seed.sh && npx serve ../../src/web --listen 3456 --no-clipboard',
    url: 'http://localhost:3456',
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
