// Issue #550 — probe topbar buttons (Bug 5 hover) + dark services (Bug 4).
const { chromium } = require('@playwright/test');
const BASE = process.env.BASE_URL || 'http://localhost:3456/app/';

function prefs(theme) {
  return `try { window.localStorage.setItem('formal-ai.preferences.v1', [
    'demo_preferences','  theme ${JSON.stringify(theme)}','  demoMode "off"',
    '  diagnosticsMode "off"','  greetingVariations "off"',
  ].join('\\n')); } catch (_e) {}`;
}
const MOCK_BRIDGE = `window.FormalAiDesktop = {
  serviceStatus: async () => ({ dockerAvailable: true, appVersion: '0.214.0', services: [
    { key: 'agent', label: 'Agent environment', running: false, state: 'absent' }]}),
  startService: async () => ({ ok: true }), stopService: async () => ({ ok: true }) };`;

async function boot(theme, bridge) {
  const browser = await chromium.launch();
  const ctx = await browser.newContext({ reducedMotion: 'reduce' });
  await ctx.addInitScript(prefs(theme));
  if (bridge) await ctx.addInitScript(MOCK_BRIDGE);
  const page = await ctx.newPage();
  await page.goto(BASE, { waitUntil: 'domcontentloaded' });
  await page.locator('.app').waitFor({ timeout: 15000 });
  await page.waitForTimeout(500);
  return { browser, page };
}

(async () => {
  // Bug 5: enumerate topbar buttons and their default background.
  let { browser, page } = await boot('light', false);
  const buttons = await page.locator('.topbar button').evaluateAll((els) =>
    els.map((el) => ({
      cls: el.className,
      testid: el.getAttribute('data-testid'),
      bg: getComputedStyle(el).backgroundColor,
    })),
  );
  console.log('=== topbar buttons (light) ===');
  for (const b of buttons) console.log(JSON.stringify(b));
  await browser.close();

  // Bug 4: dark services box background + start CTA color.
  ({ browser, page } = await boot('dark', true));
  await page.locator('[data-testid="desktop-services-panel"]').waitFor({ timeout: 8000 }).catch(() => {});
  const svc = await page.evaluate(() => {
    const box = document.querySelector('.desktop-service');
    const start = document.querySelector('.desktop-service-start');
    const cs = (el, p) => el ? getComputedStyle(el)[p] : null;
    return {
      serviceBg: cs(box, 'backgroundColor'),
      serviceBorder: cs(box, 'borderTopColor'),
      startBg: cs(start, 'backgroundColor'),
      startColor: cs(start, 'color'),
    };
  });
  console.log('=== dark services ===');
  console.log(JSON.stringify(svc, null, 2));
  await browser.close();
})().catch((e) => { console.error(e); process.exit(1); });
