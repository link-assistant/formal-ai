// @ts-check
const { test, expect } = require('@playwright/test');

const searchPrompts = [
  {
    language: 'en',
    prompt: 'search the web for formal ai',
    query: 'formal ai',
  },
  { language: 'ru', prompt: 'найди formal ai в интернете', query: 'formal ai' },
  { language: 'hi', prompt: 'सेब के बारे में इंटरनेट पर खोजो', query: 'सेब' },
  { language: 'zh', prompt: '查找苹果网上信息', query: '苹果' },
];

test('multilingual desktop web search works with agent permission off and no local API', async ({
  page,
}) => {
  await page.addInitScript(() => {
    window.localStorage.setItem(
      'formal-ai.preferences.v1',
      'demo_preferences\n  demoMode "off"\n  greetingVariations "off"\n  diagnosticsMode "off"\n  uiLanguage "en"',
    );
    window.__issue747Calls = [];
    window.FormalAiDesktop = {
      getStatus: async () => ({
        shell: 'Electron',
        apiBase: '',
        staticBase: '',
        graphUrl: '',
        traceUrl: '',
        memory: 'formal_ai_bundle',
        agentModeDefault: false,
        toolCallPolicy: 'explicit-permission',
        apiReady: false,
      }),
      setToolGrants: async () => ({}),
      invokeTool: async (request) => {
        window.__issue747Calls.push(request);
        return {
          ok: true,
          tool: request.tool,
          status: 'ok',
          executed: true,
          servedBy: 'web-capture',
          body: 'Fused Google, Bing, and DuckDuckGo results',
          results: [],
        };
      },
    };
  });

  await page.goto('./');
  const input = page.locator('[data-testid="chat-composer-input"]');
  await expect(input).toBeEnabled({ timeout: 10_000 });
  const messages = page.locator('[data-testid="chat-message"]');
  for (const { prompt } of searchPrompts) {
    await input.fill(prompt);
    await page.locator('[data-testid="chat-composer-submit"]').click();
    await expect(messages.last()).toContainText(
      'Fused Google, Bing, and DuckDuckGo results',
      {
        timeout: 20_000,
      },
    );
  }

  await expect
    .poll(() => page.evaluate(() => window.__issue747Calls.length))
    .toBe(4);
  const calls = await page.evaluate(() => window.__issue747Calls);
  expect(calls.map((call) => call.tool)).toEqual(
    searchPrompts.map(() => 'web_search'),
  );
  expect(calls.map((call) => call.input.language)).toEqual(
    searchPrompts.map(({ language }) => language),
  );
  expect(calls.map((call) => call.input.query)).toEqual(
    searchPrompts.map(({ query }) => query),
  );
  await expect(
    page.locator('[data-testid="desktop-agent-permission"]'),
  ).toHaveText('Off');
});
