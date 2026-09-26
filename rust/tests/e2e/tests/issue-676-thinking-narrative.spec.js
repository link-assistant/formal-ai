// @ts-check
// Issue #676 (R8): the thinking display should read as a human narrative, not a
// robotic list of identical category steps. The reporter reached Formal AI
// through an agentic CLI (OpenCode) that renders the reasoning verbatim, and the
// traces for unrelated prompts differed only by a route label buried mid-list.
//
// The web surface now leads every thinking preview with a single first-person
// headline of what the assistant understood and decided (mirroring the Rust
// `thinking_narrative`), while keeping the concrete steps beneath it as the
// recursive "robotic detail" layer. These tests assert that headline is present,
// human, and genuinely per-intent (greeting vs wellbeing vs calculation).
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

function narrativeOf(assistantMessage) {
  return assistantMessage
    .locator('[data-testid="thinking-preview"]')
    .locator('[data-testid="thinking-narrative"]');
}

test.describe('Issue #676 (R8) - human thinking narrative', () => {
  test('leads the trace with a human headline and keeps the robotic detail', async ({
    page,
  }) => {
    await bootManualChat(page);

    const assistantMessage = await sendPrompt(page, 'Hi');
    const narrative = narrativeOf(assistantMessage);
    await expect(narrative).toBeVisible();
    await expect(narrative).toHaveText('You said hello, so I greeted you back.');

    // The concrete steps are still available beneath the headline as the
    // recursive "robotic detail" layer once the trace is expanded.
    const preview = assistantMessage.locator(
      '[data-testid="thinking-preview"]',
    );
    await preview.locator('[data-testid="thinking-preview-toggle"]').click();
    await expect(
      preview.locator('[data-testid="thinking-expanded-list"]'),
    ).toContainText('Read the request:');
  });

  test('gives greeting and wellbeing distinct human headlines', async ({
    page,
  }) => {
    await bootManualChat(page);

    const greeting = await sendPrompt(page, 'Hi');
    await expect(narrativeOf(greeting)).toHaveText(
      'You said hello, so I greeted you back.',
    );

    const wellbeing = await sendPrompt(page, 'How are you?');
    await expect(narrativeOf(wellbeing)).toHaveText(
      "You asked how I'm doing, so I told you and offered to help.",
    );
  });

  test('summarizes a calculation route in plain language', async ({ page }) => {
    await bootManualChat(page);

    const assistantMessage = await sendPrompt(page, '2 + 2');
    await expect(narrativeOf(assistantMessage)).toContainText('calculation');
  });
});
