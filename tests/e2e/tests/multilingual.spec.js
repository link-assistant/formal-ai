// @ts-check
const { test, expect } = require('@playwright/test');

async function switchToManualMode(page) {
  const demoToggle = page.locator('.mode-toggle');
  await expect(demoToggle).toContainText('Demo on');
  await demoToggle.click();
  await expect(demoToggle).toContainText('Demo');
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

test.describe('multilingual chat surface', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
    await expect(page.locator('.app')).toBeVisible({ timeout: 15_000 });
    await switchToManualMode(page);
  });

  test('Russian greeting replies in Russian', async ({ page }) => {
    const last = await sendPrompt(page, 'Привет');
    await expect(last).toHaveClass(/assistant/);
    await expect(last).toContainText(/Здравствуйте|Привет/);
  });

  test('Hindi greeting replies in Hindi', async ({ page }) => {
    const last = await sendPrompt(page, 'नमस्ते');
    await expect(last).toHaveClass(/assistant/);
    await expect(last).toContainText('नमस्ते');
  });

  test('Chinese identity question replies in Chinese', async ({ page }) => {
    const last = await sendPrompt(page, '你是谁?');
    await expect(last).toHaveClass(/assistant/);
    await expect(last).toContainText('formal-ai');
    await expect(last).toContainText(/符号|确定性/);
  });

  test('Russian "What is X?" returns the offline concept summary', async ({ page }) => {
    const last = await sendPrompt(page, 'Что такое Википедия?');
    await expect(last).toHaveClass(/assistant/);
    await expect(last).toContainText(/Wikipedia|encyclopedia/i);
  });

  test('Chinese "X 是什么?" returns the offline concept summary', async ({ page }) => {
    const last = await sendPrompt(page, '维基百科是什么?');
    await expect(last).toHaveClass(/assistant/);
    await expect(last).toContainText(/Wikipedia|encyclopedia/i);
  });
});

test.describe('Wikipedia REST fallback', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
    await expect(page.locator('.app')).toBeVisible({ timeout: 15_000 });
    await switchToManualMode(page);
  });

  test('"What is X?" for an out-of-corpus term fetches a Wikipedia summary', async ({ page }) => {
    // Stub the Wikipedia REST endpoint so the test is hermetic and does not depend
    // on external network availability or rate limiting.
    await page.route('**/api/rest_v1/page/summary/**', async (route) => {
      const json = {
        title: 'Albert Einstein',
        extract: 'Albert Einstein was a German-born theoretical physicist...',
        type: 'standard',
        content_urls: {
          desktop: { page: 'https://en.wikipedia.org/wiki/Albert_Einstein' },
        },
      };
      await route.fulfill({
        status: 200,
        contentType: 'application/json',
        body: JSON.stringify(json),
      });
    });

    const last = await sendPrompt(page, 'What is Albert Einstein?');
    await expect(last).toHaveClass(/assistant/);
    await expect(last).toContainText('Albert Einstein');
    await expect(last).toContainText('theoretical physicist');
    await expect(last).toContainText('en.wikipedia.org');
  });
});

test.describe('memory export/import', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
    await expect(page.locator('.app')).toBeVisible({ timeout: 15_000 });
    await switchToManualMode(page);
  });

  test('Export memory and Import memory buttons are present', async ({ page }) => {
    await expect(page.locator('[data-testid="memory-export"]')).toBeVisible();
    await expect(page.locator('[data-testid="memory-import"]')).toBeVisible();
  });

  test('Export memory downloads a Links Notation file', async ({ page }) => {
    // Send one message so there is at least one event in the log.
    await sendPrompt(page, 'Hi');

    const [download] = await Promise.all([
      page.waitForEvent('download'),
      page.locator('[data-testid="memory-export"]').click(),
    ]);

    expect(download.suggestedFilename()).toBe('formal-ai-memory.lino');

    const path = await download.path();
    expect(path).toBeTruthy();
    const fs = require('node:fs');
    const text = fs.readFileSync(path, 'utf8');
    expect(text).toContain('demo_memory');
    expect(text).toContain('role "user"');
    expect(text).toContain('content "Hi"');
    // Status indicator should reflect the export count.
    await expect(page.locator('[data-testid="memory-status"]')).toContainText(/Exported \d+ events/);
  });

  test('Import memory accepts a Links Notation file', async ({ page }) => {
    const importInput = page.locator('[data-testid="memory-import-input"]');
    const lino = [
      'demo_memory',
      '  event "1"',
      '    role "user"',
      '    content "Imported greeting"',
      '    sentAt "2026-05-15T12:00:00.000Z"',
      '  event "2"',
      '    role "assistant"',
      '    intent "greeting"',
      '    content "Hi, how may I help you?"',
      '    sentAt "2026-05-15T12:00:01.000Z"',
      '',
    ].join('\n');
    await importInput.setInputFiles({
      name: 'memory.lino',
      mimeType: 'text/plain',
      buffer: Buffer.from(lino, 'utf8'),
    });
    await expect(page.locator('[data-testid="memory-status"]')).toContainText('Imported 2 events');
  });

  test('Memory module exposes no delete/forget operation', async ({ page }) => {
    const api = await page.evaluate(() => Object.keys(window.FormalAiMemory || {}));
    expect(api).toContain('appendEvent');
    expect(api).toContain('listEvents');
    expect(api).toContain('importEvents');
    expect(api).toContain('exportLinksNotation');
    expect(api).not.toContain('delete');
    expect(api).not.toContain('deleteEvent');
    expect(api).not.toContain('forget');
    expect(api).not.toContain('clear');
    expect(api).not.toContain('remove');
  });
});
