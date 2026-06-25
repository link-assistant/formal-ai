// @ts-check
const { test, expect } = require('@playwright/test');

const UNKNOWN_ANSWER_MARKER = 'cannot answer that from local links rules';

const REPORTED_PROMPT = `Search for information about:
1. Machine learning algorithms
2. Deep learning vs traditional ML
3. Neural networks basics

Then create a comparison table showing:
- Key differences
- Use cases for each
- Advantages and disadvantages`;

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

async function mockMachineLearningSearchProviders(page) {
  await page.route('**://api.duckduckgo.com/**', async (route) => {
    await route.fulfill({
      status: 200,
      contentType: 'application/json',
      body: JSON.stringify({
        Heading: 'Machine learning',
        AbstractText:
          'Machine learning includes algorithms that learn patterns from data, including neural networks and deep learning.',
        AbstractURL: 'https://en.wikipedia.org/wiki/Machine_learning',
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
              identifier: 'machine-learning-overview',
              title: 'Machine learning overview',
              description:
                'Overview of machine learning algorithms, neural networks, and deep learning methods.',
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
            key: 'Machine_learning',
            title: 'Machine learning',
            excerpt:
              'Machine learning algorithms build models from sample data to make predictions or decisions.',
            description: 'field of study in artificial intelligence',
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
            id: 'Q2539',
            label: 'Machine learning',
            description: 'scientific study of algorithms and statistical models',
            concepturi: 'https://www.wikidata.org/wiki/Q2539',
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
        'learning',
        ['learning'],
        ['The acquisition of knowledge or skill.'],
        ['https://en.wiktionary.org/wiki/learning'],
      ]),
    });
  });
}

test.describe('Issue #339 — agent research table follow-up', () => {
  test.beforeEach(async ({ page }) => {
    await disableGreetingVariations(page);
    await mockMachineLearningSearchProviders(page);
    await page.goto('./');
    await expect(page.locator('.app')).toBeVisible({ timeout: 15_000 });
    await switchToManualMode(page);
  });

  test('reported search-then-table prompt does not fall through to unknown', async ({ page }) => {
    await page.locator('[data-testid="mode-option-agent"]').click();

    const last = await sendPrompt(page, REPORTED_PROMPT);

    await expect(last).toContainText('Agent plan (2 steps)');
    await expect(last).toContainText('Step 1: Search for information about');
    await expect(last).toContainText('Search results for');
    await expect(last).toContainText('Step 2: create a comparison table showing');
    await expect(last).toContainText('Research comparison table');
    await expect(last).toContainText('Machine learning algorithms');
    await expect(last).toContainText('Deep learning vs traditional ML');
    await expect(last).toContainText('Neural networks basics');
    await expect(last).not.toContainText(UNKNOWN_ANSWER_MARKER);
  });
});
