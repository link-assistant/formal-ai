// @ts-check
// Issue #157: A Russian question about the creator of Formal AI resolves through the worker's seed facts.
const { test, expect } = require('@playwright/test');
const { WORKER_READY_TIMEOUT_MS } = require('./support/worker-ready');

const UNKNOWN_ANSWER_MARKER = 'cannot answer that from local links rules';

async function switchToManualMode(page) {
  const demoToggle = page.locator('.mode-toggle');
  await expect(demoToggle).toContainText(/Demo on|Demo off|Демо/, {
    timeout: 10_000,
  });
  await demoToggle.click();
  await expect(page.locator('[data-testid="chat-composer-input"]')).toBeEnabled({ timeout: WORKER_READY_TIMEOUT_MS });
}

async function sendPrompt(page, text) {
  const input = page.locator('[data-testid="chat-composer-input"]');
  await expect(input).toBeEnabled({ timeout: WORKER_READY_TIMEOUT_MS });
  await input.fill(text);
  const messages = page.locator('[data-testid="chat-message"]');
  const initial = await messages.count();
  await page.locator('[data-testid="chat-composer-submit"]').click();
  await expect(messages).toHaveCount(initial + 2, { timeout: 20_000 });
  return messages.last();
}

test.describe('Issue #157 — Formal AI creator prompt', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('./');
    await expect(page.locator('.app')).toBeVisible({ timeout: 15_000 });
    await switchToManualMode(page);
  });

  test('Russian creator question resolves through the worker seed facts', async ({ page }) => {
    const lastMessage = await sendPrompt(page, 'кто тебя создал?');

    await expect(lastMessage).toHaveClass(/assistant/);
    await expect(lastMessage).toContainText('github.com/konard');
    await expect(lastMessage).toContainText('github.com/link-assistant/hive-mind');
    await expect(lastMessage).not.toContainText(UNKNOWN_ANSWER_MARKER);
  });
});
