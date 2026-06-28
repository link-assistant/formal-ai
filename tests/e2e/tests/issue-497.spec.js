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

  const assistantMessage = messages.last();
  await expect(assistantMessage).toHaveClass(/assistant/);
  const body = assistantMessage.locator('.markdown-body');
  await expect(body).toBeVisible();
  return { assistantMessage, body };
}

test.describe('Issue #497 GitHub repository traffic prompt', () => {
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

  test('answers the reported Russian visitor-visibility question', async ({ page }) => {
    const { assistantMessage, body } = await sendPrompt(
      page,
      'можно ли узнать заходил ли кто либо в твое репо на github?',
    );

    await expect(assistantMessage.locator('.intent')).toContainText(
      'intent:github_repository_traffic',
    );
    await expect(body).toContainText('GitHub');
    await expect(body).toContainText('link-assistant/formal-ai');
    await expect(body).toContainText('docs.github.com/en/rest/metrics/traffic');
    await expect(assistantMessage).toContainText(
      'github_repository_traffic:privacy:no_individual_identity',
    );
  });
});
