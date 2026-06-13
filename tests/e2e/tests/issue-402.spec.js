// @ts-check
const { test, expect } = require('@playwright/test');

async function pinManualMode(page) {
  await page.addInitScript(() => {
    try {
      window.localStorage.setItem(
        'formal-ai.preferences.v1',
        [
          'demo_preferences',
          '  demoMode "off"',
          '  diagnosticsMode "on"',
          '  greetingVariations "off"',
        ].join('\n'),
      );
    } catch (_error) {
      // localStorage may be unavailable in hardened browser contexts.
    }
  });
}

async function openApp(page) {
  await pinManualMode(page);
  await page.goto('./');
  await expect(page.locator('.app')).toBeVisible({ timeout: 15_000 });
  await expect(page.locator('[data-testid="demo-status"]')).toHaveText(
    'Manual mode',
  );
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

test.describe('Issue #402 - Russian free-time small talk', () => {
  test('answers with assistant free-time intent instead of unknown', async ({
    page,
  }) => {
    await openApp(page);

    const last = await sendPrompt(page, 'Что делаешь в свободное время?');
    await expect(last).toHaveClass(/assistant/);
    await expect(last).toContainText('свободного времени');
    await expect(last.locator('.intent')).toContainText(
      'intent:assistant_free_time',
    );
    await expect(last).not.toContainText('Я ещё не научился');
    await expect(last).not.toContainText('Я тебя не понял');
    await expect(last.locator('.intent')).not.toContainText('intent:unknown');
  });
});
