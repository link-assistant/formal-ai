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

async function mockResearchSearchProviders(page) {
  await page.route('**://api.duckduckgo.com/**', async (route) => {
    await route.fulfill({
      status: 200,
      contentType: 'application/json',
      body: JSON.stringify({
        Heading: 'WMT Test Sets',
        AbstractText:
          'WMT newstest and MQM evaluation data are common benchmarks for machine translation quality evaluation.',
        AbstractURL: 'https://www2.statmt.org/wmt24/translation-task.html',
        RelatedTopics: [],
      }),
    });
  });

  await page.route('**://archive.org/advancedsearch.php**', async (route) => {
    await route.fulfill({
      status: 200,
      contentType: 'application/json',
      body: JSON.stringify({
        response: {
          docs: [
            {
              identifier: 'wmt-translation-task',
              title: 'WMT translation task',
              description:
                'Workshop on Machine Translation benchmark material for translation quality evaluation.',
            },
          ],
        },
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
            key: 'Workshop_on_Machine_Translation',
            title: 'Workshop on Machine Translation',
            excerpt:
              'The Workshop on Machine Translation publishes shared-task test sets for machine translation evaluation.',
            description: 'machine translation benchmark',
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
            id: 'Q17005157',
            label: 'Workshop on Machine Translation',
            description: 'machine translation evaluation shared task',
            concepturi: 'https://www.wikidata.org/wiki/Q17005157',
          },
        ],
      }),
    });
  });

  await page.route('**://*.wiktionary.org/w/api.php**', async (route) => {
    await route.fulfill({
      status: 200,
      contentType: 'application/json',
      body: JSON.stringify([
        'validation',
        ['validation'],
        ['The act of validating something.'],
        ['https://en.wiktionary.org/wiki/validation'],
      ]),
    });
  });
}

test.describe('Issue #224 — implicit research questions use web search', () => {
  test.beforeEach(async ({ page }) => {
    await disableGreetingVariations(page);
    await mockResearchSearchProviders(page);
    await page.goto('./');
    await expect(page.locator('.app')).toBeVisible({ timeout: 15_000 });
    await switchToManualMode(page);
  });

  test('translation-quality dataset question routes to web_search', async ({ page }) => {
    await page.locator('.diagnostics-toggle').click();

    const last = await sendPrompt(
      page,
      'What is the most popular dataset for translation quality validation?',
    );

    await expect(last).toContainText('Search results for');
    await expect(last).toContainText('WMT Test Sets');
    await expect(last).not.toContainText(UNKNOWN_ANSWER_MARKER);
    await expect(last.locator('.intent')).toContainText('intent:web_search');
    await expect(last.locator('.evidence-list')).toContainText(
      'web_search:request:most popular dataset for translation quality validation',
    );
    await expect(last.locator('.evidence-list')).toContainText(
      'web_search:query_kind:implicit_research_question',
    );
  });
});
