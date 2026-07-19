import { createRequire } from 'node:module';

// Issue #557 (comment 39): close-up inspection of the settings / configurability
// panel — the maintainer's stated especial interest ("glass and other
// skins/themes configurability and quality/polishing"). Expands the sidebar
// Settings section and screenshots it per skin (glass expands its opacity/blur/
// refraction/mode sliders). Scratch output, not committed.
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
      glassOpacity: '0.72',
      glassBlur: '18',
      glassRefraction: '60',
      glassMode: 'balanced',
      sidebarSettingsCollapsed: 'off',
      sidebarPromptsCollapsed: 'on',
      ...o,
    }).map(([k, v]) => `  ${k} "${v}"`),
  ].join('\n');

const browser = await chromium.launch();
for (const theme of themes) {
  for (const skin of skins) {
    const ctx = await browser.newContext({
      viewport: { width: 1280, height: 1200 },
      deviceScaleFactor: 2,
      colorScheme: theme,
    });
    const page = await ctx.newPage();
    await page.addInitScript((s) => {
      window.localStorage.setItem('formal-ai.preferences.v1', s);
    }, prefsLines({ uiSkin: skin, theme }));
    await page.goto(BASE, { waitUntil: 'networkidle' });
    const panel = page.locator('.settings-panel').first();
    await panel.waitFor({ timeout: 15000 });
    await page.waitForTimeout(300);
    const box = await panel.boundingBox();
    if (box) {
      await page.screenshot({
        path: `${OUT}/settings-${skin}-${theme}.png`,
        clip: {
          x: Math.max(0, box.x - 8),
          y: Math.max(0, box.y - 8),
          width: Math.min(1280, box.width + 16),
          height: Math.min(1200 - box.y, box.height + 16),
        },
      });
      console.log('saved', `${OUT}/settings-${skin}-${theme}.png`);
    }
    await ctx.close();
  }
}
await browser.close();
