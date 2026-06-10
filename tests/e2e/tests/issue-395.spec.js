// @ts-check
// Issue #395: "У меня есть числа 3, 5, 6, 7, 8 отсортируй их в JavaScript, дай
// мне код и результат" used to answer intent:unknown. The browser path must now
// route to write_program, show runnable JavaScript, and print the
// deterministically-computed sorted result.
const { test, expect } = require('@playwright/test');

async function disableGreetingVariations(page) {
  await page.addInitScript(() => {
    try {
      window.localStorage.setItem(
        'formal-ai.preferences.v1',
        'demo_preferences\n  greetingVariations "off"',
      );
    } catch (_error) {
      // localStorage may be unavailable in some sandboxes.
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

test.describe('Issue #395 - sort numbers, give code and result', () => {
  test.beforeEach(async ({ page }) => {
    await disableGreetingVariations(page);
    await page.goto('./');
    await expect(page.locator('.app')).toBeVisible({ timeout: 15_000 });
    await switchToManualMode(page);
  });

  test('the exact Russian prompt returns JS code and the sorted result (was: unknown)', async ({
    page,
  }) => {
    const answer = await sendPrompt(
      page,
      'У меня есть числа 3, 5, 6, 7, 8 отсортируй их в JavaScript, дай мне код и результат',
    );
    const body = answer.locator('.markdown-body');

    // Never the "I didn't understand you" fallback.
    await expect(body).not.toContainText('Я тебя не понял');
    // Runnable JavaScript with the ascending comparator.
    await expect(body).toContainText('const numbers = [3, 5, 6, 7, 8];');
    await expect(body).toContainText('sort((a, b) => a - b)');
    // The deterministically-computed result, localized to Russian.
    await expect(body).toContainText('Результат: 3, 5, 6, 7, 8');
  });

  test('an English descending Python request computes the reversed result', async ({
    page,
  }) => {
    const answer = await sendPrompt(
      page,
      'Sort the numbers 4, 2, 7, 1 in descending order in Python and show me the code and result',
    );
    const body = answer.locator('.markdown-body');

    await expect(body).toContainText('sorted(numbers, reverse=True)');
    await expect(body).toContainText('Result: 7, 4, 2, 1');
  });
});
