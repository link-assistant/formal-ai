// @ts-check
const { test, expect } = require('@playwright/test');

const FIRST_PROMPT =
  'Напиши мне программу на Rust, которая выдаёт список файлов в текущей директории';
const PATH_ARG_PROMPT = 'Сделай так, чтобы программа принимала путь как аргумент';
const REVERSE_SORT_PROMPT = 'Сделай сортировку результатов в обратном порядке';

async function disableGreetingVariations(page) {
  await page.addInitScript(() => {
    try {
      window.localStorage.setItem(
        'formal-ai.preferences.v1',
        'demo_preferences\n  greetingVariations "off"',
      );
    } catch (_error) {
      // localStorage may be unavailable in some sandboxes.
    }
  });
}

async function switchToManualMode(page) {
  const demoToggle = page.locator('.mode-toggle');
  await expect(demoToggle).toContainText(/Demo on|Demo off|Демо/, {
    timeout: 10_000,
  });
  await demoToggle.click();
  await expect(page.locator('[data-testid="chat-composer-input"]')).toBeEnabled({
    timeout: 5_000,
  });
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

test.describe('Issue #363 - reasoning-first report pressure', () => {
  test.beforeEach(async ({ page }) => {
    await disableGreetingVariations(page);
    await page.goto('./');
    await expect(page.locator('.app')).toBeVisible({ timeout: 15_000 });
    await switchToManualMode(page);
  });

  test('resolved reverse-sort follow-up does not offer a response-level report action', async ({
    page,
  }) => {
    await sendPrompt(page, FIRST_PROMPT);
    await sendPrompt(page, PATH_ARG_PROMPT);
    const last = await sendPrompt(page, REVERSE_SORT_PROMPT);

    await expect(last.locator('.markdown-body')).toContainText(
      'names.sort_by(|a, b| b.cmp(a))',
    );
    await expect(last.locator('.message-actions a')).toHaveCount(0);
  });

  test('missing-rule report carries the reasoning trace for triage', async ({ page }) => {
    const prompt = 'Quxblort fnordwarble plimsy gabble what?';
    const last = await sendPrompt(page, prompt);

    const reportLink = last.locator('.message-actions a');
    await expect(reportLink).toHaveText('Report missing rule');

    const href = await reportLink.getAttribute('href');
    expect(href).toBeTruthy();

    const body = new URL(href || '').searchParams.get('body') || '';
    expect(body).toContain('## Reasoning Trace');
    expect(body).toContain('diagnostics_steps:');
    expect(body).toContain('fallback: unknown');
    expect(body).toContain('evidence:');
    expect(href.length).toBeLessThanOrEqual(8192);
  });
});
