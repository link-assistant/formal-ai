// Issue #550 — rigorous DOM/computed-style verification of the fixes.
//
//   cd tests/e2e && NODE_PATH="$(pwd)/node_modules" \
//     node ../../experiments/issue-550-verify.cjs
//
// Drives the running app on :3456 and prints the computed values the fixes
// depend on, so the e2e regression assertions are grounded in real numbers.
const { chromium } = require('@playwright/test');

const BASE = process.env.BASE_URL || 'http://localhost:3456/app/';

function prefs(theme) {
  return `try { window.localStorage.setItem('formal-ai.preferences.v1', [
    'demo_preferences',
    '  theme ${JSON.stringify(theme)}',
    '  demoMode "off"',
    '  diagnosticsMode "off"',
    '  greetingVariations "off"',
  ].join('\\n')); } catch (_e) {}`;
}

const MOCK_BRIDGE = `window.FormalAiDesktop = {
  serviceStatus: async () => ({ dockerAvailable: true, appVersion: '0.214.0', services: [
    { key: 'agent', label: 'Agent environment', running: false, state: 'absent' },
  ]}),
  startService: async () => ({ ok: true }), stopService: async () => ({ ok: true }),
};`;

async function sendHi(page) {
  const input = page.locator('[data-testid="chat-composer-input"]');
  await input.waitFor({ state: 'visible', timeout: 10000 });
  await input.fill('Hi');
  const messages = page.locator('[data-testid="chat-message"]');
  const before = await messages.count();
  await page.locator('[data-testid="chat-composer-submit"]').click();
  await messages.nth(before + 1).waitFor({ timeout: 15000 });
  await page.waitForTimeout(400);
}

(async () => {
  const browser = await chromium.launch();
  const ctx = await browser.newContext({ reducedMotion: 'reduce' });
  await ctx.addInitScript(prefs('light'));
  await ctx.addInitScript(MOCK_BRIDGE);
  const page = await ctx.newPage();
  await page.goto(BASE, { waitUntil: 'domcontentloaded' });
  await page.locator('.app').waitFor({ timeout: 15000 });
  await sendHi(page);

  const preview = page.locator('[data-testid="thinking-preview"]').last();
  const out = await preview.evaluate((root) => {
    const cs = (el, p) => el ? getComputedStyle(el)[p] : null;
    const history = root.querySelector('.thinking-preview-history');
    const steps = [...root.querySelectorAll('.thinking-preview-step')];
    const current = root.querySelector('.thinking-preview-current');
    return {
      historyExists: !!history,
      historyMask: cs(history, 'maskImage') || cs(history, 'webkitMaskImage'),
      historyMaxHeight: cs(history, 'maxHeight'),
      historyOverflow: cs(history, 'overflow'),
      stepCount: steps.length,
      // Bug 2: steps must NOT be single-line nowrap/ellipsis.
      stepWhiteSpace: steps.map((s) => cs(s, 'whiteSpace')),
      stepTextOverflow: steps.map((s) => cs(s, 'textOverflow')),
      // Bug 1: each step must NOT carry its own gradient mask.
      stepMask: steps.map((s) => cs(s, 'maskImage') || cs(s, 'webkitMaskImage')),
      stepTexts: steps.map((s) => s.textContent.trim().slice(0, 60)),
      currentText: current ? current.textContent.trim() : null,
      currentMask: cs(current, 'maskImage') || cs(current, 'webkitMaskImage'),
    };
  });
  console.log(JSON.stringify(out, null, 2));

  // Bug 3: pending body width must match a revealed message body (no 116px clamp).
  const revealedBodyWidth = await page
    .locator('[data-testid="chat-message"].assistant .message-body')
    .last()
    .evaluate((el) => el.getBoundingClientRect().width);
  console.log('revealed .message-body width:', Math.round(revealedBodyWidth));

  await browser.close();
})().catch((e) => { console.error(e); process.exit(1); });
