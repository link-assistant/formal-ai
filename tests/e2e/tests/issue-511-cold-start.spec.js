// @ts-check
const { test, expect } = require('@playwright/test');
const fs = require('node:fs');
const os = require('node:os');
const path = require('node:path');

const PREF_KEY = 'formal-ai.preferences.v1';
const COMMAND = 'ls ~';

function cleanPreferences() {
  return [
    'demo_preferences',
    '  demoMode "off"',
    '  diagnosticsMode "on"',
    '  greetingVariations "off"',
    '  uiLanguage "en"',
  ].join('\n');
}

function makeHermeticHome() {
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), 'formal-ai-issue-511-home-'));
  for (const name of ['Desktop', 'Documents', 'issue-511-home-marker.txt']) {
    const target = path.join(dir, name);
    if (name.endsWith('.txt')) {
      fs.writeFileSync(target, 'issue 511 marker\n');
    } else {
      fs.mkdirSync(target);
    }
  }
  return dir;
}

function lsListing(homeDir) {
  return `${fs.readdirSync(homeDir).sort((left, right) => left.localeCompare(right)).join('\n')}\n`;
}

function agentAnswerForListing({ provider, listing }) {
  return {
    intent: 'agent_cli_turn',
    source: 'agent_provider',
    confidence: 'medium',
    content: listing.trim(),
    evidence: ['agent_provider:ndjson', 'agent_events:1', `provider:${provider}`, 'status:ok'],
    steps: [
      { step: 'agent_tool_start', detail: `shell: ${COMMAND}` },
      { step: 'agent_tool_result', detail: `shell: ${COMMAND} (exit 0)` },
    ],
    toolCalls: [
      {
        tool: 'shell',
        inputs: { command: COMMAND },
        outputs: {
          stdout: listing,
          stderr: '',
          exitCode: 0,
          status: 'completed',
        },
      },
    ],
  };
}

async function installColdStartBridge(page, options = {}) {
  const provider = options.provider || 'in-process';
  const apiBase = options.apiBase || '';
  await page.addInitScript(
    ({ prefKey, preferences, providerType, baseUrl }) => {
      try {
        window.localStorage.clear();
        window.localStorage.setItem(prefKey, preferences);
      } catch (_error) {
        // localStorage can be unavailable in hardened browser contexts.
      }

      window.__agentProviderCalls = [];
      window.__ensureAgentServerCalls = [];
      window.__grantHistory = [];
      window.__toolGrants = {};

      const status = () => ({
        shell: 'Electron',
        apiBase: baseUrl,
        staticBase: '',
        graphUrl: baseUrl ? `${baseUrl}/v1/graph` : '',
        traceUrl: '',
        memory: 'formal_ai_bundle',
        agentModeDefault: false,
        toolCallPolicy: 'explicit-permission',
        apiReady: Boolean(baseUrl),
        agentExecutionProvider: { type: providerType },
        agentProvider: baseUrl
          ? {
              type: 'local-openai-compatible',
              apiBase: baseUrl,
              openAiBaseUrl: `${baseUrl}/v1`,
              model: 'formal-ai',
            }
          : null,
      });

      window.FormalAiDesktop = {
        getStatus: async () => status(),
        ensureAgentServer: async () => {
          window.__ensureAgentServerCalls.push({ at: Date.now() });
          return status();
        },
        setToolGrants: async (grants) => {
          window.__toolGrants = { ...(grants || {}) };
          window.__grantHistory.push({ ...window.__toolGrants });
          return window.__toolGrants;
        },
        invokeTool: async (request) => ({
          ok: false,
          tool: request && request.tool,
          status: 'refused',
          executed: false,
          reason: 'issue 511 expects the agent provider path',
        }),
        runAgentProvider: async (request) => {
          window.__agentProviderCalls.push(request);
          return window.__issue511RunAgentProvider(request);
        },
      };
    },
    {
      prefKey: PREF_KEY,
      preferences: cleanPreferences(),
      providerType: provider,
      baseUrl: apiBase,
    },
  );
}

async function bootColdStart(page) {
  await page.goto('./');
  await expect(page.locator('.app')).toBeVisible({ timeout: 15_000 });
  await expect(page.locator('[data-testid="chat-composer-input"]')).toBeEnabled({
    timeout: 10_000,
  });
  await expect(page.locator('[data-testid="mode-option-chat"]')).toHaveAttribute(
    'aria-checked',
    'true',
  );
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

async function proveColdStartJourney(page) {
  await bootColdStart(page);

  await expect(page.locator('[data-testid="desktop-tool-permission"]')).toHaveText(
    '0/6 tools granted',
  );

  await page.locator('[data-testid="mode-option-agent"]').click();
  await expect(page.locator('[data-testid="mode-option-agent"]')).toHaveAttribute(
    'aria-checked',
    'true',
  );
  await expect(page.locator('[data-testid="desktop-permission-panel-message"]')).toBeVisible();
  await expect(page.locator('[data-testid="chat-message"].system')).toContainText(
    'Grant or decline each tool separately',
  );
  await expect.poll(() =>
    page.evaluate((prefKey) => window.localStorage.getItem(prefKey), PREF_KEY),
  ).toContain('agentOnboardingSeen "on"');

  await page.locator('[data-testid="mode-option-fullAuto"]').click();
  await expect(page.locator('[data-testid="mode-option-fullAuto"]')).toHaveAttribute(
    'aria-checked',
    'true',
  );
  await page.locator('[data-testid="mode-option-chat"]').click();
  await expect(page.locator('[data-testid="mode-option-chat"]')).toHaveAttribute(
    'aria-checked',
    'true',
  );
  await page.locator('[data-testid="mode-option-agent"]').click();
  await expect(page.locator('[data-testid="chat-message"].system')).toHaveCount(1);

  await page.locator('[data-testid="desktop-permission-panel-sidebar-grant-shell"]').click();
  await expect(
    page.locator('[data-testid="desktop-permission-panel-sidebar-state-shell"]'),
  ).toHaveText('Granted');
  await page.locator('[data-testid="desktop-permission-panel-sidebar-decline-http_fetch"]').click();
  await expect(
    page.locator('[data-testid="desktop-permission-panel-sidebar-state-http_fetch"]'),
  ).toHaveText('Declined');
  await expect.poll(() => page.evaluate(() => window.__toolGrants.shell)).toBe(true);

  await sendPrompt(page, 'run `ls ~` in terminal');
  await expect(page.locator('[data-testid="command-approval"]')).toBeVisible();
  await expect(page.locator('[data-testid="command-approval"] code').last()).toHaveText(COMMAND);
  await expect.poll(() => page.evaluate(() => window.__agentProviderCalls.length)).toBe(0);

  await page.locator('[data-testid="command-deny"]').last().click();
  await expect(page.locator('[data-testid="chat-message"]').last()).toContainText(
    'Command declined',
  );
  await expect.poll(() => page.evaluate(() => window.__agentProviderCalls.length)).toBe(0);

  await sendPrompt(page, 'run `ls ~` in terminal');
  await page
    .locator(
      '[data-testid="command-approval"][data-status="pending"] [data-testid="command-approve"]',
    )
    .click();
  await expect.poll(() => page.evaluate(() => window.__agentProviderCalls.length)).toBe(1);

  const providerRequest = await page.evaluate(() => window.__agentProviderCalls[0]);
  expect(providerRequest).toMatchObject({
    mode: 'agent',
    tool: 'shell',
    command: COMMAND,
    transcript: true,
  });
  expect(providerRequest.grants.shell).toBe(true);
  expect(providerRequest.grants.http_fetch).toBe(false);

  const lastAssistant = page.locator('[data-testid="chat-message"].assistant').last();
  await expect(lastAssistant).toContainText('issue-511-home-marker.txt');
  await expect(lastAssistant).toContainText('Desktop');
  await expect(lastAssistant).toContainText('Documents');
  await expect(lastAssistant).toContainText('intent:agent_cli_turn');

  const diagnosticsTools = lastAssistant.locator('[data-testid="diagnostics-tools"]');
  await expect(diagnosticsTools).toContainText('shell');
  await expect(diagnosticsTools).toContainText(COMMAND);
  await expect(diagnosticsTools).toContainText('issue-511-home-marker.txt');
}

test.describe('Issue #511/#519: cold-start ls home journey', () => {
  test('hermetic in-process provider proves onboarding, mode switch, command denial, approval, and rendered listing', async ({
    page,
  }) => {
    const homeDir = makeHermeticHome();
    await page.exposeFunction('__issue511RunAgentProvider', async (request) => {
      const command = String((request && request.command) || '');
      if (command !== COMMAND) {
        return {
          ok: false,
          provider: 'in-process',
          status: 'error',
          executed: false,
          reason: `unexpected command: ${command}`,
        };
      }
      const listing = lsListing(homeDir);
      return {
        ok: true,
        provider: 'in-process',
        status: 'ok',
        executed: true,
        command,
        events: [
          {
            type: 'tool_result',
            tool: 'shell',
            command,
            output: { stdout: listing, stderr: '', exitCode: 0, status: 'completed' },
          },
        ],
        answer: agentAnswerForListing({ provider: 'in-process', listing }),
      };
    });

    try {
      await installColdStartBridge(page, { provider: 'in-process' });
      await proveColdStartJourney(page);
    } finally {
      fs.rmSync(homeDir, { recursive: true, force: true });
    }
  });

  test('container-gated commander provider runs the same browser journey on demand', async ({
    page,
  }) => {
    test.skip(
      process.env.FORMAL_AI_E2E_AGENT_COMMANDER !== '1',
      'Set FORMAL_AI_E2E_AGENT_COMMANDER=1 with a ready formal-ai-agent container to run the real commander journey.',
    );

    const repoRoot = path.resolve(__dirname, '../../..');
    const apiBase = process.env.FORMAL_AI_E2E_AGENT_API_BASE || 'http://127.0.0.1:8080';
    const containerName = process.env.FORMAL_AI_E2E_AGENT_CONTAINER || 'formal-ai-agent';
    const commanderCommand = process.env.FORMAL_AI_E2E_AGENT_COMMANDER_COMMAND || 'start-agent';
    const { createAgentProvider } = require('../../../desktop/lib/agent-provider.cjs');
    const provider = createAgentProvider({
      type: 'commander',
      commanderCommand,
      containerName,
      workingDirectory: repoRoot,
    });

    await page.exposeFunction('__issue511RunAgentProvider', async (request) =>
      provider.run({
        ...(request || {}),
        apiBase,
        agentProvider: {
          apiBase,
          openAiBaseUrl: `${apiBase.replace(/\/+$/, '')}/v1`,
          model: 'formal-ai',
        },
        containerName,
        workingDirectory: repoRoot,
      }),
    );

    await installColdStartBridge(page, { provider: 'commander', apiBase });
    await bootColdStart(page);
    await page.locator('[data-testid="mode-option-agent"]').click();
    await page.locator('[data-testid="desktop-permission-panel-sidebar-grant-shell"]').click();
    await sendPrompt(page, 'run `ls ~` in terminal');
    await page.locator('[data-testid="command-approve"]').last().click();
    await expect.poll(() => page.evaluate(() => window.__agentProviderCalls.length)).toBe(1);

    const lastAssistant = page.locator('[data-testid="chat-message"].assistant').last();
    await expect(lastAssistant).toContainText('intent:agent_cli_turn', { timeout: 30_000 });
    await expect(lastAssistant.locator('[data-testid="diagnostics-tools"]')).toContainText(COMMAND);
  });
});
