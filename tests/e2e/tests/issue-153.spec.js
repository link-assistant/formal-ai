// @ts-check
const { test, expect } = require('@playwright/test');

// Issue #153 — search UX, formalization SVO view, cross-source dedupe, and
// localized search-result templates. Each test mocks the three providers so
// the assertions are hermetic.

const UNKNOWN_ANSWER_MARKER = 'cannot answer that from local Links Notation rules';

async function disableGreetingVariations(page) {
  await page.addInitScript(() => {
    try {
      window.localStorage.setItem(
        'formal-ai.preferences.v1',
        'demo_preferences\n  greetingVariations "off"',
      );
    } catch (_error) {
      // ignore — localStorage may be unavailable in some sandboxes.
    }
  });
}

async function switchToManualMode(page) {
  const demoToggle = page.locator('.mode-toggle');
  await expect(demoToggle).toContainText(/Demo on|Демо/);
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

async function mockAppleSearch(page) {
  await page.route('**://api.duckduckgo.com/**', async (route) => {
    await route.fulfill({
      status: 200,
      contentType: 'application/json',
      body: JSON.stringify({
        Heading: 'Apple',
        AbstractText: 'Apple is a fruit produced by an apple tree.',
        AbstractURL: 'https://duckduckgo.com/Apple',
        RelatedTopics: [],
      }),
    });
  });
  await page.route('**/w/rest.php/v1/search/page**', async (route) => {
    await route.fulfill({
      status: 200,
      contentType: 'application/json',
      body: JSON.stringify({
        pages: [
          {
            id: 856,
            key: 'Apple',
            title: 'Apple',
            excerpt: 'Apple is the edible fruit of an apple tree.',
            description: 'fruit and species of plant',
          },
        ],
      }),
    });
  });
  await page.route('**/wikidata.org/w/api.php**', async (route) => {
    const url = new URL(route.request().url());
    expect(url.searchParams.get('action')).toBe('wbsearchentities');
    expect(url.searchParams.get('props')).toContain('sitelinks/urls');
    await route.fulfill({
      status: 200,
      contentType: 'application/json',
      body: JSON.stringify({
        search: [
          {
            id: 'Q89',
            label: 'Apple',
            description: 'fruit of the apple tree',
            concepturi: 'https://www.wikidata.org/wiki/Q89',
            sitelinks: {
              enwiki: {
                site: 'enwiki',
                title: 'Apple',
                url: 'https://en.wikipedia.org/wiki/Apple',
              },
            },
          },
        ],
      }),
    });
  });
}

test.describe('Issue #153 — search UX, formalization, and dedupe', () => {
  test.beforeEach(async ({ page }) => {
    await disableGreetingVariations(page);
    await page.goto('./');
    await expect(page.locator('.app')).toBeVisible({ timeout: 15_000 });
    await switchToManualMode(page);
  });

  test('top menu uses a lab emoji for the diagnostics toggle', async ({ page }) => {
    const diagnosticsToggle = page.locator('.diagnostics-toggle');
    await expect(diagnosticsToggle).toBeVisible();
    await expect(diagnosticsToggle.locator('.btn-icon')).toHaveText('🧪');
  });

  test('top menu links to the GitHub source code', async ({ page }) => {
    const link = page.locator('[data-testid="source-code"]');
    await expect(link).toBeVisible();
    await expect(link).toHaveAttribute(
      'href',
      'https://github.com/link-assistant/formal-ai',
    );
  });

  test('"New conversation" button is disabled until the chat has content', async ({
    page,
  }) => {
    const newBtn = page.locator('[data-testid="conversation-new"]');
    await expect(newBtn).toBeDisabled();
    await sendPrompt(page, 'Hi');
    await expect(newBtn).toBeEnabled();
  });

  test('sidebar can be collapsed and expanded', async ({ page }) => {
    const sidebarToggle = page.locator('[data-testid="sidebar-toggle"]');
    await expect(sidebarToggle).toBeVisible();
    await sidebarToggle.click();
    await expect(page.locator('.app')).toHaveClass(/sidebar-collapsed/);
    await sidebarToggle.click();
    await expect(page.locator('.app')).not.toHaveClass(/sidebar-collapsed/);
  });

  test('diagnostics shows the SVO formalization view with @USER + OP:* + Q-id', async ({
    page,
  }) => {
    await mockAppleSearch(page);
    await page.locator('.diagnostics-toggle').click();

    const last = await sendPrompt(page, 'Search the web for Apple');
    const formalizationViews = last.locator('[data-testid="formalization"]');
    await expect(formalizationViews.first()).toBeVisible({ timeout: 10_000 });

    // The placeholder formalize step uses the bare query as the object slot.
    const first = formalizationViews.first();
    await expect(first).toContainText('@USER');
    await expect(first).toContainText('OP:search');
    await expect(first.locator('.formalization-slot').nth(0)).toHaveText('S');
    await expect(first.locator('.formalization-slot').nth(1)).toHaveText('V');
    await expect(first.locator('.formalization-slot').nth(2)).toHaveText('O');

    // The resolved formalize step must fold the Wikidata Q-id back into the
    // SVO tuple alongside the original placeholder.
    const resolved = formalizationViews.nth(1);
    await expect(resolved).toBeVisible();
    await expect(resolved).toContainText('Q89');
    await expect(resolved.locator('.formalization-tuple')).toContainText(
      '(@USER OP:search Q89)',
    );
  });

  test('search results dedupe cross-provider entries by Wikidata sitelinks', async ({
    page,
  }) => {
    await mockAppleSearch(page);
    await page.locator('.diagnostics-toggle').click();

    const last = await sendPrompt(page, 'Search the web for Apple');

    await expect(last).toContainText('Search results for');
    // Wikidata returned a `Q89` Q-id and the Wikipedia provider returned the
    // same entity via `enwiki: Apple` — they collapse into one bullet.
    await expect(last).toContainText('Q89');
    await expect(last).toContainText('Other sources');

    // Evidence trail must list the deduplication and the formalized id.
    const evidence = last.locator('.evidence-list');
    await expect(evidence).toContainText('web_search:dedupe:Q:Q89');
    await expect(evidence).toContainText('web_search:formal:1:Q89');
  });

  test('DuckDuckGo provider contributes results (regression: signature mismatch)', async ({
    page,
  }) => {
    // Issue #153: searchDuckDuckGo was declared as (query, limit) but the
    // dispatcher calls every provider as (query, language, providerLimit).
    // Verify the fix by mocking only DDG and asserting it shows up in the
    // ranked output.
    await page.route('**://api.duckduckgo.com/**', async (route) => {
      await route.fulfill({
        status: 200,
        contentType: 'application/json',
        body: JSON.stringify({
          Heading: 'Geometric Tesseract',
          AbstractText: 'A tesseract is the four-dimensional analogue of a cube.',
          AbstractURL: 'https://duckduckgo.com/Tesseract',
          RelatedTopics: [
            {
              FirstURL: 'https://duckduckgo.com/Hypercube',
              Text: 'Hypercube - generalised cube',
            },
          ],
        }),
      });
    });
    await page.route('**/w/rest.php/v1/search/page**', async (route) => {
      await route.fulfill({
        status: 200,
        contentType: 'application/json',
        body: JSON.stringify({ pages: [] }),
      });
    });
    await page.route('**/wikidata.org/w/api.php**', async (route) => {
      await route.fulfill({
        status: 200,
        contentType: 'application/json',
        body: JSON.stringify({ search: [] }),
      });
    });

    await page.locator('.diagnostics-toggle').click();
    const last = await sendPrompt(page, 'Search the web for Geometric Tesseract');

    await expect(last).toContainText('Geometric Tesseract');
    await expect(last).toContainText('duckduckgo');
    await expect(last.locator('.evidence-list')).toContainText(
      'web_search:provider:duckduckgo:count:2',
    );
  });

  test('search header is localized when the UI language is Russian', async ({
    page,
  }) => {
    await page.addInitScript(() => {
      Object.defineProperty(window.navigator, 'language', {
        configurable: true,
        get: () => 'ru-RU',
      });
      Object.defineProperty(window.navigator, 'languages', {
        configurable: true,
        get: () => ['ru-RU', 'en-US'],
      });
    });
    // Re-load to apply the language override (`beforeEach` already navigated).
    await page.goto('./');
    await expect(page.locator('.app')).toBeVisible({ timeout: 15_000 });
    await switchToManualMode(page);
    await mockAppleSearch(page);

    const last = await sendPrompt(page, 'Найди в интернете яблоко');
    await expect(last).toContainText('Результаты поиска для');
    await expect(last).toContainText('Apple');
    await expect(last).not.toContainText(UNKNOWN_ANSWER_MARKER);
  });
});
