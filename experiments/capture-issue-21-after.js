// Standalone Playwright capture for the issue-21 case study.
// Run via:
//   cd tests/e2e && NODE_PATH=$(pwd)/node_modules \
//     node ../../experiments/capture-issue-21-after.js
//
// Expects `npx serve src/web --listen 3456` (or the local Playwright web
// server) to be reachable at http://localhost:3456. The `@playwright/test`
// package lives under tests/e2e/node_modules, hence the NODE_PATH override.
const path = require('node:path');
const { chromium } = require('@playwright/test');

const SUMMARY_JSON = {
  title: 'Изумруд',
  extract:
    'Изумруд — драгоценный камень берилловой группы зелёного цвета, окраска которого обусловлена примесями хрома.',
  type: 'standard',
  content_urls: {
    desktop: {
      page: 'https://ru.wikipedia.org/wiki/%D0%98%D0%B7%D1%83%D0%BC%D1%80%D1%83%D0%B4',
    },
  },
};

(async () => {
  const browser = await chromium.launch();
  const context = await browser.newContext();
  const page = await context.newPage();
  await page.route('**/api/rest_v1/page/summary/**', async (route) => {
    await route.fulfill({
      status: 200,
      contentType: 'application/json',
      body: JSON.stringify(SUMMARY_JSON),
    });
  });
  await page.goto('http://localhost:3456/');
  await page.locator('.app').waitFor({ timeout: 15_000 });
  const toggle = page.locator('.mode-toggle');
  await toggle.click();
  const input = page.locator('[data-testid="chat-composer-input"]');
  await input.waitFor({ state: 'attached', timeout: 5_000 });
  await input.fill('Что такое изумруд?');
  await page.locator('[data-testid="chat-composer-submit"]').click();
  const messages = page.locator('[data-testid="chat-message"]');
  await messages.last().waitFor({ timeout: 20_000 });
  await page.waitForFunction(
    () =>
      Array.from(document.querySelectorAll('[data-testid="chat-message"]'))
        .map((node) => node.textContent || '')
        .some((text) => text.includes('Изумруд')),
    null,
    { timeout: 20_000 },
  );
  const out = path.resolve(
    __dirname,
    '..',
    'docs/case-studies/issue-21/screenshots/after.png',
  );
  await messages.last().screenshot({ path: out });
  console.log('wrote', out);
  await browser.close();
})();
