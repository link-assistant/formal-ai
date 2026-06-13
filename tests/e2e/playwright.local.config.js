// @ts-check
const { defineConfig, devices } = require('@playwright/test');

const PORT = process.env.E2E_PORT || 3456;
const BASE_URL = `http://localhost:${PORT}`;

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
  ],
  timeout: 30_000,
  retries: 1,
  reporter: [['html', { open: 'never' }], ['list']],
  use: {
    baseURL: BASE_URL,
    trace: 'on-first-retry',
  },
  webServer: {
    // The seed mirror under src/web/seed/ is generated from the canonical
    // data/seed/ tree on every server start so we never serve stale data.
    command:
      `bun --cwd ../.. run build:web && ../../scripts/sync-seed.sh && npx serve ../../src/web --listen ${PORT} --no-clipboard`,
    url: BASE_URL,
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
