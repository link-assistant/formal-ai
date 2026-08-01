// @ts-check
const { test, expect } = require('@playwright/test');

async function sendPrompt(page, text) {
  const input = page.locator('[data-testid="chat-composer-input"]');
  await expect(input).toBeEnabled({ timeout: 5_000 });
  await input.fill(text);
  const messages = page.locator('[data-testid="chat-message"]');
  const initialCount = await messages.count();
  await page.locator('[data-testid="chat-composer-submit"]').click();
  await expect(messages).toHaveCount(initialCount + 2, { timeout: 20_000 });
  const assistant = messages.last();
  await expect(assistant).toHaveClass(/assistant/);
  await expect(assistant.locator('.markdown-body')).toBeVisible();
  return assistant;
}

test.describe('Issue #710 conversational requirement recovery', () => {
  test.beforeEach(async ({ page }) => {
    await page.addInitScript(() => {
      window.localStorage.setItem(
        'formal-ai.preferences.v1',
        'demo_preferences\n  demoMode "off"\n  diagnosticsMode "on"\n  greetingVariations "off"\n  temperature "0"',
      );
    });
    await page.goto('./');
    await expect(page.locator('.app')).toBeVisible({ timeout: 15_000 });
    await expect(page.locator('[data-testid="demo-status"]')).toHaveText('Manual mode');
    await expect(page.locator('.status')).toContainText('wasm worker');
  });

  test('answers independent questions in source order', async ({ page }) => {
    const reply = await sendPrompt(page, 'Who are you? What can you do? What is 2 + 2?');
    await expect(reply).toContainText('intent:compound_response');
    await expect(reply.locator('.markdown-body')).toContainText('formal-ai');
    await expect(reply.locator('.markdown-body')).toContainText('2 + 2 = 4');
    const body = (await reply.locator('.markdown-body').textContent()) || '';
    expect(body.trimEnd().endsWith('4')).toBe(true);
    await expect(reply.locator('.evidence-list')).toContainText('sub_impulse:Who are you?');
    await expect(reply.locator('.evidence-list')).toContainText('sub_impulse:What can you do?');
    await expect(reply.locator('.evidence-list')).toContainText('sub_impulse:What is 2 + 2?');
  });

  test('sets and recalls an assistant name in Russian', async ({ page }) => {
    const acknowledgement = await sendPrompt(page, 'Теперь тебя зовут Инеффа.');
    await expect(acknowledgement).toContainText('intent:configure_assistant_name');
    await expect(acknowledgement.locator('.markdown-body')).toContainText('Инеффа');
    const storedPreferences = await page.evaluate(() =>
      window.localStorage.getItem('formal-ai.preferences.v1'),
    );
    expect(storedPreferences).toContain('assistantName "Инеффа"');
    expect(storedPreferences).not.toContain('assistantName "Инеффа."');

    const recall = await sendPrompt(page, 'Как тебя зовут?');
    await expect(recall).toContainText('intent:assistant_name');
    await expect(recall.locator('.markdown-body')).toContainText('Инеффа');
  });

  test('asks exactly one question for a target-less modification', async ({ page }) => {
    const reply = await sendPrompt(page, '修改它。');
    await expect(reply).toContainText('intent:ambiguous_modification_clarification');
    const body = (await reply.locator('.markdown-body').textContent()) || '';
    expect((body.match(/[?？]/gu) || []).length).toBe(1);
  });

  test('free-time variation is prompt-stable rather than random', async ({ page }) => {
    const prompts = [
      'What do you do in your free time?',
      'How do you spend your free time?',
      'What do you do when you are not working?',
    ];
    const answers = [];
    for (const prompt of prompts) {
      const reply = await sendPrompt(page, prompt);
      await expect(reply).toContainText('intent:assistant_free_time');
      answers.push(((await reply.locator('.markdown-body').textContent()) || '').trim());
    }
    const replay = await sendPrompt(page, prompts[0]);
    const replayText = ((await replay.locator('.markdown-body').textContent()) || '').trim();
    expect(replayText).toBe(answers[0]);
    expect(new Set(answers).size).toBeGreaterThanOrEqual(2);
  });
});
