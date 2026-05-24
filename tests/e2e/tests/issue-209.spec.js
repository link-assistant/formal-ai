// @ts-check
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

test.describe('Issue #209 prime proof prompts', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('./');
    await expect(page.locator('.app')).toBeVisible({ timeout: 15_000 });
    await switchToManualMode(page);
  });

  test('answers compact multilingual proof requests with Euclid proof', async ({
    page,
  }) => {
    const cases = [
      {
        language: 'en',
        prompt: 'Hello. Prove that there are infinitely many prime numbers',
        expected: /There are infinitely many prime numbers|p₁/u,
      },
      {
        language: 'ru',
        prompt: 'привет. докажи что простых бесконечно',
        expected: /Простых чисел бесконечно много|p₁/u,
      },
      {
        language: 'hi',
        prompt: 'नमस्ते. साबित करो कि अभाज्य संख्याएँ अनंत हैं',
        expected: /अभाज्य संख्याएँ अनंत हैं|p₁/u,
      },
      {
        language: 'zh',
        prompt: '你好。证明素数有无穷多个',
        expected: /素数有无穷多个|p₁/u,
      },
    ];

    for (const item of cases) {
      const reply = await sendPrompt(page, item.prompt);
      await expect(reply).toContainText(item.expected);
      await expect(reply).toContainText(/relative-meta-logic|Peano|Пеано/u);
      await expect(reply).not.toContainText('План доказательства');
      await expect(reply).not.toContainText('Я пока не знаю');
    }
  });
});
