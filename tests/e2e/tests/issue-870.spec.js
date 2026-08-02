// @ts-check
const { test, expect } = require('@playwright/test');

const REPORTED_PROMPT = 'Проверь какие процессы запущены на моём компьютере';
const UNKNOWN_PROMPT = 'zyxqv glorbax ritual';
const PREF_KEY = 'formal-ai.preferences.v1';

function preferences({ diagnostics = false } = {}) {
  return [
    'demo_preferences',
    '  demoMode "off"',
    '  greetingVariations "off"',
    `  diagnosticsMode "${diagnostics ? 'on' : 'off'}"`,
    '  uiLanguage "en"',
  ].join('\n');
}

async function installDesktopBridge(page, shell) {
  await page.addInitScript(
    ({ prefKey, preferencesText, shellName }) => {
      window.localStorage.setItem(prefKey, preferencesText);
      window.__agentProviderCalls = [];
      window.__toolGrants = {};
      const status = {
        shell: shellName,
        platform: 'win32',
        apiBase: '',
        staticBase: '',
        graphUrl: '',
        traceUrl: '',
        memory: 'formal_ai_bundle',
        agentModeDefault: false,
        toolCallPolicy: 'explicit-permission',
        apiReady: false,
      };
      window.FormalAiDesktop = {
        getStatus: async () => status,
        ensureAgentServer: async () => status,
        setToolGrants: async (grants) => {
          window.__toolGrants = { ...(grants || {}) };
          return window.__toolGrants;
        },
        runAgentProvider: async (request) => {
          window.__agentProviderCalls.push(request);
          return {
            ok: false,
            provider: 'commander',
            status: 'unavailable',
            executed: false,
            reason: 'no provider configured in test',
          };
        },
      };
    },
    {
      prefKey: PREF_KEY,
      preferencesText: preferences(),
      shellName: shell,
    },
  );
}

async function boot(page) {
  await page.goto('./');
  await expect(page.locator('.app')).toBeVisible({ timeout: 15_000 });
  await expect(page.locator('[data-testid="chat-composer-input"]')).toBeEnabled({
    timeout: 10_000,
  });
}

async function sendPrompt(page, text) {
  const input = page.locator('[data-testid="chat-composer-input"]');
  const assistants = page.locator('[data-testid="chat-message"].assistant');
  const initial = await assistants.count();
  await expect(input).toBeEnabled({ timeout: 10_000 });
  await input.fill(text);
  await page.locator('[data-testid="chat-composer-submit"]').click();
  await expect
    .poll(() => assistants.count(), { timeout: 20_000 })
    .toBeGreaterThan(initial);
  await expect(input).toBeEnabled({ timeout: 20_000 });
  return assistants.last();
}

for (const surface of ['Electron', 'VS Code']) {
  test(`${surface} routes the reported Russian request through permission and tasklist`, async ({
    page,
  }) => {
    await installDesktopBridge(page, surface);
    await boot(page);

    const answer = await sendPrompt(page, REPORTED_PROMPT);
    await expect(answer).toContainText('ps');
    await expect(answer).not.toContainText('That one is new to me');

    const cta = page.locator(
      '[data-testid="desktop-permission-panel-message-grant-all"]',
    );
    await expect(cta).toBeVisible();
    await expect(cta).toHaveAttribute('data-has-pending-task', 'true');
    expect(await page.evaluate(() => window.__agentProviderCalls.length)).toBe(0);

    await cta.click();
    await expect
      .poll(() => page.evaluate(() => window.__agentProviderCalls.length))
      .toBeGreaterThanOrEqual(1);
    const request = await page.evaluate(
      () => window.__agentProviderCalls[window.__agentProviderCalls.length - 1],
    );
    expect(request.command).toBe('tasklist');
    expect(request.tool).toBe('shell');
    expect(request.mode).toBe('agent');
    expect(request.grants.shell).toBe(true);
  });
}

async function mockEmptyRegularSearchProviders(page) {
  await page.route('**://api.duckduckgo.com/**', (route) =>
    route.fulfill({ status: 200, contentType: 'application/json', body: '{}' }),
  );
  await page.route('**://archive.org/advancedsearch.php**', (route) =>
    route.fulfill({
      status: 200,
      contentType: 'application/json',
      body: JSON.stringify({ response: { docs: [] } }),
    }),
  );
  await page.route('**://*.wikipedia.org/w/rest.php/v1/search/page**', (route) =>
    route.fulfill({
      status: 200,
      contentType: 'application/json',
      body: JSON.stringify({ pages: [] }),
    }),
  );
  await page.route('**://www.wikidata.org/w/api.php**', (route) =>
    route.fulfill({
      status: 200,
      contentType: 'application/json',
      body: JSON.stringify({ search: [] }),
    }),
  );
  await page.route('**://*.wiktionary.org/w/api.php**', (route) =>
    route.fulfill({
      status: 200,
      contentType: 'application/json',
      body: JSON.stringify([UNKNOWN_PROMPT, [], [], []]),
    }),
  );
  await page.route('**://*.wikinews.org/w/api.php**', (route) =>
    route.fulfill({
      status: 200,
      contentType: 'application/json',
      body: JSON.stringify([UNKNOWN_PROMPT, [], [], []]),
    }),
  );
}

test('unknown requests fuse trusted research and reuse the learned Links association', async ({
  page,
}) => {
  await page.addInitScript(
    ({ prefKey, preferencesText }) => {
      window.localStorage.setItem(prefKey, preferencesText);
    },
    {
      prefKey: PREF_KEY,
      preferencesText: preferences({ diagnostics: true }),
    },
  );
  await mockEmptyRegularSearchProviders(page);

  let stackExchangeCalls = 0;
  await page.route('**://api.stackexchange.com/**', async (route) => {
    stackExchangeCalls += 1;
    await route.fulfill({
      status: 200,
      contentType: 'application/json',
      body: JSON.stringify({
        items: [
          {
            title: 'Zyxqv glorbax ritual',
            link: 'https://stackoverflow.com/questions/870/example',
            body: 'A trusted explanation for the zyxqv glorbax ritual.',
          },
        ],
      }),
    });
  });
  for (const pattern of [
    '**://www.wikihow.com/api.php**',
    '**://www.wikifunctions.org/w/api.php**',
    '**://rosettacode.org/w/api.php**',
  ]) {
    await page.route(pattern, (route) =>
      route.fulfill({
        status: 200,
        contentType: 'application/json',
        body: JSON.stringify([UNKNOWN_PROMPT, [], [], []]),
      }),
    );
  }

  await boot(page);
  const researched = await sendPrompt(page, UNKNOWN_PROMPT);
  await expect(researched.locator('.intent')).toContainText('intent:web_search');
  await expect(researched.locator('.evidence-list')).toContainText(
    'web_search:query_kind:unknown_intent_research',
  );
  await expect(researched.locator('.evidence-list')).toContainText(
    'associative_research:learned:',
  );
  await expect(researched).toContainText('Zyxqv glorbax ritual');
  expect(stackExchangeCalls).toBe(1);
  const learnedEvents = await page.evaluate(() =>
    window.FormalAiMemory.listEvents(),
  );
  const learnedAssociation = learnedEvents.find(
    (event) => event && event.kind === 'associative_research',
  );
  expect(learnedAssociation).toBeTruthy();
  expect(learnedAssociation.role).toBe('system');
  expect(learnedAssociation.content).toContain('associative_research');

  const recalled = await sendPrompt(page, UNKNOWN_PROMPT);
  await expect(recalled.locator('.intent')).toContainText('intent:web_search');
  await expect(recalled.locator('.evidence-list')).toContainText(
    'associative_research:memory_hit:',
  );
  expect(stackExchangeCalls).toBe(1);
});
