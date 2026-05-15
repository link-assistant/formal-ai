// @ts-check
const { test, expect } = require('@playwright/test');

async function switchToManualMode(page) {
  const demoToggle = page.locator('.mode-toggle');
  await expect(demoToggle).toContainText('Demo on');
  await demoToggle.click();
  await expect(demoToggle).toContainText('Demo');
  await expect(page.locator('[data-testid="demo-status"]')).toHaveText('Manual mode');
  await expect(page.locator('[data-testid="chat-composer-input"]')).toBeEnabled({
    timeout: 5_000,
  });
  await expect(page.locator('[data-testid="tool-entry"]').first()).toBeVisible({
    timeout: 10_000,
  });
}

// Issue #27: greeting randomisation defaults to ON. Tests below pin the
// canonical greeting text, so disable randomisation up-front for stability.
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

test.describe('multilingual chat surface', () => {
  test.beforeEach(async ({ page }) => {
    await disableGreetingVariations(page);
    await page.goto('./');
    await expect(page.locator('.app')).toBeVisible({ timeout: 15_000 });
    await switchToManualMode(page);
  });

  test('Russian greeting replies in Russian', async ({ page }) => {
    const last = await sendPrompt(page, 'Привет');
    await expect(last).toHaveClass(/assistant/);
    await expect(last).toContainText(/Здравствуйте|Привет/);
  });

  test('Hindi greeting replies in Hindi', async ({ page }) => {
    const last = await sendPrompt(page, 'नमस्ते');
    await expect(last).toHaveClass(/assistant/);
    await expect(last).toContainText('नमस्ते');
  });

  test('Chinese identity question replies in Chinese', async ({ page }) => {
    const last = await sendPrompt(page, '你是谁?');
    await expect(last).toHaveClass(/assistant/);
    await expect(last).toContainText('formal-ai');
    await expect(last).toContainText(/符号|确定性/);
  });

  test('Russian "What is X?" returns the offline concept summary', async ({ page }) => {
    const last = await sendPrompt(page, 'Что такое Википедия?');
    await expect(last).toHaveClass(/assistant/);
    await expect(last).toContainText(/Wikipedia|encyclopedia/i);
  });

  test('Chinese "X 是什么?" returns the offline concept summary', async ({ page }) => {
    const last = await sendPrompt(page, '维基百科是什么?');
    await expect(last).toHaveClass(/assistant/);
    await expect(last).toContainText(/Wikipedia|encyclopedia/i);
  });
});

test.describe('Wikipedia REST fallback', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('./');
    await expect(page.locator('.app')).toBeVisible({ timeout: 15_000 });
    await switchToManualMode(page);
  });

  test('"What is X?" for an out-of-corpus term fetches a Wikipedia summary', async ({ page }) => {
    // Stub the Wikipedia REST endpoint so the test is hermetic and does not depend
    // on external network availability or rate limiting.
    await page.route('**/api/rest_v1/page/summary/**', async (route) => {
      const json = {
        title: 'Albert Einstein',
        extract: 'Albert Einstein was a German-born theoretical physicist...',
        type: 'standard',
        content_urls: {
          desktop: { page: 'https://en.wikipedia.org/wiki/Albert_Einstein' },
        },
      };
      await route.fulfill({
        status: 200,
        contentType: 'application/json',
        body: JSON.stringify(json),
      });
    });

    const last = await sendPrompt(page, 'What is Albert Einstein?');
    await expect(last).toHaveClass(/assistant/);
    await expect(last).toContainText('Albert Einstein');
    await expect(last).toContainText('theoretical physicist');
    await expect(last).toContainText('en.wikipedia.org');
  });

  // Issue #21: Wikipedia returns percent-encoded URLs for non-ASCII titles.
  // The chat must display the readable Cyrillic form while the underlying
  // link still points at the canonical (encoded) URL.
  test('Russian Wikipedia summary displays decoded Cyrillic URL with encoded href', async ({ page }) => {
    const encodedUrl =
      'https://ru.wikipedia.org/wiki/%D0%98%D0%B7%D1%83%D0%BC%D1%80%D1%83%D0%B4';
    const humanUrl = 'https://ru.wikipedia.org/wiki/Изумруд';
    await page.route('**/api/rest_v1/page/summary/**', async (route) => {
      const json = {
        title: 'Изумруд',
        extract: 'Изумруд — драгоценный камень берилловой группы.',
        type: 'standard',
        content_urls: { desktop: { page: encodedUrl } },
      };
      await route.fulfill({
        status: 200,
        contentType: 'application/json',
        body: JSON.stringify(json),
      });
    });

    const last = await sendPrompt(page, 'Что такое изумруд?');
    await expect(last).toHaveClass(/assistant/);
    // Display text is the readable IRI form.
    await expect(last).toContainText(humanUrl);
    // And the percent-encoded form must not leak into the visible message.
    await expect(last).not.toContainText(
      '%D0%98%D0%B7%D1%83%D0%BC%D1%80%D1%83%D0%B4',
    );
    // The anchor's href stays the canonical encoded URL so clicking it still resolves.
    const anchor = last.locator(`a[href="${encodedUrl}"]`);
    await expect(anchor).toHaveCount(1);
    await expect(anchor).toHaveText(humanUrl);
  });

});

test.describe('memory export/import', () => {
  test.beforeEach(async ({ page }) => {
    await disableGreetingVariations(page);
    await page.goto('./');
    await expect(page.locator('.app')).toBeVisible({ timeout: 15_000 });
    await switchToManualMode(page);
  });

  test('Export memory and Import memory buttons are present', async ({ page }) => {
    await expect(page.locator('[data-testid="memory-export"]')).toBeVisible();
    await expect(page.locator('[data-testid="memory-import"]')).toBeVisible();
  });

  test('Export memory downloads a full formal_ai_bundle by default (R109)', async ({ page }) => {
    // Send one message so there is at least one event in the log.
    await sendPrompt(page, 'Hi');

    const [download] = await Promise.all([
      page.waitForEvent('download'),
      page.locator('[data-testid="memory-export"]').click(),
    ]);

    expect(download.suggestedFilename()).toBe('formal-ai-memory.lino');

    const path = await download.path();
    expect(path).toBeTruthy();
    const fs = require('node:fs');
    const text = fs.readFileSync(path, 'utf8');
    // R109: the default export is now the full self-contained bundle —
    // seed files + UI preferences + environment metadata + the embedded
    // demo_memory log. The user must not have to click a second button to
    // get the full state.
    expect(text.startsWith('formal_ai_bundle\n')).toBe(true);
    expect(text).toContain('seed_files');
    expect(text).toContain('seed/agent-info.lino');
    expect(text).toContain('preferences');
    expect(text).toContain('demo_memory');
    expect(text).toContain('role "user"');
    expect(text).toContain('content "Hi"');
    // Status indicator should reflect the full-memory shape.
    await expect(page.locator('[data-testid="memory-status"]')).toContainText(/Exported full memory:/);
  });

  test('Import memory accepts a Links Notation file', async ({ page }) => {
    const importInput = page.locator('[data-testid="memory-import-input"]');
    const lino = [
      'demo_memory',
      '  event "1"',
      '    role "user"',
      '    content "Imported greeting"',
      '    sentAt "2026-05-15T12:00:00.000Z"',
      '  event "2"',
      '    role "assistant"',
      '    intent "greeting"',
      '    content "Hi, how may I help you?"',
      '    sentAt "2026-05-15T12:00:01.000Z"',
      '',
    ].join('\n');
    await importInput.setInputFiles({
      name: 'memory.lino',
      mimeType: 'text/plain',
      buffer: Buffer.from(lino, 'utf8'),
    });
    // R110: legacy demo_memory imports must still succeed. R111: importing a
    // legacy log surfaces a migration suggestion because no seed metadata is
    // attached, so the status indicator reports "Migration: ..." alongside
    // the import count.
    await expect(page.locator('[data-testid="memory-status"]')).toContainText('Imported 2 events');
    await expect(page.locator('[data-testid="memory-status"]')).toContainText(/Migration:.*legacy demo_memory/);
  });

  test('Import memory accepts a formal_ai_bundle and reports seed migrations (R110, R111)', async ({ page }) => {
    const importInput = page.locator('[data-testid="memory-import-input"]');
    const bundle = [
      'formal_ai_bundle',
      '  exported_at "2026-05-15T12:00:00.000Z"',
      '  version "0.0.1"',
      '  seed_files',
      '    file "seed/agent-info.lino"',
      '      agent_info',
      '        field "version"',
      '          value "0.0.1"',
      '  preferences',
      '    demo_mode "off"',
      '  demo_memory',
      '    event "1"',
      '      role "user"',
      '      content "Imported via bundle"',
      '      sentAt "2026-05-15T12:00:00.000Z"',
      '',
    ].join('\n');
    await importInput.setInputFiles({
      name: 'bundle.lino',
      mimeType: 'text/plain',
      buffer: Buffer.from(bundle, 'utf8'),
    });
    await expect(page.locator('[data-testid="memory-status"]')).toContainText('Imported 1 event(s) from full bundle');
    await expect(page.locator('[data-testid="memory-status"]')).toContainText(/Migration: Seed version 0\.0\.1 →/);
  });

  test('Memory module exposes no delete/forget operation', async ({ page }) => {
    const api = await page.evaluate(() => Object.keys(window.FormalAiMemory || {}));
    expect(api).toContain('appendEvent');
    expect(api).toContain('listEvents');
    expect(api).toContain('importEvents');
    expect(api).toContain('exportLinksNotation');
    expect(api).toContain('exportBundle');
    // R109/R110/R111: full-memory export, header-agnostic import, and
    // migration suggestions must all be reachable from the public API.
    expect(api).toContain('exportFullMemory');
    expect(api).toContain('importFullMemory');
    expect(api).toContain('suggestMigrations');
    expect(api).not.toContain('delete');
    expect(api).not.toContain('deleteEvent');
    expect(api).not.toContain('forget');
    expect(api).not.toContain('clear');
    expect(api).not.toContain('remove');
  });

  test('Issue #27: Download bundle button is removed (duplicate of Export memory)', async ({ page }) => {
    await expect(page.locator('[data-testid="memory-bundle"]')).toHaveCount(0);
    // The underlying exportBundle helper must remain on the public API for
    // Rust/CLI parity; only the redundant UI button is gone.
    const api = await page.evaluate(() => Object.keys(window.FormalAiMemory || {}));
    expect(api).toContain('exportBundle');
  });

  test('Issue #27: Export memory does not surface a "Bundled N events + seed" label', async ({ page }) => {
    await sendPrompt(page, 'Hi');
    const [download] = await Promise.all([
      page.waitForEvent('download'),
      page.locator('[data-testid="memory-export"]').click(),
    ]);
    expect(download.suggestedFilename()).toBe('formal-ai-memory.lino');
    const status = await page.locator('[data-testid="memory-status"]').innerText();
    expect(status).not.toMatch(/bundled\s+\d+\s+events\s+\+\s+seed/i);
  });

  test('Issue #27: typing "Export memory" triggers the export button', async ({ page }) => {
    const input = page.locator('[data-testid="chat-composer-input"]');
    await expect(input).toBeEnabled({ timeout: 5_000 });
    await input.fill('Export memory');
    const [download] = await Promise.all([
      page.waitForEvent('download'),
      page.locator('[data-testid="chat-composer-submit"]').click(),
    ]);
    expect(download.suggestedFilename()).toBe('formal-ai-memory.lino');
    const messages = page.locator('[data-testid="chat-message"]');
    await expect(messages.last()).toContainText('Triggered Export memory');
  });

  test('Issue #27: typing "Export your memory" also triggers the export button', async ({ page }) => {
    const input = page.locator('[data-testid="chat-composer-input"]');
    await expect(input).toBeEnabled({ timeout: 5_000 });
    await input.fill('Export your memory');
    const [download] = await Promise.all([
      page.waitForEvent('download'),
      page.locator('[data-testid="chat-composer-submit"]').click(),
    ]);
    expect(download.suggestedFilename()).toBe('formal-ai-memory.lino');
  });

  test('Issue #27: typing "Import memory" opens the file picker', async ({ page }) => {
    const input = page.locator('[data-testid="chat-composer-input"]');
    await expect(input).toBeEnabled({ timeout: 5_000 });
    // We cannot programmatically observe a native file dialog opening, but we
    // can confirm the assistant acknowledges the trigger and the file input
    // remains in the DOM ready to accept a file.
    await input.fill('Import memory');
    await page.locator('[data-testid="chat-composer-submit"]').click();
    const messages = page.locator('[data-testid="chat-message"]');
    await expect(messages.last()).toContainText('Triggered Import memory');
    await expect(page.locator('[data-testid="memory-import-input"]')).toHaveCount(1);
  });

  test('Report issue link is present in the topbar and prefills full-memory + zip instructions (R112)', async ({ page }) => {
    const reportLink = page.locator('[data-testid="report-issue"]');
    await expect(reportLink).toBeVisible();
    const href = await reportLink.getAttribute('href');
    expect(href).toBeTruthy();
    const url = new URL(href);
    expect(url.origin + url.pathname).toBe('https://github.com/link-assistant/formal-ai/issues/new');
    const body = url.searchParams.get('body') || '';
    // R112: the prefilled body must tell the user to attach the full memory
    // export, wrap it in a .zip (GitHub does not accept .lino), and redact
    // sensitive content before attaching.
    expect(body).toContain('formal-ai-memory.lino');
    expect(body).toContain('Export memory');
    expect(body).toMatch(/\.zip/);
    expect(body).toMatch(/redact/i);
  });

  test('Tool registry surfaces seed-loaded tools with mode badges', async ({ page }) => {
    const registry = page.locator('[data-testid="tool-registry"]');
    await expect(registry).toBeVisible({ timeout: 10_000 });
    const entries = page.locator('[data-testid="tool-entry"]');
    await expect(entries.first()).toBeVisible();
    const count = await entries.count();
    expect(count).toBeGreaterThan(0);
    const modes = await entries.evaluateAll((nodes) =>
      nodes.map((node) => node.getAttribute('data-tool-mode')),
    );
    expect(modes).toContain('thinking');
  });

  test('Reasoning steps and tool calls land in the append-only log', async ({ page }) => {
    await sendPrompt(page, 'Hi');
    const events = await page.evaluate(async () => {
      const list = await window.FormalAiMemory.listEvents();
      return list.map((event) => ({ kind: event.kind, role: event.role }));
    });
    const kinds = new Set(events.map((event) => event.kind).filter(Boolean));
    expect(kinds.has('message')).toBe(true);
    expect(kinds.has('reasoning')).toBe(true);
  });
});

test.describe('Issue #27: random greeting variations', () => {
  test.beforeEach(async ({ page }) => {
    // Default-on: do NOT call disableGreetingVariations — the seed-driven
    // randomisation must be observable when the user accepts the defaults.
    await page.goto('./');
    await expect(page.locator('.app')).toBeVisible({ timeout: 15_000 });
    await switchToManualMode(page);
  });

  test('English greeting falls within the seeded variant list', async ({ page }) => {
    const last = await sendPrompt(page, 'Hi');
    const text = (await last.innerText()).trim();
    const variants = [
      'Hi, how may I help you?',
      'Hello! How can I assist you today?',
      'Hi there! What can I do for you?',
      'Hey, how can I help?',
      'Hello — what would you like to explore?',
    ];
    expect(variants.some((variant) => text.includes(variant))).toBe(true);
  });

  test('disabling variations pins the canonical English greeting', async ({ page, context }) => {
    await context.addInitScript(() => {
      try {
        window.localStorage.setItem(
          'formal-ai.preferences.v1',
          'demo_preferences\n  greetingVariations "off"',
        );
      } catch (_error) {}
    });
    await page.goto('./');
    await expect(page.locator('.app')).toBeVisible({ timeout: 15_000 });
    await switchToManualMode(page);
    for (let attempt = 0; attempt < 3; attempt += 1) {
      const last = await sendPrompt(page, 'Hi');
      await expect(last).toContainText('Hi, how may I help you?');
    }
  });
});
