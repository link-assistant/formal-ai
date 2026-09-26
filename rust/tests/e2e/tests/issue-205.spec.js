// @ts-check
const { test, expect } = require('@playwright/test');

const ONE_PIXEL_PNG = Buffer.from(
  'iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAQAAAC1HAwCAAAAC0lEQVR42mP8/x8AAwMCAO+/p9sAAAAASUVORK5CYII=',
  'base64',
);

async function switchToManualMode(page) {
  const demoToggle = page.locator('.mode-toggle');
  await expect(demoToggle).toContainText(/Demo on|Demo off|Демо/, {
    timeout: 10_000,
  });
  await demoToggle.click();
  await expect(page.locator('[data-testid="demo-status"]')).toHaveText('Manual mode');
  await expect(page.locator('[data-testid="chat-composer-input"]')).toBeEnabled({
    timeout: 5_000,
  });
}

async function expandSidebarSection(page, testId) {
  const section = page.locator(`[data-testid="${testId}"]`);
  await expect(section).toBeVisible();
  if ((await section.getAttribute('data-collapsed')) === 'true') {
    await section.locator('.sidebar-section-header').click();
  }
  await expect(section).toHaveAttribute('data-collapsed', 'false');
}

test.describe('Issue #205 optional OCR image attachments', () => {
  test('keeps OCR out of the initial page and exposes an explicit data warning', async ({
    page,
  }) => {
    await page.goto('./');
    await expect(page.locator('.app')).toBeVisible({ timeout: 15_000 });

    const loadedScripts = await page.evaluate(() =>
      Array.from(document.scripts).map((script) => script.src),
    );
    expect(loadedScripts.some((src) => src.includes('ocr.bundle.js'))).toBe(false);

    await expandSidebarSection(page, 'sidebar-settings');
    const toggle = page.locator('[data-testid="setting-experimental-ocr"]');
    await expect(toggle).toBeVisible();
    await expect(toggle).not.toBeChecked();
    await expect(page.locator('[data-testid="setting-experimental-ocr-warning"]'))
      .toContainText(/downloads about .*MB/i);
  });

  test('when enabled, image attachments export base64 and OCR text in memory', async ({
    page,
  }) => {
    await page.addInitScript(() => {
      window.localStorage.setItem(
        'formal-ai.preferences.v1',
        'demo_preferences\n  greetingVariations "off"\n  experimentalOcr "on"',
      );
      window.FormalAiOcr = {
        VERSION: 'test-double',
        recognizeImage: async () => ({
          text: 'mock ocr text',
          confidence: 98.25,
        }),
      };
    });

    await page.goto('./');
    await expect(page.locator('.app')).toBeVisible({ timeout: 15_000 });
    await switchToManualMode(page);

    await expect(page.locator('[data-testid="setting-experimental-ocr"]')).toBeChecked();
    await page.locator('[data-testid="composer-attachment-input"]').setInputFiles({
      name: 'tiny.png',
      mimeType: 'image/png',
      buffer: ONE_PIXEL_PNG,
    });
    await expect(page.locator('[data-testid="composer-attachment-status"]'))
      .toContainText(/1 attached/i);

    await page.locator('[data-testid="chat-composer-input"]').fill('Read attached image');
    const messages = page.locator('[data-testid="chat-message"]');
    const initial = await messages.count();
    await page.locator('[data-testid="chat-composer-submit"]').click();
    await expect(messages).toHaveCount(initial + 2, { timeout: 20_000 });

    const [download] = await Promise.all([
      page.waitForEvent('download'),
      page.locator('[data-testid="memory-export"]').click(),
    ]);
    const path = await download.path();
    expect(path).toBeTruthy();

    const fs = require('node:fs');
    const text = fs.readFileSync(path, 'utf8');
    expect(text).toContain('attachments "');
    expect(text).toContain('data:image/png;base64,');
    expect(text).toContain('mock ocr text');
    expect(text).toContain('tiny.png');
  });
});
