// @ts-check
const { test, expect } = require('@playwright/test');

// Issue #353: the same committed web chat runs inside a VS Code Webview through
// the shared `window.FormalAiDesktop` bridge. The extension reports its host with
// a `shell` of "VS Code" (Node desktop/remote host) or "VS Code Web" (the Web
// Worker host on vscode.dev / github.dev). The web app must label either surface
// "VS Code", route to the local server only when it is genuinely ready, and keep
// the permission / network / memory surfaces the desktop shell exposes.
//
// These tests mirror issue-280.spec.js (the Electron desktop bridge) but assert
// the VS Code labelling for both hosts: the Node host with a ready local server,
// and the in-process web host with no server at all.

const supportedUiLanguages = [
  { language: 'en', name: 'English' },
  { language: 'ru', name: 'Russian' },
  { language: 'hi', name: 'Hindi' },
  { language: 'zh', name: 'Chinese' },
];

// Pin demo mode off and keep whatever UI language is already selected, so the
// permission panel renders deterministically (same approach as issue-280).
function pinPreferences(page) {
  return page.addInitScript(() => {
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
  });
}

// Install a fake VS Code bridge that returns the given status, then load the app.
async function bootWithStatus(page, status) {
  await pinPreferences(page);
  await page.addInitScript((injected) => {
    window.FormalAiDesktop = {
      getStatus: async () => injected,
    };
  }, status);
  await page.goto('./');
  await expect(page.locator('.app')).toBeVisible({ timeout: 15_000 });
}

test.describe('Issue #353: VS Code extension bridge', () => {
  test('Node host with a ready local server reads as VS Code', async ({ page }) => {
    await bootWithStatus(page, {
      shell: 'VS Code',
      apiBase: 'http://127.0.0.1:18080',
      staticBase: '',
      graphUrl: 'http://127.0.0.1:18080/v1/graph',
      traceUrl: 'http://127.0.0.1:18080/v1/graph?trace=answer_greeting_hi',
      memory: 'formal_ai_bundle',
      agentModeDefault: false,
      toolCallPolicy: 'explicit-permission',
      apiReady: true,
    });

    await expect(page.locator('[data-testid="desktop-shell-status"]')).toContainText(
      'VS Code - API local - agent permission off',
    );
    await expect(page.locator('[data-testid="sidebar-desktop"]')).toBeVisible();
    // The panel reports the raw shell string from the extension host.
    await expect(page.locator('[data-testid="desktop-shell-panel"]')).toContainText('VS Code');
    await expect(page.locator('[data-testid="desktop-api-base"]')).toHaveText('127.0.0.1:18080');
    await expect(page.locator('[data-testid="desktop-network-link"]')).toHaveAttribute(
      'href',
      'http://127.0.0.1:18080/v1/graph',
    );
    await expect(page.locator('[data-testid="desktop-memory-bundle"]')).toHaveText(
      'formal_ai_bundle',
    );
    await expect(page.locator('[data-testid="desktop-agent-permission"]')).toHaveText('Off');
    await expect(page.locator('[data-testid="desktop-tool-permission"]')).toHaveText(
      'Permission gated',
    );

    // The agent toggle is the explicit opt-in for the permission-gated tool router.
    await page.locator('[data-testid="agent-toggle"]').click();
    await expect(page.locator('[data-testid="desktop-agent-permission"]')).toHaveText('Opted in');
    await expect(page.locator('[data-testid="desktop-tool-permission"]')).toHaveText(
      'Agent tools visible',
    );
  });

  test('Web host (vscode.dev) reads as VS Code and stays in-process', async ({ page }) => {
    // The Web Worker host cannot spawn a process, so it advertises no apiBase and
    // apiReady false: the app must fall back to the in-process symbolic engine
    // while still labelling the surface "VS Code".
    await bootWithStatus(page, {
      shell: 'VS Code Web',
      apiBase: '',
      graphUrl: '',
      traceUrl: '',
      memory: 'formal_ai_bundle',
      agentModeDefault: false,
      toolCallPolicy: 'explicit-permission',
      apiReady: false,
    });

    await expect(page.locator('[data-testid="desktop-shell-status"]')).toContainText(
      'VS Code - in-process - agent permission off',
    );
    await expect(page.locator('[data-testid="sidebar-desktop"]')).toBeVisible();
    // The panel still shows the precise host: "VS Code Web".
    await expect(page.locator('[data-testid="desktop-shell-panel"]')).toContainText('VS Code Web');
    await expect(page.locator('[data-testid="desktop-memory-bundle"]')).toHaveText(
      'formal_ai_bundle',
    );
    await expect(page.locator('[data-testid="desktop-tool-permission"]')).toHaveText(
      'Permission gated',
    );
  });

  test('VS Code permission panel survives supported UI language choices', async ({ page }) => {
    await bootWithStatus(page, {
      shell: 'VS Code',
      apiBase: 'http://127.0.0.1:18080',
      graphUrl: 'http://127.0.0.1:18080/v1/graph',
      traceUrl: 'http://127.0.0.1:18080/v1/graph?trace=answer_greeting_hi',
      memory: 'formal_ai_bundle',
      agentModeDefault: false,
      toolCallPolicy: 'explicit-permission',
      apiReady: true,
    });

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
        'VS Code - API local - agent permission off',
      );
      await expect(page.locator('[data-testid="desktop-tool-permission"]')).toHaveText(
        'Permission gated',
      );
    }
  });
});
