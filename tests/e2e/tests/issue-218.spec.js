// @ts-check
//
// Issue #218 (umbrella): translation must work for the apple noun in both
// directions and for unquoted "translate X to Y" prompts, mirroring the
// Rust pipeline in the browser worker.
//
// Sub-issues exercised here:
//   - #216: `translate apple to russian` (no quotes)
//   - #217: `переведи "яблоко" на английский` (single Russian noun)
const { test, expect } = require('@playwright/test');

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

test.describe('Issue #218 apple/яблоко translation prompts', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('./');
    await expect(page.locator('.app')).toBeVisible({ timeout: 15_000 });
    await switchToManualMode(page);
  });

  test('#216 — translate apple to russian without quotes', async ({ page }) => {
    const reply = await sendPrompt(page, 'translate apple to russian');
    await expect(reply).toContainText('яблоко');
    await expect(reply).not.toContainText('[ru]');
    await expect(reply).not.toContainText('formal-ai');
  });

  test('#216 — Translate Apple to Russian preserves capitalization', async ({ page }) => {
    const reply = await sendPrompt(page, 'Translate Apple to Russian');
    await expect(reply).toContainText(/яблоко/i);
    await expect(reply).not.toContainText('[ru]');
  });

  test('#216 — unquoted apple covers all supported target languages', async ({ page }) => {
    const cases = [
      { prompt: 'translate apple to english', expected: 'apple', placeholder: '[en]' },
      { prompt: 'translate apple to russian', expected: 'яблоко', placeholder: '[ru]' },
      { prompt: 'translate apple to hindi', expected: 'सेब', placeholder: '[hi]' },
      { prompt: 'translate apple to chinese', expected: '苹果', placeholder: '[zh]' },
    ];

    for (const { prompt, expected, placeholder } of cases) {
      const reply = await sendPrompt(page, prompt);
      await expect(reply).toContainText(expected);
      await expect(reply).not.toContainText(placeholder);
    }
  });

  test('native Hindi and Chinese translation prompts work without quotes', async ({ page }) => {
    const hindi = await sendPrompt(page, 'apple का हिंदी में अनुवाद करो');
    await expect(hindi).toContainText('सेब');
    await expect(hindi).not.toContainText('[hi]');

    const chinese = await sendPrompt(page, '把 apple 翻译成中文');
    await expect(chinese).toContainText('苹果');
    await expect(chinese).not.toContainText('[zh]');
  });

  test('#217 — single Russian noun quoted with ASCII quotes', async ({ page }) => {
    const reply = await sendPrompt(page, 'переведи "яблоко" на английский');
    await expect(reply).toContainText('apple');
    await expect(reply).not.toContainText('[en]');
    await expect(reply).not.toContainText('formal-ai');
  });

  test('#217 — single Russian noun quoted with chevron quotes', async ({ page }) => {
    const reply = await sendPrompt(page, 'переведи «яблоко» на английский');
    await expect(reply).toContainText('apple');
    await expect(reply).not.toContainText('[en]');
  });

  test('round-trip: ru→en→ru lands on the original noun', async ({ page }) => {
    const ruToEn = await sendPrompt(page, 'переведи "яблоко" на английский');
    await expect(ruToEn).toContainText('apple');

    const enToRu = await sendPrompt(page, 'translate "apple" to russian');
    await expect(enToRu).toContainText('яблоко');
  });
});
