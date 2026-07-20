// @ts-check
// Issue #672 (F1): "Snapshot-based dark-theme regression coverage".
//
// Background. Issue #541 (R1) fixed a family of widgets that kept their light
// palette after switching to dark mode. Its spec (`issue-541-theme.spec.js`)
// asserts colour *bands* on two anchor widgets only — `mode-status` and
// `sidebar-toggle.is-collapsed`. The other three widgets named in the R1 fix
// (`drawer-menu-section h2`, `.tool-mode`, `.tool-mode-agent .tool-mode`) were
// left to visual inspection of the PNGs under
// `docs/case-studies/issue-541/assets/`, and the follow-up F1 in
// `docs/case-studies/issue-541/proposed-issues.md` asked for a snapshot-based
// regression mode covering the full widget set under light + dark + auto.
//
// What this spec locks down. For every theme preference (`light`, `dark`,
// `auto` under both OS colour schemes) and for both surfaces (plain web, and
// the desktop shell — the desktop app loads this same `/app/index.html` with a
// `FormalAiDesktop` bridge attached), it reads the *computed* colour of each of
// the five widget classes and snapshots the whole table as text. Any future CSS
// change that moves one of those colours — including the specific regression
// R1 fixed, a light value bleeding through under `data-theme="dark"` — fails
// the run and prints a readable diff of exactly which widget moved.
//
// Why a computed-style snapshot instead of `toHaveScreenshot()`. F1's original
// sketch proposed pixel baselines with a 5 % threshold, and explicitly named
// the reason it was not shipped in PR #542: "snapshot baselines add ~100 KB of
// PNGs to the repo and a flaky-screenshot blast radius into CI". That trade-off
// has not changed — pixel baselines captured on a contributor's machine
// disagree with CI over font hinting and subpixel AA, which is exactly the
// class of flake `playwright.local.config.js` is tuned to avoid. A computed
// colour table is the same regression guard for the bug F1 cares about (a
// widget rendering in the wrong theme's palette), is byte-stable across
// machines, and diffs in review as text rather than as an opaque binary. The
// pixel view is not lost either: `theme-gallery.spec.js`-style full-page PNGs
// are still written to `docs/screenshots/issue-672/` for human review (see the
// last test in this file), they are just not *asserted* on.
//
// See `docs/case-studies/issue-672/README.md` for the full reconciliation.

const fs = require('node:fs');
const path = require('node:path');
const { test, expect } = require('@playwright/test');

const PREF_KEY = 'formal-ai.preferences.v1';

// The five widget classes the R1 fix touched. Each entry names the properties
// that carry the theme — asserting `color` on a badge whose bug was a
// background would silently pass, so the property list is per-widget and
// deliberate.
const THEMED_WIDGETS = [
  {
    id: 'topbar-mode-status',
    selector: '[data-testid="mode-status"]',
    properties: ['color'],
  },
  {
    id: 'sidebar-toggle',
    selector: '[data-testid="sidebar-toggle"]',
    properties: ['color', 'backgroundColor'],
  },
  {
    id: 'drawer-menu-section-heading',
    selector: '.drawer-menu-section h2',
    properties: ['color'],
  },
  {
    id: 'tool-mode-thinking',
    selector: '.tool:not(.tool-mode-agent) .tool-mode',
    properties: ['color', 'backgroundColor'],
  },
  {
    id: 'tool-mode-agent',
    selector: '.tool-mode-agent .tool-mode',
    properties: ['color', 'backgroundColor'],
  },
];

function preferencesWithTheme(theme) {
  return [
    'demo_preferences',
    '  demoMode "off"',
    '  greetingVariations "off"',
    '  diagnosticsMode "off"',
    '  uiLanguage "en"',
    `  theme "${theme}"`,
  ].join('\n');
}

// The desktop shell renders the same bundle behind a `FormalAiDesktop` bridge,
// which unlocks the desktop-only sidebar section and permission panel. Stubbing
// the bridge is how every other desktop spec in this suite reaches that surface
// (see `issue-541-permissions.spec.js`); the snapshot only reads colours, so a
// status-only stub is enough.
const DESKTOP_STATUS = {
  shell: 'Electron',
  apiBase: '',
  staticBase: '',
  graphUrl: '',
  traceUrl: '',
  memory: 'formal_ai_bundle',
  agentModeDefault: false,
  toolCallPolicy: 'explicit-permission',
  apiReady: false,
};

async function bootThemeSurface(page, { theme, surface, colorScheme }) {
  await page.emulateMedia({ colorScheme });
  await page.addInitScript(
    ({ prefKey, preferences, desktop, status }) => {
      try {
        window.localStorage.setItem(prefKey, preferences);
      } catch (_error) {
        // localStorage can be unavailable in hardened browser contexts.
      }
      if (desktop) {
        window.FormalAiDesktop = {
          getStatus: async () => status,
          ensureAgentServer: async () => status,
          setToolGrants: async (grants) => ({ ...(grants || {}) }),
          invokeTool: async () => ({ ok: false, executed: false }),
          runAgentProvider: async () => ({ ok: false, executed: false }),
        };
      }
    },
    {
      prefKey: PREF_KEY,
      preferences: preferencesWithTheme(theme),
      desktop: surface === 'desktop',
      status: DESKTOP_STATUS,
    },
  );
  await page.goto('./');
  await expect(page.locator('.app')).toBeVisible({ timeout: 15_000 });

  // The boot effect mirrors the stored theme onto <html data-theme>; `auto`
  // removes the attribute and lets the media query decide. Waiting on the
  // resolved state guarantees the snapshot measures the post-mirror colours
  // rather than the CSS default.
  if (theme === 'auto') {
    await expect(page.locator('html')).not.toHaveAttribute('data-theme', /.*/, {
      timeout: 10_000,
    });
  } else {
    await expect(page.locator('html')).toHaveAttribute('data-theme', theme, {
      timeout: 10_000,
    });
  }
  // The tools sidebar section holds the two `.tool-mode` badges and is
  // populated from the seed bundle, which loads asynchronously.
  await expect(page.locator('.tool-mode-agent .tool-mode').first()).toHaveCount(
    1,
    { timeout: 15_000 },
  );
}

// Read the computed value of each themed property. Returns a stable, sorted
// text table so the committed baseline diffs line-by-line.
async function readThemeTable(page) {
  const rows = await page.evaluate((widgets) => {
    return widgets.map((widget) => {
      const node = document.querySelector(widget.selector);
      if (!node) {
        return { id: widget.id, values: ['(missing)'] };
      }
      const computed = getComputedStyle(node);
      return {
        id: widget.id,
        values: widget.properties.map(
          (property) => `${property}=${computed[property]}`,
        ),
      };
    });
  }, THEMED_WIDGETS);

  const missing = rows.filter((row) => row.values[0] === '(missing)');
  // A selector that stopped matching would otherwise snapshot as a stable
  // "(missing)" row and quietly stop guarding its widget forever.
  expect(
    missing.map((row) => row.id),
    'every themed widget must be present in the DOM',
  ).toEqual([]);

  return rows
    .map((row) => `${row.id}: ${row.values.join(' ')}`)
    .sort()
    .join('\n');
}

// The `sidebar-toggle` bug R1 fixed only shows in the `.is-collapsed` variant,
// which the boot state never reaches on its own.
async function collapseSidebar(page) {
  const toggle = page.locator('[data-testid="sidebar-toggle"]');
  await expect(toggle).toBeVisible();
  await toggle.click();
  await expect(toggle).toHaveClass(/is-collapsed/);
}

const SURFACES = ['web', 'desktop'];
const THEME_CASES = [
  { theme: 'light', colorScheme: 'light', label: 'light' },
  { theme: 'dark', colorScheme: 'light', label: 'dark' },
  // `auto` must follow the OS: the same preference has to produce the light
  // table under a light OS and the dark table under a dark one.
  { theme: 'auto', colorScheme: 'light', label: 'auto-os-light' },
  { theme: 'auto', colorScheme: 'dark', label: 'auto-os-dark' },
];

test.describe('Issue #672 (F1): theme regression — full widget set', () => {
  for (const surface of SURFACES) {
    for (const { theme, colorScheme, label } of THEME_CASES) {
      test(`${surface} surface, ${label} theme matches the committed colour table`, async ({
        page,
      }) => {
        await bootThemeSurface(page, { theme, surface, colorScheme });
        await collapseSidebar(page);
        const table = await readThemeTable(page);
        expect(table).toMatchSnapshot(`${surface}-${label}.txt`);
      });
    }
  }

  // A pure-CSS guard that does not depend on the baseline: whatever the exact
  // palette is, the dark table must never equal the light one. If a future
  // regression drops `:root[data-theme="dark"]` rules altogether, the baseline
  // could be regenerated away — this assertion cannot be.
  test('dark and light tables differ on every themed widget', async ({ page, context }) => {
    await bootThemeSurface(page, { theme: 'light', surface: 'web', colorScheme: 'light' });
    await collapseSidebar(page);
    const light = await readThemeTable(page);

    const darkPage = await context.newPage();
    await bootThemeSurface(darkPage, { theme: 'dark', surface: 'web', colorScheme: 'light' });
    await collapseSidebar(darkPage);
    const dark = await readThemeTable(darkPage);
    await darkPage.close();

    const lightRows = light.split('\n');
    const darkRows = dark.split('\n');
    expect(darkRows).toHaveLength(lightRows.length);
    for (let index = 0; index < lightRows.length; index += 1) {
      expect(
        darkRows[index],
        `widget row must change between themes: ${lightRows[index]}`,
      ).not.toEqual(lightRows[index]);
    }
  });

  // Human-review artefacts. Not asserted (that is the flake trade-off F1 named)
  // but regenerated on every run and uploaded by CI, so a reviewer can see the
  // dark surfaces the colour table describes.
  test('writes dark-theme review screenshots for both surfaces', async ({ page }) => {
    const outDir = path.resolve(__dirname, '../../../docs/screenshots/issue-672');
    fs.mkdirSync(outDir, { recursive: true });
    for (const surface of SURFACES) {
      await bootThemeSurface(page, {
        theme: 'dark',
        surface,
        colorScheme: 'light',
      });
      await page.screenshot({
        path: path.join(outDir, `theme-dark-${surface}.png`),
        fullPage: false,
      });
    }
  });
});
