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

  test('known assistant dialogs include a prefilled issue report link', async ({ page }) => {
    await switchToManualMode(page);

    const input = page.locator('[data-testid="chat-composer-input"]');
    await input.fill('Hello');

    const messages = page.locator('[data-testid="chat-message"]');
    const initialCount = await messages.count();

    await page.locator('[data-testid="chat-composer-submit"]').click();
    await expect(messages).toHaveCount(initialCount + 2, { timeout: 15_000 });

    const lastMsg = messages.last();
    await expect(lastMsg).toHaveClass(/assistant/);

    const reportLink = lastMsg.locator('.message-actions a');
    await expect(reportLink).toHaveText('Report issue');

    const href = await reportLink.getAttribute('href');
    expect(href).toBeTruthy();

    const url = new URL(href || '');
    const body = url.searchParams.get('body') || '';

    expect(`${url.origin}${url.pathname}`).toBe(
      'https://github.com/link-assistant/formal-ai/issues/new',
    );
    expect(url.searchParams.get('labels')).toBe('bug');
    expect(body).toContain('## Environment');
    expect(body).toContain('## Dialog');
    expect(body).toContain('Hello');
    expect(body).toContain('Hi, how may I help you?');
  });

  test('asking who are you produces an identity response', async ({ page }) => {
    await switchToManualMode(page);

    const input = page.locator('[data-testid="chat-composer-input"]');
    await input.fill('Who are you?');

    const messages = page.locator('[data-testid="chat-message"]');
    const initialCount = await messages.count();

    await page.locator('[data-testid="chat-composer-submit"]').click();
    await expect(messages).toHaveCount(initialCount + 2, { timeout: 15_000 });

    const lastMsg = messages.last();
    await expect(lastMsg).toHaveClass(/assistant/);
    await expect(lastMsg).toContainText('formal-ai');
    await expect(lastMsg).not.toContainText('learned symbolic rule for that prompt yet');
  });

  test('unknown prompts include a prefilled missing-rule issue link', async ({ page }) => {
    await switchToManualMode(page);

    const prompt = 'What is the capital of France?';
    const input = page.locator('[data-testid="chat-composer-input"]');
    await input.fill(prompt);

    const messages = page.locator('[data-testid="chat-message"]');
    const initialCount = await messages.count();

    await page.locator('[data-testid="chat-composer-submit"]').click();
    await expect(messages).toHaveCount(initialCount + 2, { timeout: 15_000 });

    const lastMsg = messages.last();
    await expect(lastMsg).toHaveClass(/assistant/);
    await expect(lastMsg).toContainText('learned symbolic rule for that prompt yet');

    const reportLink = lastMsg.locator('.message-actions a');
    await expect(reportLink).toHaveText('Report missing rule');

    const href = await reportLink.getAttribute('href');
    expect(href).toBeTruthy();

    const url = new URL(href || '');
    const body = url.searchParams.get('body') || '';

    expect(url.searchParams.get('title')).toContain('Unknown prompt');
    expect(url.searchParams.get('labels')).toBe('bug');
    expect(body).toContain('## Environment');
    expect(body).toContain('**Version**');
    expect(body).toContain('**User Agent**');
    expect(body).toContain('## Dialog');
    expect(body).toContain(prompt);
    expect(body).toContain('intent: unknown');
    expect(body).toContain('## Reproduction Steps');
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

  test('composer does not expose an unused preview control', async ({ page }) => {
    await switchToManualMode(page);

    await expect(page.locator('.preview-toggle')).toHaveCount(0);
    await expect(page.locator('.composer-preview')).toHaveCount(0);
  });
});
