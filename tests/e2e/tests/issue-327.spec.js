// @ts-check
const { test, expect } = require('@playwright/test');
const parityCases = require('../../../data/parity/cross-runtime-synthesis.json');

async function sendPrompt(page, text) {
  const input = page.locator('[data-testid="chat-composer-input"]');
  await expect(input).toBeEnabled({ timeout: 5_000 });
  await input.fill(text);

  const messages = page.locator('[data-testid="chat-message"]');
  const initialCount = await messages.count();
  await page.locator('[data-testid="chat-composer-submit"]').click();
  await expect(messages).toHaveCount(initialCount + 2, { timeout: 20_000 });

  const assistantMessage = messages.last();
  await expect(assistantMessage).toHaveClass(/assistant/);
  await expect(assistantMessage.locator('.markdown-body')).toBeVisible();
  return assistantMessage;
}

test.describe('Issue #327 cross-runtime synthesis parity', () => {
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
    await expect(page.locator('.status')).toContainText('wasm worker');
  });

  for (const item of parityCases) {
    test(`${item.id} matches the Rust parity fixture`, async ({ page }) => {
      const message = await sendPrompt(page, item.prompt);
      const body = message.locator('.markdown-body');
      const evidence = message.locator('.evidence-list');

      await expect(message).toContainText(`intent:${item.expectedIntent}`);
      for (const expected of item.expectedAnswerFragments) {
        if (expected.startsWith('```')) continue;
        await expect(body).toContainText(expected);
      }
      for (const forbidden of item.forbiddenAnswerFragments) {
        await expect(body).not.toContainText(forbidden);
        if (/[A-Za-z_]/.test(forbidden)) {
          await expect(evidence).not.toContainText(forbidden);
        }
      }
      for (const prefix of item.expectedEvidencePrefixes) {
        await expect(evidence, `${item.id} evidence should include ${prefix}`).toContainText(
          prefix,
        );
      }
    });
  }
});
