// @ts-check
// Assistant behavior settings (issue #82) and memory export / import
// (append-only memory, bundle download, seed migrations).
// (Split from the former multilingual.spec.js; shared helpers live in
// ./support/multilingual.js.)
const { test, expect } = require('@playwright/test');
const { WORKER_READY_TIMEOUT_MS } = require('./support/worker-ready');
const {
  switchToManualMode,
  disableGreetingVariations,
  sendPrompt,
  setRangeValue,
  emulateReducedMotion,
} = require('./support/multilingual');

// Issue #541 (R5/R6): emulate prefers-reduced-motion so answers render at
// once instead of racing the paced reveal (see support/multilingual.js).
emulateReducedMotion(test);

test.describe('Issue #82: assistant behavior settings', () => {
  test.beforeEach(async ({ page }) => {
    await disableGreetingVariations(page);
    await page.goto('./');
    await expect(page.locator('.app')).toBeVisible({ timeout: 15_000 });
    await switchToManualMode(page);
  });

  test('settings sidebar exposes ambiguity, temperature, language, skins, and location controls', async ({ page }) => {
    const settings = page.locator('[data-testid="sidebar-settings"]');
    await expect(settings).toBeVisible();
    if ((await settings.getAttribute('data-collapsed')) === 'true') {
      await settings.locator('.sidebar-section-header').click();
    }
    await expect(page.locator('[data-testid="setting-guess-probability"]')).toBeVisible();
    await expect(page.locator('[data-testid="setting-follow-up-probability"]')).toBeVisible();
    await expect(page.locator('[data-testid="setting-temperature"]')).toBeVisible();
    await expect(page.locator('[data-testid="setting-definition-fusion"]')).toBeVisible();
    await expect(page.locator('[data-testid="setting-ui-language"]')).toBeVisible();
    await expect(page.locator('[data-testid="setting-theme"]')).toBeVisible();
    await expect(page.locator('[data-testid="setting-ui-skin"]')).toBeVisible();
    await expect(page.locator('[data-testid="setting-chat-style"]')).toBeVisible();
    await expect(page.locator('[data-testid="setting-assistant-name"]')).toBeVisible();
    await expect(page.locator('[data-testid="setting-location"]')).toBeVisible();

    await setRangeValue(page, 'setting-temperature', 0);
    await setRangeValue(page, 'setting-follow-up-probability', 0);
    await page.locator('[data-testid="setting-definition-fusion"]').selectOption('auto');
    await page.locator('[data-testid="setting-assistant-name"]').fill('Astra');
    await page.locator('[data-testid="setting-location"]').fill('Berlin');
    await page.locator('[data-testid="setting-theme"]').selectOption('dark');

    await expect.poll(() =>
      page.evaluate(() => window.localStorage.getItem('formal-ai.preferences.v1') || ''),
    ).toContain('theme "dark"');
    const stored = await page.evaluate(() =>
      window.localStorage.getItem('formal-ai.preferences.v1') || '',
    );
    expect(stored).toContain('temperature "0"');
    expect(stored).toContain('followUpProbability "0"');
    expect(stored).toContain('definitionFusion "auto"');
    expect(stored).toContain('assistantName "Astra"');
    expect(stored).toContain('location "Berlin"');
    await expect(page.locator('html')).toHaveAttribute('data-theme', 'dark');
  });

  test('low ambiguity guessing asks before using a fuzzy Wikipedia match', async ({ page }) => {
    await setRangeValue(page, 'setting-guess-probability', 0);

    await page.route('**/api/rest_v1/page/summary/**', async (route) => {
      const slug = decodeURIComponent(route.request().url().split('/').pop() || '');
      if (slug === 'Грамматика') {
        await route.fulfill({
          status: 200,
          contentType: 'application/json',
          body: JSON.stringify({
            title: 'Грамматика',
            type: 'standard',
            extract: 'Грамматика — раздел лингвистики, изучающий грамматический строй языка.',
            content_urls: {
              desktop: { page: 'https://ru.wikipedia.org/wiki/Грамматика' },
            },
          }),
        });
        return;
      }
      await route.fulfill({
        status: 404,
        contentType: 'application/json',
        body: JSON.stringify({ httpCode: 404, httpReason: 'Not Found' }),
      });
    });

    await page.route('**/rest.php/v1/search/page**', async (route) => {
      await route.fulfill({
        status: 200,
        contentType: 'application/json',
        body: JSON.stringify({
          pages: [{ key: 'Грамматика', title: 'Грамматика' }],
        }),
      });
    });

    const last = await sendPrompt(page, 'что такое граматика');
    await expect(last).toHaveClass(/assistant/);
    await expect(last).toContainText(/Грамматика/);
    await expect(last).toContainText(/уточните|Did you mean/i);
    await expect(last).not.toContainText('раздел лингвистики');
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

  test('Memory module exposes explicit destructive operations only', async ({ page }) => {
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
    expect(api).toContain('purgeDeletedConversations');
    expect(api).toContain('deleteEventsByConversationId');
    expect(api).toContain('clearEvents');
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
    await expect(input).toBeEnabled({ timeout: WORKER_READY_TIMEOUT_MS });
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
    await expect(input).toBeEnabled({ timeout: WORKER_READY_TIMEOUT_MS });
    await input.fill('Export your memory');
    const [download] = await Promise.all([
      page.waitForEvent('download'),
      page.locator('[data-testid="chat-composer-submit"]').click(),
    ]);
    expect(download.suggestedFilename()).toBe('formal-ai-memory.lino');
  });

  test('Issue #27: typing "Import memory" opens the file picker', async ({ page }) => {
    const input = page.locator('[data-testid="chat-composer-input"]');
    await expect(input).toBeEnabled({ timeout: WORKER_READY_TIMEOUT_MS });
    // We cannot programmatically observe a native file dialog opening, but we
    // can confirm the assistant acknowledges the trigger and the file input
    // remains in the DOM ready to accept a file.
    await input.fill('Import memory');
    await page.locator('[data-testid="chat-composer-submit"]').click();
    const messages = page.locator('[data-testid="chat-message"]');
    await expect(messages.last()).toContainText('Triggered Import memory');
    await expect(page.locator('[data-testid="memory-import-input"]')).toHaveCount(1);
  });

  test('Issue #196: reset memory phrases are recognised in every supported language', async ({ page }) => {
    const resetPromptCases = [
      { language: 'en', phrase: 'Reset memory' },
      { language: 'ru', phrase: 'сбросить память' },
      { language: 'hi', phrase: 'स्मृति रीसेट करें' },
      { language: 'zh', phrase: '重置记忆' },
    ];
    const dialogs = [];
    page.on('dialog', async (dialog) => {
      dialogs.push(dialog.message());
      if (dialogs.length % 2 === 1) {
        await dialog.dismiss();
      } else {
        await dialog.accept();
      }
    });

    for (const { language, phrase } of resetPromptCases) {
      await sendPrompt(page, `Memory reset seed ${language}`);
      const input = page.locator('[data-testid="chat-composer-input"]');
      await expect(input).toBeEnabled({ timeout: WORKER_READY_TIMEOUT_MS });
      await input.fill(phrase);
      await page.locator('[data-testid="chat-composer-submit"]').click();
      await expect(page.locator('[data-testid="chat-message"]')).toHaveCount(0);
      await expect.poll(() =>
        page.evaluate(() =>
          window.FormalAiMemory.listEvents().then((events) => events.length),
        ),
      ).toBe(0);
    }

    expect(dialogs.length).toBe(resetPromptCases.length * 2);
  });

  test('Report issue link is present in the topbar and links to the upload-memory guide (R112 + issue #78)', async ({ page }) => {
    const reportLink = page.locator('[data-testid="report-issue"]');
    await expect(reportLink).toBeVisible();
    const href = await reportLink.getAttribute('href');
    expect(href).toBeTruthy();
    const url = new URL(href);
    expect(url.origin + url.pathname).toBe('https://github.com/link-assistant/formal-ai/issues/new');
    const body = url.searchParams.get('body') || '';
    // Issue #78: the prefilled body must stay short. It still mentions the
    // export filename, the Export memory action, .zip / Gist upload paths, and
    // redaction — but only in one line that links to docs/upload-memory.md for
    // the full walkthrough (R112).
    expect(body).toContain('formal-ai-memory.lino');
    expect(body).toContain('Export memory');
    expect(body).toMatch(/\.zip/);
    expect(body).toMatch(/redact/i);
    expect(body).toContain('docs/upload-memory.md');
    // The long block of per-OS zip instructions that used to live in the body
    // must be gone (it has moved into docs/upload-memory.md so a single link
    // is enough).
    expect(body).not.toMatch(/Send to.*Compressed/);
    expect(body).not.toMatch(/right-click.*Compress/);
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
    await expect(registry).toContainText('calculator');
  });

  test('Issue #112: tool registry includes all supported tools and localizes descriptions', async ({ page }) => {
    await page.locator('[data-testid="setting-ui-language"]').selectOption('ru');
    const entries = page.locator('[data-testid="tool-entry"]');
    const toolIds = await entries.evaluateAll((nodes) =>
      nodes.map((node) => node.getAttribute('data-tool-id')),
    );
    expect(toolIds).toEqual(expect.arrayContaining([
      'tool_http_fetch',
      'tool_url_navigate',
      'tool_web_search',
      'tool_wikipedia_lookup',
      'tool_calculator',
      'tool_eval_js',
      'tool_read_local_file',
      'tool_append_memory',
      'tool_export_memory',
      'tool_import_memory',
      'tool_conversation_recall',
      'tool_concept_lookup',
      'tool_write_program',
      'tool_intent_routing',
      'tool_fact_lookup',
      'tool_summarize_conversation',
      'tool_brainstorm',
      'tool_coreference',
      'tool_roleplay',
    ]));
    await expect(page.locator('[data-tool-id="tool_calculator"] .tool-desc')).toContainText(/Вычисляет|математические/);
    await expect(page.locator('[data-tool-id="tool_web_search"] .tool-desc')).not.toContainText('Search the open web');
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
