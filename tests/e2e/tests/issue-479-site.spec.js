// @ts-check
//
// Issue #479: the site is restructured so a visitor lands on a chooser and can
// pick where to go — the interactive web app, the documentation, or the desktop
// download — mirroring the vk-bot-desktop best practices the issue asks us to
// follow:
//
//   /            landing page (this chooser)
//   /app/        the interactive web app (moved off the root)
//   /docs/       the documentation hub (links to /docs/api/ + the prose docs)
//   /download/   the desktop download page
//
// These specs exercise the two new chooser pages (landing + docs) rendered by
// the shared src/web/site-chrome.js: the nav cards point at the right targets,
// the language/theme switchers work and persist via the shared
// formal-ai.preferences.v1 store, and the footer version label stays hidden on
// an un-stamped build. Navigation is written relative to the /app/ baseURL
// (../, ../docs/) so the identical specs run against the local server *and* the
// GitHub Pages path prefix (…/formal-ai/).
const { test, expect } = require('@playwright/test');

// Seed the shared preference store (Links-Notation format, identical to
// issue-479.spec.js / issue-347.spec.js) so the rendered locale never depends
// on the CI browser's Accept-Language header.
function seedPreferences(data) {
  try {
    // addInitScript runs on *every* navigation, including page.reload(). Only
    // seed when the store is empty so a reload re-hydrates the choice the test
    // just made instead of clobbering it back to the seeded default.
    if (window.localStorage.getItem('formal-ai.preferences.v1') !== null) {
      return;
    }
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

test.describe('Issue #479 — landing page (/) chooser', () => {
  test.beforeEach(async ({ page }) => {
    await page.addInitScript(seedPreferences, { theme: 'light', locale: 'en' });
    // ../ resolves to the site root from the /app/ baseURL on both the local
    // server and the GitHub Pages path prefix.
    await page.goto('../');
    await expect(page.locator('[data-testid="landing-app-summary"]')).toBeVisible({
      timeout: 15_000,
    });
  });

  test('renders three navigation cards pointing at app, docs and download', async ({ page }) => {
    await expect(page.locator('.hero h1')).toHaveText('formal-ai');

    const cards = page.locator('[data-testid="nav-cards"] .nav-card');
    await expect(cards).toHaveCount(3);

    // In-site routes are relative so the Pages path prefix is preserved.
    await expect(page.locator('[data-testid="nav-app"]')).toHaveAttribute('href', 'app/');
    await expect(page.locator('[data-testid="nav-docs"]')).toHaveAttribute('href', 'docs/');
    await expect(page.locator('[data-testid="nav-download"]')).toHaveAttribute('href', 'download/');

    // In-site cards stay in the same tab so theme/locale carry over; the brand
    // returns home.
    await expect(page.locator('[data-testid="nav-app"]')).not.toHaveAttribute('target', '_blank');
    await expect(page.locator('[data-testid="brand-home"]')).toHaveAttribute('href', './');
  });

  test('the web-app card opens the app, which boots under <base href="../">', async ({ page }) => {
    await page.locator('[data-testid="nav-app"]').click();
    await expect(page).toHaveURL(/\/app\/$/);
    // The app shell mounts — proving the relative asset/worker/seed URLs resolve
    // to the site root via the app page's <base href="../">.
    await expect(page.locator('.app')).toBeVisible({ timeout: 30_000 });
  });

  test('the documentation card opens the docs hub', async ({ page }) => {
    await page.locator('[data-testid="nav-docs"]').click();
    await expect(page).toHaveURL(/\/docs\/$/);
    await expect(page.locator('[data-testid="docs-app-summary"]')).toBeVisible({ timeout: 15_000 });
    await expect(page.locator('.hero h1')).toHaveText('Documentation');
  });

  test('switching language re-renders the chooser and persists across reloads', async ({ page }) => {
    // Seeded English baseline.
    await expect(page.locator('[data-testid="landing-app-summary"]')).toContainText(
      'A local, in-process',
    );

    await page.locator('.locale-switch button[data-value="ru"]').click();
    await expect(page.locator('[data-testid="landing-app-summary"]')).toContainText('Локальный');
    await expect(page.locator('html')).toHaveAttribute('lang', 'ru');

    // The choice is written to the shared store, so a reload keeps Russian.
    await page.reload();
    await expect(page.locator('[data-testid="landing-app-summary"]')).toContainText('Локальный');
    await expect(page.locator('html')).toHaveAttribute('lang', 'ru');
  });

  test('switching theme updates data-theme and persists across reloads', async ({ page }) => {
    await page.locator('.theme-switch button[data-value="dark"]').click();
    await expect(page.locator('html')).toHaveAttribute('data-theme', 'dark');

    await page.reload();
    await expect(page.locator('html')).toHaveAttribute('data-theme', 'dark');

    await page.locator('.theme-switch button[data-value="light"]').click();
    await expect(page.locator('html')).toHaveAttribute('data-theme', 'light');
  });

  test('hides the footer version label on an un-stamped build', async ({ page }) => {
    // The dev/local server serves the raw __FORMAL_AI_VERSION__ placeholder;
    // readVersion() omits the label rather than printing the placeholder. Only
    // the Pages stamp pipeline injects a real semver.
    const version = page.locator('[data-testid="landing-version"]');
    const meta = await page
      .locator('meta[name="formal-ai-version"]')
      .getAttribute('content');
    if (meta && /^v?\d/.test(meta)) {
      // Stamped build (GitHub Pages): the label is shown and carries the semver.
      await expect(version).toBeVisible();
      await expect(version).toContainText(meta.replace(/^v/, ''));
    } else {
      // Un-stamped build (local server): the label is absent entirely.
      await expect(version).toHaveCount(0);
    }
  });
});

test.describe('Issue #479 — documentation hub (/docs/)', () => {
  test.beforeEach(async ({ page }) => {
    await page.addInitScript(seedPreferences, { theme: 'light', locale: 'en' });
    await page.goto('../docs/');
    await expect(page.locator('[data-testid="docs-app-summary"]')).toBeVisible({ timeout: 15_000 });
  });

  test('renders the API reference card plus the three prose-doc cards', async ({ page }) => {
    await expect(page.locator('.hero h1')).toHaveText('Documentation');

    const cards = page.locator('[data-testid="nav-cards"] .nav-card');
    await expect(cards).toHaveCount(4);

    // In-site API reference (generated by `cargo doc` at deploy time); stays in
    // the same tab and has no path prefix so it works under Pages.
    const api = page.locator('[data-testid="nav-api"]');
    await expect(api).toHaveAttribute('href', 'api/');
    await expect(api).not.toHaveAttribute('target', '_blank');

    // Prose docs live in the repo as Markdown — they link out to GitHub in a
    // new, safe tab.
    const cases = page.locator('[data-testid="nav-cases"]');
    await expect(cases).toHaveAttribute('href', /\/tree\/main\/docs\/case-studies$/);
    await expect(cases).toHaveAttribute('target', '_blank');
    await expect(cases).toHaveAttribute('rel', 'noopener noreferrer');

    await expect(page.locator('[data-testid="nav-journeys"]')).toHaveAttribute(
      'href',
      /\/blob\/main\/docs\/USER-JOURNEYS\.md$/,
    );
    await expect(page.locator('[data-testid="nav-contributing"]')).toHaveAttribute(
      'href',
      /\/blob\/main\/CONTRIBUTING\.md$/,
    );

    // The brand returns to the site root (one level up from /docs/).
    await expect(page.locator('[data-testid="brand-home"]')).toHaveAttribute('href', '../');
  });

  test('localizes the documentation heading for every supported UI language', async ({ page }) => {
    await expect(page.locator('.hero h1')).toHaveText('Documentation');

    for (const [locale, heading] of [
      ['ru', 'Документация'],
      ['zh', '文档'],
      ['hi', 'दस्तावेज़'],
    ]) {
      await page.locator(`.locale-switch button[data-value="${locale}"]`).click();
      await expect(page.locator('.hero h1')).toHaveText(heading);
      await expect(page.locator('html')).toHaveAttribute('lang', locale);
    }
  });
});
