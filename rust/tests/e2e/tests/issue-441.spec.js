// @ts-check
const { test, expect } = require('@playwright/test');

const UNKNOWN_ANSWER_MARKER = 'cannot answer that from local links rules';

async function sendPrompt(page, text) {
  const input = page.locator('[data-testid="chat-composer-input"]');
  await expect(input).toBeEnabled({ timeout: 5_000 });
  await input.fill(text);

  const messages = page.locator('[data-testid="chat-message"]');
  const initialCount = await messages.count();
  await page.locator('[data-testid="chat-composer-submit"]').click();
  await expect(messages).toHaveCount(initialCount + 2, { timeout: 20_000 });

  const lastMessage = messages.last();
  await expect(lastMessage).toHaveClass(/assistant/);
  return lastMessage;
}

test.describe('Issue #441 mixed-script Wikipedia lookup', () => {
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
  });

  test('Russian definition prompt with Latin technical term stays Russian', async ({
    page,
  }) => {
    const requestedSummaries = [];

    await page.route('**/*', async (route) => {
      const url = new URL(route.request().url());
      if (['localhost', '127.0.0.1'].includes(url.hostname)) {
        await route.continue();
        return;
      }

      if (url.pathname.includes('/api/rest_v1/page/summary/')) {
        const slug = decodeURIComponent(url.pathname.split('/').pop() || '');
        requestedSummaries.push({ host: url.hostname, slug });
        if (url.hostname === 'ru.wikipedia.org' && slug.toLowerCase() === 'vulkan_layer') {
          await route.fulfill({
            status: 200,
            contentType: 'application/json',
            body: JSON.stringify({
              title: 'Vulkan layer',
              extract:
                'A Vulkan layer can intercept Vulkan API calls before they reach the driver.',
              type: 'standard',
              content_urls: {
                desktop: { page: 'https://ru.wikipedia.org/wiki/Vulkan_layer' },
              },
            }),
          });
          return;
        }
        await route.fulfill({
          status: 404,
          contentType: 'application/json',
          body: JSON.stringify({ httpCode: 404, httpReason: 'Not Found' }),
        });
        return;
      }

      await route.abort();
    });

    const answer = await sendPrompt(page, 'Что такое vulkan layer');
    const body = answer.locator('.markdown-body');

    await expect(answer.locator('.intent')).toContainText('intent:wikipedia_lookup');
    await expect(body).toContainText('Vulkan layer');
    await expect(body).toContainText('intercept Vulkan API calls');
    await expect(answer.locator('.evidence-list')).toContainText('language:ru');
    await expect(body).not.toContainText(UNKNOWN_ANSWER_MARKER);
    expect(requestedSummaries[0]).toMatchObject({
      host: 'ru.wikipedia.org',
      slug: 'vulkan_layer',
    });
  });
});
