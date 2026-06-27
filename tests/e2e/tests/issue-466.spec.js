// @ts-check
//
// Issue #466: authorship questions for unseeded works must route through the
// Wikidata fact-query pipeline instead of falling through to `unknown`.
const { test, expect } = require('@playwright/test');

const WAR_AND_PEACE_QID = 'Q161531';
const TOLSTOY_QID = 'Q7243';

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

async function mockWikidataAuthorLookup(page, requests) {
  await page.route('**://*.wikipedia.org/**', (route) => route.abort());
  await page.route('**://*.wiktionary.org/**', (route) => route.abort());
  await page.route('**://www.wikidata.org/w/api.php**', async (route) => {
    const url = new URL(route.request().url());
    const action = url.searchParams.get('action') || '';
    const search = url.searchParams.get('search') || '';
    const ids = url.searchParams.get('ids') || '';
    requests.push({ action, search, ids });

    if (action === 'wbsearchentities') {
      await route.fulfill({
        contentType: 'application/json',
        body: JSON.stringify({
          search: [
            {
              id: WAR_AND_PEACE_QID,
              label: /войн/i.test(search) ? 'Война и мир' : 'War and Peace',
              description: 'novel by Leo Tolstoy',
              concepturi: `https://www.wikidata.org/wiki/${WAR_AND_PEACE_QID}`,
              display: {
                label: {
                  value: /войн/i.test(search) ? 'Война и мир' : 'War and Peace',
                },
                description: { value: 'novel by Leo Tolstoy' },
              },
            },
          ],
        }),
      });
      return;
    }

    if (action === 'wbgetentities' && ids === WAR_AND_PEACE_QID) {
      await route.fulfill({
        contentType: 'application/json',
        body: JSON.stringify({
          entities: {
            [WAR_AND_PEACE_QID]: {
              labels: {
                en: { value: 'War and Peace' },
                ru: { value: 'Война и мир' },
              },
              sitelinks: {
                enwiki: {
                  title: 'War and Peace',
                  url: 'https://en.wikipedia.org/wiki/War_and_Peace',
                },
                ruwiki: {
                  title: 'Война и мир',
                  url: 'https://ru.wikipedia.org/wiki/Война_и_мир',
                },
              },
              claims: {
                P50: [
                  {
                    mainsnak: {
                      datavalue: {
                        value: { id: TOLSTOY_QID },
                      },
                    },
                  },
                ],
              },
            },
          },
        }),
      });
      return;
    }

    if (action === 'wbgetentities' && ids === TOLSTOY_QID) {
      await route.fulfill({
        contentType: 'application/json',
        body: JSON.stringify({
          entities: {
            [TOLSTOY_QID]: {
              labels: {
                en: { value: 'Leo Tolstoy' },
                ru: { value: 'Лев Толстой' },
              },
              sitelinks: {
                enwiki: {
                  title: 'Leo Tolstoy',
                  url: 'https://en.wikipedia.org/wiki/Leo_Tolstoy',
                },
                ruwiki: {
                  title: 'Лев Толстой',
                  url: 'https://ru.wikipedia.org/wiki/Лев_Толстой',
                },
              },
            },
          },
        }),
      });
      return;
    }

    await route.fulfill({
      contentType: 'application/json',
      body: JSON.stringify({ entities: {}, search: [] }),
    });
  });
}

test.describe('Issue #466 - authorship fact query routing', () => {
  test.beforeEach(async ({ page }) => {
    await page.addInitScript(() => {
      window.localStorage.setItem(
        'formal-ai.preferences.v1',
        'demo_preferences\n  demoMode "off"\n  diagnosticsMode "on"\n  greetingVariations "off"',
      );
    });
    await page.goto('./');
    await expect(page.locator('.app')).toBeVisible({ timeout: 15_000 });
    await expect(page.locator('[data-testid="demo-status"]')).toHaveText('Manual mode');
    await expect(page.locator('[data-testid="chat-composer-input"]')).toBeEnabled({
      timeout: 5_000,
    });
  });

  test('resolves English authorship questions through Wikidata P50', async ({ page }) => {
    const requests = [];
    await mockWikidataAuthorLookup(page, requests);

    const message = await sendPrompt(page, 'Who wrote War and Peace?');
    const evidence = message.locator('.evidence-list');

    await expect(message).toContainText('War and Peace was written by Leo Tolstoy.');
    await expect(message).not.toContainText('unknown');
    await expect(message.locator('.intent')).toContainText('intent:fact_query');
    await expect(evidence).toContainText('fact_query:relation:author_of_book');
    await expect(evidence).toContainText(`wikidata:${WAR_AND_PEACE_QID}`);
    await expect(evidence).toContainText(`wikidata:${TOLSTOY_QID}`);
    await expect(evidence).toContainText('source:https://en.wikipedia.org/wiki/Leo_Tolstoy');
    expect(requests.some((request) => request.search === 'War and Peace')).toBe(true);
    expect(requests.some((request) => request.ids === WAR_AND_PEACE_QID)).toBe(true);
    expect(requests.some((request) => request.ids === TOLSTOY_QID)).toBe(true);
  });

  test('resolves Russian authorship questions through Wikidata P50', async ({ page }) => {
    const requests = [];
    await mockWikidataAuthorLookup(page, requests);

    const message = await sendPrompt(page, 'Кто написал «Войну и мир»?');
    const evidence = message.locator('.evidence-list');

    await expect(message).toContainText('Автор произведения «Война и мир»: Лев Толстой.');
    await expect(message).not.toContainText('unknown');
    await expect(message.locator('.intent')).toContainText('intent:fact_query');
    await expect(evidence).toContainText('fact_query:relation:author_of_book');
    await expect(evidence).toContainText('language:ru');
    await expect(evidence).toContainText(`wikidata:${WAR_AND_PEACE_QID}`);
    await expect(evidence).toContainText(`wikidata:${TOLSTOY_QID}`);
    await expect(evidence).toContainText('source:https://ru.wikipedia.org/wiki/Лев_Толстой');
    expect(requests.some((request) => request.search === 'Войну и мир')).toBe(true);
    expect(requests.some((request) => request.ids === WAR_AND_PEACE_QID)).toBe(true);
    expect(requests.some((request) => request.ids === TOLSTOY_QID)).toBe(true);
  });
});
