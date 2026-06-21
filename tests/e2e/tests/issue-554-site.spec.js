// @ts-check
//
// Issue #554: every interface gets a dedicated landing page with copy-paste
// install instructions, reachable from the site chooser:
//
//   /vscode/     VS Code extension — manual .vsix / one-liner / desktop one-click
//   /cli/        the command-line tool — Cargo / installer / usage
//   /telegram/   the Telegram bot — install / run / webhook
//
// All three are rendered by the shared src/web/site-chrome.js, the same module
// the #479 landing/docs pages use. These specs prove each page boots, renders
// its copy-paste install sections (the `info-sections` block with the expected
// `section-<id>`, `command-<testid>` and `copy-<testid>` controls), exposes its
// cross-links as nav cards with the correct relative hrefs (so the GitHub Pages
// path prefix is preserved), and localizes via the shared
// formal-ai.preferences.v1 store. Navigation is written relative to the /app/
// baseURL (../vscode/ …) so the identical specs run against the local server
// *and* the Pages path prefix (…/formal-ai/).
const { test, expect } = require('@playwright/test');

// Seed the shared preference store (Links-Notation format, identical to
// issue-479-site.spec.js) so the rendered locale never depends on the CI
// browser's Accept-Language header.
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

test.describe('Issue #554 — VS Code extension page (/vscode/)', () => {
  test.beforeEach(async ({ page }) => {
    await page.addInitScript(seedPreferences, { theme: 'light', locale: 'en' });
    await page.goto('../vscode/');
    await expect(page.locator('[data-testid="vscode-app-summary"]')).toBeVisible({
      timeout: 15_000,
    });
  });

  test('renders the heading, eyebrow and summary', async ({ page }) => {
    await expect(page.locator('.hero h1')).toHaveText('formal-ai for VS Code');
    await expect(page.locator('[data-testid="vscode-app-summary"]')).toContainText(
      'not on the Marketplace yet',
    );
  });

  test('renders the install sections with copy-paste commands', async ({ page }) => {
    await expect(page.locator('[data-testid="info-sections"]')).toBeVisible();

    // The four documented flows: quick one-liner, manual .vsix, desktop
    // one-click and the web-host caveat.
    for (const id of ['quick', 'manual', 'desktop', 'web']) {
      await expect(page.locator(`[data-testid="section-${id}"]`)).toBeVisible();
    }

    // The quick-install one-liners — the *only* install path while the
    // extension is off the Marketplace (R3) — are shown verbatim with a copy
    // button each.
    const curl = page.locator('[data-testid="command-vscode-curl"]');
    await expect(curl).toContainText('curl -fsSL');
    await expect(curl).toContainText('install.sh');
    await expect(curl).toContainText('vscode');
    await expect(page.locator('[data-testid="copy-vscode-curl"]')).toBeVisible();

    await expect(page.locator('[data-testid="command-vscode-ps"]')).toContainText('irm');
    await expect(page.locator('[data-testid="copy-vscode-ps"]')).toBeVisible();

    // The "VS Code Extension only" manual flow renders its ordered steps plus
    // the `code --install-extension` command.
    await expect(
      page.locator('[data-testid="section-manual"] .info-steps li'),
    ).toHaveCount(4);
    await expect(page.locator('[data-testid="command-vscode-manual"]')).toContainText(
      'code --install-extension',
    );
  });

  test('links straight to the latest release and the raw install.sh (R3)', async ({ page }) => {
    const release = page.locator('[data-testid="vscode-release"]');
    await expect(release).toHaveAttribute(
      'href',
      'https://github.com/link-assistant/formal-ai/releases/latest',
    );
    await expect(release).toHaveAttribute('target', '_blank');
    await expect(release).toHaveAttribute('rel', 'noopener noreferrer');

    const raw = page.locator('[data-testid="vscode-raw"]');
    await expect(raw).toHaveAttribute(
      'href',
      'https://raw.githubusercontent.com/link-assistant/formal-ai/main/scripts/install.sh',
    );
    await expect(raw).toHaveAttribute('target', '_blank');

    // The desktop one-click flow (R2) points back to the download page.
    await expect(page.locator('[data-testid="vscode-desktop-link"]')).toHaveAttribute(
      'href',
      '../download/',
    );
  });

  test('renders the cross-link nav cards with in-site relative hrefs', async ({ page }) => {
    const cards = page.locator('[data-testid="nav-cards"] .nav-card');
    await expect(cards).toHaveCount(2);

    await expect(page.locator('[data-testid="nav-download"]')).toHaveAttribute('href', '../download/');
    await expect(page.locator('[data-testid="nav-docs"]')).toHaveAttribute('href', '../docs/');
    // In-site cards stay in the same tab so theme/locale carry over.
    await expect(page.locator('[data-testid="nav-download"]')).not.toHaveAttribute('target', '_blank');
    await expect(page.locator('[data-testid="brand-home"]')).toHaveAttribute('href', '../');
  });

  test('localizes the heading for every supported UI language', async ({ page }) => {
    await expect(page.locator('.hero h1')).toHaveText('formal-ai for VS Code');

    for (const [locale, heading] of [
      ['ru', 'formal-ai для VS Code'],
      ['zh', 'VS Code 版 formal-ai'],
      ['hi', 'VS Code के लिए formal-ai'],
    ]) {
      await page.locator(`.locale-switch button[data-value="${locale}"]`).click();
      await expect(page.locator('.hero h1')).toHaveText(heading);
      await expect(page.locator('html')).toHaveAttribute('lang', locale);
    }

    // The chosen locale is written to the shared store, so a reload keeps Hindi.
    await page.reload();
    await expect(page.locator('.hero h1')).toHaveText('VS Code के लिए formal-ai');
    await expect(page.locator('html')).toHaveAttribute('lang', 'hi');
  });
});

test.describe('Issue #554 — CLI page (/cli/)', () => {
  test.beforeEach(async ({ page }) => {
    await page.addInitScript(seedPreferences, { theme: 'light', locale: 'en' });
    await page.goto('../cli/');
    await expect(page.locator('[data-testid="cli-app-summary"]')).toBeVisible({
      timeout: 15_000,
    });
  });

  test('renders the heading and install/usage sections', async ({ page }) => {
    await expect(page.locator('.hero h1')).toHaveText('formal-ai CLI');
    await expect(page.locator('[data-testid="info-sections"]')).toBeVisible();

    for (const id of ['cargo', 'installer', 'usage']) {
      await expect(page.locator(`[data-testid="section-${id}"]`)).toBeVisible();
    }

    // The Cargo install, the universal one-liner (R4) and the usage commands
    // are all shown with copy buttons.
    await expect(page.locator('[data-testid="command-cli-cargo"]')).toContainText('cargo install');
    await expect(page.locator('[data-testid="copy-cli-cargo"]')).toBeVisible();
    await expect(page.locator('[data-testid="command-cli-curl"]')).toContainText('curl -fsSL');
    await expect(page.locator('[data-testid="command-cli-ps"]')).toContainText('irm');
    await expect(page.locator('[data-testid="command-cli-help"]')).toBeVisible();
    await expect(page.locator('[data-testid="command-cli-serve"]')).toContainText('serve');

    // Direct link to the raw installer (R3).
    await expect(page.locator('[data-testid="cli-raw"]')).toHaveAttribute(
      'href',
      'https://raw.githubusercontent.com/link-assistant/formal-ai/main/scripts/install.sh',
    );
  });

  test('renders the cross-link nav cards with in-site relative hrefs', async ({ page }) => {
    const cards = page.locator('[data-testid="nav-cards"] .nav-card');
    await expect(cards).toHaveCount(3);

    await expect(page.locator('[data-testid="nav-download"]')).toHaveAttribute('href', '../download/');
    await expect(page.locator('[data-testid="nav-vscode"]')).toHaveAttribute('href', '../vscode/');
    await expect(page.locator('[data-testid="nav-docs"]')).toHaveAttribute('href', '../docs/');
    await expect(page.locator('[data-testid="brand-home"]')).toHaveAttribute('href', '../');
  });

  test('localizes the eyebrow for every supported UI language', async ({ page }) => {
    // The CLI heading is identical across most locales, so assert the eyebrow,
    // which is translated in each — covering all four supported languages.
    await expect(page.locator('.hero .eyebrow')).toHaveText('Command-line interface');

    for (const [locale, eyebrow] of [
      ['ru', 'Интерфейс командной строки'],
      ['zh', '命令行界面'],
      ['hi', 'कमांड-लाइन इंटरफ़ेस'],
    ]) {
      await page.locator(`.locale-switch button[data-value="${locale}"]`).click();
      await expect(page.locator('.hero .eyebrow')).toHaveText(eyebrow);
      await expect(page.locator('html')).toHaveAttribute('lang', locale);
    }

    await page.reload();
    await expect(page.locator('.hero .eyebrow')).toHaveText('कमांड-लाइन इंटरफ़ेस');
    await expect(page.locator('html')).toHaveAttribute('lang', 'hi');
  });
});

test.describe('Issue #554 — Telegram bot page (/telegram/)', () => {
  test.beforeEach(async ({ page }) => {
    await page.addInitScript(seedPreferences, { theme: 'light', locale: 'en' });
    await page.goto('../telegram/');
    await expect(page.locator('[data-testid="telegram-app-summary"]')).toBeVisible({
      timeout: 15_000,
    });
  });

  test('renders the heading and install/run/webhook sections', async ({ page }) => {
    await expect(page.locator('.hero h1')).toHaveText('formal-ai Telegram bot');
    await expect(page.locator('[data-testid="info-sections"]')).toBeVisible();

    for (const id of ['install', 'run', 'webhook']) {
      await expect(page.locator(`[data-testid="section-${id}"]`)).toBeVisible();
    }

    await expect(page.locator('[data-testid="command-telegram-cargo"]')).toContainText('cargo install');
    await expect(page.locator('[data-testid="command-telegram-curl"]')).toContainText('curl -fsSL');
    await expect(page.locator('[data-testid="command-telegram-ps"]')).toContainText('irm');
    await expect(page.locator('[data-testid="command-telegram-run"]')).toBeVisible();
    await expect(page.locator('[data-testid="command-telegram-webhook"]')).toBeVisible();
    await expect(page.locator('[data-testid="copy-telegram-run"]')).toBeVisible();

    // Direct link to the raw installer (R3).
    await expect(page.locator('[data-testid="telegram-raw"]')).toHaveAttribute(
      'href',
      'https://raw.githubusercontent.com/link-assistant/formal-ai/main/scripts/install.sh',
    );
  });

  test('renders the cross-link nav cards with in-site relative hrefs', async ({ page }) => {
    const cards = page.locator('[data-testid="nav-cards"] .nav-card');
    await expect(cards).toHaveCount(3);

    await expect(page.locator('[data-testid="nav-cli"]')).toHaveAttribute('href', '../cli/');
    await expect(page.locator('[data-testid="nav-download"]')).toHaveAttribute('href', '../download/');
    await expect(page.locator('[data-testid="nav-docs"]')).toHaveAttribute('href', '../docs/');
    await expect(page.locator('[data-testid="brand-home"]')).toHaveAttribute('href', '../');
  });

  test('localizes the heading for every supported UI language', async ({ page }) => {
    await expect(page.locator('.hero h1')).toHaveText('formal-ai Telegram bot');

    for (const [locale, heading] of [
      ['ru', 'Telegram-бот formal-ai'],
      ['zh', 'formal-ai Telegram 机器人'],
      ['hi', 'formal-ai Telegram बॉट'],
    ]) {
      await page.locator(`.locale-switch button[data-value="${locale}"]`).click();
      await expect(page.locator('.hero h1')).toHaveText(heading);
      await expect(page.locator('html')).toHaveAttribute('lang', locale);
    }

    await page.reload();
    await expect(page.locator('.hero h1')).toHaveText('formal-ai Telegram बॉट');
    await expect(page.locator('html')).toHaveAttribute('lang', 'hi');
  });
});
