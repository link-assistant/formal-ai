// @ts-check
const { test, expect } = require('@playwright/test');

const wasmParityCases = [
  {
    language: 'en',
    name: 'English',
    prompt: 'blorfblarf',
    expected: "I'm not sure how to respond to that yet.",
  },
  {
    language: 'ru',
    name: 'Russian',
    prompt: 'неведомослово',
    expected: 'Я ещё не научился отвечать на это.',
    forbidden: 'Я тебя не понял.',
  },
  {
    language: 'hi',
    name: 'Hindi',
    prompt: 'अज्ञातशब्द',
    expected: 'मैं समझ नहीं पाया।',
  },
  {
    language: 'zh',
    name: 'Chinese',
    prompt: '未知词',
    expected: '我不太明白你说的意思。',
  },
];

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

test.describe('Issue #282 Rust/WASM worker parity', () => {
  test.beforeEach(async ({ page }) => {
    await page.addInitScript(() => {
      window.localStorage.setItem(
        'formal-ai.preferences.v1',
        'demo_preferences\n  demoMode "off"\n  diagnosticsMode "on"\n  greetingVariations "off"',
      );
    });
    await page.goto('./');
    await expect(page.locator('.app')).toBeVisible({ timeout: 15_000 });
    await expect(page.locator('[data-testid="demo-status"]')).toHaveText('Manual mode');
    await expect(page.locator('[data-testid="chat-composer-input"]')).toBeEnabled({
      timeout: 5_000,
    });
  });

  for (const { language, name, prompt, expected, forbidden } of wasmParityCases) {
    test(`${name} unknown prompts use the native Rust stable-id opener`, async ({ page }) => {
      await expect(page.locator('.status')).toContainText('wasm worker');

      const reply = await sendPrompt(page, prompt);
      await expect(reply, `${language} opener should match native Rust`).toContainText(expected);
      if (forbidden) {
        await expect(reply).not.toContainText(forbidden);
      }
    });
  }
});
