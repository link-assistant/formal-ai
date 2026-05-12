// @ts-check
const { test, expect } = require('@playwright/test');

test.describe('formal-ai demo UI', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
    // Wait for React to mount and the app shell to be visible
    await expect(page.locator('.app')).toBeVisible({ timeout: 15_000 });
  });

  test('page title is formal-ai', async ({ page }) => {
    await expect(page).toHaveTitle('formal-ai');
  });

  test('brand header is visible', async ({ page }) => {
    await expect(page.locator('.brand')).toBeVisible();
    await expect(page.locator('.brand strong')).toContainText('formal-ai');
  });

  test('initial messages show greeting exchange', async ({ page }) => {
    const messageList = page.locator('[data-testid="message-list"]');
    await expect(messageList).toBeVisible();

    const messages = page.locator('[data-testid="chat-message"]');
    await expect(messages).toHaveCount(2);

    const userMsg = messages.first();
    await expect(userMsg).toHaveClass(/user/);
    await expect(userMsg).toContainText('Hi');

    const assistantMsg = messages.nth(1);
    await expect(assistantMsg).toHaveClass(/assistant/);
    await expect(assistantMsg).toContainText('Hi');
  });

  test('quick prompts sidebar is visible with expected prompts', async ({ page }) => {
    const promptList = page.locator('.prompt-list');
    await expect(promptList).toBeVisible();

    const buttons = promptList.locator('button');
    await expect(buttons.first()).toContainText('Hi');
    await expect(buttons.nth(1)).toContainText('Rust');
  });

  test('chat input and send button are present', async ({ page }) => {
    const input = page.locator('[data-testid="chat-composer-input"]');
    await expect(input).toBeVisible();
    await expect(input).toBeEnabled();

    const sendBtn = page.locator('[data-testid="chat-composer-submit"]');
    await expect(sendBtn).toBeVisible();
  });

  test('send button is disabled when input is empty', async ({ page }) => {
    const sendBtn = page.locator('[data-testid="chat-composer-submit"]');
    await expect(sendBtn).toBeDisabled();
  });

  test('clicking a quick prompt populates the input', async ({ page }) => {
    const hiButton = page.locator('.prompt-list button').first();
    await hiButton.click();

    const input = page.locator('[data-testid="chat-composer-input"]');
    await expect(input).toHaveValue('Hi');
  });

  test('sending a greeting produces an assistant reply', async ({ page }) => {
    const input = page.locator('[data-testid="chat-composer-input"]');
    await input.fill('Hello');

    const sendBtn = page.locator('[data-testid="chat-composer-submit"]');
    await sendBtn.click();

    // Wait for the assistant response to appear (worker or fallback)
    const messages = page.locator('[data-testid="chat-message"]');
    await expect(messages).toHaveCount(4, { timeout: 15_000 });

    const lastMsg = messages.last();
    await expect(lastMsg).toHaveClass(/assistant/);
    await expect(lastMsg).toContainText('Hi');
  });

  test('sending a hello world request produces a code block', async ({ page }) => {
    const input = page.locator('[data-testid="chat-composer-input"]');
    await input.fill('Write me hello world program in Rust');

    const sendBtn = page.locator('[data-testid="chat-composer-submit"]');
    await sendBtn.click();

    // Wait for assistant message with Rust code
    const messages = page.locator('[data-testid="chat-message"]');
    await expect(messages).toHaveCount(4, { timeout: 15_000 });

    const lastMsg = messages.last();
    await expect(lastMsg).toHaveClass(/assistant/);
    await expect(lastMsg).toContainText('rust');
  });

  test('pressing Enter submits the message', async ({ page }) => {
    const input = page.locator('[data-testid="chat-composer-input"]');
    await input.fill('Hi');
    await input.press('Enter');

    const messages = page.locator('[data-testid="chat-message"]');
    await expect(messages).toHaveCount(4, { timeout: 15_000 });
  });

  test('demo mode toggle button is present', async ({ page }) => {
    const demoToggle = page.locator('.mode-toggle');
    await expect(demoToggle).toBeVisible();
    await expect(demoToggle).toContainText('Demo');
  });

  test('toggling demo mode disables the input', async ({ page }) => {
    const demoToggle = page.locator('.mode-toggle');
    await demoToggle.click();

    const input = page.locator('[data-testid="chat-composer-input"]');
    await expect(input).toBeDisabled({ timeout: 5_000 });

    // Toggle back off
    await demoToggle.click();
    await expect(input).toBeEnabled({ timeout: 5_000 });
  });

  test('trace panel shows model and intent metadata', async ({ page }) => {
    const traceList = page.locator('.trace-list');
    await expect(traceList).toBeVisible();
    await expect(traceList).toContainText('formal-symbolic-poc');
    await expect(traceList).toContainText('Intent');
  });

  test('preview toggle switches between write and preview mode', async ({ page }) => {
    const previewToggle = page.locator('.preview-toggle');
    await expect(previewToggle).toBeVisible();
    await expect(previewToggle).toContainText('Preview');

    await previewToggle.click();
    await expect(previewToggle).toContainText('Write');

    await previewToggle.click();
    await expect(previewToggle).toContainText('Preview');
  });
});
