// @ts-check
const { test, expect } = require('@playwright/test');

const UNKNOWN_ANSWER_MARKER = 'не могу ответить на это по локальным правилам';

async function sendPrompt(page, text) {
  const input = page.locator('[data-testid="chat-composer-input"]');
  await expect(input).toBeEnabled({ timeout: 5_000 });
  await input.fill(text);

  const messages = page.locator('[data-testid="chat-message"]');
  const initialCount = await messages.count();
  await page.locator('[data-testid="chat-composer-submit"]').click();
  await expect(messages).toHaveCount(initialCount + 2, { timeout: 20_000 });

  const lastMessage = messages.last();
  await expect(lastMessage).toHaveClass(/assistant/);
  const body = lastMessage.locator('.markdown-body');
  await expect(body).toBeVisible();
  return body;
}

test.describe('Issue #286 Russian antiregime concept lookup', () => {
  test.beforeEach(async ({ page }) => {
    await page.addInitScript(() => {
      window.localStorage.setItem(
        'formal-ai.preferences.v1',
        'demo_preferences\n  demoMode "off"\n  diagnosticsMode "on"\n  greetingVariations "off"',
      );
    });
    await page.route('**://*.wikipedia.org/**', (route) => route.abort());
    await page.route('**://*.wiktionary.org/**', (route) => route.abort());
    await page.goto('./');
    await expect(page.locator('.app')).toBeVisible({ timeout: 15_000 });
    await expect(page.locator('[data-testid="demo-status"]')).toHaveText('Manual mode');
    await expect(page.locator('.status')).toContainText('wasm worker');
  });

  test('reported prompt resolves from local seed data instead of unknown fallback', async ({
    page,
  }) => {
    const reply = await sendPrompt(page, 'Что такое антирежим?');
    await expect(reply).toContainText('Антирежим');
    await expect(reply).toContainText('политического режима');
    await expect(reply).toContainText('wiktionary.org/wiki/antiregime');
    await expect(reply).not.toContainText(UNKNOWN_ANSWER_MARKER);
  });
});
