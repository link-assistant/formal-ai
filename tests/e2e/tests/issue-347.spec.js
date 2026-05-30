// @ts-check
//
// Issue #347: add a cross-platform `/download` page (mirroring the design and
// feature set of github.com/konard/vk-bot-desktop) and generate its preview
// screenshots in CI/CD.
//
// This suite has two jobs:
//
//   1. Functional assertions for the `/download` page itself — it renders, lists
//      every cross-platform installer (macOS / Windows / Linux — requirement R1),
//      respects the shared theme + UI-language preferences (R2), detects the OS,
//      exposes the offline SHA-256 checksum verifier, and links back to the app.
//
//   2. Screenshot generation (R2 — "generates screenshots in CI/CD"). For each of
//      the four UI languages (en/ru/zh/hi) and both themes (light/dark) it writes
//        - src/web/download/assets/app-preview-<locale>-<theme>.png  (the in-window
//          preview the /download page layers over its placeholder), and
//        - docs/screenshots/issue-347/download-<locale>-<theme>.png (a full-page
//          capture of the /download page itself, for PR review).
//      The committed copies are what GitHub Pages publishes; regenerating them on
//      every CI run proves the pipeline can produce them deterministically.
//
// The release data is injected through the page's documented test hook
// (`window.__FORMAL_AI_DOWNLOAD_RELEASE__`) so the suite never depends on the
// live GitHub Releases API (rate limits / offline CI) and the screenshots always
// show resolvable, version-stamped download links.
const { test, expect } = require('@playwright/test');
const crypto = require('node:crypto');
const fs = require('node:fs');
const path = require('node:path');

const REPO_ROOT = path.resolve(__dirname, '..', '..', '..');
const PREVIEW_DIR = path.join(REPO_ROOT, 'src', 'web', 'download', 'assets');
const SHOTS_DIR = path.join(REPO_ROOT, 'docs', 'screenshots', 'issue-347');

const LOCALES = ['en', 'ru', 'zh', 'hi'];
const THEMES = ['light', 'dark'];

// Version is the single source of truth in Cargo.toml; the desktop assets and
// the /download links are stamped with it.
const CARGO_TOML = fs.readFileSync(path.join(REPO_ROOT, 'Cargo.toml'), 'utf8');
const PACKAGE_SECTION = CARGO_TOML.split(/^\[/m).find((s) => s.startsWith('package]')) || '';
const VERSION = (PACKAGE_SECTION.match(/^\s*version\s*=\s*"([^"]+)"/m) || [])[1];

// The 14 desktop installers the /download page resolves from a release. This
// list mirrors `download.js`'s `downloadOptions`; a functional test below asserts
// the page still agrees with it, so drift fails loudly instead of silently.
const DOWNLOAD_OPTIONS = [
  { id: 'macos-arm64', os: 'macos', assetPrefix: 'formal-ai-desktop-macos-arm64', extension: 'dmg' },
  { id: 'macos-arm64-zip', os: 'macos', assetPrefix: 'formal-ai-desktop-macos-arm64', extension: 'zip' },
  { id: 'macos-x64', os: 'macos', assetPrefix: 'formal-ai-desktop-macos-x64', extension: 'dmg' },
  { id: 'macos-x64-zip', os: 'macos', assetPrefix: 'formal-ai-desktop-macos-x64', extension: 'zip' },
  { id: 'windows-x64', os: 'windows', assetPrefix: 'formal-ai-desktop-windows-installer-x64', extension: 'exe' },
  { id: 'windows-arm64', os: 'windows', assetPrefix: 'formal-ai-desktop-windows-installer-arm64', extension: 'exe' },
  { id: 'windows-portable-x64', os: 'windows', assetPrefix: 'formal-ai-desktop-windows-portable-x64', extension: 'exe' },
  { id: 'windows-portable-arm64', os: 'windows', assetPrefix: 'formal-ai-desktop-windows-portable-arm64', extension: 'exe' },
  { id: 'linux-appimage-x64', os: 'linux', assetPrefix: 'formal-ai-desktop-linux-x64', extension: 'AppImage' },
  { id: 'linux-appimage-arm64', os: 'linux', assetPrefix: 'formal-ai-desktop-linux-arm64', extension: 'AppImage' },
  { id: 'linux-deb-x64', os: 'linux', assetPrefix: 'formal-ai-desktop-linux-x64', extension: 'deb' },
  { id: 'linux-deb-arm64', os: 'linux', assetPrefix: 'formal-ai-desktop-linux-arm64', extension: 'deb' },
  { id: 'linux-tar-x64', os: 'linux', assetPrefix: 'formal-ai-desktop-linux-x64', extension: 'tar.gz' },
  { id: 'linux-tar-arm64', os: 'linux', assetPrefix: 'formal-ai-desktop-linux-arm64', extension: 'tar.gz' },
];

const RELEASE_BASE =
  'https://github.com/link-assistant/formal-ai/releases/download/v' + VERSION + '/';

function assetName(option) {
  return option.assetPrefix + '-' + VERSION + '.' + option.extension;
}

// A deterministic GitHub-release-shaped object (the fields download.js reads).
function buildRelease() {
  const assets = DOWNLOAD_OPTIONS.map((option) => ({
    name: assetName(option),
    browser_download_url: RELEASE_BASE + assetName(option),
    size: 12_345_678,
    content_type: 'application/octet-stream',
  }));
  for (const extra of ['SHA256SUMS.txt', 'BUILD-PROVENANCE.txt']) {
    assets.push({
      name: extra,
      browser_download_url: RELEASE_BASE + extra,
      size: 4096,
      content_type: 'text/plain',
    });
  }
  return {
    tag_name: 'v' + VERSION,
    name: 'v' + VERSION,
    html_url: 'https://github.com/link-assistant/formal-ai/releases/tag/v' + VERSION,
    published_at: '2026-01-01T00:00:00Z',
    assets,
  };
}

const RELEASE = buildRelease();

// addInitScript payload: seed the shared preference store (theme + UI language)
// and the deterministic release before any page script runs.
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
  window.__FORMAL_AI_DOWNLOAD_RELEASE__ = data.release;
}

test.describe('Issue #347 — /download page', () => {
  test.beforeEach(async ({ page }) => {
    await page.addInitScript(seedPreferences, { theme: 'light', locale: 'en', release: RELEASE });
    await page.goto('/download/');
    await expect(page.locator('[data-testid="download-app"]')).toBeVisible({ timeout: 15_000 });
  });

  test('Cargo.toml version is readable (release stamping)', async () => {
    expect(VERSION, 'could not parse [package] version from Cargo.toml').toMatch(
      /^\d+\.\d+\.\d+/,
    );
  });

  test('lists every cross-platform installer with a resolvable link (R1)', async ({ page }) => {
    // The page's option table still matches this spec's expectations (drift guard).
    const pageOptions = await page.evaluate(() =>
      window.FormalAiDownload.downloadOptions.map((o) => ({
        id: o.id,
        assetPrefix: o.assetPrefix,
        extension: o.extension,
      })),
    );
    expect(pageOptions).toEqual(
      DOWNLOAD_OPTIONS.map(({ id, assetPrefix, extension }) => ({ id, assetPrefix, extension })),
    );

    // All three desktop platforms are represented.
    expect(new Set(DOWNLOAD_OPTIONS.map((o) => o.os))).toEqual(
      new Set(['macos', 'windows', 'linux']),
    );

    // Each option renders exactly once and points at the version-stamped asset.
    for (const option of DOWNLOAD_OPTIONS) {
      const link = page.locator('[data-testid="download-' + option.id + '"]');
      await expect(link, option.id).toHaveCount(1);
      await expect(link, option.id).toHaveAttribute('href', RELEASE_BASE + assetName(option));
    }

    // The three OS groups are present in the downloads grid.
    for (const os of ['macos', 'windows', 'linux']) {
      await expect(page.locator('.download-group[data-os="' + os + '"]')).toHaveCount(1);
    }
  });

  test('shows the resolved release tag in the status row', async ({ page }) => {
    const status = page.locator('[data-testid="release-status"]');
    await expect(status).toBeVisible();
    await expect(status).toContainText('v' + VERSION);
  });

  test('offers a primary download and a working back-to-app link', async ({ page }) => {
    await expect(page.locator('[data-testid="primary-download"]')).toBeVisible();
    const back = page.locator('[data-testid="back-to-app"]');
    await expect(back).toBeVisible();
    await expect(back).toHaveAttribute('href', '../');
  });

  test('respects the theme preference and live theme switching (R2)', async ({ page }) => {
    const html = page.locator('html');
    // Seeded as light.
    await expect(html).toHaveAttribute('data-theme', 'light');

    // The segmented theme control flips the document theme attribute.
    await page.locator('.theme-switch button[data-value="dark"]').click();
    await expect(html).toHaveAttribute('data-theme', 'dark');
    await page.locator('.theme-switch button[data-value="light"]').click();
    await expect(html).toHaveAttribute('data-theme', 'light');

    // And the choice is persisted to the shared preference store.
    const stored = await page.evaluate(() =>
      window.localStorage.getItem('formal-ai.preferences.v1'),
    );
    expect(stored).toContain('theme "light"');
  });

  test('localizes the UI across every supported language and persists the choice (R2)', async ({
    page,
  }) => {
    const html = page.locator('html');
    const downloadsTitle = page.locator('#downloads-title');
    // Seeded as English.
    await expect(html).toHaveAttribute('lang', 'en');
    await expect(downloadsTitle).toHaveText('Latest release');

    // Switching the UI language localizes the "Latest release" heading and the
    // <html lang> attribute, and persists the choice to the shared preference
    // store. Asserting each native heading pins every non-English language so a
    // fix can't regress one of them silently: Russian (Cyrillic),
    // Chinese (Han), and Hindi (Devanagari).
    const localeCases = [
      { locale: 'ru', heading: 'Последний релиз' }, // Russian
      { locale: 'zh', heading: '最新版本' }, // Chinese
      { locale: 'hi', heading: 'नवीनतम रिलीज़' }, // Hindi
    ];
    for (const { locale, heading } of localeCases) {
      await page.locator('.locale-switch button[data-value="' + locale + '"]').click();
      await expect(html).toHaveAttribute('lang', locale);
      await expect(downloadsTitle).toHaveText(heading);
      const stored = await page.evaluate(() =>
        window.localStorage.getItem('formal-ai.preferences.v1'),
      );
      expect(stored).toContain('uiLanguage "' + locale + '"');
    }
  });

  test('verifies a matching SHA-256 checksum offline', async ({ page }) => {
    const content = Buffer.from('formal-ai issue 347 checksum fixture', 'utf8');
    const digest = crypto.createHash('sha256').update(content).digest('hex');
    const fileName = 'formal-ai-desktop-linux-x64-' + VERSION + '.AppImage';
    const sums = digest + '  ' + fileName + '\n';

    await page
      .locator('[data-testid="verify-file"]')
      .setInputFiles({ name: fileName, mimeType: 'application/octet-stream', buffer: content });
    await page
      .locator('[data-testid="verify-sums"]')
      .setInputFiles({ name: 'SHA256SUMS.txt', mimeType: 'text/plain', buffer: Buffer.from(sums, 'utf8') });

    await expect(page.locator('[data-testid="verify-result"]')).toHaveClass(
      'verification-result match',
      { timeout: 10_000 },
    );
  });

  test('flags a mismatched SHA-256 checksum offline', async ({ page }) => {
    const content = Buffer.from('formal-ai issue 347 checksum fixture', 'utf8');
    const fileName = 'formal-ai-desktop-linux-x64-' + VERSION + '.AppImage';
    // A valid-looking but wrong digest for the same filename.
    const wrong = 'f'.repeat(64);
    const sums = wrong + '  ' + fileName + '\n';

    await page
      .locator('[data-testid="verify-file"]')
      .setInputFiles({ name: fileName, mimeType: 'application/octet-stream', buffer: content });
    await page
      .locator('[data-testid="verify-sums"]')
      .setInputFiles({ name: 'SHA256SUMS.txt', mimeType: 'text/plain', buffer: Buffer.from(sums, 'utf8') });

    await expect(page.locator('[data-testid="verify-result"]')).toHaveClass(
      'verification-result mismatch',
      { timeout: 10_000 },
    );
  });
});

test.describe('Issue #347 — preview & page screenshots (R2)', () => {
  test.beforeAll(() => {
    fs.mkdirSync(PREVIEW_DIR, { recursive: true });
    fs.mkdirSync(SHOTS_DIR, { recursive: true });
  });

  for (const locale of LOCALES) {
    for (const theme of THEMES) {
      test('generates ' + locale + '/' + theme + ' app preview and /download capture', async ({
        page,
      }) => {
        await page.addInitScript(seedPreferences, { theme, locale, release: RELEASE });
        await page.emulateMedia({ reducedMotion: 'reduce' });
        await page.setViewportSize({ width: 1280, height: 832 });

        // --- In-window app preview (screenshot of the real chat app) ----------
        await page.goto('/');
        await expect(page.locator('.app')).toBeVisible({ timeout: 15_000 });
        await expect(page.locator('html')).toHaveAttribute('data-theme', theme);
        await expect(page.locator('html')).toHaveAttribute('lang', locale);
        // Let fonts/layout settle (these captures are for human review, not pixel diffing).
        await page.waitForTimeout(500);
        await page.screenshot({
          path: path.join(PREVIEW_DIR, 'app-preview-' + locale + '-' + theme + '.png'),
        });

        // --- Full /download page capture --------------------------------------
        await page.goto('/download/');
        await expect(page.locator('[data-testid="download-app"]')).toBeVisible({ timeout: 15_000 });
        await expect(page.locator('html')).toHaveAttribute('data-theme', theme);
        await expect(page.locator('[data-testid="release-status"]')).toContainText('v' + VERSION);
        await page.waitForTimeout(400);
        await page.screenshot({
          path: path.join(SHOTS_DIR, 'download-' + locale + '-' + theme + '.png'),
          fullPage: true,
        });
      });
    }
  }

  // The /download page preloads assets/app-preview.png as the fallback used when
  // a locale/theme-specific preview is unavailable. Generate it from the English
  // dark-theme app so the fallback is never a broken image.
  test('generates the default app-preview.png fallback', async ({ page }) => {
    await page.addInitScript(seedPreferences, { theme: 'dark', locale: 'en', release: RELEASE });
    await page.emulateMedia({ reducedMotion: 'reduce' });
    await page.setViewportSize({ width: 1280, height: 832 });
    await page.goto('/');
    await expect(page.locator('.app')).toBeVisible({ timeout: 15_000 });
    await expect(page.locator('html')).toHaveAttribute('data-theme', 'dark');
    await page.waitForTimeout(500);
    await page.screenshot({ path: path.join(PREVIEW_DIR, 'app-preview.png') });
  });
});
