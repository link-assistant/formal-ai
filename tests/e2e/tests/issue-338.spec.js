// @ts-check
//
// Issue #338: the production demo (wasm worker, manual mode) returned the
// unknown fallback for a simple box/apple arithmetic word problem. The worker
// must reduce the object relations to arithmetic and show the requested steps.
const { test, expect } = require('@playwright/test');

const PROMPT =
  'I have 3 boxes. Box A has twice as many apples as Box B. ' +
  'Box C has 5 more apples than Box A. If Box B has 10 apples, ' +
  'how many apples are there in total? Show your reasoning step by step.';

const supportedUiLanguages = [
  { language: 'en', name: 'English' },
  { language: 'ru', name: 'Russian' },
  { language: 'hi', name: 'Hindi' },
  { language: 'zh', name: 'Chinese' },
];

function preferencesForUiLanguage(language) {
  return (
    'demo_preferences\n' +
    '  demoMode "off"\n' +
    '  diagnosticsMode "on"\n' +
    '  greetingVariations "off"\n' +
    `  uiLanguage "${language}"`
  );
}

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

test.describe('Issue #338 - relational apple-box arithmetic', () => {
  test.beforeEach(async ({ page }) => {
    await page.addInitScript(() => {
      const KEY = 'formal-ai.preferences.v1';
      const existing = window.localStorage.getItem(KEY) || '';
      const languageMatch = existing.match(/^\s+uiLanguage "([^"]+)"/m);
      const uiLanguage = languageMatch ? languageMatch[1] : 'auto';
      window.localStorage.setItem(
        KEY,
        [
          'demo_preferences',
          '  demoMode "off"',
          '  diagnosticsMode "on"',
          '  greetingVariations "off"',
          `  uiLanguage "${uiLanguage}"`,
        ].join('\n'),
      );
    });
    await page.goto('./');
    await expect(page.locator('.app')).toBeVisible({ timeout: 15_000 });
    await expect(page.locator('[data-testid="demo-status"]')).toHaveText('Manual mode');
    await expect(page.locator('.status')).toContainText('wasm worker');
  });

  test('reduces box relations to a total apple count across UI languages', async ({
    page,
  }) => {
    for (const { language, name } of supportedUiLanguages) {
      const preferences = preferencesForUiLanguage(language);
      await page.evaluate(
        ({ nextPreferences }) => {
          window.localStorage.setItem('formal-ai.preferences.v1', nextPreferences);
        },
        { nextPreferences: preferences, language, name },
      );
      await page.reload();
      await expect(page.locator('.app'), `${name} UI loaded`).toBeVisible({
        timeout: 15_000,
      });
      await expect(page.locator('html'), `${name} UI language is active`).toHaveAttribute(
        'lang',
        language,
      );
      await expect(page.locator('.status'), `${name} worker status`).toContainText(
        'wasm worker',
      );

      const message = await sendPrompt(page, PROMPT);

      await expect(message, language).toContainText('Step 1: Box B = 10 apples.');
      await expect(message, language).toContainText(
        'Step 2: Box A = 2 * 10 = 20 apples.',
      );
      await expect(message, language).toContainText(
        'Step 3: Box C = 20 + 5 = 25 apples.',
      );
      await expect(message, language).toContainText('20 + 10 + 25 = 55');
      await expect(message, language).toContainText('there are 55 apples in total');
      await expect(message, language).not.toContainText('cannot answer');
      await expect(message, language).not.toContainText("I don't know how to answer");
    }
  });
});
