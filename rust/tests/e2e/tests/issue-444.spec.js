// @ts-check
const { test, expect } = require('@playwright/test');

// Issue #444: after a "how to …" answer, a bare elaboration follow-up such as
// "Can you give me specific instructions?" used to dead-end at the unknown
// opener. It must rebind to the procedure recovered from the prior turn and
// answer as procedural_how_to in the original language. Mirrors the Rust
// reproducing tests in tests/unit/specification/reasoning_paths_procedures.rs.
const CASES = [
  {
    language: 'en',
    howTo: 'how to publish to npm',
    followUp: 'Can you give me specific instructions?',
    task: 'publish to npm',
  },
  {
    language: 'ru',
    howTo: 'как опубликовать пакет в npm',
    followUp: 'дай конкретные инструкции',
    task: 'опубликовать пакет в npm',
  },
  {
    language: 'zh',
    howTo: '如何发布到 npm',
    followUp: '给我具体步骤',
    task: '发布到 npm',
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
  return { lastMessage, body };
}

test.describe('Issue #444 procedural elaboration follow-up', () => {
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

  test('elaboration follow-up rebinds to the prior how-to', async ({ page }) => {
    for (const { language, howTo, followUp, task } of CASES) {
      await test.step(language, async () => {
        const first = await sendPrompt(page, howTo);
        await expect(first.lastMessage.locator('.intent')).toContainText(
          'intent:procedural_how_to',
        );
        await expect(first.body).toContainText(task);

        const second = await sendPrompt(page, followUp);
        // The bare follow-up must resolve as the same procedure, not unknown.
        await expect(second.lastMessage.locator('.intent')).toContainText(
          'intent:procedural_how_to',
        );
        await expect(second.body).toContainText(task);
        await expect(second.lastMessage).toContainText('procedural_how_to:followup');
      });

      // Reset the conversation between languages so each case starts clean.
      await page.reload();
      await expect(page.locator('.app')).toBeVisible({ timeout: 15_000 });
    }
  });
});
