// @ts-check
const { test, expect } = require('@playwright/test');

const UNKNOWN_ANSWER_MARKER = 'cannot answer that from local links rules';

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

async function mockCursorSearchProviders(page) {
  await page.route('**://*.wikipedia.org/api/rest_v1/page/summary/**', async (route) => {
    await route.fulfill({
      status: 404,
      contentType: 'application/json',
      body: JSON.stringify({ type: 'not_found' }),
    });
  });

  await page.route('**://api.duckduckgo.com/**', async (route) => {
    await route.fulfill({
      status: 200,
      contentType: 'application/json',
      body: JSON.stringify({
        Heading: 'Cursor',
        AbstractText: 'Cursor is an AI-assisted code editor.',
        AbstractURL: 'https://www.cursor.com/',
        RelatedTopics: [],
      }),
    });
  });

  await page.route('**://*.wikipedia.org/w/rest.php/v1/search/page**', async (route) => {
    await route.fulfill({
      status: 200,
      contentType: 'application/json',
      body: JSON.stringify({
        pages: [
          {
            id: 1,
            key: 'Cursor_(user_interface)',
            title: 'Cursor (user interface)',
            excerpt: 'A cursor is an indicator used to show position for user interaction.',
            description: 'user interface indicator',
          },
        ],
      }),
    });
  });

  await page.route('**://www.wikidata.org/w/api.php**', async (route) => {
    await route.fulfill({
      status: 200,
      contentType: 'application/json',
      body: JSON.stringify({
        search: [
          {
            id: 'Q181127',
            label: 'Cursor',
            description: 'indicator used to show position on a display',
            concepturi: 'https://www.wikidata.org/wiki/Q181127',
          },
        ],
      }),
    });
  });

  await page.route('**://*.wiktionary.org/w/api.php**', async (route) => {
    await route.fulfill({
      status: 200,
      contentType: 'application/json',
      body: JSON.stringify(['cursor', ['cursor'], ['An indicator on a screen.'], []]),
    });
  });

  await page.route('**://archive.org/advancedsearch.php**', async (route) => {
    await route.fulfill({
      status: 200,
      contentType: 'application/json',
      body: JSON.stringify({ response: { docs: [] } }),
    });
  });
}

test.describe('Issue #500 — unresolved bare terms use web search', () => {
  test.beforeEach(async ({ page }) => {
    await disableGreetingVariations(page);
    await mockCursorSearchProviders(page);
    await page.goto('./');
    await expect(page.locator('.app')).toBeVisible({ timeout: 15_000 });
    await switchToManualMode(page);
  });

  test('cursor routes to web_search instead of unknown', async ({ page }) => {
    await page.locator('.diagnostics-toggle').click();

    const last = await sendPrompt(page, 'cursor');

    await expect(last).toContainText('Cursor');
    await expect(last.locator('.intent')).toContainText('intent:web_search');
    await expect(last.locator('.markdown-body')).not.toContainText(UNKNOWN_ANSWER_MARKER);
    await expect(last.locator('.evidence-list')).toContainText('web_search:request:cursor');
    await expect(last.locator('.evidence-list')).toContainText(
      'web_search:query_kind:unresolved_bare_term',
    );
  });
});
