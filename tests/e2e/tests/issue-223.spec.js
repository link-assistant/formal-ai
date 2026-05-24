// @ts-check
const { test, expect } = require('@playwright/test');

const UNKNOWN_ANSWER_MARKER = 'cannot answer that from local Links Notation rules';
const PANDAS_JOIN_DOCS_URL =
  'https://pandas.pydata.org/docs/reference/api/pandas.DataFrame.join.html';

async function disableGreetingVariations(page) {
  await page.addInitScript(() => {
    try {
      window.localStorage.setItem(
        'formal-ai.preferences.v1',
        'demo_preferences\n  greetingVariations "off"',
      );
    } catch (_error) {
      // ignore
    }
  });
}

async function switchToManualMode(page) {
  const demoToggle = page.locator('.mode-toggle');
  await expect(demoToggle).toContainText(/Demo on|Demo off|Демо/, {
    timeout: 10_000,
  });
  await demoToggle.click();
  await expect(page.locator('[data-testid="chat-composer-input"]')).toBeEnabled({
    timeout: 5_000,
  });
}

async function sendPrompt(page, text) {
  const input = page.locator('[data-testid="chat-composer-input"]');
  await expect(input).toBeEnabled({ timeout: 5_000 });
  await input.fill(text);
  const messages = page.locator('[data-testid="chat-message"]');
  const initial = await messages.count();
  await page.locator('[data-testid="chat-composer-submit"]').click();
  await expect(messages).toHaveCount(initial + 2, { timeout: 20_000 });
  return messages.last();
}

test.describe('Issue #223 - pandas join method docs summary', () => {
  test.beforeEach(async ({ page }) => {
    await disableGreetingVariations(page);
    await page.goto('./');
    await expect(page.locator('.app')).toBeVisible({ timeout: 15_000 });
    await switchToManualMode(page);
  });

  test('pandas join-method prompt returns an official-docs answer', async ({ page }) => {
    await page.locator('.diagnostics-toggle').click();

    const last = await sendPrompt(page, 'how the join method works in pandas');

    await expect(last).toContainText('DataFrame.join');
    await expect(last).toContainText('index');
    await expect(last).toContainText('other');
    await expect(last).toContainText('how');
    await expect(last).not.toContainText(UNKNOWN_ANSWER_MARKER);
    await expect(last.locator('.intent')).toContainText(
      'intent:docs_method_explanation',
    );
    await expect(last.locator('.evidence-list')).toContainText(
      'docs_method:project:pandas',
    );
    await expect(last.locator('.evidence-list')).toContainText(
      'docs_method:method:pandas.DataFrame.join',
    );
    await expect(last.locator('.evidence-list')).toContainText(
      'docs_method:source_kind:official-docs',
    );
    await expect(last.locator('.evidence-list')).toContainText(
      `source:${PANDAS_JOIN_DOCS_URL}`,
    );
  });
});
