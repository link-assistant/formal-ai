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

// A single line naming BTC (cross-asset), the Russian ETH alias with a year
// whose $1,700 is *inside* the recorded range (within-range, not contradicted),
// and the Chinese ETH alias (non-ASCII) with the false 2024 price. This proves
// the browser worker generalises the fact check across assets, languages, and
// the within-range vs contradicted distinction — not just the ETH example.
const GENERALIZATION_OCR_TEXT = [
  'BTC in 2024: $1,700',
  'эфириум в 2023: $1,700',
  '以太坊 2024: $1,700',
].join('\n');

async function installOcr(page, ocrText) {
  await page.addInitScript((text) => {
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
        text,
        confidence: 90,
      }),
    };
  }, ocrText);
  await page.goto('./');
  await expect(page.locator('.app')).toBeVisible({ timeout: 15_000 });
  await expect(page.locator('[data-testid="demo-status"]')).toHaveText('Manual mode');
}

async function attachAndVerify(page, name) {
  await expect(page.locator('[data-testid="setting-experimental-ocr"]')).toBeChecked();
  await page.locator('[data-testid="composer-attachment-input"]').setInputFiles({
    name,
    mimeType: 'image/png',
    buffer: ONE_PIXEL_PNG,
  });
  await expect(page.locator('[data-testid="composer-attachment-status"]'))
    .toContainText(/1 attached/i);
  return sendPrompt(page, 'Verify factual accuracy of this attached image');
}

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
  test('uses OCR text to flag the false ETH 2024 price claim', async ({ page }) => {
    await installOcr(page, ISSUE_493_OCR_TEXT);
    const last = await attachAndVerify(page, 'issue-493-eth-claim.png');

    await expect(last.locator('.intent')).toContainText('intent:document_originality_check');
    await expect(last).toContainText('Price claim check');
    await expect(last).toContainText('ETH in 2024: $1,700 is contradicted');

    const evidence = last.locator('.evidence-list');
    await expect(evidence).toContainText('market_price_claim:claim_count:6');
    await expect(evidence).toContainText(
      'market_price_claim:assessment:asset=ETH period=2024 claimed=1700.00 status=contradicted source=binance_ethusdt_1d_2024 min=2100.00 min_date=2024-01-03 max=4107.80 max_date=2024-12-16 posterior=0.030000',
    );
  });

  test('generalises across assets, languages, and within-range vs contradicted', async ({
    page,
  }) => {
    await installOcr(page, GENERALIZATION_OCR_TEXT);
    const last = await attachAndVerify(page, 'issue-493-multi-asset-claim.png');

    await expect(last.locator('.intent')).toContainText('intent:document_originality_check');
    await expect(last).toContainText('Price claim check');

    // Cross-asset: the same machinery flags an impossible BTC price with the
    // BTCUSDT reference, no ETH-specific path.
    await expect(last).toContainText('BTC in 2024: $1,700 is contradicted');
    // Non-ASCII Chinese alias resolves to ETH and the false 2024 price is caught.
    await expect(last).toContainText('以太坊 2024: $1,700 is contradicted');
    // Within-range claims are not over-claimed: only contradicted lines appear in
    // the answer body, so the within-range ETH 2023 line must NOT be summarized
    // as contradicted. Its within-range verdict is asserted on the evidence trace
    // below instead.
    await expect(last).not.toContainText('эфириум в 2023: $1,700 is contradicted');

    const evidence = last.locator('.evidence-list');
    await expect(evidence).toContainText('market_price_claim:claim_count:3');
    await expect(evidence).toContainText(
      'market_price_claim:assessment:asset=BTC period=2024 claimed=1700.00 status=contradicted source=binance_btcusdt_1d_2024 min=38555.00 min_date=2024-01-23 max=108353.00 max_date=2024-12-17 posterior=0.030000',
    );
    await expect(evidence).toContainText(
      'market_price_claim:assessment:asset=ETH period=2023 claimed=1700.00 status=within_recorded_range source=binance_ethusdt_1d_2023 min=1190.57 min_date=2023-01-01 max=2445.80 max_date=2023-12-28 posterior=0.980000',
    );
    await expect(evidence).toContainText(
      'market_price_claim:assessment:asset=ETH period=2024 claimed=1700.00 status=contradicted source=binance_ethusdt_1d_2024 min=2100.00 min_date=2024-01-03 max=4107.80 max_date=2024-12-16 posterior=0.030000',
    );
  });
});
