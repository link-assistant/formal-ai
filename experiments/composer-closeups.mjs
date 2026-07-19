import { createRequire } from 'node:module';

// Issue #557 (PR #643, comment 39): close-up composer inspection across every
// skin in the ENABLED (typed) state, where the send button lights up. The
// committed gallery only carries glass/disabled closeups; this fills the gap so
// each skin's send affordance, attach circle, and pill uniformity can be
// checked side by side. Output goes to a scratch dir (not committed) for
// inspection; anything that reveals a defect gets fixed in source.
const requireFromE2e = createRequire(
  new URL('../tests/e2e/package.json', import.meta.url),
);
const { chromium } = requireFromE2e('@playwright/test');

const BASE = process.env.BASE || 'http://localhost:3456/app/';
const OUT = process.env.OUT || 'experiments/composer-out';
const skins = ['flat', 'glass', 'mui-flat', 'material', 'contrast'];
const themes = ['light', 'dark'];

const prefsLines = (overrides) => {
  const base = {
    demoMode: 'off',
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
      viewport: { width: 1280, height: 860 },
      deviceScaleFactor: 2,
      colorScheme: theme,
    });
    const page = await ctx.newPage();
    await page.addInitScript((s) => {
      window.localStorage.setItem('formal-ai.preferences.v1', s);
    }, prefsLines({ uiSkin: skin, theme }));
    await page.goto(BASE, { waitUntil: 'networkidle' });
    await page.waitForSelector('.composer', { timeout: 15000 });
    // Type into the textarea to enable the send button.
    const ta = page.locator('.composer-input, textarea').first();
    await ta.click();
    await ta.fill('Hello world');
    await page.waitForTimeout(400);
    const composer = page.locator('form.composer').first();
    const box = await composer.boundingBox();
    if (box) {
      const pad = 24;
      await page.screenshot({
        path: `${OUT}/composer-${skin}-${theme}.png`,
        clip: {
          x: Math.max(0, box.x - pad),
          y: Math.max(0, box.y - pad),
          width: Math.min(1280, box.width + pad * 2),
          height: box.height + pad * 2,
        },
      });
      console.log('saved', `${OUT}/composer-${skin}-${theme}.png`);
    }
    await ctx.close();
  }
}

await browser.close();
