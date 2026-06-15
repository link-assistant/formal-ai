// @ts-check
const { test, expect } = require('@playwright/test');

async function bootManualChat(page) {
  await page.addInitScript(() => {
    try {
      window.localStorage.setItem(
        'formal-ai.preferences.v1',
        [
          'demo_preferences',
          '  theme "light"',
          '  demoMode "off"',
          '  diagnosticsMode "off"',
          '  greetingVariations "off"',
        ].join('\n'),
      );
    } catch (_error) {
      // localStorage may be unavailable in hardened browser contexts.
    }
  });
  await page.goto('./');
  await expect(page.locator('.app')).toBeVisible({ timeout: 15_000 });
  await expect(page.locator('[data-testid="demo-status"]')).toHaveText(
    'Manual mode',
  );
}

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
  return assistantMessage;
}

test.describe('Issue #488 - visible thinking preview', () => {
  test('shows collapsed human-readable thinking by default and expands details', async ({
    page,
  }) => {
    await bootManualChat(page);

    const assistantMessage = await sendPrompt(page, 'Hi');

    await expect(assistantMessage.locator('.thinking-steps')).toHaveCount(0);
    const preview = assistantMessage.locator('[data-testid="thinking-preview"]');
    await expect(preview).toBeVisible();

    const toggle = preview.locator('[data-testid="thinking-preview-toggle"]');
    await expect(toggle).toHaveAttribute('aria-expanded', 'false');
    await expect(
      preview.locator('[data-testid="thinking-preview-previous"]'),
    ).toBeVisible();
    await expect(
      preview.locator('[data-testid="thinking-preview-current"]'),
    ).toContainText('Applied available context:');
    await expect(preview).not.toContainText(
      /match_rule|dispatch_handler|deformalize|formalize/i,
    );

    await toggle.click();
    await expect(toggle).toHaveAttribute('aria-expanded', 'true');

    const expandedList = preview.locator(
      '[data-testid="thinking-expanded-list"]',
    );
    await expect(expandedList).toBeVisible();
    expect(await expandedList.locator('li').count()).toBeGreaterThanOrEqual(6);
    await expect(expandedList).toContainText('Received the user request.');
    await expect(expandedList).toContainText('Matched the greeting rule.');
    await expect(expandedList).toContainText(
      'Prepared the answer in readable text.',
    );
    await expect(expandedList).not.toContainText(
      /match_rule|dispatch_handler|deformalize|formalize/i,
    );

    await page
      .locator('[data-testid="setting-thinking-detail"]')
      .selectOption('brief');
    await expect(expandedList.locator('li')).toHaveCount(1);
    await expect(expandedList).toContainText('Applied available context:');
    await expect(
      page.locator('[data-testid="settings-reset-thinkingDetailLevel"]'),
    ).toBeVisible();
  });
});
