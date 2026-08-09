// @ts-check
// Issue #917: every registered seed language round-trips through the
// seed-defined FOL projection in the browser's Rust-to-WASM path.
const { test, expect } = require('@playwright/test');

async function switchToManualMode(page) {
  const demoToggle = page.locator('.mode-toggle');
  await expect(demoToggle).toContainText(/Demo on|Demo off|Демо/, {
    timeout: 10_000,
  });
  await demoToggle.click();
  await expect(page.locator('[data-testid="demo-status"]')).toHaveText('Manual mode');
  await expect(page.locator('[data-testid="chat-composer-input"]')).toBeEnabled();
}

async function enableDiagnostics(page) {
  const diagnostics = page.locator('.diagnostics-toggle');
  await expect(diagnostics).toBeVisible();
  await diagnostics.click();
  await expect(diagnostics).toHaveAttribute('aria-pressed', 'true');
}

async function sendPrompt(page, text) {
  const input = page.locator('[data-testid="chat-composer-input"]');
  await input.fill(text);
  const messages = page.locator('[data-testid="chat-message"]');
  const before = await messages.count();
  await page.locator('[data-testid="chat-composer-submit"]').click();
  await expect(messages).toHaveCount(before + 2, { timeout: 20_000 });
  return messages.last();
}

const cases = [
  { language: 'en', name: 'English', statement: 'apple is a fruit' },
  { language: 'ru', name: 'Russian', statement: 'яблоко это фрукт' },
  { language: 'hi', name: 'Hindi', statement: 'सेब फल है' },
  { language: 'zh', name: 'Chinese', statement: '苹果是水果' },
  { language: 'es', name: 'Spanish', statement: 'manzana es una fruta' },
];

test.describe('Issue #917 formal language projections', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('./');
    await expect(page.locator('.app')).toBeVisible({ timeout: 15_000 });
    await switchToManualMode(page);
    await enableDiagnostics(page);
  });

  test('every supported natural statement projects to the same FOL meaning', async ({ page }) => {
    for (const item of cases) {
      const reply = await sendPrompt(
        page,
        `Translate \`${item.statement}\` from ${item.name} to FOL`,
      );
      await expect(reply.locator('.markdown-body')).toHaveText('P31(Q89, Q3314483)');
      await expect(reply.locator('.evidence-list')).toContainText(
        'meaning:statement:P31(Q89,Q3314483)',
      );
    }
  });

  test('the FOL meaning projects back into every supported natural language', async ({ page }) => {
    for (const item of cases) {
      const reply = await sendPrompt(
        page,
        `Translate \`P31(Q89, Q3314483)\` from FOL to ${item.name}`,
      );
      await expect(reply.locator('.markdown-body')).toHaveText(item.statement);
      await expect(reply.locator('.evidence-list')).toContainText(
        `language_to:${item.language}`,
      );
    }
  });
});
