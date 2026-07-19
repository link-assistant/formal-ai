import { createRequire } from 'node:module';

// Playwright belongs to the dedicated E2E package, not the repository root.
// Resolve it from that package so this checked-in experiment works after the
// documented `npm ci --prefix tests/e2e` setup without a duplicate install.
const requireFromE2e = createRequire(
  new URL('../tests/e2e/package.json', import.meta.url),
);
const { chromium } = requireFromE2e('@playwright/test');

// Issue #557: regenerate the skin gallery + glass component closeups.
//
// Full-skin frames keep demo mode ON so the chat area is populated with real
// message cards — the frosted-glass / Material tonal surfaces only read well on
// actual conversation content, so an empty chat under-sells the polish. The
// per-component glass closeups seed one deterministic exchange, disable the
// demo, and crop to a single Chakra element so each component's liquid-glass
// treatment can be verified in isolation (as requested on PR #643).
const BASE = process.env.BASE || 'http://localhost:3456/app/';
const OUT = process.env.OUT || 'docs/case-studies/issue-557/screenshots';
const skins = ['flat', 'glass', 'mui-flat', 'material', 'contrast'];
const themes = ['light', 'dark'];
// Issue #557 (PR #643 follow-up): the selectable brand colour themes. Each one
// re-tints the accent tokens and ships a light + dark variant.
const colorThemes = [
  'emerald',
  'ocean',
  'indigo',
  'violet',
  'rose',
  'amber',
  'graphite',
];

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
    sidebarPromptsCollapsed: 'off',
  };
  const merged = { ...base, ...overrides };
  return [
    'demo_preferences',
    ...Object.entries(merged).map(([k, v]) => `  ${k} "${v}"`),
  ].join('\n');
};

const seedPrefs = (page, overrides) =>
  page.addInitScript((s) => {
    window.localStorage.setItem('formal-ai.preferences.v1', s);
  }, prefsLines(overrides));

const browser = await chromium.launch();

// 1) Full-skin gallery: every skin, light + dark, desktop + mobile.
for (const view of [
  { name: 'desktop', width: 1280, height: 860 },
  { name: 'tablet', width: 820, height: 1180 },
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
      await seedPrefs(page, { uiSkin: skin, theme });
      await page.goto(BASE, { waitUntil: 'networkidle' });
      await page.waitForSelector('.app', { timeout: 15000 });
      await page
        .waitForFunction(
          () =>
            document.querySelectorAll('[data-testid="chat-message"]').length >= 2,
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

// 2) Glass component closeups: seed one exchange, demo OFF, crop per component.
// Each entry is [filename, selector]. Missing selectors are skipped gracefully.
const glassComponents = [
  ['glass-component-composer.png', 'form.composer'],
  ['glass-component-message-assistant.png', '.message.assistant .message-body'],
  ['glass-component-message-user.png', '.message.user .message-body'],
  ['glass-component-topbar.png', '.topbar'],
  ['glass-component-sidebar-conversations.png', '.sidebar-section-body'],
  ['glass-component-settings.png', '.settings-panel'],
  ['glass-component-prompts.png', '.prompt-list'],
  ['glass-component-tool.png', '.tool'],
  ['glass-component-chat-panel.png', '.chat-panel'],
];

async function screenshotVisibleElement(page, locator, path) {
  await locator.scrollIntoViewIfNeeded();
  const box = await locator.boundingBox();
  if (!box) {
    throw new Error('component is not visible');
  }
  const viewport = page.viewportSize() || { width: 1280, height: 860 };
  const x = Math.max(0, Math.floor(box.x));
  const y = Math.max(0, Math.floor(box.y));
  const maxWidth = Math.max(1, viewport.width - x);
  const maxHeight = Math.max(1, viewport.height - y);
  await page.screenshot({
    path,
    clip: {
      x,
      y,
      width: Math.max(1, Math.ceil(Math.min(box.width, maxWidth))),
      height: Math.max(1, Math.ceil(Math.min(box.height, maxHeight, 720))),
    },
  });
}

for (const theme of themes) {
  const ctx = await browser.newContext({
    viewport: { width: 1280, height: 860 },
    deviceScaleFactor: 2,
    colorScheme: theme,
  });
  const page = await ctx.newPage();
  await seedPrefs(page, {
    uiSkin: 'glass',
    theme,
    demoMode: 'off',
    sidebarSettingsCollapsed: 'off',
  });
  await page.goto(BASE, { waitUntil: 'networkidle' });
  await page.waitForSelector('.app', { timeout: 15000 });
  // Seed one deterministic user+assistant exchange via the example prompt.
  await page.locator('button[data-prompt-text="Hi"]').first().click();
  await page.locator('button[data-testid="chat-composer-submit"]').click();
  await page
    .waitForFunction(
      () => document.querySelectorAll('[data-testid="chat-message"]').length >= 2,
      { timeout: 15000 },
    )
    .catch(() => {});
  await page.waitForTimeout(1000);

  for (const [file, selector] of glassComponents) {
    const el = page.locator(selector).first();
    if ((await el.count()) === 0) {
      console.log('skip (not found)', selector);
      continue;
    }
    try {
      await screenshotVisibleElement(page, el, `${OUT}/${theme}-${file}`);
      console.log('saved', `${OUT}/${theme}-${file}`);
    } catch (err) {
      console.log('skip (error)', selector, err.message);
    }
  }
  await ctx.close();
}

// 3) React Bits-informed glass appearance modes. The three modes deliberately
// share one accessible DOM and differ only in the resolved optical profile.
for (const theme of themes) {
  for (const glassMode of ['balanced', 'clear', 'frosted']) {
    const ctx = await browser.newContext({
      viewport: { width: 1280, height: 860 },
      deviceScaleFactor: 2,
      colorScheme: theme,
    });
    const page = await ctx.newPage();
    await seedPrefs(page, { uiSkin: 'glass', theme, glassMode });
    await page.goto(BASE, { waitUntil: 'networkidle' });
    await page.waitForSelector('.app', { timeout: 15000 });
    await page.waitForTimeout(1200);
    const file = `${OUT}/glass-mode-${theme}-${glassMode}.png`;
    await page.screenshot({ path: file, fullPage: false });
    console.log('saved', file);
    await ctx.close();
  }
}

// 4) Colour-theme gallery: every brand theme, light + dark, on the flat Chakra
// skin so the accent recolouring (links, the solid send button, hover/focus)
// reads clearly against the neutral surfaces. Demo mode stays ON so the
// populated chat shows the accent applied to real message chrome.
for (const theme of themes) {
  for (const colorTheme of colorThemes) {
    const ctx = await browser.newContext({
      viewport: { width: 1280, height: 860 },
      deviceScaleFactor: 2,
      colorScheme: theme,
    });
    const page = await ctx.newPage();
    await seedPrefs(page, { uiSkin: 'flat', theme, colorTheme });
    await page.goto(BASE, { waitUntil: 'networkidle' });
    await page.waitForSelector('.app', { timeout: 15000 });
    await page
      .waitForFunction(
        () =>
          document.querySelectorAll('[data-testid="chat-message"]').length >= 2,
        { timeout: 20000 },
      )
      .catch(() => {});
    await page.waitForTimeout(1000);
    const file = `${OUT}/color-theme-${theme}-${colorTheme}.png`;
    await page.screenshot({ path: file, fullPage: false });
    console.log('saved', file);
    await ctx.close();
  }
}

await browser.close();
