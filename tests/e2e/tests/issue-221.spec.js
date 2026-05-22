// @ts-check
//
// Issue #221 (umbrella): common-noun translation in the browser demo.
// #218 fixed the apple noun; #221 demands the same machinery for any
// common noun (tomato/cucumber/potato/...) plus the unquoted variants.
//
// The browser worker now loads a shared `seed/translations.lino`
// dictionary (capped at 128 entries) alongside the rest of the seed
// data so it can resolve common nouns without CORS-blocked Wiktionary
// calls. The same `.lino` file is embedded into the Rust binary via
// `include_str!` so the two surfaces stay in sync.
const { test, expect } = require('@playwright/test');

async function switchToManualMode(page) {
  const demoToggle = page.locator('.mode-toggle');
  await expect(demoToggle).toContainText(/Demo on|Demo off|Демо/, { timeout: 10_000 });
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

test.describe('Issue #221 common-noun translation', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('./');
    await expect(page.locator('.app')).toBeVisible({ timeout: 15_000 });
    await switchToManualMode(page);
  });

  test('quoted Russian common nouns translate to English', async ({ page }) => {
    const cases = [
      { prompt: 'Переведи "помидор" на английский.', expected: 'tomato' },
      { prompt: 'Переведи "огурец" на английский.', expected: 'cucumber' },
      { prompt: 'переведи "картофель" на английский', expected: 'potato' },
      { prompt: 'переведи "морковь" на английский', expected: 'carrot' },
      { prompt: 'переведи "хлеб" на английский', expected: 'bread' },
      { prompt: 'переведи "вода" на английский', expected: 'water' },
    ];
    for (const { prompt, expected } of cases) {
      const reply = await sendPrompt(page, prompt);
      await expect(reply).toContainText(expected);
      await expect(reply).not.toContainText('[en]');
    }
  });

  test('quoted English common nouns translate to Russian', async ({ page }) => {
    const cases = [
      { prompt: 'translate "tomato" to russian', expected: /помидор|томат/i },
      { prompt: 'translate "cucumber" to russian', expected: /огурец/i },
      { prompt: 'translate "potato" to russian', expected: /картофель|картошка/i },
      { prompt: 'translate "carrot" to russian', expected: /морковь/i },
      { prompt: 'translate "bread" to russian', expected: /хлеб/i },
    ];
    for (const { prompt, expected } of cases) {
      const reply = await sendPrompt(page, prompt);
      await expect(reply).toContainText(expected);
      await expect(reply).not.toContainText('[ru]');
    }
  });

  test('unquoted common-noun prompts work in both directions', async ({ page }) => {
    const cases = [
      { prompt: 'translate tomato to russian', expected: /помидор|томат/i, placeholder: '[ru]' },
      { prompt: 'translate cucumber to russian', expected: /огурец/i, placeholder: '[ru]' },
      { prompt: 'переведи помидор на английский', expected: 'tomato', placeholder: '[en]' },
      { prompt: 'переведи огурец на английский', expected: 'cucumber', placeholder: '[en]' },
    ];
    for (const { prompt, expected, placeholder } of cases) {
      const reply = await sendPrompt(page, prompt);
      await expect(reply).toContainText(expected);
      await expect(reply).not.toContainText(placeholder);
    }
  });

  test('Russian inflected forms resolve to the lemma translation', async ({ page }) => {
    // The dictionary aliases include common Russian case forms so
    // `помидоры`, `помидору`, `яблоки` resolve to the lemma.
    const reply = await sendPrompt(page, 'переведи "помидоры" на английский');
    await expect(reply).toContainText('tomato');
    await expect(reply).not.toContainText('[en]');
  });

  test('round-trip: tomato → помидор → tomato', async ({ page }) => {
    const enToRu = await sendPrompt(page, 'translate "tomato" to russian');
    await expect(enToRu).toContainText(/помидор|томат/i);

    const ruToEn = await sendPrompt(page, 'Переведи "помидор" на английский.');
    await expect(ruToEn).toContainText('tomato');
  });
});
