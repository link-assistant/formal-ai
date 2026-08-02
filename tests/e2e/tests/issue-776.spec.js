// @ts-check
// Issue #776: source-first translation commands and semantic round trips.
const { test, expect } = require('@playwright/test');

async function switchToManualMode(page) {
  const demoToggle = page.locator('.mode-toggle');
  await expect(demoToggle).toContainText(/Demo on|Demo off|Демо/, {
    timeout: 10_000,
  });
  await demoToggle.click();
  await expect(page.locator('[data-testid="demo-status"]')).toHaveText('Manual mode');
}

async function sendPrompt(page, text) {
  const input = page.locator('[data-testid="chat-composer-input"]');
  await expect(input).toBeEnabled({ timeout: 5_000 });
  await input.fill(text);
  const messages = page.locator('[data-testid="chat-message"]');
  const initialCount = await messages.count();
  await page.locator('[data-testid="chat-composer-submit"]').click();
  await expect(messages).toHaveCount(initialCount + 2, { timeout: 20_000 });
  return messages.last().locator('.markdown-body');
}

test.describe('Issue #776 source-first translation', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('./');
    await expect(page.locator('.app')).toBeVisible({ timeout: 15_000 });
    await switchToManualMode(page);
  });

  test('reported prompt translates through the browser worker', async ({ page }) => {
    const reply = await sendPrompt(
      page,
      'любая формальная система либо неполна, либо противоречива - translate to english',
    );
    await expect(reply).toContainText(
      'any formal system is either incomplete or inconsistent',
    );
    await expect(reply).not.toContainText('translation gap');
  });

  test('one proposition is renderable in en, ru, hi, and zh', async ({ page }) => {
    const cases = [
      ['translate "any formal system is either incomplete or inconsistent" to english', 'any formal system is either incomplete or inconsistent'],
      ['translate "any formal system is either incomplete or inconsistent" to russian', 'любая формальная система либо неполна, либо противоречива'],
      ['translate "any formal system is either incomplete or inconsistent" to hindi', 'कोई भी औपचारिक प्रणाली या तो अपूर्ण होती है या असंगत'],
      ['translate "any formal system is either incomplete or inconsistent" to chinese', '任何形式系统要么是不完备的，要么是不一致的'],
    ];
    for (const [prompt, expected] of cases) {
      await expect(await sendPrompt(page, prompt)).toContainText(expected);
    }
  });

  test('composer waits for the browser worker before accepting a prompt', async ({ page }) => {
    let agentInfoRequests = 0;
    page.on('request', (request) => {
      if (/\/seed\/agent-info\.lino(?:\?|$)/.test(request.url())) {
        agentInfoRequests += 1;
      }
    });
    let releaseWasm;
    let markWasmBlocked;
    const wasmRelease = new Promise((resolve) => {
      releaseWasm = resolve;
    });
    const wasmBlocked = new Promise((resolve) => {
      markWasmBlocked = resolve;
    });
    await page.route('**/formal_ai_worker.wasm*', async (route) => {
      markWasmBlocked();
      await wasmRelease;
      await route.continue();
    });

    await page.reload();
    await wasmBlocked;
    const input = page.locator('[data-testid="chat-composer-input"]');
    try {
      await expect(input).toBeDisabled();
      expect(agentInfoRequests).toBe(1);
    } finally {
      releaseWasm();
    }
    await expect(input).toBeEnabled({ timeout: 15_000 });
  });
});
