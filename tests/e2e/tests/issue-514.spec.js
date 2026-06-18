// @ts-check
const { test, expect } = require('@playwright/test');

const PREF_KEY = 'formal-ai.preferences.v1';
const supportedUiLanguages = [
  { language: 'en', name: 'English' },
  { language: 'ru', name: 'Russian' },
  { language: 'hi', name: 'Hindi' },
  { language: 'zh', name: 'Chinese' },
];

function preferencesForUiLanguage(language) {
  return [
    'demo_preferences',
    '  demoMode "off"',
    '  greetingVariations "off"',
    `  uiLanguage "${language}"`,
  ].join('\n');
}

async function sendPrompt(page, text) {
  const input = page.locator('[data-testid="chat-composer-input"]');
  const messages = page.locator('[data-testid="chat-message"]');
  const initial = await messages.count();
  await expect(input).toBeEnabled({ timeout: 5_000 });
  await input.fill(text);
  await page.locator('[data-testid="chat-composer-submit"]').click();
  await expect.poll(async () => messages.count(), { timeout: 20_000 }).toBeGreaterThan(initial);
}

async function bootIssue514(page) {
  await page.addInitScript((prefKey) => {
    try {
      if (!window.localStorage.getItem(prefKey)) {
        window.localStorage.setItem(
          prefKey,
          'demo_preferences\n  demoMode "off"\n  greetingVariations "off"',
        );
      }
    } catch (_error) {
      // localStorage can be unavailable in hardened browser contexts.
    }

    window.__grantHistory = [];
    window.__toolGrants = {};
    window.__toolInvocations = [];
    window.FormalAiDesktop = {
      getStatus: async () => ({
        shell: 'Electron',
        apiBase: '',
        staticBase: '',
        graphUrl: '',
        traceUrl: '',
        memory: 'formal_ai_bundle',
        agentModeDefault: false,
        toolCallPolicy: 'explicit-permission',
        apiReady: false,
      }),
      setToolGrants: async (grants) => {
        window.__toolGrants = { ...(grants || {}) };
        window.__grantHistory.push({ ...window.__toolGrants });
        return window.__toolGrants;
      },
      invokeTool: async (request) => {
        window.__toolInvocations.push(request);
        const grants = window.__toolGrants || {};
        const allowed = grants.all === true || grants[request.tool] === true;
        if (!allowed) {
          return {
            ok: false,
            tool: request.tool,
            status: 'refused',
            executed: false,
            reason: 'tool call denied by explicit-permission policy',
          };
        }
        return {
          ok: true,
          tool: request.tool,
          status: 'ok',
          executed: true,
          servedBy: 'test-bridge',
          body: `ran ${request.input.command || request.input.url || ''}`.trim(),
        };
      },
    };
  }, PREF_KEY);
  await page.goto('./');
  await expect(page.locator('.app')).toBeVisible({ timeout: 15_000 });
  await expect(page.locator('[data-testid="chat-composer-input"]')).toBeEnabled({
    timeout: 10_000,
  });
}

test.describe('Issue #514: per-tool permissions and command approval', () => {
  test.beforeEach(async ({ page }) => {
    await bootIssue514(page);
  });

  test('onboarding appears once and per-tool grants persist', async ({ page }) => {
    await expect.poll(() => page.evaluate(() => window.__toolGrants.all)).toBe(false);
    await expect(page.locator('[data-testid="desktop-tool-permission"]')).toHaveText(
      '0/6 tools granted',
    );

    await page.locator('[data-testid="mode-option-agent"]').click();
    await expect(page.locator('[data-testid="desktop-permission-panel-message"]')).toBeVisible();
    await expect(page.locator('[data-testid="chat-message"].system')).toContainText(
      'Grant or decline each tool separately',
    );

    await page.locator('[data-testid="desktop-permission-panel-sidebar-grant-shell"]').click();
    await expect(
      page.locator('[data-testid="desktop-permission-panel-sidebar-state-shell"]'),
    ).toHaveText('Granted');
    await expect.poll(() => page.evaluate(() => window.__toolGrants.shell)).toBe(true);
    await expect(page.locator('[data-testid="desktop-tool-permission"]')).toHaveText(
      '1/6 tools granted',
    );

    await page.locator('[data-testid="desktop-permission-panel-sidebar-decline-http_fetch"]').click();
    await expect(
      page.locator('[data-testid="desktop-permission-panel-sidebar-state-http_fetch"]'),
    ).toHaveText('Declined');
    await expect.poll(() => page.evaluate(() => window.__toolGrants.http_fetch)).toBe(false);

    await page.reload();
    await expect(page.locator('.app')).toBeVisible({ timeout: 15_000 });
    await expect(
      page.locator('[data-testid="desktop-permission-panel-sidebar-state-shell"]'),
    ).toHaveText('Granted');
    await expect.poll(() => page.evaluate(() => window.__toolGrants.shell)).toBe(true);
    await expect.poll(() =>
      page.evaluate((prefKey) => window.localStorage.getItem(prefKey), PREF_KEY),
    ).toContain('agentOnboardingSeen "on"');
    await expect.poll(() =>
      page.evaluate((prefKey) => window.localStorage.getItem(prefKey), PREF_KEY),
    ).toContain('desktopToolGrants "http_fetch:off,shell:on"');
  });

  test('permission panel works across supported UI language choices', async ({ page }) => {
    for (const { language, name } of supportedUiLanguages) {
      await page.evaluate(
        ({ prefKey, preferences }) => {
          window.localStorage.setItem(prefKey, preferences);
        },
        {
          prefKey: PREF_KEY,
          preferences: preferencesForUiLanguage(language),
          language,
          name,
        },
      );
      await page.reload();

      await expect(page.locator('.app'), `${name} UI loaded`).toBeVisible({
        timeout: 15_000,
      });
      await expect(page.locator('html'), `${name} UI language is active`).toHaveAttribute(
        'lang',
        language,
      );
      await expect(page.locator('[data-testid="desktop-permission-panel-sidebar"]')).toBeVisible();
      await expect(page.locator('[data-testid="desktop-tool-permission"]')).toHaveText(
        '0/6 tools granted',
      );

      await page.locator('[data-testid="desktop-permission-panel-sidebar-grant-shell"]').click();
      await expect(
        page.locator('[data-testid="desktop-permission-panel-sidebar-state-shell"]'),
      ).toHaveText('Granted');
      await expect(page.locator('[data-testid="desktop-tool-permission"]')).toHaveText(
        '1/6 tools granted',
      );
    }
  });

  test('Agent mode asks before each shell command', async ({ page }) => {
    await page.locator('[data-testid="mode-option-agent"]').click();
    await page.locator('[data-testid="desktop-permission-panel-sidebar-grant-shell"]').click();
    await expect.poll(() => page.evaluate(() => window.__toolGrants.shell)).toBe(true);

    await sendPrompt(page, 'run `ls ~` in terminal');
    await expect(page.locator('[data-testid="command-approval"]')).toBeVisible();
    await expect.poll(() => page.evaluate(() => window.__toolInvocations.length)).toBe(0);

    await page.locator('[data-testid="command-deny"]').last().click();
    await expect(page.locator('[data-testid="chat-message"]').last()).toContainText(
      'Command declined',
    );
    await expect.poll(() => page.evaluate(() => window.__toolInvocations.length)).toBe(0);

    await sendPrompt(page, 'run `ls ~` in terminal');
    await page.locator('[data-testid="command-approve"]').last().click();
    await expect.poll(() => page.evaluate(() => window.__toolInvocations.length)).toBe(1);
    await expect(page.locator('[data-testid="chat-message"]').last()).toContainText('ran ls ~');
  });

  test('Full Auto skips command prompts but still gates ungranted tools', async ({ page }) => {
    await page.locator('[data-testid="mode-option-fullAuto"]').click();
    await page.locator('[data-testid="desktop-permission-panel-sidebar-grant-shell"]').click();
    await expect.poll(() => page.evaluate(() => window.__toolGrants.shell)).toBe(true);

    await sendPrompt(page, 'run `ls ~` in terminal');
    await expect(page.locator('[data-testid="command-approval"]')).toHaveCount(0);
    await expect.poll(() => page.evaluate(() => window.__toolInvocations.length)).toBe(1);
    await expect(page.locator('[data-testid="chat-message"]').last()).toContainText('ran ls ~');

    const fetchResult = await page.evaluate(() =>
      window.formalAiDesktopToolCall('http_fetch', { url: 'https://example.com' }),
    );
    expect(fetchResult.executed).toBe(false);
    expect(fetchResult.status).toBe('refused');
  });
});
