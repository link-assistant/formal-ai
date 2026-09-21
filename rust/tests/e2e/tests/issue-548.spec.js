// @ts-check
const { test, expect } = require('@playwright/test');

const supportedUiLanguages = [
  { language: 'en', name: 'English' },
  { language: 'ru', name: 'Russian' },
  { language: 'hi', name: 'Hindi' },
  { language: 'zh', name: 'Chinese' },
];

function preserveUiLanguage() {
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

function installDesktopBridge(initialUpdater) {
  window.__updateCalls = [];
  window.__updateListeners = [];
  let updater = {
    supported: true,
    enabled: true,
    platform: 'darwin',
    currentVersion: '0.212.0',
    state: 'idle',
    updateAvailable: false,
    downloaded: false,
    latestVersion: '',
    progressPercent: 0,
    checkedAt: '',
    error: '',
    message: '',
    ...initialUpdater,
  };

  window.FormalAiDesktop = {
    getStatus: async () => ({
      shell: 'Electron',
      appVersion: '0.212.0',
      apiBase: 'http://127.0.0.1:18080',
      staticBase: 'http://127.0.0.1:18081',
      graphUrl: 'http://127.0.0.1:18080/v1/graph',
      memory: 'formal_ai_bundle',
      agentModeDefault: false,
      toolCallPolicy: 'explicit-permission',
      apiReady: true,
      updater,
    }),
    checkForUpdates: async () => {
      window.__updateCalls.push('check');
      return updater;
    },
    installUpdate: async () => {
      window.__updateCalls.push('install');
      updater = {
        ...updater,
        state: 'installing',
        updateAvailable: true,
        downloaded: true,
      };
      return updater;
    },
    onUpdateStatus: (callback) => {
      window.__updateListeners.push(callback);
      return () => {
        window.__updateListeners = window.__updateListeners.filter((item) => item !== callback);
      };
    },
  };
}

test.describe('Issue #548: desktop updates and version display', () => {
  test.beforeEach(async ({ page }) => {
    await page.addInitScript(preserveUiLanguage);
  });

  test('uses the Electron app version instead of the unstamped web fallback', async ({ page }) => {
    await page.addInitScript(installDesktopBridge, {
      state: 'available',
      updateAvailable: true,
      latestVersion: '0.213.0',
    });
    await page.goto('./');
    await expect(page.locator('.app')).toBeVisible({ timeout: 15_000 });

    await expect(page.locator('[data-testid="app-version"]')).toHaveText('v0.212.0');
    await expect(page.locator('[data-testid="desktop-app-version"]')).toHaveText('v0.212.0');
    await expect(page.locator('[data-testid="desktop-update-state"]')).toContainText(
      '0.213.0',
    );

    await page.locator('[data-testid="desktop-update-install"]').click();
    await expect.poll(() => page.evaluate(() => window.__updateCalls)).toContainEqual('install');
  });

  test('renders an in-app update notification from the desktop event stream', async ({ page }) => {
    await page.addInitScript(installDesktopBridge, {
      state: 'idle',
      updateAvailable: false,
      latestVersion: '',
    });
    await page.goto('./');
    await expect(page.locator('.app')).toBeVisible({ timeout: 15_000 });

    await page.evaluate(() => {
      for (const callback of window.__updateListeners) {
        callback({
          supported: true,
          enabled: true,
          platform: 'darwin',
          currentVersion: '0.212.0',
          state: 'available',
          updateAvailable: true,
          downloaded: false,
          latestVersion: '0.213.0',
          progressPercent: 0,
          checkedAt: '2026-06-20T00:00:00.000Z',
          error: '',
          message: '',
        });
      }
    });

    await expect(page.locator('[data-testid="desktop-update-state"]')).toContainText(
      '0.213.0',
    );
    await expect(page.locator('[data-testid="desktop-update-install"]')).toBeEnabled();
  });

  test('update panel survives supported UI language choices', async ({ page }) => {
    await page.addInitScript(installDesktopBridge, {
      state: 'available',
      updateAvailable: true,
      latestVersion: '0.213.0',
    });
    await page.goto('./');
    await expect(page.locator('.app')).toBeVisible({ timeout: 15_000 });

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
      const expectedState = await page.evaluate(async (nextLanguage) => {
        await window.FormalAiI18n.ready;
        return window.FormalAiI18n.t('updates.state.available', nextLanguage, {
          version: '0.213.0',
        });
      }, language);
      await expect(
        page.locator('[data-testid="desktop-update-state"]'),
        `update notification is translated for ${name}`,
      ).toHaveText(expectedState);
      await expect(
        page.locator('[data-testid="desktop-update-install"]'),
        `update action renders for ${name}`,
      ).toBeEnabled();
    }
  });
});
