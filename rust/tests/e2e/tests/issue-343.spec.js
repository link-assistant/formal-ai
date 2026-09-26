// @ts-check
const { test, expect } = require('@playwright/test');

const PROMPTS = [
  {
    language: 'en',
    prompt: 'How to do SPEC dirven development step by step?',
  },
  {
    language: 'ru',
    prompt: 'как сделать SPEC dirven development? напиши по шагам',
  },
  {
    language: 'hi',
    prompt: 'कैसे करें SPEC dirven development? चरणों में बताओ',
  },
  {
    language: 'zh',
    prompt: '如何做 SPEC dirven development？按步骤写',
  },
];
const UNKNOWN_ANSWER_MARKER = 'не могу ответить на это по локальным правилам';

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

test.describe('Issue #343 Russian spec-driven how-to typo', () => {
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

  test('supported-language prompts route to procedural how-to with corrected typo', async ({
    page,
  }) => {
    for (const { language, prompt } of PROMPTS) {
      await test.step(language, async () => {
        const { lastMessage, body } = await sendPrompt(page, prompt);

        await expect(lastMessage.locator('.intent')).toContainText(
          'intent:procedural_how_to',
        );
        await expect(body).toContainText('spec driven development');
        await expect(body).toContainText('web search');
        await expect(body).not.toContainText(UNKNOWN_ANSWER_MARKER);
        await expect(lastMessage).toContainText(
          'spelling_correction:dirven->driven',
        );
      });
    }
  });
});
