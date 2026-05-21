// @ts-check
const { test, expect } = require('@playwright/test');

// Issue #180 — every solve() turn must end with a `deformalize` diagnostics
// step that projects the resolved formalization back to natural language. The
// fact-style and web-search handlers must additionally emit a
// `formalize_resolved` step that folds the resolved Q-id into the SVO tuple.
// These tests exercise the visible diagnostics panel so a regression in the
// worker's finalize() routine surfaces as a UI bug too.

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

async function openAllDetails(messageLocator) {
  await messageLocator.evaluate((node) => {
    for (const det of node.querySelectorAll('details.diagnostics-detail')) {
      det.open = true;
    }
  });
}

async function readStepNames(messageLocator) {
  // The worker tags every diagnostics row with a stable `data-step` attribute so
  // assertions stay independent of the i18n-localised display label (e.g. the
  // formalize/formalize_resolved entries both render as "Formalization").
  return await messageLocator
    .locator('[data-testid="diagnostics-step"]')
    .evaluateAll((nodes) => nodes.map((node) => node.getAttribute('data-step') || ''));
}

// `user_context` is appended by the App layer after the worker's pipeline
// completes; it carries environment metadata and is not a reasoning step.
const REASONING_STEPS = (names) => names.filter((name) => name !== 'user_context');

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
  await page.route('**://*.wikipedia.org/w/rest.php/v1/search/page**', async (route) => {
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
  await page.route('**://www.wikidata.org/w/api.php**', async (route) => {
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

test.describe('Issue #180 — always-on deformalize step', () => {
  test.beforeEach(async ({ page }) => {
    await disableGreetingVariations(page);
    await page.goto('./');
    await expect(page.locator('.app')).toBeVisible({ timeout: 15_000 });
    await switchToManualMode(page);
  });

  test('greeting prompt emits a deformalize step as the last reasoning step', async ({
    page,
  }) => {
    await page.locator('.diagnostics-toggle').click();
    const last = await sendPrompt(page, 'hi');
    await openAllDetails(last);

    const stepNames = await readStepNames(last);
    const reasoning = REASONING_STEPS(stepNames);
    expect(reasoning).toContain('impulse');
    expect(reasoning).toContain('deformalize');
    // `deformalize` must be the final *reasoning* step — `user_context` rows
    // emitted by the App layer afterwards are environment metadata, not part
    // of the pipeline, and are filtered out above.
    expect(reasoning[reasoning.length - 1]).toBe('deformalize');
  });

  test('unknown prompt still ends with deformalize', async ({ page }) => {
    await page.locator('.diagnostics-toggle').click();
    const last = await sendPrompt(page, 'asdfasdfasdf');
    await openAllDetails(last);

    const stepNames = await readStepNames(last);
    const reasoning = REASONING_STEPS(stepNames);
    expect(reasoning).toContain('impulse');
    expect(reasoning).toContain('deformalize');
    expect(reasoning[reasoning.length - 1]).toBe('deformalize');
  });

  test('web_search emits formalize_resolved followed by deformalize', async ({
    page,
  }) => {
    await mockAppleSearch(page);
    await page.locator('.diagnostics-toggle').click();

    const last = await sendPrompt(page, 'Search the web for Apple');
    await openAllDetails(last);

    const stepNames = await readStepNames(last);
    const reasoning = REASONING_STEPS(stepNames);
    expect(reasoning).toContain('impulse');
    // Both the initial `formalize` step and the rule-resolved
    // `formalize_resolved` step must be visible — they share the localised
    // "Formalization" display label but the raw step keys differ.
    expect(reasoning).toContain('formalize');
    expect(reasoning).toContain('formalize_resolved');
    // The resolved formalization must come before the deformalize hand-off.
    expect(reasoning.indexOf('formalize_resolved')).toBeLessThan(
      reasoning.indexOf('deformalize'),
    );
    expect(reasoning[reasoning.length - 1]).toBe('deformalize');

    // The deformalize summary must use the ⇒ glyph that the worker emits in
    // `projection.summary` so the symbolic-to-natural-language hand-off is
    // visible in the diagnostics row, not just in the underlying step.
    const deformalizeSummary = last
      .locator('[data-testid="diagnostics-step"][data-step="deformalize"]')
      .locator('.diagnostics-step-summary');
    await expect(deformalizeSummary).toContainText('⇒');
  });
});

test.describe('Issue #212 — Russian web-search word order', () => {
  test.beforeEach(async ({ page }) => {
    await disableGreetingVariations(page);
    await page.goto('./');
    await expect(page.locator('.app')).toBeVisible({ timeout: 15_000 });
    await switchToManualMode(page);
  });

  test('Найди яблоко в интернете routes to web_search instead of unknown', async ({
    page,
  }) => {
    await mockAppleSearch(page);
    await page.locator('.diagnostics-toggle').click();

    const last = await sendPrompt(page, 'Найди яблоко в интернете');

    await expect(last.locator('.intent')).toContainText('intent:web_search');
    await expect(last.locator('.markdown-body')).toContainText('яблоко');
    await expect(last.locator('.markdown-body')).not.toContainText(
      'Я пока не могу ответить',
    );
  });
});
