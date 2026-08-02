// @ts-check
// Issue #709: live search is presented as ranked statements with source-level
// provenance, including cross-language facts and explicit disagreements.
const { test, expect } = require('@playwright/test');

async function sendPrompt(page, text) {
  const input = page.locator('[data-testid="chat-composer-input"]');
  await expect(input).toBeEnabled({ timeout: 5_000 });
  await input.fill(text);
  const messages = page.locator('[data-testid="chat-message"]');
  const before = await messages.count();
  await page.locator('[data-testid="chat-composer-submit"]').click();
  await expect(messages).toHaveCount(before + 2, { timeout: 20_000 });
  return messages.last();
}

async function mockProviders(page) {
  await page.route('**://api.duckduckgo.com/**', async (route) => {
    const query = new URL(route.request().url()).searchParams.get('q') || '';
    let body = { Heading: '', AbstractText: '', AbstractURL: '', RelatedTopics: [] };
    if (/apple taxonomy/i.test(query)) {
      body = {
        Heading: 'Русский ботанический справочник',
        AbstractText: 'Яблоко это фрукт.',
        AbstractURL: 'https://foreign.invalid/apple',
        SourceTier: 'original_first_party',
        SourceLanguage: 'ru',
        RelatedTopics: [],
      };
    } else if (/parser speed/i.test(query)) {
      body = {
        Heading: 'Official benchmark',
        AbstractText: 'The parser is fast.',
        AbstractURL: 'https://speed.invalid/official',
        SourceTier: 'original_first_party',
        SourceLanguage: 'en',
        RelatedTopics: [{
          FirstURL: 'https://speed.invalid/lab',
          Text: 'Independent lab - The parser is not fast.',
          SourceTier: 'independent_corroboration',
          SourceLanguage: 'en',
        }],
      };
    }
    await route.fulfill({
      status: 200,
      contentType: 'application/json',
      body: JSON.stringify(body),
    });
  });
  await page.route('**/w/rest.php/v1/search/page**', async (route) => {
    await route.fulfill({ status: 200, contentType: 'application/json', body: '{"pages":[]}' });
  });
  await page.route('**://*.wikidata.org/w/api.php**', async (route) => {
    await route.fulfill({ status: 200, contentType: 'application/json', body: '{"search":[]}' });
  });
  await page.route('**://*.wiktionary.org/w/api.php**', async (route) => {
    await route.fulfill({ status: 200, contentType: 'application/json', body: '["",[],[],[]]' });
  });
  await page.route('**://*.wikinews.org/w/api.php**', async (route) => {
    await route.fulfill({ status: 200, contentType: 'application/json', body: '["",[],[],[]]' });
  });
  await page.route('**://archive.org/advancedsearch.php**', async (route) => {
    await route.fulfill({
      status: 200,
      contentType: 'application/json',
      body: '{"response":{"docs":[]}}',
    });
  });
}

test.describe('Issue #709 - ranked provenance answers', () => {
  test.beforeEach(async ({ page }) => {
    await page.addInitScript(() => {
      window.localStorage.setItem(
        'formal-ai.preferences.v1',
        'demo_preferences\n  demoMode "off"\n  diagnosticsMode "on"\n  greetingVariations "off"',
      );
    });
    await mockProviders(page);
    await page.goto('./');
    await expect(page.locator('.app')).toBeVisible({ timeout: 15_000 });
  });

  test('deformalizes a decisive foreign fact and renders its normalized source card', async ({
    page,
  }) => {
    const last = await sendPrompt(page, 'Search the web for apple taxonomy');
    await expect(last).toContainText('Apple is a fruit.');
    await expect(last).toContainText('Русский ботанический справочник');
    await expect(last).toContainText('Яблоко это фрукт.');
    await expect(last).toContainText('posterior=1.000000');
    await expect(last).toContainText('source_tier=original_first_party');
    await expect(last.getByRole('link', { name: 'Read more' })).toHaveAttribute(
      'href',
      'https://foreign.invalid/apple',
    );
    await expect(last.locator('.evidence-list')).toContainText('wikidata:Q89');
    await expect(last.locator('.evidence-list')).toContainText('wikidata:P31');
    await expect(last.locator('.evidence-list')).toContainText('wikidata:Q3314483');
    if (process.env.ISSUE_709_SCREENSHOT) {
      await last.locator('[data-testid="message-markdown-body"]').screenshot({
        path: 'docs/screenshots/issue-709-search-fusion.png',
      });
    }
  });

  test('keeps both contradiction sides with tiers and posteriors', async ({ page }) => {
    const last = await sendPrompt(page, 'Search the web for parser speed');
    await expect(last).toContainText('The parser is fast.');
    await expect(last).toContainText('The parser is not fast.');
    await expect(last).toContainText('conflict=source_disagreement');
    await expect(last).toContainText('source_tier=original_first_party');
    await expect(last).toContainText('source_tier=independent_corroboration');
    await expect(last.getByRole('link', { name: 'Read more' })).toHaveCount(2);
    await expect(last.locator('.evidence-list')).toContainText('conflict:source_disagreement');
  });

  test('bounds a full provider result set without wedging the WASM worker', async ({ page }) => {
    const archivePattern = '**://archive.org/advancedsearch.php**';
    await page.unroute(archivePattern);
    await page.route(archivePattern, async (route) => {
      const description = 'Allocator apple evidence is bounded for fusion. '.repeat(70);
      const docs = Array.from({ length: 10 }, (_, index) => ({
        identifier: `bounded-${index}`,
        title: `Bounded source ${index}`,
        description,
      }));
      await route.fulfill({
        status: 200,
        contentType: 'application/json',
        body: JSON.stringify({ response: { docs } }),
      });
    });

    const last = await sendPrompt(page, 'Search the web for allocator apple');
    await expect(last).toContainText('Bounded source 0');
    await expect(last).toContainText('source_count=10');
    await expect(last.getByRole('link', { name: 'Read more' })).toHaveCount(10);
  });
});
