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

async function mockRecordsSearchProviders(page) {
  await page.route('**://api.duckduckgo.com/**', async (route) => {
    await route.fulfill({
      status: 200,
      contentType: 'application/json',
      body: JSON.stringify({
        Heading: 'Boeing financial records',
        AbstractText:
          'Boeing publishes annual reports and SEC filings that summarize its financial records after major events.',
        AbstractURL: 'https://www.boeing.com/company/general-info/',
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
            key: 'Boeing',
            title: 'Boeing',
            excerpt: 'Boeing is an American aerospace company with public financial records.',
            description: 'aerospace company',
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
            id: 'Q66',
            label: 'Boeing',
            description: 'American aerospace company',
            concepturi: 'https://www.wikidata.org/wiki/Q66',
          },
        ],
      }),
    });
  });

  await page.route('**://*.wiktionary.org/w/api.php**', async (route) => {
    await route.fulfill({
      status: 200,
      contentType: 'application/json',
      body: JSON.stringify(['record', ['record'], ['A piece of recorded information.'], []]),
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

// The originally reported prompt (issue #426) plus multilingual siblings. Each
// is a verbless "records/financials/statistics about a subject" request that
// previously fell through to the unknown-prompt report.
const RECORDS_PROMPTS = [
  {
    language: 'en',
    prompt: 'Financical records for boeing after crysis with icas system',
    expectedQuery: 'financical records for boeing after crysis with icas system',
  },
  {
    language: 'ru',
    prompt: 'финансовые отчёты о boeing после кризиса',
    expectedQuery: 'финансовые отчёты о boeing после кризиса',
  },
  {
    language: 'hi',
    prompt: 'boeing के बारे में वित्तीय रिकॉर्ड',
    expectedQuery: 'boeing के बारे में वित्तीय रिकॉर्ड',
  },
  {
    language: 'zh',
    prompt: '关于波音的财务记录',
    expectedQuery: '关于波音的财务记录',
  },
];

test.describe('Issue #426 — records/financials about a subject use web search', () => {
  test.beforeEach(async ({ page }) => {
    await disableGreetingVariations(page);
    await mockRecordsSearchProviders(page);
    await page.goto('./');
    await expect(page.locator('.app')).toBeVisible({ timeout: 15_000 });
    await switchToManualMode(page);
  });

  for (const testCase of RECORDS_PROMPTS) {
    test(`${testCase.language} records prompt routes to web_search`, async ({ page }) => {
      await page.locator('.diagnostics-toggle').click();

      const last = await sendPrompt(page, testCase.prompt);

      // The "Search results for" heading is localized per language, so assert
      // on the rendered provider content (English in our mocks) plus the
      // language-independent intent and evidence signals instead.
      await expect(last).toContainText('Boeing financial records');
      await expect(last).not.toContainText(UNKNOWN_ANSWER_MARKER);
      await expect(last.locator('.intent')).toContainText('intent:web_search');
      await expect(last.locator('.evidence-list')).toContainText(
        `web_search:request:${testCase.expectedQuery}`,
      );
      await expect(last.locator('.evidence-list')).toContainText(
        'web_search:query_kind:records_information_request',
      );
    });
  }
});
