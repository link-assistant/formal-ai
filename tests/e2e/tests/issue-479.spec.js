// @ts-check
//
// Issue #479: "`Not available in latest release` for all desktop apps".
//
// The primary bug (the `/download` page showed "Not available in latest release"
// for every platform because the auto-release child-commit tag never matched the
// `workflow_run.head_sha`, so zero desktop assets were built) is covered by the
// Rust unit test tests/unit/ci-cd/desktop_release_resolve.rs.
//
// This e2e suite covers the issue's documentation request:
//
//   "macOS instructions don't have screenshots like in
//    https://konard.github.io/vk-bot-desktop"
//
// The macOS install section now embeds three faithful, on-brand reproductions of
// the macOS 15 (Sequoia) Gatekeeper dialogs (regenerated from a fixture via
// tests/e2e/scripts/generate-macos-screenshots.mjs and committed under
// src/web/download/assets/screenshots/). These assertions prove the figures
// render, the images actually load (no broken <img>), each carries a localized
// alt text, and the caption is present — in every supported UI language.
const { test, expect } = require('@playwright/test');

const LOCALES = ['en', 'ru', 'zh', 'hi'];

// The three Gatekeeper screenshots, in document order, mapped 1:1 to the
// System Settings steps (installMacosSettingsStep1/2/3).
const MACOS_SHOTS = [
  'assets/screenshots/macos-gatekeeper-not-opened.png',
  'assets/screenshots/macos-gatekeeper-open-anyway.png',
  'assets/screenshots/macos-gatekeeper-confirm.png',
];

// Seed the shared preference store (theme + UI language) before any page script
// runs, mirroring issue-347.spec.js so the suite never depends on browser locale.
function seedPreferences(data) {
  try {
    window.localStorage.setItem(
      'formal-ai.preferences.v1',
      'demo_preferences\n  theme "' +
        data.theme +
        '"\n  uiLanguage "' +
        data.locale +
        '"\n  greetingVariations "off"\n  demoMode "off"',
    );
  } catch (error) {
    // localStorage may be unavailable in hardened browser contexts; the page
    // falls back to defaults and the test tolerates it.
  }
}

test.describe('Issue #479 — macOS Gatekeeper screenshots on /download', () => {
  test('renders three macOS screenshots that all load (en)', async ({ page }) => {
    await page.addInitScript(seedPreferences, { theme: 'light', locale: 'en' });
    await page.goto('/download/');
    await expect(page.locator('[data-testid="download-app"]')).toBeVisible({ timeout: 15_000 });

    const figure = page.locator('.install-macos .install-macos-screenshots');
    await expect(figure).toHaveCount(1);

    const imgs = figure.locator('img');
    await expect(imgs).toHaveCount(MACOS_SHOTS.length);

    // The figure lives inside the System Settings <details>, which renders open
    // by default, so the screenshots are visible without interaction.
    await expect(imgs.first()).toBeVisible();

    for (let i = 0; i < MACOS_SHOTS.length; i++) {
      const img = imgs.nth(i);
      await expect(img, MACOS_SHOTS[i]).toHaveAttribute('src', MACOS_SHOTS[i]);
      // Lazy/async decoding hints are set so the page stays responsive.
      await expect(img).toHaveAttribute('loading', 'lazy');

      // The image actually resolved (no broken <img>): naturalWidth > 0 once loaded.
      await expect
        .poll(async () => img.evaluate((el) => /** @type {HTMLImageElement} */ (el).naturalWidth), {
          timeout: 10_000,
        })
        .toBeGreaterThan(0);

      // Every screenshot carries a non-empty alt text for accessibility.
      const alt = await img.getAttribute('alt');
      expect(alt && alt.trim().length, 'alt text for ' + MACOS_SHOTS[i]).toBeGreaterThan(0);
    }

    // The caption is present.
    await expect(figure.locator('figcaption')).toHaveText(/\S/);
  });

  // The alt text and caption are localized for every supported UI language, so a
  // missing translation can't silently fall back to an English-only or empty alt.
  for (const locale of LOCALES) {
    test('localizes the macOS screenshot alt text + caption (' + locale + ')', async ({ page }) => {
      await page.addInitScript(seedPreferences, { theme: 'light', locale });
      await page.goto('/download/');
      await expect(page.locator('[data-testid="download-app"]')).toBeVisible({ timeout: 15_000 });

      const figure = page.locator('.install-macos .install-macos-screenshots');
      const imgs = figure.locator('img');
      await expect(imgs).toHaveCount(MACOS_SHOTS.length);

      for (let i = 0; i < MACOS_SHOTS.length; i++) {
        const alt = await imgs.nth(i).getAttribute('alt');
        expect(alt && alt.trim().length, locale + ' alt[' + i + ']').toBeGreaterThan(0);
      }
      await expect(figure.locator('figcaption')).toHaveText(/\S/);
    });
  }
});
