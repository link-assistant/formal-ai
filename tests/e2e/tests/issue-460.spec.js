// @ts-check
//
// Issue #460: the GitHub Pages WASM-worker demo treated a train meeting word
// problem as `unknown`. This spec drives the real browser worker and asserts
// that the relative-speed solution keeps the requested verification tags.
const { test, expect } = require('@playwright/test');

const REPORTED_PROMPT =
  'Solve this step-by-step, but with verification at each stage:\n' +
  'Problem: "A train leaves Moscow at 60 km/h. Another leaves St. Petersburg ' +
  'at 80 km/h. Distance: 700 km. When/where do they meet?"\n' +
  'Required format: [STEP 1]... [VERIFY] ... [STEP 5] ...\n' +
  'Then: Ask formal-ai to solve SAME problem... Compare approaches.';

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

test.describe('Issue #460 - train meeting word problem', () => {
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

  test('solves with verification tags and meeting location', async ({ page }) => {
    const message = await sendPrompt(page, REPORTED_PROMPT);

    await expect(message).toContainText('[STEP 1]');
    await expect(message).toContainText('[VERIFY]');
    await expect(message).toContainText('700 / (60 + 80) = 5');
    await expect(message).toContainText('5 hours');
    await expect(message).toContainText('300 km from Moscow');
    await expect(message).toContainText('400 km from St. Petersburg');
    await expect(message).toContainText('[COMPARE]');
    await expect(message).not.toContainText("I haven't learned to answer that yet");
    await expect(message).not.toContainText('I could not evaluate');
  });
});
