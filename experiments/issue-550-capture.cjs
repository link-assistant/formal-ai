// Issue #550 — UI/UX evidence capture.
//
// Reusable Playwright script that drives the web app at http://localhost:3456/app/
// and saves screenshots of every surface the issue calls out, in BOTH light and
// dark themes, so we can compare "before" vs "after" the fixes.
//
//   node experiments/issue-550-capture.js before
//   node experiments/issue-550-capture.js after
//
// Prereqs: `bun run build:web && ./scripts/sync-seed.sh` and a static server on
// :3456 (e.g. `npx serve src/web --listen 3456`). The Services panel only
// renders when the desktop bridge is present, so we inject a mock
// `window.FormalAiDesktop` via addInitScript to exercise its theming.
const path = require('path');
const fs = require('fs');
const { chromium } = require('@playwright/test');

const LABEL = process.argv[2] || 'before';
const BASE = process.env.BASE_URL || 'http://localhost:3456/app/';
const OUT = path.resolve(__dirname, '..', 'docs', 'case-studies', 'issue-550', 'screenshots', LABEL);

const MOCK_BRIDGE = `
  window.FormalAiDesktop = {
    serviceStatus: async () => ({
      dockerAvailable: true,
      appVersion: '0.214.0',
      services: [
        { key: 'agent', label: 'Agent environment', running: false, state: 'absent' },
        { key: 'telegram', label: 'Telegram bot', running: false, state: 'stopped' },
        { key: 'openai', label: 'OpenAI-compatible API', running: true, state: 'running', url: 'http://localhost:8080/v1' },
      ],
    }),
    startService: async () => ({ ok: true }),
    stopService: async () => ({ ok: true }),
  };
`;

function prefsScript(theme, demoMode) {
  return `
    try {
      window.localStorage.setItem('formal-ai.preferences.v1', [
        'demo_preferences',
        '  theme ${JSON.stringify(theme)}',
        '  demoMode ${JSON.stringify(demoMode)}',
        '  diagnosticsMode "off"',
        '  greetingVariations "off"',
      ].join('\\n'));
    } catch (_e) {}
  `;
}

async function shot(locator, file) {
  try {
    await locator.scrollIntoViewIfNeeded({ timeout: 3000 });
    await locator.screenshot({ path: path.join(OUT, file) });
    console.log('  saved', file);
  } catch (e) {
    console.log('  SKIP', file, '-', e.message.split('\n')[0]);
  }
}

async function sendPrompt(page, text) {
  const input = page.locator('[data-testid="chat-composer-input"]');
  await input.waitFor({ state: 'visible', timeout: 10000 });
  await input.fill(text);
  const messages = page.locator('[data-testid="chat-message"]');
  const before = await messages.count();
  await page.locator('[data-testid="chat-composer-submit"]').click();
  await messages.filter().nth(before + 1).waitFor({ timeout: 15000 }).catch(() => {});
  await page.waitForTimeout(600);
}

async function captureTheme(browser, theme) {
  const context = await browser.newContext({ viewport: { width: 1280, height: 900 }, deviceScaleFactor: 2 });
  await context.addInitScript(prefsScript(theme, 'off'));
  await context.addInitScript(MOCK_BRIDGE);
  const page = await context.newPage();
  await page.goto(BASE, { waitUntil: 'domcontentloaded' });
  await page.locator('.app').waitFor({ timeout: 15000 });
  await page.waitForTimeout(400);

  // Topbar (hover bug: capture a neutral state + a hovered button). Prefer a
  // control that was *missing* hover before the fix (memory/report had no hover
  // rule in either theme; source-code/download already had dark ones), so the
  // before/after diff is visible in both light and dark.
  await shot(page.locator('.topbar'), `topbar-${theme}.png`);
  const hoverTarget = page
    .locator('.memory-button, .report-button, .source-code-button, .download-button')
    .first();
  try {
    await hoverTarget.hover({ timeout: 2000 });
    await page.waitForTimeout(150);
    await shot(page.locator('.topbar'), `topbar-${theme}-hover.png`);
  } catch (_e) {}

  // Services panel (theming bug). Sidebar section renders once the mock bridge
  // reports a snapshot (polled on mount).
  await page.waitForTimeout(400);
  await shot(page.locator('[data-testid="desktop-services-panel"]'), `services-${theme}.png`);

  // Thinking preview (gradient + omitted-text bugs).
  await sendPrompt(page, 'Hi');
  const preview = page.locator('[data-testid="thinking-preview"]').first();
  await shot(preview, `thinking-collapsed-${theme}.png`);
  try {
    await preview.locator('[data-testid="thinking-preview-toggle"]').click({ timeout: 2000 });
    await page.waitForTimeout(200);
    await shot(preview, `thinking-expanded-${theme}.png`);
  } catch (_e) {}

  // Whole assistant message (width context).
  await shot(page.locator('[data-testid="chat-message"]').last(), `message-${theme}.png`);

  await context.close();
}

async function capturePending(browser, theme) {
  // Demo mode scripts realistic "thinking…" pending phases; catch the pending
  // bubble to show the width-jump bug (.pending .message-body { width: 116px }).
  const context = await browser.newContext({ viewport: { width: 1280, height: 900 }, deviceScaleFactor: 2 });
  await context.addInitScript(prefsScript(theme, 'on'));
  const page = await context.newPage();
  await page.goto(BASE, { waitUntil: 'domcontentloaded' });
  try {
    const pendingBubble = page.locator('.message.assistant.pending');
    await pendingBubble.waitFor({ timeout: 12000 });
    await page.waitForTimeout(150);
    await shot(pendingBubble, `pending-${theme}.png`);
    await shot(page.locator('.transcript, .messages, .chat-panel').first(), `pending-context-${theme}.png`);
  } catch (e) {
    console.log('  SKIP pending-' + theme, '-', e.message.split('\n')[0]);
  }
  await context.close();
}

(async () => {
  fs.mkdirSync(OUT, { recursive: true });
  const browser = await chromium.launch();
  for (const theme of ['light', 'dark']) {
    console.log('Capturing theme:', theme);
    await captureTheme(browser, theme);
  }
  console.log('Capturing pending bubble (demo mode)…');
  await capturePending(browser, 'dark');
  await capturePending(browser, 'light');
  await browser.close();
  console.log('Done →', OUT);
})().catch((e) => { console.error(e); process.exit(1); });
