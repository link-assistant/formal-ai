import { createRequire } from 'node:module';

// Issue #557 (PR #643, comment 39): close-up inspection of the command-approval
// tool card (the only card surface in the app besides message cards, and the
// only one gated behind Agent mode + the Electron desktop bridge). The maintainer
// asked to inspect UI parts closely for per-skin surface/background defects; the
// message cards and prompt list already have close-ups, this adds the tool card.
// Scratch output, gitignored (experiments/message-out/).
const requireFromE2e = createRequire(
  new URL('../tests/e2e/package.json', import.meta.url),
);
const { chromium } = requireFromE2e('@playwright/test');

const BASE = process.env.BASE || 'http://localhost:3456/app/';
const OUT = process.env.OUT || 'experiments/message-out';
const skins = ['flat', 'glass', 'mui-flat', 'material', 'contrast'];
const themes = ['light', 'dark'];
const PREF_KEY = 'formal-ai.preferences.v1';

const prefsLines = (o) =>
  [
    'demo_preferences',
    ...Object.entries({
      demoMode: 'off',
      greetingVariations: 'off',
      uiSkin: 'flat',
      colorTheme: 'emerald',
      theme: 'light',
      sidebarSettingsCollapsed: 'on',
      sidebarPromptsCollapsed: 'on',
      ...o,
    }).map(([k, v]) => `  ${k} "${v}"`),
  ].join('\n');

const browser = await chromium.launch();
for (const theme of themes) {
  for (const skin of skins) {
    const ctx = await browser.newContext({
      viewport: { width: 1280, height: 900 },
      deviceScaleFactor: 2,
      colorScheme: theme,
      reducedMotion: 'reduce',
    });
    const page = await ctx.newPage();
    await page.addInitScript(
      ({ prefKey, prefs }) => {
        window.localStorage.setItem(prefKey, prefs);
        window.__toolGrants = {};
        window.FormalAiDesktop = {
          getStatus: async () => ({
            shell: 'Electron',
            apiBase: '',
            staticBase: '',
            memory: 'formal_ai_bundle',
            agentModeDefault: false,
            toolCallPolicy: 'explicit-permission',
            apiReady: false,
          }),
          setToolGrants: async (grants) => {
            window.__toolGrants = { ...(grants || {}) };
            return window.__toolGrants;
          },
          invokeTool: async (request) => ({
            ok: false,
            tool: request.tool,
            status: 'refused',
            executed: false,
            reason: 'closeup: leave the approval pending',
          }),
        };
      },
      { prefKey: PREF_KEY, prefs: prefsLines({ uiSkin: skin, theme }) },
    );
    await page.goto(BASE, { waitUntil: 'networkidle' });
    await page.waitForSelector('.app', { timeout: 15000 });
    await page.locator('[data-testid="mode-option-agent"]').click();
    await page
      .locator('[data-testid="desktop-permission-panel-sidebar-grant-shell"]')
      .click()
      .catch(() => {});
    const input = page.locator('[data-testid="chat-composer-input"]');
    await input.waitFor({ timeout: 10000 });
    await input.fill('run `ls ~` in terminal');
    await page.locator('[data-testid="chat-composer-submit"]').click();
    const panel = page.locator('[data-testid="command-approval"]').first();
    await panel.waitFor({ timeout: 10000 }).catch(() => {});
    await page.waitForTimeout(500);
    const box = await panel.boundingBox().catch(() => null);
    if (box) {
      const pad = 12;
      await page.screenshot({
        path: `${OUT}/toolcard-${skin}-${theme}.png`,
        clip: {
          x: Math.max(0, box.x - pad),
          y: Math.max(0, box.y - pad),
          width: Math.min(1280, box.width + pad * 2),
          height: Math.min(900 - Math.max(0, box.y - pad), box.height + pad * 2),
        },
      });
      console.log('saved', `${OUT}/toolcard-${skin}-${theme}.png`);
    } else {
      console.log('NO PANEL', skin, theme);
    }
    await ctx.close();
  }
}
await browser.close();
