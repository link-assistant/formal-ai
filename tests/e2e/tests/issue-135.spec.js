// @ts-check
//
// Issue #135: a Russian request for a "Playright" script was routed to the
// unknown fallback in the browser demo.
const { test, expect } = require('@playwright/test');

const UNKNOWN_ANSWER_MARKER = 'local Links Notation rules';

async function switchToManualMode(page) {
  const demoToggle = page.locator('.mode-toggle');
  await expect(demoToggle).toContainText(/Demo on|Demo off|Демо/, {
    timeout: 10_000,
  });
  await demoToggle.click();
  await expect(page.locator('[data-testid="demo-status"]')).toHaveText('Manual mode');
  await expect(page.locator('[data-testid="chat-composer-input"]')).toBeEnabled({
    timeout: 5_000,
  });
}

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

test.describe('Issue #135 Playwright script prompt', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('./');
    await expect(page.locator('.app')).toBeVisible({ timeout: 15_000 });
    await switchToManualMode(page);
  });

  test('reported Russian prompt returns a Playwright starter script', async ({ page }) => {
    const reply = await sendPrompt(page, 'Можешь написать мне Playright скрипт?');

    await expect(reply).toContainText('Playwright');
    await expect(reply).toContainText('@playwright/test');
    await expect(reply).toContainText('https://playwright.dev/docs/writing-tests');
    await expect(reply).not.toContainText(UNKNOWN_ANSWER_MARKER);
  });
});
