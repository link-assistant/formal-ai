// @ts-check
const { test, expect } = require('@playwright/test');

const CDN_ORIGINS = new Set([
  'https://unpkg.com',
  'https://cdn.jsdelivr.net',
  'https://esm.sh',
]);

function isCdnUrl(url) {
  try {
    return CDN_ORIGINS.has(new URL(url).origin);
  } catch (_error) {
    return false;
  }
}

test.describe('Issue #193 bundled web runtime', () => {
  test('starts and translates UI when external JavaScript CDNs are unavailable', async ({
    page,
  }) => {
    const blockedCdnRequests = [];
    const scriptCdnRequests = [];

    await page.route(/^https:\/\/(?:unpkg\.com|cdn\.jsdelivr\.net|esm\.sh)\//, (route) => {
      blockedCdnRequests.push(route.request().url());
      return route.abort();
    });

    page.on('request', (request) => {
      if (request.resourceType() === 'script' && isCdnUrl(request.url())) {
        scriptCdnRequests.push(request.url());
      }
    });

    await page.goto('./');
    await expect(page.locator('.app')).toBeVisible({ timeout: 15_000 });

    const runtime = await page.evaluate(async () => {
      await window.FormalAiI18n.ready;
      return {
        engine: window.FormalAiI18n.ENGINE_SOURCE,
        russian: window.FormalAiI18n.t('buttons.reportIssue', 'ru'),
        fallback: window.FormalAiI18n.t('buttons.reportIssue', 'zz'),
        lastError: window.FormalAiI18n.lastError,
      };
    });

    expect(runtime).toEqual({
      engine: 'lino-i18n@0.1.1',
      russian: 'Сообщить о проблеме',
      fallback: 'Report issue',
      lastError: null,
    });
    expect(blockedCdnRequests).toEqual([]);
    expect(scriptCdnRequests).toEqual([]);
  });
});
