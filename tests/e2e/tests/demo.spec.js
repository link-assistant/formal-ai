// @ts-check
const { test, expect } = require('@playwright/test');

const UNKNOWN_ANSWER_MARKER = 'cannot answer that from local Links Notation rules';

async function switchToManualMode(page) {
  const demoToggle = page.locator('.mode-toggle');
  await expect(demoToggle).toContainText('Demo on');
  await demoToggle.click();
  await expect(demoToggle).toContainText('Demo');
  await expect(page.locator('[data-testid="demo-status"]')).toHaveText('Manual mode');
  await expect(page.locator('[data-testid="chat-composer-input"]')).toBeEnabled({
    timeout: 5_000,
  });
}

// Issue #27: greeting randomisation defaults to ON in production. Tests pin
// the canonical greeting text so they assert deterministic output; new tests
// that actually exercise the randomisation flip the preference back on.
async function disableGreetingVariations(page) {
  await page.addInitScript(() => {
    try {
      window.localStorage.setItem(
        'formal-ai.preferences.v1',
        'demo_preferences\n  greetingVariations "off"',
      );
    } catch (_error) {
      // localStorage may be unavailable; tests will tolerate variant text.
    }
  });
}

test.describe('formal-ai demo UI', () => {
  test.beforeEach(async ({ page }) => {
    await disableGreetingVariations(page);
    await page.goto('./');
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
    // Issue #27: the list now includes multilingual greetings before the Rust
    // hello-world entry, so match by label instead of by absolute index.
    await expect(promptList.locator('button[data-prompt-label*="Rust"]').first()).toBeVisible();
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
    // Issue #78: the dialog is now a single fenced code block with `U:` /
    // `A:` line prefixes instead of one Markdown subsection per message.
    expect(body).toContain('Legend: `U` = user, `A` = agent.');
    expect(body).toContain('U: Hello');
    // Issue #73: the reported assistant message must be annotated with its
    // intent and a "reported" marker even when the intent is not "unknown".
    expect(body).toMatch(/A \(intent: [^,)]+, reported\):/);
    // The verbose per-message subsections must be gone.
    expect(body).not.toMatch(/^### \d+\. /m);
    expect(body).not.toContain('- **Role**:');
  });

  // Issue #73: clicking "Report issue" on a TypeScript hello-world response
  // must include the intent and "reported" annotation in the dialog block.
  test('reporting a TypeScript hello world dialog annotates the message with intent and reported', async ({ page }) => {
    await switchToManualMode(page);

    const input = page.locator('[data-testid="chat-composer-input"]');
    await input.fill('Write hello world in TypeScript');

    const messages = page.locator('[data-testid="chat-message"]');
    const initialCount = await messages.count();

    await page.locator('[data-testid="chat-composer-submit"]').click();
    await expect(messages).toHaveCount(initialCount + 2, { timeout: 15_000 });

    const lastMsg = messages.last();
    await expect(lastMsg).toHaveClass(/assistant/);
    await expect(lastMsg).toContainText('typescript');

    const reportLink = lastMsg.locator('.message-actions a');
    await expect(reportLink).toHaveText('Report issue');

    const href = await reportLink.getAttribute('href');
    expect(href).toBeTruthy();

    const url = new URL(href || '');
    const body = url.searchParams.get('body') || '';

    expect(url.searchParams.get('title')).toContain('Issue with dialog');
    expect(url.searchParams.get('title')).toContain('TypeScript');
    // The reported message must carry both its intent and the "reported" marker.
    expect(body).toMatch(/A \(intent: hello_world_typescript, reported\):/);
    // The dialog must not accidentally annotate user messages.
    expect(body).not.toMatch(/U \(.*reported.*\):/);
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
    await expect(lastMsg).not.toContainText(UNKNOWN_ANSWER_MARKER);
  });

  test('unknown prompts include a prefilled missing-rule issue link', async ({ page }) => {
    await switchToManualMode(page);

    // Pick a phrase no Wikipedia article will match so the unknown-intent
    // fallback path is exercised (the Wikipedia REST API answers many
    // plausible-looking questions like "What is the capital of France?").
    const prompt = 'Quxblort fnordwarble plimsy gabble what?';
    const input = page.locator('[data-testid="chat-composer-input"]');
    await input.fill(prompt);

    const messages = page.locator('[data-testid="chat-message"]');
    const initialCount = await messages.count();

    await page.locator('[data-testid="chat-composer-submit"]').click();
    await expect(messages).toHaveCount(initialCount + 2, { timeout: 15_000 });

    const lastMsg = messages.last();
    await expect(lastMsg).toHaveClass(/assistant/);
    await expect(lastMsg).toContainText(UNKNOWN_ANSWER_MARKER);

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
    // Issue #78: intent now appears inline next to the assistant turn marker
    // ("A (intent: unknown, reported): ...") inside the dialog code block.
    expect(body).toMatch(/A \(intent: unknown[^)]*\):/);
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
    await expect(lastMsg).toContainText('rust hello world');
    await expect(lastMsg).toContainText('Execution status:');
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
    await expect(traceList).toContainText('formal-symbolic-production');
    await expect(traceList).toContainText('Intent');

    const assistantMessage = page.locator('[data-testid="chat-message"].assistant').first();
    await expect(assistantMessage.locator('.intent')).toContainText(/intent:/);
    await expect(assistantMessage.locator('.evidence-list')).toContainText(/source:/);
    await expect(assistantMessage.locator('.thinking-steps')).toContainText(/match_rule|dispatch_handler|fallback/);
  });

  test('composer does not expose an unused preview control', async ({ page }) => {
    await switchToManualMode(page);

    await expect(page.locator('.preview-toggle')).toHaveCount(0);
    await expect(page.locator('.composer-preview')).toHaveCount(0);
  });

  test('demo hint is shown in demo mode and hidden in manual mode', async ({ page }) => {
    const hint = page.locator('[data-testid="composer-demo-hint"]');
    await expect(hint).toBeVisible({ timeout: 5_000 });
    await expect(hint).toContainText('Demo is running');

    await page.locator('.mode-toggle').click();
    await expect(hint).toHaveCount(0);
  });
});

test.describe('Issue #94: theme, localization, and report context', () => {
  test('honors dark color-scheme preference automatically', async ({ page }) => {
    await disableGreetingVariations(page);
    await page.emulateMedia({ colorScheme: 'dark' });
    await page.goto('./');
    await expect(page.locator('.app')).toBeVisible({ timeout: 15_000 });

    const colors = await page.locator('.topbar').evaluate((node) => {
      const styles = window.getComputedStyle(node);
      return {
        background: styles.backgroundColor,
        color: styles.color,
      };
    });

    expect(colors.background).not.toBe('rgb(251, 252, 253)');
    expect(colors.color).not.toBe('rgb(30, 37, 43)');
  });

  test('auto-detects Russian UI language from browser preferences', async ({ page }) => {
    await page.addInitScript(() => {
      Object.defineProperty(window.navigator, 'language', {
        configurable: true,
        get: () => 'ru-RU',
      });
      Object.defineProperty(window.navigator, 'languages', {
        configurable: true,
        get: () => ['ru-RU', 'en-US'],
      });
    });
    await disableGreetingVariations(page);
    await page.goto('./');
    await expect(page.locator('.app')).toBeVisible({ timeout: 15_000 });

    await expect(page.locator('[data-testid="demo-status"]')).toContainText('Демо');
    await page.locator('.mode-toggle').click();
    await expect(page.locator('[data-testid="demo-status"]')).toHaveText('Ручной режим');
    await expect(page.locator('[data-testid="report-issue"]')).toContainText(
      'Сообщить о проблеме',
    );
    await expect(page.locator('[data-testid="report-issue"]')).toHaveAttribute(
      'title',
      /Сообщить о проблеме/,
    );
    await expect(page.locator('.diagnostics-toggle')).toHaveAttribute(
      'title',
      /Показать диагностическую трассировку/,
    );
    await expect(page.locator('[data-testid="chat-composer-input"]')).toHaveAttribute(
      'placeholder',
      'Сообщение formal-ai',
    );
    await expect(page.locator('html')).toHaveAttribute('lang', 'ru');
  });

  test('ships UI dictionaries for all required languages', async ({ page }) => {
    await disableGreetingVariations(page);
    await page.goto('./');
    await expect(page.locator('.app')).toBeVisible({ timeout: 15_000 });

    const labels = await page.evaluate(async () => {
      await window.FormalAiI18n.ready;
      return {
        en: window.FormalAiI18n.t('buttons.reportIssue', 'en'),
        ru: window.FormalAiI18n.t('buttons.reportIssue', 'ru'),
        zh: window.FormalAiI18n.t('buttons.reportIssue', 'zh'),
        hi: window.FormalAiI18n.t('buttons.reportIssue', 'hi'),
      };
    });

    expect(labels).toEqual({
      en: 'Report issue',
      ru: 'Сообщить о проблеме',
      zh: '报告问题',
      hi: 'समस्या रिपोर्ट करें',
    });
  });

  test('loads the published lino-i18n runtime for UI translations', async ({ page }) => {
    await disableGreetingVariations(page);
    await page.goto('./');
    await expect(page.locator('.app')).toBeVisible({ timeout: 15_000 });

    const runtime = await page.evaluate(async () => {
      await window.FormalAiI18n.ready;
      return {
        engine: window.FormalAiI18n.ENGINE_SOURCE,
        russian: window.FormalAiI18n.t('buttons.reportIssue', 'ru'),
        fallback: window.FormalAiI18n.t('buttons.reportIssue', 'zz'),
      };
    });

    expect(runtime).toEqual({
      engine: 'lino-i18n@0.1.1',
      russian: 'Сообщить о проблеме',
      fallback: 'Report issue',
    });
  });

  test('loads the nested Links Notation catalog with generated parent labels', async ({ page }) => {
    await disableGreetingVariations(page);
    await page.goto('./');
    await expect(page.locator('.app')).toBeVisible({ timeout: 15_000 });

    const catalog = await page.evaluate(async () => {
      await window.FormalAiI18n.ready;
      return {
        source: window.FormalAiI18n.ENGINE_SOURCE,
        reportTitle: window.FormalAiI18n.t('titles.reportIssue', 'en'),
        settingsLanguage: window.FormalAiI18n.t('settings.language', 'en'),
        timedStatus: window.FormalAiI18n.t('status.nextDialogIn', 'en', {
          seconds: 8,
        }),
        catalogUrl: window.FormalAiI18n.CATALOG_URL,
      };
    });

    expect(catalog).toMatchObject({
      source: 'lino-i18n@0.1.1',
      settingsLanguage: 'Language',
      timedStatus: 'Next dialog in 8s',
      catalogUrl: 'i18n-catalog.lino',
    });
    expect(catalog.reportTitle).toContain('pre-filled GitHub issue');
    expect(catalog.reportTitle).toContain('docs/upload-memory.md');
  });

  test('issue reports include UI, browser, and coarse location context', async ({ page }) => {
    await disableGreetingVariations(page);
    await page.emulateMedia({ colorScheme: 'dark' });
    await page.goto('./');
    await expect(page.locator('.app')).toBeVisible({ timeout: 15_000 });
    await switchToManualMode(page);

    const href = await page.locator('[data-testid="report-issue"]').getAttribute('href');
    expect(href).toBeTruthy();

    const body = new URL(href || '').searchParams.get('body') || '';
    expect(body).toContain('## User Context');
    expect(body).toContain('**UI Language**');
    expect(body).toContain('**Browser Languages**');
    expect(body).toContain('**Color Scheme**: dark');
    expect(body).toContain('**Time Zone**');
    expect(body).toContain('**Location Inference**');
  });

  test('toolbar labels switch to icon-only before controls wrap', async ({ page }) => {
    await disableGreetingVariations(page);
    await page.setViewportSize({ width: 980, height: 760 });
    await page.goto('./');
    await expect(page.locator('.app')).toBeVisible({ timeout: 15_000 });

    const labelDisplays = await page
      .locator('.topbar-actions .btn-label')
      .evaluateAll((nodes) => nodes.map((node) => window.getComputedStyle(node).display));
    expect(labelDisplays.length).toBeGreaterThan(0);
    expect(labelDisplays.every((display) => display === 'none')).toBe(true);

    const actionBoxes = await page.locator('.topbar-actions > *').evaluateAll((nodes) =>
      nodes.map((node) => {
        const rect = node.getBoundingClientRect();
        return { top: Math.round(rect.top), height: Math.round(rect.height) };
      }),
    );
    const rows = new Set(actionBoxes.filter((box) => box.height > 0).map((box) => box.top));
    expect(rows.size).toBe(1);
  });
});

test.describe('Issue #108: mobile composer and configurable input UI', () => {
  test('mobile uses a left menu, hides the wordmark, and exposes brand details inside the drawer', async ({ page }) => {
    await disableGreetingVariations(page);
    await page.setViewportSize({ width: 390, height: 780 });
    await page.goto('./');
    await expect(page.locator('.app')).toBeVisible({ timeout: 15_000 });

    const menu = page.locator('[data-testid="mobile-menu-toggle"]');
    await expect(menu).toBeVisible();
    const menuBox = await menu.boundingBox();
    expect(menuBox).toBeTruthy();
    expect(menuBox && menuBox.x).toBeLessThan(20);

    await expect(page.locator('.topbar .brand')).toBeHidden();

    await menu.click();
    const drawerBrand = page.locator('[data-testid="drawer-brand"]');
    await expect(drawerBrand).toBeVisible();
    await expect(drawerBrand).toContainText('formal-ai');
    await expect(drawerBrand).toContainText(/v(dev|\d+\.\d+\.\d+)/);
  });

  test('focused mobile composer stays in one row and keeps the top menu reachable', async ({ page }) => {
    await disableGreetingVariations(page);
    await page.setViewportSize({ width: 390, height: 780 });
    await page.goto('./');
    await expect(page.locator('.app')).toBeVisible({ timeout: 15_000 });

    await page.locator('.mode-toggle').click();
    await expect(page.locator('[data-testid="chat-composer-input"]')).toBeEnabled({
      timeout: 5_000,
    });

    const input = page.locator('[data-testid="chat-composer-input"]');
    await input.focus();

    const topbarBox = await page.locator('.topbar').boundingBox();
    const menuBox = await page.locator('[data-testid="mobile-menu-toggle"]').boundingBox();
    const actionBox = await page.locator('[data-testid="composer-menu-toggle"]').boundingBox();
    const inputBox = await input.boundingBox();
    const sendBox = await page.locator('[data-testid="chat-composer-submit"]').boundingBox();
    const viewport = page.viewportSize();

    expect(topbarBox).toBeTruthy();
    expect(menuBox).toBeTruthy();
    expect(actionBox).toBeTruthy();
    expect(inputBox).toBeTruthy();
    expect(sendBox).toBeTruthy();
    expect(viewport).toBeTruthy();

    expect(topbarBox && topbarBox.y).toBeGreaterThanOrEqual(0);
    expect(menuBox && menuBox.y).toBeGreaterThanOrEqual(0);
    expect(sendBox && inputBox && Math.abs(sendBox.y - inputBox.y)).toBeLessThan(4);
    expect(actionBox && inputBox && Math.abs(actionBox.y - inputBox.y)).toBeLessThan(4);
    expect(inputBox && viewport && inputBox.width).toBeGreaterThan(viewport.width * 0.55);
  });

  test('UI skin, chat style, composer style, and action button are configurable and persisted', async ({ page }) => {
    await disableGreetingVariations(page);
    await page.goto('./');
    await expect(page.locator('.app')).toBeVisible({ timeout: 15_000 });

    await page.locator('[data-testid="setting-ui-skin"]').selectOption('glass');
    await page.locator('[data-testid="setting-chat-style"]').selectOption('bubbles');
    await page.locator('[data-testid="setting-composer-style"]').selectOption('glass-clear');
    await page.locator('[data-testid="setting-composer-action"]').selectOption('plus');

    await expect(page.locator('.app')).toHaveClass(/ui-skin-glass/);
    await expect(page.locator('.app')).toHaveClass(/chat-style-bubbles/);
    await expect(page.locator('.app')).toHaveClass(/composer-style-glass-clear/);
    await expect(page.locator('[data-testid="composer-menu-toggle"]')).toContainText('+');

    const stored = await page.evaluate(
      () => window.localStorage.getItem('formal-ai.preferences.v1') || '',
    );
    expect(stored).toContain('uiSkin "glass"');
    expect(stored).toContain('chatStyle "bubbles"');
    expect(stored).toContain('composerStyle "glass-clear"');
    expect(stored).toContain('composerAction "plus"');
  });

  test('composer menu exposes attachment and memory actions at the input', async ({ page }) => {
    await disableGreetingVariations(page);
    await page.goto('./');
    await expect(page.locator('.app')).toBeVisible({ timeout: 15_000 });

    await page.locator('[data-testid="composer-menu-toggle"]').click();
    const menu = page.locator('[data-testid="composer-menu"]');
    await expect(menu).toBeVisible();
    await expect(menu).toContainText('Attach files');
    await expect(menu).toContainText('Export memory');
    await expect(menu).toContainText('Import memory');
    await expect(menu).toContainText('Report issue');
  });
});

test.describe('Issue #110: mobile keyboard viewport handling', () => {
  test('focused mobile input pins the app shell to the visual viewport offset', async ({ page }) => {
    await disableGreetingVariations(page);
    await page.addInitScript(() => {
      const listeners = new Map();
      const viewport = {
        width: 390,
        height: 780,
        offsetTop: 0,
        offsetLeft: 0,
        pageTop: 0,
        pageLeft: 0,
        scale: 1,
        addEventListener(type, listener) {
          const current = listeners.get(type) || [];
          current.push(listener);
          listeners.set(type, current);
        },
        removeEventListener(type, listener) {
          const current = listeners.get(type) || [];
          listeners.set(
            type,
            current.filter((entry) => entry !== listener),
          );
        },
        dispatchEvent(event) {
          for (const listener of listeners.get(event.type) || []) {
            listener.call(viewport, event);
          }
          return true;
        },
        __set(next) {
          Object.assign(viewport, next);
          viewport.dispatchEvent(new Event('resize'));
          viewport.dispatchEvent(new Event('scroll'));
        },
      };
      Object.defineProperty(window, 'visualViewport', {
        configurable: true,
        value: viewport,
      });
    });
    await page.setViewportSize({ width: 390, height: 780 });
    await page.goto('./');
    await expect(page.locator('.app')).toBeVisible({ timeout: 15_000 });

    await page.locator('.mode-toggle').click();
    const input = page.locator('[data-testid="chat-composer-input"]');
    await expect(input).toBeEnabled({ timeout: 5_000 });
    await input.focus();

    const viewportState = await page.evaluate(() => {
      window.visualViewport.__set({
        height: 520,
        offsetTop: 180,
      });
      return {
        height: window.visualViewport.height,
        offsetTop: window.visualViewport.offsetTop,
      };
    });

    const topbarBox = await page.locator('.topbar').boundingBox();
    const composerBox = await page.locator('.composer').boundingBox();

    expect(topbarBox).toBeTruthy();
    expect(composerBox).toBeTruthy();
    expect(Math.round(topbarBox.y)).toBe(viewportState.offsetTop);
    expect(
      Math.round(composerBox.y + composerBox.height),
    ).toBeLessThanOrEqual(viewportState.offsetTop + viewportState.height + 1);
  });
});
