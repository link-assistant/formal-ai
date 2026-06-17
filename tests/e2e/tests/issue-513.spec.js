// @ts-check
const { test, expect } = require('@playwright/test');

// Issue #513 (visible fix for #511): a terminal-command request used to fall
// through to the `unknown` fallback. It must now resolve to an
// `agent_suggestion` response that names the detected command and explains
// Agent mode, in both Russian and English. The toolbar also exposes a three-way
// Chat / Agent / Full-Auto radio whose one-click switch is reflected in the
// status label.

async function switchToManualMode(page) {
  const demoToggle = page.locator('.mode-toggle');
  await expect(demoToggle).toContainText(/Demo on|Demo off|Демо/, { timeout: 10_000 });
  await demoToggle.click();
  await expect(page.locator('[data-testid="demo-status"]')).toHaveText('Manual mode');
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

test.describe('Issue #513 - terminal-command intent + mode radio', () => {
  test.beforeEach(async ({ page }) => {
    await page.addInitScript(() => {
      try {
        window.localStorage.setItem(
          'formal-ai.preferences.v1',
          'demo_preferences\n  greetingVariations "off"',
        );
      } catch (_error) {
        // localStorage may be unavailable; the test tolerates greeting variants.
      }
    });
    await page.goto('./');
    await expect(page.locator('.app')).toBeVisible({ timeout: 15_000 });
    await switchToManualMode(page);
  });

  test('English "run `ls ~` in terminal" suggests Agent mode instead of unknown', async ({
    page,
  }) => {
    const last = await sendPrompt(page, 'run `ls ~` in terminal');
    await expect(last).toHaveClass(/assistant/);
    await expect(last).toContainText('ls ~');
    await expect(last).toContainText('Agent mode');
    await expect(last).not.toContainText('That one is new to me');
  });

  test('Russian "Выполни `ls ~` в терминале" suggests Agent mode instead of unknown', async ({
    page,
  }) => {
    const last = await sendPrompt(page, 'Выполни `ls ~` в терминале');
    await expect(last).toHaveClass(/assistant/);
    await expect(last).toContainText('ls ~');
    await expect(last).toContainText(/режим агента|Agent/i);
    await expect(last).not.toContainText('That one is new to me');
  });

  test('mode radio switches Chat -> Agent and the status label follows', async ({ page }) => {
    await expect(page.locator('[data-testid="mode-radio"]')).toBeVisible();
    await expect(page.locator('[data-testid="mode-option-chat"]')).toHaveAttribute(
      'aria-checked',
      'true',
    );
    await expect(page.locator('[data-testid="mode-status"]')).toContainText('Chat');

    await page.locator('[data-testid="mode-option-agent"]').click();
    await expect(page.locator('[data-testid="mode-option-agent"]')).toHaveAttribute(
      'aria-checked',
      'true',
    );
    await expect(page.locator('[data-testid="mode-status"]')).toContainText('Agent');

    await page.locator('[data-testid="mode-option-fullAuto"]').click();
    await expect(page.locator('[data-testid="mode-option-fullAuto"]')).toHaveAttribute(
      'aria-checked',
      'true',
    );
  });
});
