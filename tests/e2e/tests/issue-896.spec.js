// @ts-check
// Issue #896: the web app must cross the published web-search library and
// web-capture HTTP boundaries before using its bounded local fallbacks.
const { test, expect } = require('@playwright/test');

async function boot(page) {
  await page.addInitScript(() => {
    window.localStorage.setItem(
      'formal-ai.preferences.v1',
      'demo_preferences\n  demoMode "off"\n  diagnosticsMode "on"\n  greetingVariations "off"',
    );
  });
  await page.goto('./');
  await expect(page.locator('.app')).toBeVisible({ timeout: 15_000 });
}

async function sendPrompt(page, text) {
  const input = page.locator('[data-testid="chat-composer-input"]');
  const assistants = page.locator('[data-testid="chat-message"].assistant');
  const before = await assistants.count();
  await expect(input).toBeEnabled({ timeout: 10_000 });
  await input.fill(text);
  await page.locator('[data-testid="chat-composer-submit"]').click();
  await expect.poll(() => assistants.count(), { timeout: 20_000 }).toBeGreaterThan(before);
  return assistants.last();
}

async function mockSearchProviders(page) {
  await page.route('**://api.duckduckgo.com/**', (route) =>
    route.fulfill({
      status: 200,
      contentType: 'application/json',
      body: JSON.stringify({
        Heading: 'Published component result',
        AbstractText: 'The component boundary result is captured.',
        AbstractURL: 'https://component.invalid/search-result',
        RelatedTopics: [],
      }),
    }),
  );
  await page.route('**://archive.org/advancedsearch.php**', (route) =>
    route.fulfill({ status: 200, contentType: 'application/json', body: '{"response":{"docs":[]}}' }),
  );
  await page.route('**/w/rest.php/v1/search/page**', (route) =>
    route.fulfill({ status: 200, contentType: 'application/json', body: '{"pages":[]}' }),
  );
  await page.route('**://www.wikidata.org/w/api.php**', (route) =>
    route.fulfill({ status: 200, contentType: 'application/json', body: '{"search":[]}' }),
  );
  await page.route('**://*.wiktionary.org/w/api.php**', (route) =>
    route.fulfill({ status: 200, contentType: 'application/json', body: '["",[],[],[]]' }),
  );
  await page.route('**://*.wikinews.org/w/api.php**', (route) =>
    route.fulfill({ status: 200, contentType: 'application/json', body: '["",[],[],[]]' }),
  );
}

const SUPPORTED_LANGUAGE_SEARCHES = [
  { language: 'en', prompt: 'Search the web for component boundary' },
  { language: 'ru', prompt: 'Найди в интернете component boundary' },
  { language: 'hi', prompt: 'इंटरनेट पर component boundary खोजें' },
  { language: 'zh', prompt: '在网上搜索 component boundary' },
  { language: 'es', prompt: 'Busca en internet component boundary' },
];

for (const { language, prompt } of SUPPORTED_LANGUAGE_SEARCHES) {
  test(`browser search uses the published registry and merger for language: "${language}"`, async ({ page }) => {
    await mockSearchProviders(page);
    await boot(page);

    const answer = await sendPrompt(page, prompt);
    await expect(answer).toContainText('Published component result');
    await expect(answer.locator('.evidence-list')).toContainText(
      'web_search:component:@link-assistant/web-search@0.10.3:defaultProviders',
    );
    await expect(answer.locator('.evidence-list')).toContainText(
      'web_search:component:@link-assistant/web-search@0.10.3:mergeResults',
    );
  });
}

test('browser HTTP fetch prefers web-capture and preserves target bytes', async ({ page }) => {
  let directTargetCalls = 0;
  let componentCalls = 0;
  await page.route('http://localhost:3000/fetch?**', (route) => {
    componentCalls += 1;
    return route.fulfill({
      status: 503,
      contentType: 'text/plain',
      headers: { 'access-control-allow-origin': '*' },
      body: 'exact bytes returned through web-capture',
    });
  });
  await page.route('https://capture.invalid/article', (route) => {
    directTargetCalls += 1;
    return route.fulfill({ status: 200, body: 'direct fallback must not run' });
  });
  await boot(page);

  const answer = await sendPrompt(page, 'Fetch https://capture.invalid/article');
  await expect(answer).toContainText('exact bytes returned through web-capture');
  await expect(answer.locator('.evidence-list')).toContainText(
    'http_fetch:component:web-capture:http-get-fetch',
  );
  await expect(answer.locator('.evidence-list')).toContainText('http_fetch:status:503');
  expect(componentCalls).toBe(1);
  expect(directTargetCalls).toBe(0);
});

test('browser HTTP fetch reports a component transport failure before bounded fallback', async ({ page }) => {
  let directTargetCalls = 0;
  await page.route('http://localhost:3000/fetch?**', (route) =>
    route.abort('failed'),
  );
  await page.route('https://capture.invalid/fallback', (route) => {
    directTargetCalls += 1;
    return route.fulfill({ status: 200, contentType: 'text/plain', body: 'bounded direct fallback' });
  });
  await boot(page);

  const answer = await sendPrompt(page, 'Fetch https://capture.invalid/fallback');
  await expect(answer).toContainText('bounded direct fallback');
  await expect(answer.locator('.evidence-list')).toContainText(
    'http_fetch:component_error:network',
  );
  expect(directTargetCalls).toBe(1);
});

test('browser web-capture request is cancelled before bounded fallback', async ({ page }) => {
  let directTargetCalls = 0;
  await page.route('http://localhost:3000/fetch?**', async (route) => {
    await new Promise((resolve) => setTimeout(resolve, 2_500));
    try {
      await route.fulfill({ status: 200, contentType: 'text/plain', body: 'too late' });
    } catch (_error) {
      // The worker-owned AbortController is expected to cancel this request.
    }
  });
  await page.route('https://capture.invalid/timeout', (route) => {
    directTargetCalls += 1;
    return route.fulfill({ status: 200, contentType: 'text/plain', body: 'fallback after cancellation' });
  });
  await boot(page);

  const answer = await sendPrompt(page, 'Fetch https://capture.invalid/timeout');
  await expect(answer).toContainText('fallback after cancellation');
  await expect(answer.locator('.evidence-list')).toContainText(
    'http_fetch:component_error:timeout',
  );
  expect(directTargetCalls).toBe(1);
});
