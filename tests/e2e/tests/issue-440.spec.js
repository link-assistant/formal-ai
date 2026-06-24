// @ts-check
//
// Issue #440: list-files examples must not show Rust-specific files for Python,
// the browser "Copy..." instruction must be a separate paragraph, and code
// blocks should use a light code theme when the app theme is light.
const { test, expect } = require('@playwright/test');

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

test.describe('Issue #440 - list-files output and light code blocks', () => {
  test.beforeEach(async ({ page }) => {
    await page.addInitScript(() => {
      window.localStorage.setItem(
        'formal-ai.preferences.v1',
        'demo_preferences\n  theme "light"\n  demoMode "off"\n  diagnosticsMode "off"\n  greetingVariations "off"',
      );
    });
    await page.goto('./');
    await expect(page.locator('.app')).toBeVisible({ timeout: 15_000 });
    await expect(page.locator('html')).toHaveAttribute('data-theme', 'light');
    await expect(page.locator('[data-testid="demo-status"]')).toHaveText('Manual mode');
  });

  test('renders Python list-files output with matching sample files', async ({ page }) => {
    const message = await sendPrompt(
      page,
      'Write me a Python program that lists files in the current directory in reverse-sorted order',
    );

    const body = message.locator('.markdown-body');
    await expect(body).toContainText('main.py');
    await expect(body).toContainText('data.txt');
    await expect(body).toContainText('README.md');
    await expect(body).not.toContainText('Cargo.toml');
    await expect(body).not.toContainText('main.rs');

    const paragraphs = await body.locator('p').allTextContents();
    const status = paragraphs.find((text) => text.startsWith('Execution status:'));
    expect(status).toBeTruthy();
    expect(status).not.toContain('Copy the snippet');
    expect(paragraphs.some((text) => text.startsWith('Copy the snippet'))).toBe(true);

    const codeBlock = body.locator('.code-block').first();
    const colors = await codeBlock.evaluate((node) => {
      const block = window.getComputedStyle(node);
      const code = window.getComputedStyle(node.querySelector('code'));
      return {
        blockBackground: block.backgroundColor,
        codeColor: code.color,
      };
    });
    expect(colors.blockBackground).toBe('rgb(248, 250, 252)');
    expect(colors.codeColor).toBe('rgb(31, 41, 55)');
  });
});
