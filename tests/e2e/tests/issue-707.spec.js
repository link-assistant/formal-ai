const { test, expect } = require('@playwright/test');
const fs = require('node:fs');
const path = require('node:path');

const REPO_ROOT = path.resolve(__dirname, '../../..');
const tasks = fs.readFileSync(
  path.join(REPO_ROOT, 'data/seed/computer-use-tasks.lino'),
  'utf8',
);
const permissions = fs.readFileSync(
  path.join(REPO_ROOT, 'src/web/i18n-catalog-permissions.lino'),
  'utf8',
);

const localeCases = [
  { locale: 'en' },
  { locale: 'ru' },
  { locale: 'hi' },
  { locale: 'zh' },
];
const computerUsePermissions = [
  ['fs.read', 'computer_fs_read'],
  ['fs.write', 'computer_fs_write'],
  ['fs.list', 'computer_fs_list'],
  ['fs.move', 'computer_fs_move'],
  ['shell.run', 'computer_shell_run'],
  ['http.fetch', 'computer_http_fetch'],
  ['http.post', 'computer_http_post'],
  ['dom.query', 'computer_dom_query'],
  ['dom.extract', 'computer_dom_extract'],
  ['archive.pack', 'computer_archive_pack'],
  ['archive.unpack', 'computer_archive_unpack'],
  ['process.status', 'computer_process_status'],
];

function localeBlock(source, locale) {
  const header = new RegExp(`^${locale}\\n`, 'm').exec(source);
  expect(header).not.toBeNull();
  const rest = source.slice(header.index + header[0].length);
  const nextHeader = /^(?:en|ru|hi|zh)\n/m.exec(rest);
  return nextHeader ? rest.slice(0, nextHeader.index) : rest;
}

for (const { locale } of localeCases) {
  test(`computer-use seed and permission UI cover ${locale}`, async () => {
    const promptPattern = new RegExp(`^    prompt ${locale} `, 'gm');
    expect(tasks.match(promptPattern)).toHaveLength(10);

    const catalog = localeBlock(permissions, locale);
    expect(catalog.match(/^      computer_[a-z_]+$/gm)).toHaveLength(12);
    expect(catalog.match(/^        label /gm).length).toBeGreaterThanOrEqual(12);
    expect(catalog.match(/^        description /gm).length).toBeGreaterThanOrEqual(12);
  });
}

test('desktop permission panel exposes the complete computer-use taxonomy', async ({ page }) => {
  await page.addInitScript(() => {
    window.localStorage.setItem(
      'formal-ai.preferences.v1',
      'demo_preferences\n  demoMode "off"\n  greetingVariations "off"\n  uiLanguage "en"',
    );
    window.FormalAiDesktop = {
      getStatus: async () => ({
        shell: 'Electron',
        apiBase: 'http://127.0.0.1:18080',
        graphUrl: 'http://127.0.0.1:18080/v1/graph',
        traceUrl: 'http://127.0.0.1:18080/v1/graph?trace=answer_greeting_hi',
        memory: 'formal_ai_bundle',
        activeEngine: 'out-of-box',
        engines: [
          { id: 'out-of-box', label: 'Out of the box', type: 'native', available: true },
        ],
        agentModeDefault: false,
        toolCallPolicy: 'explicit-permission',
        apiReady: true,
      }),
    };
  });
  await page.goto('./');
  await expect(page.locator('.app')).toBeVisible({ timeout: 15_000 });

  const panel = page.locator('[data-testid="desktop-permission-panel-sidebar"]');
  await expect(panel.locator('.permission-tool-row')).toHaveCount(18);
  await expect(page.locator('[data-testid="desktop-tool-permission"]')).toHaveText(
    '0/18 tools granted',
  );

  for (const [primitive, i18nKey] of computerUsePermissions) {
    const row = page.locator(
      `[data-testid="desktop-permission-panel-sidebar-row-${primitive}"]`,
    );
    const expectedLabel = await page.evaluate(async (key) => {
      await window.FormalAiI18n.ready;
      return window.FormalAiI18n.t(`permissions.tool.${key}.label`, 'en');
    }, i18nKey);
    await expect(row).toBeVisible();
    await expect(row.locator('strong')).toHaveText(expectedLabel);
  }
});
