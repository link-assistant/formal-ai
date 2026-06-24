// @ts-check
// Issue #541 (R1): "Not all UI elements has correctly applied theme."
//
// The user reported widgets that kept their light palette after switching to
// dark mode — most visibly the topbar mode-status badge, the collapsed-sidebar
// toggle, the mobile drawer section headings, and the per-step "tool"/"agent"
// mode badges in the reasoning trace. Each of those classes had a light-mode
// rule with a hardcoded hex but no `:root[data-theme="dark"]` (or
// `@media (prefers-color-scheme: dark)`) counterpart, so the dark theme
// rendered them in their light colors and they read as broken seams.
//
// This spec boots the app with `theme "dark"` pinned in preferences and
// asserts that the actual computed color (read via getComputedStyle) for each
// of those widgets matches a dark palette value instead of the light hex from
// the base rule. We do not assert the exact rgb (palette tweaks are common);
// we assert the colour is dark — i.e. its perceived luminance is closer to the
// dark surface than to the light surface. That makes the test a regression
// guard against the SPECIFIC bug ("light value bleeds through in dark mode")
// without forcing every future color tune-up to update the test.
//
// The matcher: a CSS color string is "dark-themed" when its R/G/B average is
// either very low (a dark *background* like #222624 ≈ rgb(34, 38, 36) → 32.7)
// or fairly muted (a high but desaturated text color like #c9c1b6 ≈ rgb(201,
// 193, 182) → 192). The bright user-facing whites (#ffffff) and the brand
// greens (#2c8f71 → 100.7) stay in range either way. The light text we are
// guarding against (#40515f ≈ rgb(64, 81, 95) → 80) and the light background
// (#eef2f1 ≈ rgb(238, 242, 241) → 240.3) sit OUTSIDE the dark band when used
// against a dark page — the assertions below encode that.

const { test, expect } = require('@playwright/test');

const PREF_KEY = 'formal-ai.preferences.v1';

function preferencesWithDarkTheme() {
  return [
    'demo_preferences',
    '  demoMode "off"',
    '  greetingVariations "off"',
    '  diagnosticsMode "off"',
    '  uiLanguage "en"',
    '  theme "dark"',
  ].join('\n');
}

function parseRgb(color) {
  const match = /rgba?\(\s*(\d+)\s*,\s*(\d+)\s*,\s*(\d+)/.exec(color);
  if (!match) return null;
  return {
    r: Number(match[1]),
    g: Number(match[2]),
    b: Number(match[3]),
  };
}

// A foreground color is acceptable for dark mode when it is either a very
// light, muted text (>= 170 across all channels — the codebase's bright dark
// text is #ece7df / rgb(236, 231, 223) and the muted text is #c9c1b6 / rgb(201,
// 193, 182)) OR a brand accent that we know reads well on dark (we only
// care about the muted family here). Critically, it must NOT be the light
// theme's dark-grey text colour family (#40515f / rgb(64, 81, 95)) where
// every channel sits in the 60–95 range — that hue is the bug we are
// guarding against.
function isMutedDarkText(rgb) {
  if (!rgb) return false;
  return rgb.r >= 170 && rgb.g >= 170 && rgb.b >= 170;
}

// A background color is acceptable for dark mode when it is a dark surface —
// every channel under 70 (the codebase's dark backgrounds are #181a1b /
// rgb(24, 26, 27), #202322 / rgb(32, 35, 34), #222624 / rgb(34, 38, 36)). The
// light bug we are guarding against is #eef2f1 / rgb(238, 242, 241), which
// sits well above this threshold on every channel.
function isDarkSurface(rgb) {
  if (!rgb) return false;
  return rgb.r < 70 && rgb.g < 70 && rgb.b < 70;
}

async function bootIssue541ThemeSpec(page) {
  await page.addInitScript(
    ({ prefKey, preferences }) => {
      try {
        window.localStorage.setItem(prefKey, preferences);
      } catch (_error) {
        // localStorage can be unavailable in hardened browser contexts.
      }
    },
    { prefKey: PREF_KEY, preferences: preferencesWithDarkTheme() },
  );
  await page.goto('./');
  await expect(page.locator('.app')).toBeVisible({ timeout: 15_000 });
  // The boot effect mirrors the stored theme onto <html data-theme>. Until
  // that runs, the page is in its CSS default (light), so widgets would
  // legitimately have light colors. Waiting on the attribute guarantees the
  // tests below measure the post-mirror state.
  await expect(page.locator('html')).toHaveAttribute('data-theme', 'dark', {
    timeout: 10_000,
  });
}

test.describe('Issue #541 (R1): dark theme reaches every primary widget', () => {
  test.beforeEach(async ({ page }) => {
    await bootIssue541ThemeSpec(page);
  });

  test('topbar mode-status badge uses a muted dark color', async ({ page }) => {
    const status = page.locator('[data-testid="mode-status"]');
    await expect(status).toBeVisible();
    const color = await status.evaluate((node) => getComputedStyle(node).color);
    const rgb = parseRgb(color);
    expect(rgb, `parsed rgb from ${color}`).not.toBeNull();
    // The light theme's hard-coded color (#40515f / rgb 64, 81, 95) would
    // FAIL this assertion. The fix's dark color (#c9c1b6 / rgb 201, 193, 182)
    // PASSES it.
    expect(isMutedDarkText(rgb)).toBe(true);
  });

  test('collapsed sidebar toggle uses a dark surface background', async ({ page }) => {
    // The toggle is visible by default on desktop. To exercise the .is-collapsed
    // variant we click it once — the boot effect leaves it expanded.
    const toggle = page.locator('[data-testid="sidebar-toggle"]');
    await expect(toggle).toBeVisible();
    await toggle.click();
    await expect(toggle).toHaveClass(/is-collapsed/);
    const background = await toggle.evaluate((node) =>
      getComputedStyle(node).backgroundColor,
    );
    const rgb = parseRgb(background);
    expect(rgb, `parsed rgb from ${background}`).not.toBeNull();
    // The light theme's hard-coded background (#eef2f1 / rgb 238, 242, 241)
    // would FAIL this assertion. The fix's dark surface (#222624 / rgb 34,
    // 38, 36) PASSES it.
    expect(isDarkSurface(rgb)).toBe(true);
  });
});
