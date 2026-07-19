import { createRequire } from 'node:module';

// Issue #557 (PR #643, comment 39): close-up inspection of the chat transcript
// (user + assistant message cards, and the first tool card if present) across
// every skin in light + dark. The maintainer asked to "get smaller screenshots
// of parts of the UI and inspect them much more closely" — the committed
// gallery carries full frames and per-component glass crops, but no tight
// side-by-side of the message cards themselves across ALL skins. This fills
// that gap so per-skin card background/border/uniformity defects surface.
// Output goes to a scratch dir (gitignored) for inspection.
const requireFromE2e = createRequire(
  new URL('../tests/e2e/package.json', import.meta.url),
);
const { chromium } = requireFromE2e('@playwright/test');

const BASE = process.env.BASE || 'http://localhost:3456/app/';
const OUT = process.env.OUT || 'experiments/message-out';
const skins = ['flat', 'glass', 'mui-flat', 'material', 'contrast'];
const themes = ['light', 'dark'];

const prefsLines = (overrides) => {
  const base = {
    demoMode: 'on',
    greetingVariations: 'off',
    uiSkin: 'flat',
    colorTheme: 'emerald',
    theme: 'light',
    glassOpacity: '0.72',
    glassBlur: '18',
    glassRefraction: '60',
    glassMode: 'balanced',
    sidebarSettingsCollapsed: 'on',
    sidebarPromptsCollapsed: 'on',
  };
  const merged = { ...base, ...overrides };
  return [
    'demo_preferences',
    ...Object.entries(merged).map(([k, v]) => `  ${k} "${v}"`),
  ].join('\n');
};

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
    await page.addInitScript((s) => {
      window.localStorage.setItem('formal-ai.preferences.v1', s);
    }, prefsLines({ uiSkin: skin, theme }));
    await page.goto(BASE, { waitUntil: 'networkidle' });
    await page.waitForSelector('.app', { timeout: 15000 });
    // Demo mode auto-populates a deterministic transcript with real message cards.
    await page
      .waitForFunction(
        () => document.querySelectorAll('[data-testid="chat-message"]').length >= 2,
        { timeout: 15000 },
      )
      .catch(() => {});
    await page.waitForTimeout(1000);

    const panel = page.locator('.chat-scroll, .chat-transcript, .messages, .chat-panel').first();
    const target = (await panel.count()) > 0 ? panel : page.locator('.app');
    const box = await target.boundingBox();
    if (box) {
      const pad = 8;
      await page.screenshot({
        path: `${OUT}/messages-${skin}-${theme}.png`,
        clip: {
          x: Math.max(0, box.x - pad),
          y: Math.max(0, box.y - pad),
          width: Math.min(1280, box.width + pad * 2),
          height: Math.min(880, box.height + pad * 2),
        },
      });
      console.log('saved', `${OUT}/messages-${skin}-${theme}.png`);
    }
    await ctx.close();
  }
}

await browser.close();
