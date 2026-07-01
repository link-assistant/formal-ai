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

test.describe('Issue #556 repository lookup response-language follow-up', () => {
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

  test('Russian language request reanswers the prior GitHub lookup', async ({ page }) => {
    const first = await sendPrompt(
      page,
      'ты можешь сделать кодревью https://github.com/netkeep80/anum_docs ?',
    );
    await expect(first.lastMessage.locator('.intent')).toContainText(
      'intent:project_lookup',
    );
    await expect(first.body).toContainText('netkeep80/anum_docs');

    const second = await sendPrompt(page, 'я не понимаю по английски, напиши по русски');
    await expect(second.lastMessage.locator('.intent')).toContainText(
      'intent:project_lookup',
    );
    await expect(second.body).toContainText('Это запрос о репозитории');
    await expect(second.body).toContainText('netkeep80/anum_docs');
    await expect(second.body).not.toContainText('This is a repository lookup');
    await expect(second.lastMessage).toContainText('response_language_followup');
    await expect(second.lastMessage).toContainText('language_to:ru');
  });
});
