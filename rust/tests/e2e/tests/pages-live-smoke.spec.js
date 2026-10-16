// @ts-check
// One live context verifies delivery and the real engine without a CDN load test.
const { test, expect } = require('@playwright/test');
const { WORKER_READY_TIMEOUT_MS } = require('./support/worker-ready');

test('live Pages serves landing, native-engine chat, and documentation', async ({ page }) => {
  await page.addInitScript(() => {
    window.localStorage.setItem(
      'formal-ai.preferences.v1',
      'demo_preferences\n  demoMode "off"\n  diagnosticsMode "on"\n  uiLanguage "en"\n  greetingVariations "off"',
    );
  });

  const landing = await page.goto('../');
  expect(landing.status()).toBe(200);
  await expect(page.locator('[data-testid="landing-app-summary"]')).toBeVisible();

  const app = await page.goto('./');
  expect(app.status()).toBe(200);
  await expect(page.locator('.app')).toBeVisible();
  expect(await page.evaluate(() => window.localStorage.getItem('formalAiDiagnosticJsFallback'))).not.toBe('1');
  await expect(page.locator('[data-testid="demo-status"]')).toHaveText('Manual mode');
  const input = page.locator('[data-testid="chat-composer-input"]');
  await expect(input).toBeEnabled({ timeout: WORKER_READY_TIMEOUT_MS });
  await expect(page.locator('.status')).toContainText('wasm worker');

  const messages = page.locator('[data-testid="chat-message"]');
  const before = await messages.count();
  await input.fill('Can you calculate 2 + 2?');
  await page.locator('[data-testid="chat-composer-submit"]').click();
  await expect(messages).toHaveCount(before + 2, { timeout: 20_000 });
  const answer = messages.last();
  await expect(answer).toHaveClass(/assistant/);
  await expect(answer).toContainText('2 + 2 = 4');
  await expect(answer).not.toContainText('engine unavailable');
  await expect(answer).not.toContainText('diagnostic override');

  const docs = await page.goto('../docs/');
  expect(docs.status()).toBe(200);
  await expect(page.locator('[data-testid="docs-app-summary"]')).toBeVisible();
  await expect(page.locator('.hero h1')).toHaveText('Documentation');
});
