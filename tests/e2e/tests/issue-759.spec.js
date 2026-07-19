// @ts-check
const { test, expect } = require('@playwright/test');

const supportedUiLanguages = [
  { language: 'en', name: 'English' },
  { language: 'ru', name: 'Russian' },
  { language: 'hi', name: 'Hindi' },
  { language: 'zh', name: 'Chinese' },
];

test('desktop selector routes installed Agent and native turns through one UI', async ({ page }) => {
  await page.route('**/v1/chat/completions', async (route) => {
    await page.evaluate(() => window.__issue759ProxyLog.push('out-of-box'));
    await route.fulfill({
      contentType: 'application/json',
      body: JSON.stringify({
        choices: [{ message: { role: 'assistant', content: 'native roundtrip' } }],
      }),
    });
  });
  await page.addInitScript(() => {
    const existingPreferences = window.localStorage.getItem('formal-ai.preferences.v1') || '';
    const languageMatch = existingPreferences.match(/^\s+uiLanguage "([^"]+)"/m);
    const uiLanguage = languageMatch ? languageMatch[1] : 'en';
    window.localStorage.setItem(
      'formal-ai.preferences.v1',
      `demo_preferences\n  demoMode "off"\n  greetingVariations "off"\n  diagnosticsMode "off"\n  uiLanguage "${uiLanguage}"`,
    );
    window.__issue759ProxyLog = [];
    let activeEngine = 'agent';
    let eventListener = null;
    const status = () => ({
      shell: 'Electron',
      apiBase: window.location.origin,
      graphUrl: `${window.location.origin}/v1/graph`,
      memory: 'formal_ai_bundle',
      apiReady: true,
      activeEngine,
      engines: [
        { id: 'out-of-box', label: 'Out of the box', type: 'native', available: true },
        { id: 'agent', label: 'Agent', type: 'passthrough', available: true },
      ],
    });
    window.FormalAiDesktop = {
      getStatus: async () => status(),
      setEngine: async (engine) => {
        activeEngine = engine;
        return status();
      },
      setToolGrants: async () => ({}),
      syncMemory: async () => ({ ok: true }),
      onAgentEvent: (callback) => {
        eventListener = callback;
        return () => { eventListener = null; };
      },
      runAgentProvider: async (request) => {
        window.__issue759ProxyLog.push(request.commanderTool || activeEngine);
        if (eventListener) {
          eventListener({ requestId: request.requestId, engine: activeEngine,
            event: { type: 'assistant', content: 'agent streaming' } });
        }
        // Keep the mocked turn pending long enough for Playwright to observe the
        // transient stream surface before the completed answer replaces it.
        await new Promise((resolve) => setTimeout(resolve, 500));
        return {
          ok: true,
          answer: {
            intent: 'agent_cli_turn',
            content: 'agent roundtrip',
            evidence: ['provider:commander'],
            steps: [],
            toolCalls: [],
          },
        };
      },
    };
  });

  await page.goto('./');
  const selector = page.locator('[data-testid="desktop-engine-selector"]');
  await expect(selector).toHaveValue('agent');
  await expect(selector.locator('option')).toHaveText(['Out of the box', 'Agent']);
  const desktopHeader = page.locator(
    '[data-testid="sidebar-desktop"] .sidebar-section-header',
  );
  if ((await desktopHeader.getAttribute('aria-expanded')) === 'false') {
    await desktopHeader.click();
  }
  await expect(selector).toBeVisible();
  await page.screenshot({
    path: '../../docs/case-studies/issue-759/engine-selector.png',
    fullPage: true,
  });

  const input = page.locator('[data-testid="chat-composer-input"]');
  await input.fill('use installed agent');
  await page.locator('[data-testid="chat-composer-submit"]').click();
  await expect(page.locator('[data-testid="desktop-agent-stream"]')).toContainText('agent streaming');
  await expect(page.locator('[data-testid="chat-message"]').last()).toContainText('agent roundtrip');

  await selector.selectOption('out-of-box');
  await input.fill('use native engine');
  await page.locator('[data-testid="chat-composer-submit"]').click();
  await expect(page.locator('[data-testid="chat-message"]').last()).toContainText('native roundtrip');
  await expect.poll(() => page.evaluate(() => window.__issue759ProxyLog)).toEqual([
    'agent',
    'out-of-box',
  ]);

  for (const { language, name } of supportedUiLanguages) {
    await page.evaluate((nextLanguage) => {
      window.localStorage.setItem(
        'formal-ai.preferences.v1',
        `demo_preferences\n  demoMode "off"\n  greetingVariations "off"\n  diagnosticsMode "off"\n  uiLanguage "${nextLanguage}"`,
      );
    }, language);
    await page.reload();
    await expect(page.locator('html'), `${name} UI keeps the engine selector`).toHaveAttribute(
      'lang',
      language,
    );
    await expect(page.locator('[data-testid="desktop-engine-selector"]')).toHaveValue('agent');
  }
});
