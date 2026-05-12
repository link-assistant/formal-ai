// @ts-check
const { test, expect } = require('@playwright/test');

async function switchToManualMode(page) {
  const demoToggle = page.locator('.mode-toggle');
  await expect(demoToggle).toContainText('Demo on');
  await demoToggle.click();
  await expect(demoToggle).toContainText('Demo');
}

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

  test('demo mode starts automatically and exposes a live countdown', async ({ page }) => {
    const demoToggle = page.locator('.mode-toggle');
    await expect(demoToggle).toBeVisible();
    await expect(demoToggle).toContainText('Demo on');

    const input = page.locator('[data-testid="chat-composer-input"]');
    await expect(input).toBeDisabled();

    const demoStatus = page.locator('[data-testid="demo-status"]');
    await expect(demoStatus).toBeVisible();
    await expect(demoStatus).toContainText(/Demo playing|Next dialog in/);
    await expect(demoStatus).toContainText(/Next dialog in \d+s/, { timeout: 15_000 });

    const firstCountdown = await demoStatus.textContent();
    await page.waitForTimeout(1200);
    await expect(demoStatus).not.toHaveText(firstCountdown || '');
  });

  test('automatic demo renders a chat exchange', async ({ page }) => {
    const messageList = page.locator('[data-testid="message-list"]');
    await expect(messageList).toBeVisible();

    const messages = page.locator('[data-testid="chat-message"]');
    await expect(messages.first()).toBeVisible({ timeout: 15_000 });

    await expect(page.locator('[data-testid="chat-message"].user').first()).toBeVisible();
    await expect(page.locator('[data-testid="chat-message"].assistant').first()).toBeVisible({
      timeout: 15_000,
    });
  });

  test('quick prompts sidebar is visible with expected prompts', async ({ page }) => {
    const promptList = page.locator('.prompt-list');
    await expect(promptList).toBeVisible();

    const buttons = promptList.locator('button');
    await expect(buttons.first()).toContainText('Hi');
    await expect(buttons.nth(1)).toContainText('Rust');
  });

  test('chat input and send button are present', async ({ page }) => {
    await switchToManualMode(page);

    const input = page.locator('[data-testid="chat-composer-input"]');
    await expect(input).toBeVisible();
    await expect(input).toBeEnabled();

    const sendBtn = page.locator('[data-testid="chat-composer-submit"]');
    await expect(sendBtn).toBeVisible();
  });

  test('send button is disabled when input is empty', async ({ page }) => {
    await switchToManualMode(page);

    const sendBtn = page.locator('[data-testid="chat-composer-submit"]');
    await expect(sendBtn).toBeDisabled();
  });

  test('clicking a quick prompt populates the input', async ({ page }) => {
    const hiButton = page.locator('.prompt-list button').first();
    await hiButton.click();

    const input = page.locator('[data-testid="chat-composer-input"]');
    await expect(input).toBeEnabled({ timeout: 5_000 });
    await expect(input).toHaveValue('Hi');
  });

  test('sending a greeting produces an assistant reply', async ({ page }) => {
    await switchToManualMode(page);

    const input = page.locator('[data-testid="chat-composer-input"]');
    await input.fill('Hello');

    const messages = page.locator('[data-testid="chat-message"]');
    const initialCount = await messages.count();

    const sendBtn = page.locator('[data-testid="chat-composer-submit"]');
    await sendBtn.click();

    // Wait for the assistant response to appear (worker or fallback)
    await expect(messages).toHaveCount(initialCount + 2, { timeout: 15_000 });

    const lastMsg = messages.last();
    await expect(lastMsg).toHaveClass(/assistant/);
    await expect(lastMsg).toContainText('Hi');
  });

  test('sending a hello world request produces a code block', async ({ page }) => {
    await switchToManualMode(page);

    const input = page.locator('[data-testid="chat-composer-input"]');
    await input.fill('Write me hello world program in Rust');

    const messages = page.locator('[data-testid="chat-message"]');
    const initialCount = await messages.count();

    const sendBtn = page.locator('[data-testid="chat-composer-submit"]');
    await sendBtn.click();

    // Wait for assistant message with Rust code
    await expect(messages).toHaveCount(initialCount + 2, { timeout: 15_000 });

    const lastMsg = messages.last();
    await expect(lastMsg).toHaveClass(/assistant/);
    await expect(lastMsg).toContainText('Rust');
    await expect(lastMsg).toContainText('Execution status: compiled and ran');
  });

  test('pressing Enter submits the message', async ({ page }) => {
    await switchToManualMode(page);

    const input = page.locator('[data-testid="chat-composer-input"]');
    await input.fill('Hi');

    const messages = page.locator('[data-testid="chat-message"]');
    const initialCount = await messages.count();

    await input.press('Enter');

    await expect(messages).toHaveCount(initialCount + 2, { timeout: 15_000 });
  });

  test('demo mode toggle button is present', async ({ page }) => {
    const demoToggle = page.locator('.mode-toggle');
    await expect(demoToggle).toBeVisible();
    await expect(demoToggle).toContainText('Demo on');

    await demoToggle.click();
    await expect(demoToggle).toContainText('Demo');
  });

  test('toggling demo mode disables the input', async ({ page }) => {
    const demoToggle = page.locator('.mode-toggle');

    const input = page.locator('[data-testid="chat-composer-input"]');
    await expect(input).toBeDisabled({ timeout: 5_000 });

    await demoToggle.click();
    await expect(input).toBeEnabled({ timeout: 5_000 });
  });

  test('diagnostics are hidden by default', async ({ page }) => {
    await expect(page.locator('.trace-list')).toHaveCount(0);
    await expect(page.locator('.intent')).toHaveCount(0);
    await expect(page.locator('.evidence-list')).toHaveCount(0);
    await expect(page.locator('.thinking-steps')).toHaveCount(0);
  });

  test('diagnostics toggle shows trace, intent, evidence, and thinking steps', async ({ page }) => {
    await expect(page.locator('[data-testid="chat-message"].assistant').first()).toBeVisible({
      timeout: 15_000,
    });

    const diagnosticsToggle = page.locator('.diagnostics-toggle');
    await expect(diagnosticsToggle).toBeVisible();
    await expect(diagnosticsToggle).toContainText('Diagnostics');
    await diagnosticsToggle.click();

    const traceList = page.locator('.trace-list');
    await expect(traceList).toBeVisible();
    await expect(traceList).toContainText('formal-symbolic-poc');
    await expect(traceList).toContainText('Intent');

    const assistantMessage = page.locator('[data-testid="chat-message"].assistant').first();
    await expect(assistantMessage.locator('.intent')).toContainText(/intent:/);
    await expect(assistantMessage.locator('.evidence-list')).toContainText(/source:/);
    await expect(assistantMessage.locator('.thinking-steps')).toContainText('Select symbolic intent');
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
