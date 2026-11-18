// @ts-check
// Hermetic transport faults exercise the real local CI worker, without GitHub side effects.
const { test, expect } = require('@playwright/test');
const { readFileSync } = require('node:fs');
const { parseRegisteredLanguages, parseSupportedLanguagesFromAgentInfo } = require('../scripts/lino-seed-parser.mjs');
const captured = require('../../fixtures/web/pages-worker-startup-network-error.json');
const { WORKER_READY_TIMEOUT_MS } = require('./support/worker-ready');
const { withWorkerDiagnostics } = require('./support/worker-diagnostics');

test.describe.configure({ retries: 0 });
test.use({ locale: 'en-US' });

const registered_languages = parseRegisteredLanguages(readFileSync(
  require.resolve('../../../../data/seed/languages.lino'), 'utf8'));
const supportedUiLanguages = new Set(parseSupportedLanguagesFromAgentInfo(readFileSync(
  require.resolve('../../../../data/seed/agent-info.lino'), 'utf8')));

async function configurePreferences(page, language) {
  await page.addInitScript(nextLanguage => localStorage.setItem('formal-ai.preferences.v1',
    `demo_preferences\n  demoMode "off"\n  diagnosticsMode "on"\n  uiLanguage "${nextLanguage}"\n  greetingVariations "off"`), language);
}

test('transient captured HTTP failures recover into the real native worker', async ({ page }, testInfo) => withWorkerDiagnostics(page, testInfo, async () => {
  await configurePreferences(page, 'en');
  const attempts = new Map();
  await page.route(url => captured.startupAssets.some(asset => url.pathname.endsWith('/' + asset)), async route => {
    const asset = captured.startupAssets.find(asset => new URL(route.request().url()).pathname.endsWith('/' + asset));
    const count = (attempts.get(asset) || 0) + 1;
    attempts.set(asset, count);
    if (count === 1) await route.fulfill({ status: captured.status, body: 'Temporarily unavailable' });
    else await route.continue();
  });
  await page.goto('./');
  const input = page.locator('[data-testid="chat-composer-input"]');
  await expect(input).toBeEnabled({ timeout: WORKER_READY_TIMEOUT_MS });
  await expect(page.locator('.status')).toContainText('wasm worker');
  expect(await page.evaluate(() => localStorage.getItem('formalAiDiagnosticJsFallback'))).not.toBe('1');
  for (const asset of captured.startupAssets) expect(attempts.get(asset)).toBe(2);
  const messages = page.locator('[data-testid="chat-message"]');
  const before = await messages.count();
  await input.fill('Can you calculate 2 + 2?');
  await page.locator('[data-testid="chat-composer-submit"]').click();
  await expect(messages).toHaveCount(before + 2, { timeout: 20_000 });
  await expect(messages.last()).toContainText('2 + 2 = 4');
}));

for (const language of registered_languages) test(`persistent startup failure is reported after two requests (${language})`, async ({ page }, testInfo) => withWorkerDiagnostics(page, testInfo, async () => {
  await configurePreferences(page, language);
  const asset = captured.startupAssets[1];
  let attempts = 0;
  await page.route(url => url.pathname.endsWith('/' + asset), async route => {
    attempts++;
    await route.fulfill({ status: captured.status, body: 'Temporarily unavailable' });
  });
  await page.goto('./');
  // Partial solver languages use the supported browser locale for the UI.
  const uiLanguage = supportedUiLanguages.has(language) ? language : 'en';
  await expect(page.locator('html')).toHaveAttribute('lang', uiLanguage);
  const failure = page.locator('[data-testid="engine-unavailable"]');
  await expect(failure).toBeVisible({ timeout: WORKER_READY_TIMEOUT_MS });
  await expect(failure).toHaveAttribute('title', /importScripts/);
  await expect(page.locator('.status')).toContainText('engine unavailable');
  await expect(page.locator('[data-testid="chat-composer-input"]')).toBeDisabled();
  expect(attempts).toBe(2);
}));
