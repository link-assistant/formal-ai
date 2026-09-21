// @ts-check
//
// Issue #230: a quoted Russian phrase that describes a search action was
// routed to translation but fell back to the fake `[en] ...` placeholder in
// the browser worker.
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

test.describe('Issue #230 Russian search-phrase translation', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('./');
    await expect(page.locator('.app')).toBeVisible({ timeout: 15_000 });
    await switchToManualMode(page);
  });

  test('reported quoted phrase translates instead of using a placeholder', async ({ page }) => {
    const reply = await sendPrompt(
      page,
      'Переведи "Найти синонимы или примеры согласования" на ангилйский',
    );
    await expect(reply).toContainText('Find synonyms or examples of agreement');
    await expect(reply).not.toContainText('[en]');
    await expect(reply).not.toContainText('[En]');
  });

  test('unknown translation gaps are explicit for every supported target language', async ({
    page,
  }) => {
    const cases = [
      {
        language: 'en',
        prompt: 'Переведи "неведомослово" на английский',
        placeholders: ['[en]', '[En]'],
      },
      {
        language: 'ru',
        prompt: 'Translate "zzqxqv" to Russian',
        placeholders: ['[ru]', '[Ru]'],
      },
      {
        language: 'hi',
        prompt: 'Translate "zzqxqv" to Hindi',
        placeholders: ['[hi]', '[Hi]'],
      },
      {
        language: 'zh',
        prompt: 'Translate "zzqxqv" to Chinese',
        placeholders: ['[zh]', '[Zh]'],
      },
    ];

    for (const { language, prompt, placeholders } of cases) {
      const reply = await sendPrompt(page, prompt);
      await expect(reply).toContainText('could not translate');
      await expect(reply).toContainText(`to ${language}`);
      for (const placeholder of placeholders) {
        await expect(reply).not.toContainText(placeholder);
      }
    }
  });
});
