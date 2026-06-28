// @ts-check
const { test, expect } = require('@playwright/test');

const EMOJI_RE = /[\u{1F300}-\u{1FAFF}\u{2600}-\u{27BF}]/u;
const ICON_PACKS = [
  'fontawesome',
  'material-symbols',
  'bootstrap-icons',
  'ionicons',
  'remix-icon',
  'tabler-icons',
  'names',
];
const LOCALE_EXPECTATIONS = [
  { language: 'en', label: 'Toolbar icons', namesOption: 'Names' },
  { language: 'ru', label: 'Иконки панели', namesOption: 'Названия' },
  { language: 'hi', label: 'टूलबार आइकन', namesOption: 'नाम' },
  { language: 'zh', label: '工具栏图标', namesOption: '名称' },
];

async function boot(page, extraPreferences = '') {
  await page.addInitScript((extra) => {
    try {
      const key = 'formal-ai.preferences.v1';
      const existing = window.localStorage.getItem(key);
      if (existing && existing.startsWith('demo_preferences')) {
        const lines = existing.split(/\r?\n/);
        for (const line of [
          '  demoMode "off"',
          '  greetingVariations "off"',
          ...String(extra).split(/\r?\n/).filter(Boolean),
        ]) {
          if (!lines.includes(line)) {
            lines.push(line);
          }
        }
        window.localStorage.setItem(key, lines.join('\n'));
        return;
      }
      window.localStorage.setItem(
        key,
        `demo_preferences
  demoMode "off"
  greetingVariations "off"${extra}`,
      );
    } catch (_error) {
      // localStorage may be unavailable in hardened browser contexts.
    }
  }, extraPreferences);
  await page.goto('./');
  await expect(page.locator('.app')).toBeVisible({ timeout: 15_000 });
  await expect(page.locator('[data-testid="demo-status"]')).toHaveText(
    'Manual mode',
  );
}

async function expandSidebarSection(page, testId) {
  const section = page.locator(`[data-testid="${testId}"]`);
  await expect(section).toBeVisible();
  if ((await section.getAttribute('data-collapsed')) === 'true') {
    await section.locator('.sidebar-section-header').click();
  }
  await expect(section).toHaveAttribute('data-collapsed', 'false');
}

test.describe('Issue #409 - toolbar icon packs', () => {
  test('toolbar and drawer default to Font Awesome metadata without emoji glyphs', async ({
    page,
  }) => {
    await boot(page);
    await expandSidebarSection(page, 'drawer-menu-actions');

    const iconMeta = await page
      .locator('.topbar-actions .btn-icon[data-icon-pack], .drawer-action .btn-icon[data-icon-pack]')
      .evaluateAll((nodes) =>
        nodes.map((node) => ({
          pack: node.getAttribute('data-icon-pack'),
          name: node.getAttribute('data-icon-font-name'),
          text: node.textContent || '',
        })),
      );

    expect(iconMeta.length).toBeGreaterThanOrEqual(16);
    expect(iconMeta.every((entry) => entry.pack === 'fontawesome')).toBe(true);
    expect(iconMeta.some((entry) => entry.name === 'fa-code')).toBe(true);
    expect(iconMeta.some((entry) => entry.name === 'fa-download')).toBe(true);
    expect(iconMeta.some((entry) => entry.name === 'fa-bug')).toBe(true);
    for (const entry of iconMeta) {
      expect(entry.text).not.toMatch(EMOJI_RE);
    }
  });

  test('icon pack setting offers popular icon fonts and persists the selected pack', async ({
    page,
  }) => {
    await boot(page);

    const select = page.locator('[data-testid="setting-toolbar-icon-pack"]');
    await expect(select).toHaveValue('fontawesome');
    const options = await select.locator('option').evaluateAll((nodes) =>
      nodes.map((node) => node.getAttribute('value')),
    );
    expect(options).toEqual(ICON_PACKS);

    await select.selectOption('tabler-icons');
    await expect(page.locator('[data-testid="source-code"] .btn-icon')).toHaveAttribute(
      'data-icon-pack',
      'tabler-icons',
    );
    await expect(page.locator('[data-testid="source-code"] .btn-icon')).toHaveAttribute(
      'data-icon-font-name',
      'IconCode',
    );

    await expect(page.locator('[data-testid="settings-reset-toolbarIconPack"]')).toBeVisible();
    await expect
      .poll(() =>
        page.evaluate(
          () => window.localStorage.getItem('formal-ai.preferences.v1') || '',
        ),
      )
      .toContain('toolbarIconPack "tabler-icons"');

    await page.reload();
    await expect(page.locator('.app')).toBeVisible({ timeout: 15_000 });
    await expect(select).toHaveValue('tabler-icons');
    await expect(page.locator('[data-testid="source-code"] .btn-icon')).toHaveAttribute(
      'data-icon-pack',
      'tabler-icons',
    );

    await page.locator('[data-testid="settings-reset-toolbarIconPack"]').click();
    await expect(select).toHaveValue('fontawesome');
    await expect(page.locator('[data-testid="source-code"] .btn-icon')).toHaveAttribute(
      'data-icon-pack',
      'fontawesome',
    );
  });

  test('toolbar icon setting label is localized across supported UI languages', async ({
    page,
  }) => {
    await boot(page);

    const languageSelect = page.locator('[data-testid="setting-ui-language"]');
    const iconSelect = page.locator('[data-testid="setting-toolbar-icon-pack"]');

    for (const { language, label, namesOption } of LOCALE_EXPECTATIONS) {
      await languageSelect.selectOption(language);
      await expect(page.locator('html')).toHaveAttribute('lang', language);
      await expect(page.getByText(label, { exact: true })).toBeVisible();
      await expect(iconSelect.locator('option[value="names"]')).toHaveText(
        namesOption,
      );
    }
  });
});
