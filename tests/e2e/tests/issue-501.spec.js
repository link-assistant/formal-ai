// @ts-check
const { test, expect } = require('@playwright/test');

const CASES = [
  {
    language: 'en',
    prompt: 'how install cursor',
    task: 'install cursor',
    officialMarker: 'official documentation',
  },
  {
    language: 'ru',
    prompt: 'как установить cursor',
    task: 'установить cursor',
    officialMarker: 'официальную документацию',
  },
  {
    language: 'hi',
    prompt: 'कैसे इंस्टॉल करें cursor',
    task: 'इंस्टॉल करें cursor',
    officialMarker: 'आधिकारिक documentation',
  },
  {
    language: 'zh',
    prompt: '如何安装 cursor',
    task: '安装 cursor',
    officialMarker: '官方 documentation',
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

  const assistantMessage = messages.last();
  await expect(assistantMessage).toHaveClass(/assistant/);
  const body = assistantMessage.locator('.markdown-body');
  await expect(body).toBeVisible();
  return { assistantMessage, body };
}

async function bootManualMode(page) {
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
}

test.describe('Issue #501 install how-to prompts', () => {
  test.beforeEach(async ({ page }) => {
    await bootManualMode(page);
  });

  test('routes install requests to official-docs-first procedural discovery', async ({
    page,
  }) => {
    for (const testCase of CASES) {
      await test.step(testCase.language, async () => {
        const { assistantMessage, body } = await sendPrompt(page, testCase.prompt);

        await expect(assistantMessage.locator('.intent')).toContainText(
          'intent:procedural_how_to',
        );
        await expect(body).toContainText(testCase.task);
        await expect(body).toContainText(testCase.officialMarker);
        await expect(assistantMessage).toContainText(
          'procedural_how_to:source_gate:official_documentation_first',
        );
        await expect(assistantMessage).toContainText(
          'web_search:request:cursor install official documentation',
        );
      });

      await page.reload();
      await expect(page.locator('.app')).toBeVisible({ timeout: 15_000 });
    }
  });
});
