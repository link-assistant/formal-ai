// @ts-check
//
// Issue #464: the WASM-worker demo accepted neither direct clock subtraction
// ("17:30 - 14:00") nor natural-language elapsed-time prompts ("how long is the
// trip?"). Both must route to the calculator instead of the unknown fallback.
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
  await expect(assistantMessage.locator('.markdown-body')).toBeVisible();
  return assistantMessage;
}

test.describe('Issue #464 - clock-time duration routing', () => {
  test.beforeEach(async ({ page }) => {
    await page.addInitScript(() => {
      window.localStorage.setItem(
        'formal-ai.preferences.v1',
        'demo_preferences\n  demoMode "off"\n  diagnosticsMode "off"\n  greetingVariations "off"',
      );
    });
    await page.goto('./');
    await expect(page.locator('.app')).toBeVisible({ timeout: 15_000 });
    await expect(page.locator('[data-testid="demo-status"]')).toHaveText('Manual mode');
    await expect(page.locator('[data-testid="chat-composer-input"]')).toBeEnabled({
      timeout: 5_000,
    });
  });

  test('direct clock subtraction delegates to the calculator', async ({ page }) => {
    const message = await sendPrompt(page, '17:30 - 14:00');
    await expect(message).toContainText('17:30 - 14:00 = 3 hours, 30 minutes');
    await expect(message).not.toContainText('I could not determine');
  });

  test('elapsed-time wording delegates to clock subtraction', async ({ page }) => {
    const message = await sendPrompt(
      page,
      'If a train leaves at 14:00 and arrives at 17:30, how long is the trip?',
    );
    await expect(message).toContainText('17:30 - 14:00 = 3 hours, 30 minutes');
    await expect(message).not.toContainText('I could not determine');
  });
});
