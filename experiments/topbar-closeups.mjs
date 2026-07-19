import { createRequire } from 'node:module';

// Issue #557 (comment 39): close-up topbar inspection across skins. The topbar
// carries toggle buttons (mode, diagnostics) whose active state is an accent
// fill; MUI skins render them as IconButtons, so the same emotion-specificity
// trap that hit the composer could strip their active fill. Scratch output.
const requireFromE2e = createRequire(
  new URL('../tests/e2e/package.json', import.meta.url),
);
const { chromium } = requireFromE2e('@playwright/test');

const BASE = process.env.BASE || 'http://localhost:3456/app/';
const OUT = process.env.OUT || 'experiments/composer-out';
const skins = ['flat', 'glass', 'mui-flat', 'material', 'contrast'];
const themes = ['light', 'dark'];

const prefsLines = (o) =>
  [
    'demo_preferences',
    ...Object.entries({
      demoMode: 'off',
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
      viewport: { width: 1280, height: 860 },
      deviceScaleFactor: 2,
      colorScheme: theme,
    });
    const page = await ctx.newPage();
    await page.addInitScript((s) => {
      window.localStorage.setItem('formal-ai.preferences.v1', s);
    }, prefsLines({ uiSkin: skin, theme }));
    await page.goto(BASE, { waitUntil: 'networkidle' });
    await page.waitForSelector('.topbar', { timeout: 15000 });
    await page.waitForTimeout(300);
    const bar = page.locator('.topbar').first();
    const box = await bar.boundingBox();
    if (box) {
      await page.screenshot({
        path: `${OUT}/topbar-${skin}-${theme}.png`,
        clip: {
          x: box.x,
          y: box.y,
          width: box.width,
          height: box.height + 4,
        },
      });
      console.log('saved', `${OUT}/topbar-${skin}-${theme}.png`);
    }
    await ctx.close();
  }
}
await browser.close();
