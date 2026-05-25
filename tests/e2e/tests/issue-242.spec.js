// @ts-check
const { test, expect } = require('@playwright/test');

const UNKNOWN_ANSWER_MARKER = 'cannot answer that from local Links Notation rules';

async function disableGreetingVariations(page) {
  await page.addInitScript(() => {
    try {
      const KEY = 'formal-ai.preferences.v1';
      const existing = window.localStorage.getItem(KEY) || '';
      if (/greetingVariations\s+"/.test(existing)) {
        window.localStorage.setItem(
          KEY,
          existing.replace(/greetingVariations\s+"[^"]*"/, 'greetingVariations "off"'),
        );
      } else if (existing.startsWith('demo_preferences')) {
        window.localStorage.setItem(KEY, `${existing}\n  greetingVariations "off"`);
      } else {
        window.localStorage.setItem(KEY, 'demo_preferences\n  greetingVariations "off"');
      }
    } catch (_error) {
      // localStorage may be unavailable in strict browser contexts.
    }
  });
}

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

async function routeDictionaryFallback(page, entriesByTerm) {
  await page.route('**/api/rest_v1/page/summary/**', async (route) => {
    await route.fulfill({
      status: 404,
      contentType: 'application/json',
      body: JSON.stringify({ httpCode: 404, httpReason: 'Not Found' }),
    });
  });
  await page.route('**/rest.php/v1/search/page**', async (route) => {
    await route.fulfill({
      status: 200,
      contentType: 'application/json',
      body: JSON.stringify({ pages: [] }),
    });
  });
  await page.route('**://*.wikidata.org/w/api.php**', async (route) => {
    await route.fulfill({
      status: 200,
      contentType: 'application/json',
      body: JSON.stringify({ search: [] }),
    });
  });
  await page.route('**://*.wiktionary.org/w/api.php**', async (route) => {
    const url = new URL(route.request().url());
    const term = url.searchParams.get('search') || '';
    const entry = entriesByTerm[term];
    expect(entry, `unexpected Wiktionary search term: ${term}`).toBeTruthy();
    await route.fulfill({
      status: 200,
      contentType: 'application/json',
      body: JSON.stringify([
        term,
        [term],
        [entry.description],
        [entry.url],
      ]),
    });
  });
}

test.describe('Issue #242 dictionary lookup prompt recovery', () => {
  test.beforeEach(async ({ page }) => {
    await disableGreetingVariations(page);
    await page.goto('./');
    await expect(page.locator('.app')).toBeVisible({ timeout: 15_000 });
    await switchToManualMode(page);
  });

  test('malformed meaning question extracts the term and falls back to Wiktionary', async ({ page }) => {
    await routeDictionaryFallback(page, {
      digress: {
        description: 'to turn aside, especially from the main subject in writing or speaking',
        url: 'https://en.wiktionary.org/wiki/digress',
      },
    });

    const reply = await sendPrompt(page, 'what i digress mean?');
    await expect(reply).toContainText('digress');
    await expect(reply).toContainText(/main subject|writing|speaking/i);
    await expect(reply).toContainText('wiktionary.org');
    await expect(reply).not.toContainText(UNKNOWN_ANSWER_MARKER);
  });

  test('supported-language definition prompts still reach dictionary fallback', async ({ page }) => {
    await routeDictionaryFallback(page, {
      flibbertigibbet: {
        description: 'a flighty or excessively talkative person',
        url: 'https://en.wiktionary.org/wiki/flibbertigibbet',
      },
    });

    for (const { language, prompt } of [
      { language: 'en', prompt: 'what does flibbertigibbet mean?' },
      { language: 'ru', prompt: 'что такое flibbertigibbet?' },
      { language: 'hi', prompt: 'flibbertigibbet क्या है?' },
      { language: 'zh', prompt: 'flibbertigibbet 是什么?' },
    ]) {
      const reply = await sendPrompt(page, prompt);
      await expect(reply, language).toContainText('flibbertigibbet');
      await expect(reply, language).toContainText(/flighty|talkative/i);
      await expect(reply, language).toContainText('wiktionary.org');
      await expect(reply, language).not.toContainText(UNKNOWN_ANSWER_MARKER);
    }
  });
});
