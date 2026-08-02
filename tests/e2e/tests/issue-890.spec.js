// @ts-check
//
// Issue #890: the browser worker mirrors native proof-to-program translation.
const { test, expect } = require('@playwright/test');

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

async function sendPrompt(page, text) {
  const input = page.locator('[data-testid="chat-composer-input"]');
  await input.fill(text);
  const messages = page.locator('[data-testid="chat-message"]');
  const initialCount = await messages.count();
  await page.locator('[data-testid="chat-composer-submit"]').click();
  await expect(messages).toHaveCount(initialCount + 2, { timeout: 20_000 });
  return messages.last().locator('.markdown-body');
}

test.describe('Issue #890 formal proof program translation', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('./');
    await expect(page.locator('.app')).toBeVisible({ timeout: 15_000 });
    await switchToManualMode(page);
  });

  test('all registered natural-language commands produce the same Rust proof', async ({ page }) => {
    const statement = 'x > 1 and x < 3 is satisfiable';
    const prompts = [
      `Translate \`${statement}\` to Rust`,
      `Переведи \`${statement}\` на Раст`,
      `\`${statement}\` का रस्ट में अनुवाद करो`,
      `把\`${statement}\`翻译成Rust`,
    ];
    for (const prompt of prompts) {
      const reply = await sendPrompt(page, prompt);
      await expect(reply).toContainText('fn main()');
      await expect(reply).toContainText('proof obligation failed');
      await expect(reply).toContainText('x: i64 = 2');
    }
  });

  test('the same proof can be projected into Python', async ({ page }) => {
    const reply = await sendPrompt(
      page,
      'Translate `x > 1 and x < 3 is satisfiable` to Python',
    );
    await expect(reply).toContainText('x = 2');
    await expect(reply).toContainText('assert x > 1 and x < 3');
    await expect(reply).toContainText('print(x)');
  });
});
