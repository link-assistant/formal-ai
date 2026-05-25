// @ts-check
const { test, expect } = require('@playwright/test');

test.describe('formal-ai connectivity diagnostics', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('./tests/');
    await expect(page.locator('[data-testid="connectivity-dashboard"]')).toBeVisible({
      timeout: 15_000,
    });
  });

  test('renders search engines, public knowledge APIs, and proxy controls', async ({ page }) => {
    await expect(page).toHaveTitle('formal-ai connectivity tests');
    await expect(page.locator('[data-testid="proxy-base-input"]')).toHaveValue(
      'http://localhost:3000',
    );
    await expect(page.locator('[data-testid="proxy-endpoint-select"]')).toHaveValue('/fetch');

    const rows = page.locator('[data-service-row="true"]');
    await expect.poll(async () => rows.count()).toBeGreaterThanOrEqual(10);

    for (const serviceId of [
      'google-web',
      'bing-web',
      'duckduckgo-web',
      'brave-web',
      'wikipedia-api',
      'wikidata-api',
      'wiktionary-api',
      'cambridge-dictionary',
      'merriam-webster-dictionary',
      'openlibrary-api',
      'openalex-api',
    ]) {
      await expect(page.locator(`[data-testid="service-${serviceId}"]`)).toBeVisible();
    }
  });

  test('lists dictionary page sources for proxy-based connectivity checks', async ({ page }) => {
    for (const serviceId of [
      'cambridge-dictionary',
      'merriam-webster-dictionary',
      'dictionary-com',
      'collins-dictionary',
    ]) {
      const row = page.locator(`[data-testid="service-${serviceId}"]`);
      await expect(row).toBeVisible();
      await expect(row.locator('[data-testid="api-status"]')).toContainText('No API');
      await expect(row.locator('[data-testid="run-api-fetch"]')).toBeDisabled();
      await expect(row.locator('.target-line').first()).toContainText(/digress/i);
    }
  });

  test('fetches a CORS-readable public knowledge API directly', async ({ page }) => {
    await page.route('https://en.wikipedia.org/w/rest.php/v1/search/page**', async (route) => {
      await route.fulfill({
        status: 200,
        contentType: 'application/json',
        body: JSON.stringify({
          pages: [
            {
              id: 18742,
              title: 'Formal language',
              excerpt: 'Formal language is a set of strings.',
            },
          ],
        }),
      });
    });

    const row = page.locator('[data-testid="service-wikipedia-api"]');
    await row.locator('[data-testid="run-api-fetch"]').click();

    await expect(row.locator('[data-testid="api-status"]')).toContainText('200 OK');
    await expect(row.locator('[data-testid="api-final-url"]')).toContainText(
      'en.wikipedia.org',
    );
    await expect(row.locator('[data-testid="result-preview"]')).toContainText(
      'Formal language',
    );
  });

  test('records blocked direct page fetches without hiding iframe diagnostics', async ({
    page,
  }) => {
    await page.route('https://www.google.com/search**', async (route) => {
      await route.abort('failed');
    });

    const row = page.locator('[data-testid="service-google-web"]');
    await row.locator('[data-testid="run-page-fetch"]').click();

    await expect(row.locator('[data-testid="page-status"]')).toContainText(/blocked|failed/i);
    await expect(row.locator('[data-testid="result-preview"]')).toContainText(
      'Direct browser fetch failed',
    );
    await expect(row.locator('[data-testid="toggle-frame"]')).toBeEnabled();
  });

  test('routes fetches through the configured web-capture proxy', async ({ page }) => {
    await page.route('http://localhost:3000/fetch?**', async (route) => {
      const proxiedUrl = new URL(route.request().url()).searchParams.get('url') || '';
      expect(proxiedUrl).toContain('www.google.com/search');
      await route.fulfill({
        status: 200,
        contentType: 'text/plain',
        body: 'proxied google search html',
      });
    });

    await page.locator('[data-testid="mode-proxy"]').click();
    await expect(page.locator('[data-testid="proxy-settings"]')).toBeVisible();

    const row = page.locator('[data-testid="service-google-web"]');
    await row.locator('[data-testid="run-page-fetch"]').click();

    await expect(row.locator('[data-testid="page-status"]')).toContainText('200 OK');
    await expect(row.locator('[data-testid="proxy-final-url"]')).toContainText(
      'localhost:3000/fetch?url=',
    );
    await expect(row.locator('[data-testid="result-preview"]')).toContainText(
      'proxied google search html',
    );
  });

  test('opens and expands iframe diagnostics for a page endpoint', async ({ page }) => {
    const row = page.locator('[data-testid="service-wikipedia-api"]');
    await row.locator('[data-testid="toggle-frame"]').click();

    await expect(row.locator('[data-testid="frame-panel"]')).toBeVisible();
    await expect(row.locator('[data-testid="frame-panel"] iframe')).toHaveAttribute(
      'src',
      /en\.wikipedia\.org/,
    );

    await row.locator('[data-testid="expand-frame"]').click();
    await expect(page.locator('[data-testid="frame-overlay"]')).toBeVisible();
    await expect(page.locator('[data-testid="frame-overlay"] iframe')).toHaveAttribute(
      'src',
      /en\.wikipedia\.org/,
    );

    await page.locator('[data-testid="close-frame-overlay"]').click();
    await expect(page.locator('[data-testid="frame-overlay"]')).toBeHidden();
  });
});
