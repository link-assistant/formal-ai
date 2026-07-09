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
//   * the material skin mounts the MUI framework root and upgrades the composer
//     controls to MuiIconButton while preserving their test ids + disabled
//     state (so the rest of the E2E suite keeps working across frameworks);
//   * the glass skin exposes exactly the opacity/blur/refraction sliders (and
//     the other skins hide them), and moving the blur slider re-drives the
//     `--fa-glass-blur` CSS variable that the frosted surfaces read;
//   * the glass composer textarea is fully transparent so it blends into the
//     rounded composer pill (the explicit request on PR #643).

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
    sidebarSettingsCollapsed: 'off',
    glassOpacity: '0.72',
    glassBlur: '18',
    glassRefraction: '60',
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
  for (const skin of ['flat', 'glass', 'material', 'contrast']) {
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

  test('does not mount the MUI root for the Chakra skins', async ({ page }) => {
    await boot(page, { uiSkin: 'glass' });
    await expect(page.locator('[data-testid="mui-framework-root"]')).toHaveCount(0);
  });

  test('composer controls become MuiIconButton yet keep their test ids and disabled state', async ({ page }) => {
    await boot(page, { uiSkin: 'material' });
    const send = page.locator('[data-testid="chat-composer-submit"]');
    const menu = page.locator('[data-testid="composer-menu-toggle"]');
    await expect(send).toHaveClass(/MuiIconButton-root/);
    await expect(menu).toHaveClass(/MuiIconButton-root/);
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
  });

  test('hides the glass sliders for non-glass skins', async ({ page }) => {
    await boot(page, { uiSkin: 'flat' });
    await expect(page.locator('[data-testid="setting-glass-opacity"]')).toHaveCount(0);
    await expect(page.locator('[data-testid="setting-glass-blur"]')).toHaveCount(0);
    await expect(page.locator('[data-testid="setting-glass-refraction"]')).toHaveCount(0);
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

  test('composer textarea is fully transparent so it blends into the pill', async ({ page }) => {
    await boot(page, { uiSkin: 'glass' });
    const bg = await page
      .locator('[data-testid="chat-composer-input"]')
      .evaluate((node) => getComputedStyle(node).backgroundColor);
    // Fully transparent renders as rgba(…, 0) or the keyword "transparent".
    expect(bg === 'transparent' || /,\s*0\s*\)$/.test(bg)).toBe(true);
  });
});
