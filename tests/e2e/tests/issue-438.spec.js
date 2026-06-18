// @ts-check
const { test, expect } = require('@playwright/test');

// Issue #438 (follow-up): the desktop sidebar gained a one-click "Services"
// panel that starts/stops the prepared Telegram bot and OpenAI-compatible
// server containers. The panel is part of the language-facing renderer
// (src/web/app.js), so it must keep working across every supported UI
// language, mirroring the issue-280 desktop-shell guard.
const supportedUiLanguages = [
  { language: 'en', name: 'English' },
  { language: 'ru', name: 'Russian' },
  { language: 'hi', name: 'Hindi' },
  { language: 'zh', name: 'Chinese' },
];

function installDesktopBridge() {
  // Only the Electron desktop shell exposes the service handlers; the mock here
  // mirrors that bridge so the renderer reveals the Services panel. Calls are
  // recorded on the window so the test can assert one-click wiring.
  window.__serviceCalls = [];
  const snapshot = {
    dockerAvailable: true,
    services: [
      { key: 'telegram', label: 'Telegram bot', state: 'stopped', running: false },
      {
        key: 'server',
        label: 'OpenAI-compatible server',
        state: 'running',
        running: true,
        url: 'http://127.0.0.1:8080/v1',
      },
    ],
  };
  window.FormalAiDesktop = {
    getStatus: async () => ({
      shell: 'Electron',
      apiBase: 'http://127.0.0.1:18080',
      staticBase: 'http://127.0.0.1:18081',
      graphUrl: 'http://127.0.0.1:18080/v1/graph',
      memory: 'formal_ai_bundle',
      agentModeDefault: false,
      toolCallPolicy: 'explicit-permission',
      apiReady: true,
    }),
    serviceStatus: async () => snapshot,
    startService: async (request) => {
      window.__serviceCalls.push({ action: 'start', request });
      return { ok: true };
    },
    stopService: async (request) => {
      window.__serviceCalls.push({ action: 'stop', request });
      return { ok: true };
    },
  };
}

function preserveUiLanguage() {
  // Init scripts run on every navigation/reload, so the existing UI language is
  // read back and preserved instead of being reset — otherwise the language
  // switch in the multilingual test would be clobbered on reload.
  try {
    const existing = window.localStorage.getItem('formal-ai.preferences.v1') || '';
    const languageMatch = existing.match(/^\s+uiLanguage "([^"]+)"/m);
    const uiLanguage = languageMatch ? languageMatch[1] : 'auto';
    window.localStorage.setItem(
      'formal-ai.preferences.v1',
      `demo_preferences\n  demoMode "off"\n  greetingVariations "off"\n  uiLanguage "${uiLanguage}"`,
    );
  } catch (_error) {
    // localStorage can be unavailable in hardened browser contexts.
  }
}

test.describe('Issue #438: desktop one-click services panel', () => {
  test.beforeEach(async ({ page }) => {
    await page.addInitScript(preserveUiLanguage);
    await page.addInitScript(installDesktopBridge);
    await page.goto('./');
    await expect(page.locator('.app')).toBeVisible({ timeout: 15_000 });
  });

  test('renders one-click start/stop controls for both prepared containers', async ({
    page,
  }) => {
    await expect(page.locator('[data-testid="sidebar-services"]')).toBeVisible();
    await expect(page.locator('[data-testid="desktop-services-panel"]')).toBeVisible();

    // Telegram bot is stopped: its start button is enabled, the stop button is
    // disabled, and the inline token field is offered before it can start.
    const telegram = page.locator('[data-testid="desktop-service-telegram"]');
    await expect(telegram).toHaveAttribute('data-state', 'stopped');
    await expect(
      page.locator('[data-testid="desktop-service-start-telegram"]'),
    ).toBeEnabled();
    await expect(
      page.locator('[data-testid="desktop-service-stop-telegram"]'),
    ).toBeDisabled();
    await expect(
      page.locator('[data-testid="desktop-service-telegram-token"]'),
    ).toBeVisible();

    // OpenAI-compatible server is running: the stop button is enabled, the start
    // button is disabled, and the live URL is linked.
    const server = page.locator('[data-testid="desktop-service-server"]');
    await expect(server).toHaveAttribute('data-state', 'running');
    await expect(
      page.locator('[data-testid="desktop-service-stop-server"]'),
    ).toBeEnabled();
    await expect(
      page.locator('[data-testid="desktop-service-start-server"]'),
    ).toBeDisabled();
    await expect(
      page.locator('[data-testid="desktop-service-url-server"]'),
    ).toHaveText('127.0.0.1:8080/v1');

    // One click forwards the request (with the typed token) through the bridge.
    await page
      .locator('[data-testid="desktop-service-telegram-token"]')
      .fill('123:abc');
    await page.locator('[data-testid="desktop-service-start-telegram"]').click();
    await expect
      .poll(() => page.evaluate(() => window.__serviceCalls))
      .toContainEqual({
        action: 'start',
        request: { service: 'telegram', token: '123:abc' },
      });

    await page.locator('[data-testid="desktop-service-stop-server"]').click();
    await expect
      .poll(() => page.evaluate(() => window.__serviceCalls))
      .toContainEqual({ action: 'stop', request: { service: 'server' } });
  });

  test('services panel survives supported UI language choices', async ({ page }) => {
    for (const { language, name } of supportedUiLanguages) {
      await page.evaluate((nextLanguage) => {
        window.localStorage.setItem(
          'formal-ai.preferences.v1',
          `demo_preferences\n  demoMode "off"\n  greetingVariations "off"\n  uiLanguage "${nextLanguage}"`,
        );
      }, language);
      await page.reload();
      await expect(page.locator('html'), `${name} UI language is active`).toHaveAttribute(
        'lang',
        language,
      );
      await expect(
        page.locator('[data-testid="sidebar-services"]'),
        `Services panel renders for ${name}`,
      ).toBeVisible();
      await expect(
        page.locator('[data-testid="desktop-service-start-telegram"]'),
        `Telegram start control renders for ${name}`,
      ).toBeVisible();
      await expect(
        page.locator('[data-testid="desktop-service-stop-server"]'),
        `Server stop control renders for ${name}`,
      ).toBeVisible();
    }
  });
});
