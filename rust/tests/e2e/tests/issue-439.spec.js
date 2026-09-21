// @ts-check
const { test, expect } = require('@playwright/test');

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
  return { lastMessage, body };
}

test.describe('Issue #439 Russian length-vs-mass unit question', () => {
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

  test('the worker explains that meters and kilograms measure different dimensions', async ({
    page,
  }) => {
    const { lastMessage, body } = await sendPrompt(page, 'Сколько метров в килограмме?');

    await expect(lastMessage.locator('.intent')).toContainText(
      'intent:unit_incompatibility',
    );
    await expect(body).toContainText('length');
    await expect(body).toContainText('mass');
    await expect(lastMessage).toContainText('unit_incompatibility:');
    await expect(lastMessage).not.toContainText('intent:unknown');
    await expect(body).not.toContainText('Пока нет символьного правила');
  });
});
