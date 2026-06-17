// @ts-check
const { test, expect } = require('@playwright/test');

const supportedUiLanguages = [
  { language: 'en', name: 'English' },
  { language: 'ru', name: 'Russian' },
  { language: 'hi', name: 'Hindi' },
  { language: 'zh', name: 'Chinese' },
];

test.describe('Issue #280: desktop shell bridge', () => {
  test.beforeEach(async ({ page }) => {
    await page.addInitScript(() => {
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

      window.FormalAiDesktop = {
        getStatus: async () => ({
          shell: 'Electron',
          apiBase: 'http://127.0.0.1:18080',
          staticBase: 'http://127.0.0.1:18081',
          graphUrl: 'http://127.0.0.1:18080/v1/graph',
          traceUrl: 'http://127.0.0.1:18080/v1/graph?trace=answer_greeting_hi',
          memory: 'formal_ai_bundle',
          agentModeDefault: false,
          toolCallPolicy: 'explicit-permission',
          apiReady: true,
        }),
      };
    });
    await page.goto('./');
    await expect(page.locator('.app')).toBeVisible({ timeout: 15_000 });
  });

  test('desktop status, network, memory, and permission surfaces are visible', async ({ page }) => {
    await expect(page.locator('[data-testid="desktop-shell-status"]')).toContainText(
      'Desktop - API local - agent permission off',
    );
    await expect(page.locator('[data-testid="sidebar-desktop"]')).toBeVisible();
    await expect(page.locator('[data-testid="desktop-api-base"]')).toHaveText(
      '127.0.0.1:18080',
    );
    await expect(page.locator('[data-testid="desktop-network-link"]')).toHaveAttribute(
      'href',
      'http://127.0.0.1:18080/v1/graph',
    );
    await expect(page.locator('[data-testid="desktop-memory-bundle"]')).toHaveText(
      'formal_ai_bundle',
    );
    await expect(page.locator('[data-testid="desktop-agent-permission"]')).toHaveText(
      'Off',
    );
    await expect(page.locator('[data-testid="desktop-tool-permission"]')).toHaveText(
      'Permission gated',
    );

    await page.locator('[data-testid="mode-option-agent"]').click();
    await expect(page.locator('[data-testid="desktop-agent-permission"]')).toHaveText(
      'Opted in',
    );
    await expect(page.locator('[data-testid="desktop-tool-permission"]')).toHaveText(
      'Agent tools visible',
    );
  });

  test('desktop permission panel survives supported UI language choices', async ({ page }) => {
    for (const { language, name } of supportedUiLanguages) {
      await page.evaluate(
        ({ language: nextLanguage }) => {
          window.localStorage.setItem(
            'formal-ai.preferences.v1',
            `demo_preferences\n  demoMode "off"\n  greetingVariations "off"\n  uiLanguage "${nextLanguage}"`,
          );
        },
        { language, name },
      );
      await page.reload();
      await expect(page.locator('html'), `${name} UI language is active`).toHaveAttribute(
        'lang',
        language,
      );
      await expect(page.locator('[data-testid="desktop-shell-status"]')).toContainText(
        'Desktop - API local - agent permission off',
      );
      await expect(page.locator('[data-testid="desktop-tool-permission"]')).toHaveText(
        'Permission gated',
      );
    }
  });
});
