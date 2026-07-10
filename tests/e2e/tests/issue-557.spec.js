// @ts-check
// Issue #557: multi-framework UI skins.
//
// The user asked us to (1) keep the basic Chakra UI skin but polish the
// composer + buttons, (2) add an Apple "Liquid Glass" skin built on top of
// Chakra via rdev/liquid-glass-react (with configurable transparency/blur/
// refraction), and (3) add a Material skin that also SWITCHES the UI framework
// from Chakra to MUI. This spec guards the observable contracts of that work:
//
//   * every skin puts its `ui-skin-<name>` marker class on `.app`;
//   * the MUI skins mount the MUI framework root and upgrade composer + shared
//     topbar button primitives to MuiIconButton while preserving test ids and
//     disabled state (so the rest of the E2E suite keeps working across
//     frameworks);
//   * the glass skin exposes exactly the opacity/blur/refraction sliders (and
//     the other skins hide them), and moving the blur slider re-drives the
//     `--fa-glass-blur` CSS variable that the frosted surfaces read;
//   * the composer textarea stays fully transparent in every skin so it blends
//     into the rounded composer pill (the explicit request on PR #643);
//   * every colour palette exposes its light accent and a distinct dark accent
//     border, guarding the complete skin × scheme configuration contract.

const { test, expect } = require('@playwright/test');

const PREF_KEY = 'formal-ai.preferences.v1';

// Build a LINO preferences blob (the app persists preferences as LINO, not
// JSON). Demo mode stays off so the composer is interactive and slider changes
// are not fighting the demo playback.
function preferences(overrides = {}) {
  const base = {
    demoMode: 'off',
    greetingVariations: 'off',
    diagnosticsMode: 'off',
    uiLanguage: 'en',
    theme: 'light',
    uiSkin: 'flat',
    colorTheme: 'emerald',
    sidebarSettingsCollapsed: 'off',
    glassOpacity: '0.72',
    glassBlur: '18',
    glassRefraction: '60',
    glassMode: 'balanced',
  };
  const merged = { ...base, ...overrides };
  return [
    'demo_preferences',
    ...Object.entries(merged).map(([key, value]) => `  ${key} "${value}"`),
  ].join('\n');
}

async function boot(page, overrides) {
  await page.addInitScript(
    ({ prefKey, blob }) => {
      try {
        window.localStorage.setItem(prefKey, blob);
      } catch (_error) {
        // localStorage can be unavailable in hardened contexts.
      }
    },
    { prefKey: PREF_KEY, blob: preferences(overrides) },
  );
  await page.goto('./');
  await expect(page.locator('.app')).toBeVisible({ timeout: 15_000 });
}

test.describe('Issue #557: UI skins apply their marker class', () => {
  for (const skin of ['flat', 'glass', 'mui-flat', 'material', 'contrast']) {
    test(`the ${skin} skin marks .app with ui-skin-${skin}`, async ({ page }) => {
      await boot(page, { uiSkin: skin });
      await expect(page.locator('.app')).toHaveClass(
        new RegExp(`\\bui-skin-${skin}\\b`),
      );
    });
  }
});

test.describe('Issue #557: material skin switches the framework to MUI', () => {
  test('mounts the MUI framework root only for the material skin', async ({ page }) => {
    await boot(page, { uiSkin: 'material' });
    await expect(page.locator('[data-testid="mui-framework-root"]')).toHaveCount(1);
    // The MUI ScopedCssBaseline wrapper carries its emotion-generated class.
    await expect(page.locator('[data-testid="mui-framework-root"]')).toHaveClass(
      /MuiScopedCssBaseline-root/,
    );
  });

  test('provides a neutral MUI-backed flat variant', async ({ page }) => {
    await boot(page, { uiSkin: 'mui-flat' });
    await expect(page.locator('[data-testid="mui-framework-root"]')).toHaveCount(1);
    await expect(page.locator('[data-testid="chat-composer-submit"]')).toHaveClass(/MuiIconButton-root/);
    await expect(page.locator('[data-testid="memory-export"]')).toHaveClass(/MuiIconButton-root/);
    await expect(page.locator('.app')).toHaveClass(/ui-skin-mui-flat/);
  });

  test('does not mount the MUI root for the Chakra skins', async ({ page }) => {
    await boot(page, { uiSkin: 'glass' });
    await expect(page.locator('[data-testid="mui-framework-root"]')).toHaveCount(0);
  });

  test('composer and topbar buttons become MuiIconButton yet keep their test ids and disabled state', async ({ page }) => {
    await boot(page, { uiSkin: 'material' });
    const send = page.locator('[data-testid="chat-composer-submit"]');
    const menu = page.locator('[data-testid="composer-menu-toggle"]');
    const memoryExport = page.locator('[data-testid="memory-export"]');
    await expect(send).toHaveClass(/MuiIconButton-root/);
    await expect(menu).toHaveClass(/MuiIconButton-root/);
    await expect(memoryExport).toHaveClass(/MuiIconButton-root/);
    await expect(memoryExport).toHaveAccessibleName(/export/i);
    // The send button is disabled while the composer is empty; MUI must forward
    // that state (Mui-disabled + the native disabled attribute) so the rest of
    // the suite's "wait until enabled" logic keeps working.
    await expect(send).toBeDisabled();
    await expect(send).toHaveClass(/Mui-disabled/);
    await page.locator('[data-testid="chat-composer-input"]').fill('hello');
    await expect(send).toBeEnabled();
  });
});

test.describe('Issue #557: glass skin configuration', () => {
  test('exposes opacity, blur and refraction sliders', async ({ page }) => {
    await boot(page, { uiSkin: 'glass' });
    await expect(page.locator('[data-testid="setting-glass-opacity"]')).toBeVisible();
    await expect(page.locator('[data-testid="setting-glass-blur"]')).toBeVisible();
    await expect(page.locator('[data-testid="setting-glass-refraction"]')).toBeVisible();
    await expect(page.locator('[data-testid="setting-glass-mode"]')).toBeVisible();
  });

  test('hides the glass sliders for non-glass skins', async ({ page }) => {
    await boot(page, { uiSkin: 'flat' });
    await expect(page.locator('[data-testid="setting-glass-opacity"]')).toHaveCount(0);
    await expect(page.locator('[data-testid="setting-glass-blur"]')).toHaveCount(0);
    await expect(page.locator('[data-testid="setting-glass-refraction"]')).toHaveCount(0);
    await expect(page.locator('[data-testid="setting-glass-mode"]')).toHaveCount(0);
  });

  test('moving the blur slider re-drives the --fa-glass-blur variable', async ({ page }) => {
    await boot(page, { uiSkin: 'glass', glassBlur: '18' });
    const readBlur = () =>
      page
        .locator('.app')
        .evaluate((node) =>
          getComputedStyle(node).getPropertyValue('--fa-glass-blur').trim(),
        );
    expect(await readBlur()).toBe('18px');
    // Drag the slider to its maximum (40). setValue on a range input then a
    // dispatched input event mirrors a user drag for React's controlled input.
    await page.locator('[data-testid="setting-glass-blur"]').evaluate((node) => {
      const input = /** @type {HTMLInputElement} */ (node);
      const setter = Object.getOwnPropertyDescriptor(
        HTMLInputElement.prototype,
        'value',
      ).set;
      setter.call(input, '40');
      input.dispatchEvent(new Event('input', { bubbles: true }));
    });
    await expect
      .poll(async () => await readBlur())
      .toBe('40px');
  });

  test('stops ambient motion when reduced motion is requested', async ({ page }) => {
    await page.emulateMedia({ reducedMotion: 'reduce' });
    await boot(page, { uiSkin: 'glass' });
    await expect.poll(() => page.locator('.app').evaluate((node) => getComputedStyle(node).animationName))
      .toBe('none');
  });

  test('persists visual mode and exposes the resolved mode on the app surface', async ({ page }) => {
    await boot(page, { uiSkin: 'glass', glassMode: 'balanced' });
    const mode = page.locator('[data-testid="setting-glass-mode"]');
    await expect(mode.locator('option')).toHaveCount(3);
    await mode.selectOption('clear');
    await expect(page.locator('.app')).toHaveAttribute('data-glass-mode', 'clear');
    await expect.poll(() => page.evaluate((key) => localStorage.getItem(key) || '', PREF_KEY))
      .toContain('glassMode "clear"');
  });

  test('all glass settings participate in individual reset behavior', async ({ page }) => {
    await boot(page, {
      uiSkin: 'glass',
      glassOpacity: '0.45',
      glassBlur: '40',
      glassRefraction: '120',
      glassMode: 'clear',
    });
    for (const key of ['glassOpacity', 'glassBlur', 'glassRefraction', 'glassMode']) {
      await expect(page.locator(`[data-testid="settings-reset-${key}"]`)).toBeVisible();
    }
    await page.locator('[data-testid="settings-reset-glassBlur"]').click();
    await expect(page.locator('[data-testid="setting-glass-blur"]')).toHaveValue('18');
    await page.locator('[data-testid="settings-reset-glassRefraction"]').click();
    await expect(page.locator('[data-testid="setting-glass-refraction"]')).toHaveValue('60');
    await page.locator('[data-testid="settings-reset-glassMode"]').click();
    await expect(page.locator('[data-testid="setting-glass-mode"]')).toHaveValue('balanced');
  });
});

test.describe('Issue #557: unified composer surface', () => {
  for (const skin of ['flat', 'glass', 'mui-flat', 'material', 'contrast']) {
    test(`${skin} keeps the actual textarea transparent`, async ({ page }) => {
      await boot(page, { uiSkin: skin });
      const bg = await page
        .locator('[data-testid="chat-composer-input"]')
        .evaluate((node) => getComputedStyle(node).backgroundColor);
      // Fully transparent renders as rgba(…, 0) or the keyword "transparent".
      expect(bg === 'transparent' || /,\s*0\s*\)$/.test(bg)).toBe(true);
    });
  }

  for (const viewport of [
    { name: 'desktop', width: 1280, height: 860 },
    { name: 'tablet', width: 820, height: 1180 },
    { name: 'mobile', width: 390, height: 800 },
  ]) {
    test(`${viewport.name} keeps focused multiline input and controls inside one surface`, async ({ page }) => {
      await page.setViewportSize({ width: viewport.width, height: viewport.height });
      await boot(page, { uiSkin: 'material', theme: 'dark' });
      const surface = page.locator('.composer-grid');
      const input = page.locator('[data-testid="chat-composer-input"]');
      await input.fill('first line\nsecond line\nthird line');
      await input.focus();
      const style = await input.evaluate((node) => {
        const computed = getComputedStyle(node);
        return {
          background: computed.backgroundColor,
          borderTop: computed.borderTopWidth,
          outline: computed.outlineStyle,
        };
      });
      expect(style.background === 'transparent' || /,\s*0\s*\)$/.test(style.background)).toBe(true);
      expect(style.borderTop).toBe('0px');
      await expect(surface).toBeVisible();
      const surfaceBox = await surface.boundingBox();
      for (const selector of [
        '[data-testid="composer-menu-toggle"]',
        '[data-testid="chat-composer-input"]',
        '[data-testid="chat-composer-submit"]',
      ]) {
        const box = await page.locator(selector).boundingBox();
        expect(box && surfaceBox && box.x).toBeGreaterThanOrEqual(surfaceBox.x - 1);
        expect(box && surfaceBox && box.x + box.width).toBeLessThanOrEqual(surfaceBox.x + surfaceBox.width + 1);
        expect(box && surfaceBox && box.y).toBeGreaterThanOrEqual(surfaceBox.y - 1);
        expect(box && surfaceBox && box.y + box.height).toBeLessThanOrEqual(surfaceBox.y + surfaceBox.height + 1);
      }
    });
  }

  test('composer controls retain accessible names across Chakra, Glass, and MUI', async ({ page }) => {
    await boot(page, { uiSkin: 'flat' });
    for (const skin of ['flat', 'glass', 'material']) {
      await page.locator('[data-testid="setting-ui-skin"]').selectOption(skin);
      await expect(page.locator('[data-testid="composer-menu-toggle"]')).toHaveAccessibleName(/.+/);
      await expect(page.locator('[data-testid="chat-composer-input"]')).toHaveAccessibleName(/.+/);
      await expect(page.locator('[data-testid="chat-composer-submit"]')).toHaveAccessibleName(/.+/);
    }
  });
});

// Issue #557 (PR #643 follow-up): multiple user-selectable colour themes, each
// with a light and a dark variant. The theme id is mirrored onto `.app` via
// `data-color-theme`, and it re-tints the shared `--fa-accent-*` tokens so the
// brand accent (links, the solid send button, hover/focus) recolours across
// every framework/skin without touching surfaces or text contrast.
test.describe('Issue #557: colour themes', () => {
  const themes = {
    emerald: { light: '#1f7a5b', darkBorder: '#2a8f6a' },
    ocean: { light: '#1668b8', darkBorder: '#3d92e0' },
    indigo: { light: '#4f46e5', darkBorder: '#818cf8' },
    violet: { light: '#7c3aed', darkBorder: '#a78bfa' },
    rose: { light: '#be123c', darkBorder: '#fb7185' },
    amber: { light: '#b45309', darkBorder: '#f59e0b' },
    graphite: { light: '#475569', darkBorder: '#94a3b8' },
  };

  const readAccent = (page) =>
    page
      .locator('.app')
      .evaluate((node) =>
        getComputedStyle(node).getPropertyValue('--fa-accent-solid-bg').trim(),
      );

  test('the default emerald theme marks .app with data-color-theme', async ({ page }) => {
    await boot(page, { colorTheme: 'emerald' });
    await expect(page.locator('.app')).toHaveAttribute(
      'data-color-theme',
      'emerald',
    );
  });

  test('a persisted non-default theme is applied on boot and re-tints the accent', async ({ page }) => {
    await boot(page, { colorTheme: 'ocean' });
    await expect(page.locator('.app')).toHaveAttribute(
      'data-color-theme',
      'ocean',
    );
    // getPropertyValue returns the raw declared custom-property value, so we
    // compare against the hex the theme sets (ocean's light brand).
    expect(await readAccent(page)).toBe('#1668b8');
  });

  test('choosing a theme in settings re-drives the accent token live', async ({ page }) => {
    await boot(page, { colorTheme: 'emerald' });
    // Emerald is the base palette (no override) — --fa-accent-solid-bg is #1f7a5b.
    expect(await readAccent(page)).toBe('#1f7a5b');
    await page
      .locator('[data-testid="setting-color-theme"]')
      .selectOption('violet');
    await expect(page.locator('.app')).toHaveAttribute(
      'data-color-theme',
      'violet',
    );
    // Violet's light brand is #7c3aed.
    await expect.poll(async () => await readAccent(page)).toBe('#7c3aed');
  });

  test('all seven palettes switch live and ship light + dark variants', async ({ page }) => {
    await boot(page, { colorTheme: 'emerald', theme: 'light' });
    const app = page.locator('.app');
    const colorSelect = page.locator('[data-testid="setting-color-theme"]');
    const schemeSelect = page.locator('[data-testid="setting-theme"]');
    const readBorder = () =>
      app.evaluate((node) =>
        getComputedStyle(node)
          .getPropertyValue('--fa-accent-solid-border')
          .trim(),
      );

    await expect(colorSelect.locator('option')).toHaveCount(
      Object.keys(themes).length,
    );
    for (const [themeId, expected] of Object.entries(themes)) {
      await colorSelect.selectOption(themeId);
      await schemeSelect.selectOption('light');
      await expect(app).toHaveAttribute('data-color-theme', themeId);
      await expect.poll(async () => await readAccent(page)).toBe(expected.light);
      await expect.poll(async () => await readBorder()).toBe(expected.light);

      await schemeSelect.selectOption('dark');
      await expect
        .poll(() => page.locator('html').getAttribute('data-theme'))
        .toBe('dark');
      await expect.poll(async () => await readBorder()).toBe(expected.darkBorder);
    }
  });
});

test.describe('Issue #557: composer visual regression matrix', () => {
  for (const viewport of [
    { name: 'desktop', width: 1280, height: 860 },
    { name: 'tablet', width: 820, height: 1180 },
    { name: 'mobile', width: 390, height: 800 },
  ]) {
    for (const theme of ['light', 'dark']) {
      for (const skin of ['flat', 'glass', 'material']) {
        test(`${viewport.name} ${theme} ${skin} multiline composer`, async ({ page }) => {
          await page.setViewportSize({ width: viewport.width, height: viewport.height });
          await page.emulateMedia({ reducedMotion: 'reduce', colorScheme: theme });
          await boot(page, { uiSkin: skin, theme });
          const input = page.locator('[data-testid="chat-composer-input"]');
          await input.fill('A focused multiline message\nwith embedded controls');
          await input.focus();
          await expect(page.locator('form.composer')).toHaveScreenshot(
            `composer-${viewport.name}-${theme}-${skin}.png`,
            { animations: 'disabled', caret: 'hide', maxDiffPixelRatio: 0.01 },
          );
        });
      }
    }
  }
});
