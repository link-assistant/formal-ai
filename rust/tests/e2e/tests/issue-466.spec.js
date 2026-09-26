// @ts-check
//
// Issue #466: authorship questions for unseeded works must route through the
// Wikidata fact-query pipeline instead of falling through to `unknown`.
const { test, expect } = require('@playwright/test');

const WAR_AND_PEACE_QID = 'Q161531';
const TOLSTOY_QID = 'Q7243';
const WAR_AND_PEACE_LABELS = {
  en: 'War and Peace',
  ru: 'Война и мир',
  hi: 'युद्ध और शान्ति',
  zh: '战争与和平',
};
const TOLSTOY_LABELS = {
  en: 'Leo Tolstoy',
  ru: 'Лев Толстой',
  hi: 'लेव तोलस्तोय',
  zh: '列夫·托尔斯泰',
};

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

function languageForSearch(search) {
  if (/войн/i.test(search)) return 'ru';
  if (/युद्ध/u.test(search)) return 'hi';
  if (/战争/u.test(search)) return 'zh';
  return 'en';
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
      const language = languageForSearch(search);
      await route.fulfill({
        contentType: 'application/json',
        body: JSON.stringify({
          search: [
            {
              id: WAR_AND_PEACE_QID,
              label: WAR_AND_PEACE_LABELS[language],
              description: 'novel by Leo Tolstoy',
              concepturi: `https://www.wikidata.org/wiki/${WAR_AND_PEACE_QID}`,
              display: {
                label: {
                  value: WAR_AND_PEACE_LABELS[language],
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
                en: { value: WAR_AND_PEACE_LABELS.en },
                ru: { value: WAR_AND_PEACE_LABELS.ru },
                hi: { value: WAR_AND_PEACE_LABELS.hi },
                zh: { value: WAR_AND_PEACE_LABELS.zh },
              },
              sitelinks: {
                enwiki: {
                  title: WAR_AND_PEACE_LABELS.en,
                  url: 'https://en.wikipedia.org/wiki/War_and_Peace',
                },
                ruwiki: {
                  title: WAR_AND_PEACE_LABELS.ru,
                  url: 'https://ru.wikipedia.org/wiki/Война_и_мир',
                },
                hiwiki: {
                  title: WAR_AND_PEACE_LABELS.hi,
                  url: 'https://hi.wikipedia.org/wiki/युद्ध_और_शान्ति',
                },
                zhwiki: {
                  title: WAR_AND_PEACE_LABELS.zh,
                  url: 'https://zh.wikipedia.org/wiki/战争与和平',
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
                en: { value: TOLSTOY_LABELS.en },
                ru: { value: TOLSTOY_LABELS.ru },
                hi: { value: TOLSTOY_LABELS.hi },
                zh: { value: TOLSTOY_LABELS.zh },
              },
              sitelinks: {
                enwiki: {
                  title: TOLSTOY_LABELS.en,
                  url: 'https://en.wikipedia.org/wiki/Leo_Tolstoy',
                },
                ruwiki: {
                  title: TOLSTOY_LABELS.ru,
                  url: 'https://ru.wikipedia.org/wiki/Лев_Толстой',
                },
                hiwiki: {
                  title: TOLSTOY_LABELS.hi,
                  url: 'https://hi.wikipedia.org/wiki/लेव_तोलस्तोय',
                },
                zhwiki: {
                  title: TOLSTOY_LABELS.zh,
                  url: 'https://zh.wikipedia.org/wiki/列夫·托尔斯泰',
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

  test('resolves Hindi authorship questions through Wikidata P50', async ({ page }) => {
    const requests = [];
    await mockWikidataAuthorLookup(page, requests);

    const message = await sendPrompt(page, 'युद्ध और शान्ति किसने लिखी?');
    const evidence = message.locator('.evidence-list');

    await expect(message).toContainText('युद्ध और शान्ति को लेव तोलस्तोय ने लिखा था।');
    await expect(message).not.toContainText('unknown');
    await expect(message.locator('.intent')).toContainText('intent:fact_query');
    await expect(evidence).toContainText('fact_query:relation:author_of_book');
    await expect(evidence).toContainText('language:hi');
    await expect(evidence).toContainText(`wikidata:${WAR_AND_PEACE_QID}`);
    await expect(evidence).toContainText(`wikidata:${TOLSTOY_QID}`);
    await expect(evidence).toContainText('source:https://hi.wikipedia.org/wiki/लेव_तोलस्तोय');
    expect(requests.some((request) => request.search === 'युद्ध और शान्ति')).toBe(true);
    expect(requests.some((request) => request.ids === WAR_AND_PEACE_QID)).toBe(true);
    expect(requests.some((request) => request.ids === TOLSTOY_QID)).toBe(true);
  });

  test('resolves Chinese authorship questions through Wikidata P50', async ({ page }) => {
    const requests = [];
    await mockWikidataAuthorLookup(page, requests);

    const message = await sendPrompt(page, '战争与和平是谁写的?');
    const evidence = message.locator('.evidence-list');

    await expect(message).toContainText('《战争与和平》由列夫·托尔斯泰创作。');
    await expect(message).not.toContainText('unknown');
    await expect(message.locator('.intent')).toContainText('intent:fact_query');
    await expect(evidence).toContainText('fact_query:relation:author_of_book');
    await expect(evidence).toContainText('language:zh');
    await expect(evidence).toContainText(`wikidata:${WAR_AND_PEACE_QID}`);
    await expect(evidence).toContainText(`wikidata:${TOLSTOY_QID}`);
    await expect(evidence).toContainText('source:https://zh.wikipedia.org/wiki/列夫·托尔斯泰');
    expect(requests.some((request) => request.search === '战争与和平')).toBe(true);
    expect(requests.some((request) => request.ids === WAR_AND_PEACE_QID)).toBe(true);
    expect(requests.some((request) => request.ids === TOLSTOY_QID)).toBe(true);
  });
});
