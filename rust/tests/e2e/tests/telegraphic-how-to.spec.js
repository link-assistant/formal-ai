// @ts-check
// Issue #481: A telegraphic how-to prompt routes through the procedural worker path.
const { test, expect } = require('@playwright/test');
const { WORKER_READY_TIMEOUT_MS } = require('./support/worker-ready');

async function sendPrompt(page, text) {
  const input = page.locator('[data-testid="chat-composer-input"]');
  await expect(input).toBeEnabled({ timeout: WORKER_READY_TIMEOUT_MS });
  await input.fill(text);

  const messages = page.locator('[data-testid="chat-message"]');
  const initialCount = await messages.count();
  await page.locator('[data-testid="chat-composer-submit"]').click();
  await expect(messages).toHaveCount(initialCount + 2, { timeout: 20_000 });

  const assistantMessage = messages.last();
  await expect(assistantMessage).toHaveClass(/assistant/);
  const body = assistantMessage.locator('.markdown-body');
  await expect(body).toBeVisible();
  return { assistantMessage, body };
}

test.describe('Issue #481 telegraphic how-to prompt', () => {
  test.beforeEach(async ({ page }) => {
    await page.addInitScript(() => {
      window.localStorage.setItem(
        'formal-ai.preferences.v1',
        'demo_preferences\n  demoMode "off"\n  diagnosticsMode "on"\n  greetingVariations "off"',
      );
    });
    await page.route('**/*', (route) => {
      const url = new URL(route.request().url());
      if (['localhost', '127.0.0.1'].includes(url.hostname)) {
        route.continue();
        return;
      }
      route.abort();
    });
    await page.goto('./');
    await expect(page.locator('.app')).toBeVisible({ timeout: 15_000 });
    await expect(page.locator('[data-testid="demo-status"]')).toHaveText('Manual mode');
    await expect(page.locator('.status')).toContainText('wasm worker');
  });

  test('routes "how order ..." through the procedural worker path', async ({ page }) => {
    const { assistantMessage, body } = await sendPrompt(
      page,
      'how order 3d print in nan chang vietnam?',
    );

    await expect(assistantMessage.locator('.intent')).toContainText(
      'intent:procedural_how_to',
    );
    await expect(body).toContainText('order 3d print in nan chang vietnam');
    await expect(assistantMessage).toContainText(
      'web_search:request:how to order 3d print in nan chang vietnam',
    );
  });
});
