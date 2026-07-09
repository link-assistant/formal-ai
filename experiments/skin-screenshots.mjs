import { chromium } from '@playwright/test';

// Regenerate the skin gallery for issue #557. Demo mode is left ON so the chat
// area is populated with real message cards — the frosted-glass / Material tonal
// surfaces are only visible on actual conversation content, so an empty chat
// under-sells the polish. We wait for a couple of turns to render, then capture.
const BASE = process.env.BASE || 'http://localhost:3499/app/';
const OUT = process.env.OUT || 'docs/screenshots';
const skins = ['flat', 'glass', 'material', 'contrast'];
const themes = ['light', 'dark'];

const prefs = (skin, theme) =>
  [
    'demo_preferences',
    '  demoMode "on"',
    '  greetingVariations "off"',
    `  uiSkin "${skin}"`,
    `  theme "${theme}"`,
    '  glassOpacity "0.7"',
  ].join('\n');

const browser = await chromium.launch();
for (const view of [
  { name: 'desktop', width: 1280, height: 860 },
  { name: 'mobile', width: 390, height: 800 },
]) {
  for (const theme of themes) {
    for (const skin of skins) {
      const ctx = await browser.newContext({
        viewport: { width: view.width, height: view.height },
        deviceScaleFactor: 2,
        colorScheme: theme,
      });
      const page = await ctx.newPage();
      await page.addInitScript(
        (s) => {
          window.localStorage.setItem('formal-ai.preferences.v1', s);
        },
        prefs(skin, theme),
      );
      await page.goto(BASE, { waitUntil: 'networkidle' });
      await page.waitForSelector('.app', { timeout: 15000 });
      // Wait for the demo to render at least one full user+assistant exchange so
      // the frosted/tonal message cards are on screen.
      await page
        .waitForFunction(
          () => document.querySelectorAll('[data-testid="chat-message"]').length >= 2,
          { timeout: 20000 },
        )
        .catch(() => {});
      await page.waitForTimeout(1200);
      const file = `${OUT}/skin-${view.name}-${theme}-${skin}.png`;
      await page.screenshot({ path: file, fullPage: false });
      console.log('saved', file);
      await ctx.close();
    }
  }
}
await browser.close();
