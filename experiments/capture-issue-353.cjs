// Capture issue-353 PR screenshots: the same committed web chat running inside a
// VS Code Webview through the `window.FormalAiDesktop` bridge. We reuse the exact
// injection the e2e spec (tests/e2e/tests/issue-353.spec.js) verifies — a fake
// bridge whose getStatus() returns a host status — so the screenshots show the
// real surface labelling, not a mock-up.
//
// Run (from the repo root, with the static server already serving src/web):
//   NODE_PATH=tests/e2e/node_modules node experiments/capture-issue-353.cjs http://localhost:3458
//
// Output: docs/screenshots/issue-353/*.png

const path = require('node:path');
const { chromium } = require('@playwright/test');

const BASE_URL = process.argv[2] || 'http://localhost:3458';
const OUT_DIR = path.resolve(__dirname, '..', 'docs', 'screenshots', 'issue-353');

// Preferences block identical to the spec's pinPreferences, plus an explicit
// theme so light/dark renders are deterministic instead of following the host.
function preferences(theme) {
  return [
    'demo_preferences',
    '  demoMode "off"',
    '  greetingVariations "off"',
    '  uiLanguage "en"',
    `  theme "${theme}"`,
  ].join('\n');
}

const NODE_HOST_STATUS = {
  shell: 'VS Code',
  apiBase: 'http://127.0.0.1:18080',
  staticBase: '',
  graphUrl: 'http://127.0.0.1:18080/v1/graph',
  traceUrl: 'http://127.0.0.1:18080/v1/graph?trace=answer_greeting_hi',
  memory: 'formal_ai_bundle',
  agentModeDefault: false,
  toolCallPolicy: 'explicit-permission',
  apiReady: true,
};

const WEB_HOST_STATUS = {
  shell: 'VS Code Web',
  apiBase: '',
  graphUrl: '',
  traceUrl: '',
  memory: 'formal_ai_bundle',
  agentModeDefault: false,
  toolCallPolicy: 'explicit-permission',
  apiReady: false,
};

async function boot(browser, status, theme) {
  // A fresh context per scenario keeps localStorage (e.g. an agent opt-in) from
  // bleeding between the Node and Web host shots.
  const context = await browser.newContext({
    viewport: { width: 1280, height: 860 },
    deviceScaleFactor: 2,
    colorScheme: theme,
  });
  const page = await context.newPage();
  await page.addInitScript((prefs) => {
    try {
      window.localStorage.setItem('formal-ai.preferences.v1', prefs);
    } catch (_error) {
      /* hardened contexts */
    }
  }, preferences(theme));
  await page.addInitScript((injected) => {
    window.FormalAiDesktop = { getStatus: async () => injected };
  }, status);
  await page.goto(`${BASE_URL}/`);
  await page.locator('.app').waitFor({ state: 'visible', timeout: 15000 });
  await page.locator('[data-testid="desktop-shell-status"]').waitFor({ timeout: 15000 });
  return { context, page };
}

async function shot(page, name) {
  const file = path.join(OUT_DIR, `${name}.png`);
  await page.screenshot({ path: file });
  // eslint-disable-next-line no-console
  console.log(`wrote ${path.relative(path.resolve(__dirname, '..'), file)}`);
}

async function shotLocator(locator, name) {
  const file = path.join(OUT_DIR, `${name}.png`);
  await locator.screenshot({ path: file });
  // eslint-disable-next-line no-console
  console.log(`wrote ${path.relative(path.resolve(__dirname, '..'), file)}`);
}

// Collapse every expanded sidebar accordion section except VS CODE so its panel
// (api base, network link, memory bundle, agent / tool permission) gets the full
// height and renders without the accordion clipping it.
async function focusVsCodeSection(page) {
  const labels = await page
    .locator('.app aside button[aria-expanded="true"]')
    .evaluateAll((els) => els.map((el) => (el.textContent || '').replace(/^[▼▶▾▸]\s*/, '').trim()));
  for (const label of labels) {
    if (/^VS Code$/i.test(label)) continue;
    await page
      .locator('.app aside button[aria-expanded="true"]', { hasText: new RegExp(`^[▼▶▾▸]?\\s*${label}$`) })
      .first()
      .click();
  }
}

async function main() {
  const browser = await chromium.launch();
  try {
    for (const theme of ['light', 'dark']) {
      // Node host: ready local server → topbar "VS Code - API local", sidebar
      // panel shows the raw host string "VS Code".
      const node = await boot(browser, NODE_HOST_STATUS, theme);
      await node.page
        .getByTestId('desktop-shell-status')
        .filter({ hasText: 'VS Code - API local' })
        .waitFor({ timeout: 15000 });
      await shot(node.page, `vscode-node-host-${theme}`);

      // Opt in to agent mode so the permission panel flips to "Agent tools visible".
      await node.page.getByTestId('agent-toggle').click();
      await node.page
        .getByTestId('desktop-tool-permission')
        .filter({ hasText: 'Agent tools visible' })
        .waitFor({ timeout: 15000 });
      await shot(node.page, `vscode-node-host-agent-opted-in-${theme}`);
      await node.context.close();

      // Web host (vscode.dev): no server → topbar "VS Code - in-process"; the
      // sidebar panel shows the precise host "VS Code Web".
      const web = await boot(browser, WEB_HOST_STATUS, theme);
      await web.page
        .getByTestId('desktop-shell-status')
        .filter({ hasText: 'VS Code - in-process' })
        .waitFor({ timeout: 15000 });
      await web.page
        .getByTestId('desktop-shell-panel')
        .filter({ hasText: 'VS Code Web' })
        .waitFor({ timeout: 15000 });
      await shot(web.page, `vscode-web-host-${theme}`);
      await web.context.close();

      // Focused VS CODE panel (Node host, default permission-gated state): the
      // full integration surface — API base, network link, memory bundle, and
      // the agent / tool permission rows — with sibling sections collapsed.
      const panel = await boot(browser, NODE_HOST_STATUS, theme);
      await panel.page
        .getByTestId('desktop-api-base')
        .filter({ hasText: '127.0.0.1:18080' })
        .waitFor({ timeout: 15000 });
      await focusVsCodeSection(panel.page);
      await shotLocator(panel.page.getByTestId('sidebar-desktop'), `vscode-desktop-panel-${theme}`);
      await panel.context.close();
    }
  } finally {
    await browser.close();
  }
}

main().catch((error) => {
  // eslint-disable-next-line no-console
  console.error(error);
  process.exit(1);
});
