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
  return lastMessage;
}

test.describe('Issue #478 neural inference concept lookup', () => {
  test.beforeEach(async ({ page }) => {
    await page.route('**/*', async (route) => {
      const url = new URL(route.request().url());
      if (['localhost', '127.0.0.1'].includes(url.hostname)) {
        await route.continue();
        return;
      }
      await route.abort();
    });
    await page.addInitScript(() => {
      window.localStorage.setItem(
        'formal-ai.preferences.v1',
        'demo_preferences\n  demoMode "off"\n  diagnosticsMode "on"\n  greetingVariations "off"',
      );
    });
    await page.goto('./');
    await expect(page.locator('.app')).toBeVisible({ timeout: 15_000 });
    await expect(page.locator('[data-testid="demo-status"]')).toHaveText('Manual mode');
  });

  test('supported-language neural inference prompts use the seeded concept', async ({
    page,
  }) => {
    const cases = [
      {
        language: 'en',
        prompt: 'What is neural inference?',
        expectedTerm: 'neural inference',
        expectedSummary: 'trained neural network',
      },
      {
        language: 'ru',
        prompt: 'что такое нейросетевой инференс?',
        expectedTerm: 'Нейросетевой инференс',
        expectedSummary: 'обученной нейронной сети',
      },
      {
        language: 'hi',
        prompt: 'न्यूरल इन्फरेंस क्या है?',
        expectedTerm: 'न्यूरल इन्फरेंस',
        expectedSummary: 'प्रशिक्षित neural network',
      },
      {
        language: 'zh',
        prompt: '什么是神经网络推理?',
        expectedTerm: '神经网络推理',
        expectedSummary: '已经训练好的神经网络',
      },
    ];

    for (const scenario of cases) {
      const answer = await sendPrompt(page, scenario.prompt);
      const body = answer.locator('.markdown-body');

      await expect(answer.locator('.intent'), scenario.language).toContainText(
        'intent:concept_lookup',
      );
      await expect(body, scenario.language).toContainText(scenario.expectedTerm);
      await expect(body, scenario.language).toContainText(scenario.expectedSummary);
      await expect(body, scenario.language).toContainText(
        'cloud.google.com/discover/what-is-ai-inference',
      );
      await expect(answer.locator('.evidence-list'), scenario.language).toContainText(
        'concept_lookup:hit:concept_neural_inference',
      );
    }
  });
});
