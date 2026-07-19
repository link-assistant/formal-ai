import { createRequire } from 'node:module';

// Issue #557 (PR #643, comment 39): close-up inspection of the "Example prompts"
// sidebar section across every skin in light + dark. The maintainer asked to
// inspect small parts of the UI closely; the composer, top bar, settings and
// message cards already have dedicated close-ups, but the prompt-list buttons
// (their surface/border/hover geometry per skin) had none. This fills the gap so
// per-skin button background/border/uniformity defects surface. Scratch output,
// gitignored (experiments/message-out/).
const requireFromE2e = createRequire(
  new URL('../tests/e2e/package.json', import.meta.url),
);
const { chromium } = requireFromE2e('@playwright/test');

const BASE = process.env.BASE || 'http://localhost:3456/app/';
const OUT = process.env.OUT || 'experiments/message-out';
const skins = ['flat', 'glass', 'mui-flat', 'material', 'contrast'];
const themes = ['light', 'dark'];

const prefsLines = (o) =>
  [
    'demo_preferences',
    ...Object.entries({
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
      sidebarPromptsCollapsed: 'off',
      ...o,
    }).map(([k, v]) => `  ${k} "${v}"`),
  ].join('\n');

const browser = await chromium.launch();
for (const theme of themes) {
  for (const skin of skins) {
    const ctx = await browser.newContext({
      viewport: { width: 1280, height: 1000 },
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
    const panel = page
      .locator('[data-testid="example-prompts"]')
      .locator('xpath=ancestor::*[contains(@class,"sidebar-section")][1]')
      .first();
    const target = (await panel.count()) > 0 ? panel : page.locator('[data-testid="example-prompts"]');
    await target.waitFor({ timeout: 15000 });
    await page.waitForTimeout(400);
    const box = await target.boundingBox();
    if (box) {
      const pad = 10;
      await page.screenshot({
        path: `${OUT}/prompts-${skin}-${theme}.png`,
        clip: {
          x: Math.max(0, box.x - pad),
          y: Math.max(0, box.y - pad),
          width: Math.min(1280, box.width + pad * 2),
          height: Math.min(1000 - Math.max(0, box.y - pad), box.height + pad * 2),
        },
      });
      console.log('saved', `${OUT}/prompts-${skin}-${theme}.png`);
    }
    await ctx.close();
  }
}
await browser.close();
