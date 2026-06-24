// @ts-check
const { test, expect } = require('@playwright/test');

const PREF_KEY = 'formal-ai.preferences.v1';
const supportedUiLanguages = [
  { language: 'en', name: 'English' },
  { language: 'ru', name: 'Russian' },
  { language: 'hi', name: 'Hindi' },
  { language: 'zh', name: 'Chinese' },
];

function preferencesForUiLanguage(language) {
  return [
    'demo_preferences',
    '  demoMode "off"',
    '  diagnosticsMode "on"',
    '  greetingVariations "off"',
    `  uiLanguage "${language}"`,
  ].join('\n');
}

async function sendPrompt(page, text) {
  const input = page.locator('[data-testid="chat-composer-input"]');
  const messages = page.locator('[data-testid="chat-message"]');
  const initial = await messages.count();
  await expect(input).toBeEnabled({ timeout: 5_000 });
  await input.fill(text);
  await page.locator('[data-testid="chat-composer-submit"]').click();
  await expect.poll(async () => messages.count(), { timeout: 20_000 }).toBeGreaterThan(initial);
}

async function bootIssue518(page, language) {
  await page.addInitScript(({ prefKey, preferences }) => {
    try {
      window.localStorage.setItem(prefKey, preferences);
    } catch (_error) {
      // localStorage can be unavailable in hardened browser contexts.
    }

    window.__agentProviderCalls = [];
    window.__toolInvocations = [];
    window.__toolGrants = {};
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
        agentExecutionProvider: {
          type: 'commander',
          commanderCommand: 'start-agent',
        },
      }),
      ensureAgentServer: async () => ({
        shell: 'Electron',
        apiBase: '',
        staticBase: '',
        graphUrl: '',
        traceUrl: '',
        memory: 'formal_ai_bundle',
        agentModeDefault: false,
        toolCallPolicy: 'explicit-permission',
        apiReady: false,
        agentExecutionProvider: {
          type: 'commander',
          commanderCommand: 'start-agent',
        },
      }),
      setToolGrants: async (grants) => {
        window.__toolGrants = { ...(grants || {}) };
        return window.__toolGrants;
      },
      invokeTool: async (request) => {
        window.__toolInvocations.push(request);
        return {
          ok: true,
          tool: request.tool,
          status: 'ok',
          executed: true,
          body: `direct ${request.input.command || ''}`.trim(),
        };
      },
      runAgentProvider: async (request) => {
        window.__agentProviderCalls.push(request);
        return {
          ok: true,
          provider: 'commander',
          status: 'ok',
          executed: true,
          command: 'start-agent',
          events: [
            { type: 'text', text: 'I will inspect the home directory.\n' },
            {
              type: 'tool_use',
              id: 'call_ls_home',
              tool: 'bash',
              input: { command: 'ls ~' },
            },
            {
              type: 'tool_result',
              id: 'call_ls_home',
              tool: 'bash',
              output: {
                stdout: 'Desktop\nDocuments\n',
                stderr: '',
                exitCode: 0,
                status: 'completed',
              },
            },
            { type: 'text', text: 'Desktop and Documents are present.' },
          ],
          answer: {
            intent: 'agent_cli_turn',
            source: 'agent_provider',
            content: 'I will inspect the home directory.\n\nDesktop and Documents are present.',
            evidence: ['agent_provider:ndjson', 'agent_events:4', 'provider:commander'],
            steps: [
              { step: 'agent_text', detail: 'I will inspect the home directory.' },
              { step: 'agent_tool_start', detail: 'shell: ls ~' },
              { step: 'agent_tool_result', detail: 'shell: ls ~ (exit 0)' },
            ],
            toolCalls: [
              {
                tool: 'shell',
                inputs: { command: 'ls ~' },
                outputs: {
                  stdout: 'Desktop\nDocuments\n',
                  exitCode: 0,
                  status: 'completed',
                },
              },
            ],
          },
        };
      },
    };
  }, { prefKey: PREF_KEY, preferences: preferencesForUiLanguage(language) });
  await page.goto('./');
  await expect(page.locator('.app')).toBeVisible({ timeout: 15_000 });
  await expect(page.locator('html'), `${language} UI language is active`).toHaveAttribute(
    'lang',
    language,
  );
  await expect(page.locator('[data-testid="chat-composer-input"]')).toBeEnabled({
    timeout: 10_000,
  });
}

test.describe('Issue #518: agent CLI NDJSON renders in chat UI', () => {
  for (const { language, name } of supportedUiLanguages) {
    test(`approved terminal command renders the agent provider answer and tool diagnostics in ${name}`, async ({
      page,
    }) => {
      await bootIssue518(page, language);
      await page.locator('[data-testid="mode-option-agent"]').click();
      await page.locator('[data-testid="desktop-permission-panel-sidebar-grant-shell"]').click();
      await expect.poll(() => page.evaluate(() => window.__toolGrants.shell)).toBe(true);

      await sendPrompt(page, 'run `ls ~` in terminal');
      await expect(page.locator('[data-testid="command-approval"]')).toBeVisible();
      await expect.poll(() => page.evaluate(() => window.__toolInvocations.length)).toBe(0);

      await page.locator('[data-testid="command-approve"]').last().click();
      await expect.poll(() => page.evaluate(() => window.__agentProviderCalls.length)).toBe(1);
      await expect.poll(() => page.evaluate(() => window.__toolInvocations.length)).toBe(0);
      await expect
        .poll(() => page.evaluate(() => window.__agentProviderCalls[0]?.grants?.shell))
        .toBe(true);

      const lastAssistant = page.locator('[data-testid="chat-message"].assistant').last();
      await expect(lastAssistant).toContainText('Desktop and Documents are present');
      await expect(lastAssistant).toContainText('intent:agent_cli_turn');

      const diagnosticsSteps = lastAssistant.locator('[data-testid="diagnostics-steps"]');
      await expect(diagnosticsSteps).toContainText('agent_tool_start');
      await expect(diagnosticsSteps).toContainText('agent_tool_result');

      const diagnosticsTools = lastAssistant.locator('[data-testid="diagnostics-tools"]');
      await expect(diagnosticsTools).toContainText('shell');
      await expect(diagnosticsTools).toContainText('ls ~');
      await expect(diagnosticsTools).toContainText('Desktop');
    });
  }
});
