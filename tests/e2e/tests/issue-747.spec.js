// @ts-check
const { test, expect } = require('@playwright/test');

test('desktop web search works with agent permission off and no local API', async ({ page }) => {
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
  await input.fill('search the web for formal ai');
  await page.locator('[data-testid="chat-composer-submit"]').click();

  const messages = page.locator('[data-testid="chat-message"]');
  await expect(messages.last()).toContainText('Fused Google, Bing, and DuckDuckGo results', {
    timeout: 20_000,
  });
  await expect.poll(() => page.evaluate(() => window.__issue747Calls.length)).toBe(1);
  expect(await page.evaluate(() => window.__issue747Calls[0])).toMatchObject({
    tool: 'web_search',
    input: { query: 'formal ai', language: 'en' },
  });
  await expect(page.locator('[data-testid="desktop-agent-permission"]')).toHaveText('Off');
});
