// @ts-check
//
// Issue #336: a compound-interest prompt with a follow-up currency conversion
// fell through to unknown in the CLI and in the browser agent plan. The browser
// case split the prompt into a calculation step and a "convert the final amount"
// step, so the second step also needs to read the previous assistant answer.
const { test, expect } = require('@playwright/test');

const FULL_PROMPT =
  'If I invest $1000 at 8% annual interest compounded monthly for 5 years, ' +
  'how much will I have? Show the formula, calculate step by step, and then ' +
  'convert the final amount to EUR using current exchange rates from the web.';

function preferences(uiLanguage = 'auto') {
  return (
    'demo_preferences\n' +
    '  demoMode "off"\n' +
    '  diagnosticsMode "on"\n' +
    '  greetingVariations "off"\n' +
    `  uiLanguage "${uiLanguage}"`
  );
}

async function resetApp(page, uiLanguage = 'auto') {
  await page.addInitScript((value) => {
    window.localStorage.clear();
    window.localStorage.setItem('formal-ai.preferences.v1', value);
  }, preferences(uiLanguage));
  await page.goto('./');
  await expect(page.locator('.app')).toBeVisible({ timeout: 15_000 });
  await expect(page.locator('.status')).toContainText('wasm worker');
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

test.describe('Issue #336 — compound interest with EUR conversion', () => {
  test.beforeEach(async ({ page }) => {
    await resetApp(page);
    await expect(page.locator('[data-testid="demo-status"]')).toHaveText('Manual mode');
  });

  test('chat mode computes the formula, final USD amount, and EUR conversion', async ({
    page,
  }) => {
    const message = await sendPrompt(page, FULL_PROMPT);

    await expect(message).toContainText('Compound interest calculation');
    await expect(message).toContainText('A = P(1 + r/n)^(n*t)');
    await expect(message).toContainText('P = 1000 USD');
    await expect(message).toContainText('r = 0.08');
    await expect(message).toContainText('n = 12');
    await expect(message).toContainText('t = 5 years');
    await expect(message).toContainText('Final amount: 1489.85 USD');
    await expect(message).toContainText('Conversion: USD -> EUR');
    await expect(message).toContainText('EUR');
    await expect(message).not.toContainText("I didn't understand you");
    await expect(message).not.toContainText("I'm not sure how to respond");
    await expect(message).not.toContainText('unparseable');
  });

  test('agent mode converts the final amount from step 1 in step 2', async ({
    page,
  }) => {
    await page.locator('[data-testid="mode-option-agent"]').click();
    const message = await sendPrompt(page, FULL_PROMPT);

    await expect(message).toContainText('Agent plan (2 steps)');
    await expect(message).toContainText(
      'Step 1: If I invest $1000 at 8% annual interest compounded monthly for 5 years',
    );
    await expect(message).toContainText('Step 2: convert the final amount to EUR');
    await expect(message).toContainText('Final amount: 1489.85 USD');
    await expect(message).toContainText('Final amount conversion');
    await expect(message).toContainText('Source amount: 1489.85 USD');
    await expect(message).toContainText('Conversion: USD -> EUR');
    await expect(message).not.toContainText("I didn't understand you");
    await expect(message).not.toContainText("I'm not sure how to respond");
    await expect(message).not.toContainText('unparseable');
  });

  test('agent mode works across supported UI languages', async ({ browser, baseURL }) => {
    const cases = [
      { language: 'en' },
      { language: 'ru' },
      { language: 'hi' },
      { language: 'zh' },
    ];

    for (const { language } of cases) {
      const context = await browser.newContext();
      await context.addInitScript((value) => {
        window.localStorage.clear();
        window.localStorage.setItem('formal-ai.preferences.v1', value);
      }, preferences(language));
      const languagePage = await context.newPage();
      try {
        await languagePage.goto(baseURL || './');
        await expect(languagePage.locator('.app'), language).toBeVisible({
          timeout: 15_000,
        });
        await expect(languagePage.locator('.status'), language).toContainText(
          'wasm worker',
        );

        await languagePage.locator('[data-testid="mode-option-agent"]').click();
        const message = await sendPrompt(languagePage, FULL_PROMPT);

        await expect(message, language).toContainText('Agent plan (2 steps)');
        await expect(message, language).toContainText('Final amount: 1489.85 USD');
        await expect(message, language).toContainText('Source amount: 1489.85 USD');
        await expect(message, language).toContainText('Conversion: USD -> EUR');
        await expect(message, language).not.toContainText("I didn't understand you");
        await expect(message, language).not.toContainText(
          "I'm not sure how to respond",
        );
        await expect(message, language).not.toContainText('unparseable');
      } finally {
        await context.close();
      }
    }
  });
});
