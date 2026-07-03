// @ts-check
const { test, expect } = require('@playwright/test');

const ONE_PIXEL_PNG = Buffer.from(
  'iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAQAAAC1HAwCAAAAC0lEQVR42mP8/x8AAwMCAO+/p9sAAAAASUVORK5CYII=',
  'base64',
);

const ISSUE_493_OCR_TEXT = [
  '$ETH',
  'ETH in 2021: $1,700',
  'ETH in 2022: $1,700',
  'ETH in 2023: $1,700',
  'ETH in 2024: $1,700',
  'ETH in 2025: $1,700',
  'ETH in 2026: $1,700',
  'ETH before BitMine buying: $1,700',
  'ETH after BitMine buying: $1,700',
  'ETH before ETF approval: $1,700',
  'ETH after ETF approval: $1,700',
  'ETH during anti-crypto President: $1,700',
  'ETH during pro-crypto President: $1,700',
  'ETH before US-Iran war: $1,700',
  'ETH after US-Iran war: $1,700',
  '',
  'Performance of $ETH is an absolute joke.',
].join('\n');

async function sendPrompt(page, text) {
  const input = page.locator('[data-testid="chat-composer-input"]');
  await expect(input).toBeEnabled({ timeout: 5_000 });
  await input.fill(text);
  const messages = page.locator('[data-testid="chat-message"]');
  const initialAssistant = await page.locator('[data-testid="chat-message"].assistant').count();
  await page.locator('[data-testid="chat-composer-submit"]').click();
  await expect
    .poll(async () => page.locator('[data-testid="chat-message"].assistant').count(), {
      timeout: 20_000,
    })
    .toBeGreaterThan(initialAssistant);
  await expect(messages.last()).toHaveClass(/assistant/);
  return page.locator('[data-testid="chat-message"].assistant').last();
}

test.describe('Issue #493 - OCR market-price fact checking', () => {
  test.beforeEach(async ({ page }) => {
    await page.addInitScript((ocrText) => {
      window.localStorage.setItem(
        'formal-ai.preferences.v1',
        [
          'demo_preferences',
          '  demoMode "off"',
          '  diagnosticsMode "on"',
          '  experimentalOcr "on"',
          '  greetingVariations "off"',
        ].join('\n'),
      );
      window.FormalAiOcr = {
        VERSION: 'test-double',
        recognizeImage: async () => ({
          text: ocrText,
          confidence: 90,
        }),
      };
    }, ISSUE_493_OCR_TEXT);
    await page.goto('./');
    await expect(page.locator('.app')).toBeVisible({ timeout: 15_000 });
    await expect(page.locator('[data-testid="demo-status"]')).toHaveText('Manual mode');
  });

  test('uses OCR text to flag the false ETH 2024 price claim', async ({ page }) => {
    await expect(page.locator('[data-testid="setting-experimental-ocr"]')).toBeChecked();
    await page.locator('[data-testid="composer-attachment-input"]').setInputFiles({
      name: 'issue-493-eth-claim.png',
      mimeType: 'image/png',
      buffer: ONE_PIXEL_PNG,
    });
    await expect(page.locator('[data-testid="composer-attachment-status"]'))
      .toContainText(/1 attached/i);

    const last = await sendPrompt(
      page,
      'Verify factual accuracy of this attached image',
    );

    await expect(last.locator('.intent')).toContainText('intent:document_originality_check');
    await expect(last).toContainText('Price claim check');
    await expect(last).toContainText('ETH in 2024: $1,700 is contradicted');

    const evidence = last.locator('.evidence-list');
    await expect(evidence).toContainText('market_price_claim:claim_count:6');
    await expect(evidence).toContainText(
      'market_price_claim:assessment:asset=ETH period=2024 claimed=1700.00 status=contradicted source=binance_ethusdt_1d_2024 min=2100.00 min_date=2024-01-03 max=4107.80 max_date=2024-12-16 posterior=0.030000',
    );
  });
});
