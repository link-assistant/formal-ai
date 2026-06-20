// @ts-check
// Issue #541 (R9): "After permissions are granted nothing happens, the message
// for granting permissions should also include button to grant all permissions
// and switch to agent mode, which when clicked should actually evaluate
// pending task for execution."
//
// Before this fix:
//   1. User in Chat mode asks "run `ls ~` in terminal".
//   2. The worker detects a terminal command and returns a polite suggestion
//      ("If you want me to run it, switch to Agent mode…").
//   3. The user clicks the Agent mode toggle → onboarding system message
//      appears with six per-tool Grant/Decline buttons.
//   4. The user clicks Grant on `shell` (and only `shell`) — then nothing
//      happens. The original prompt is gone. They have to re-type "run `ls ~`
//      in terminal" and submit it again.
//
// After this fix:
//   1. Same starting state.
//   2. requestTerminalCommandExecution stashes the detected command in
//      pendingAgentTaskRef and triggers showAgentOnboarding, which adds a
//      system message with the permission panel embedded inline.
//   3. The panel renders a new primary CTA — "Grant all, switch to Agent mode,
//      and run pending task" — directly above the per-tool rows. The label
//      itself tells the user this single click will replay the queued command.
//   4. Clicking the CTA:
//        a. mirrors all six DESKTOP_TOOL_OPTIONS into the grants ref and state,
//        b. flips mode to "agent" (ref + state),
//        c. calls executeTerminalCommand("ls ~", "agent"), which surfaces a
//           command-approval entry; the user approves it once and the desktop
//           bridge runs `ls ~`.
//
// This spec verifies the new affordance end-to-end against a mocked
// FormalAiDesktop bridge that captures runAgentProvider calls and tool grants.

const { test, expect } = require('@playwright/test');

const PREF_KEY = 'formal-ai.preferences.v1';

function basePreferences() {
  return [
    'demo_preferences',
    '  demoMode "off"',
    '  greetingVariations "off"',
    '  diagnosticsMode "off"',
    '  uiLanguage "en"',
  ].join('\n');
}

async function bootIssue541Permissions(page) {
  await page.addInitScript(
    ({ prefKey, preferences }) => {
      try {
        window.localStorage.setItem(prefKey, preferences);
      } catch (_error) {
        // localStorage can be unavailable in hardened browser contexts.
      }

      // The mocked desktop bridge captures every cross-boundary call so the
      // test can assert what the renderer would have asked Electron to do.
      window.__grantHistory = [];
      window.__toolGrants = {};
      window.__toolInvocations = [];
      window.__agentProviderCalls = [];
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
        }),
        setToolGrants: async (grants) => {
          window.__toolGrants = { ...(grants || {}) };
          window.__grantHistory.push({ ...window.__toolGrants });
          return window.__toolGrants;
        },
        invokeTool: async (request) => {
          window.__toolInvocations.push(request);
          const grants = window.__toolGrants || {};
          const allowed = grants.all === true || grants[request.tool] === true;
          if (!allowed) {
            return {
              ok: false,
              tool: request.tool,
              status: 'refused',
              executed: false,
              reason: 'tool call denied by explicit-permission policy',
            };
          }
          return {
            ok: true,
            tool: request.tool,
            status: 'ok',
            executed: true,
            servedBy: 'test-bridge',
            body: `ran ${request.input.command || request.input.url || ''}`.trim(),
          };
        },
        runAgentProvider: async (request) => {
          window.__agentProviderCalls.push(request);
          // Deliberately omit `answer` so executeTerminalCommand falls through
          // to invokeTool — that gives the test a single, observable record of
          // the actual shell invocation in window.__toolInvocations.
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
    { prefKey: PREF_KEY, preferences: basePreferences() },
  );
  await page.goto('./');
  await expect(page.locator('.app')).toBeVisible({ timeout: 15_000 });
  await expect(page.locator('[data-testid="chat-composer-input"]')).toBeEnabled({
    timeout: 10_000,
  });
}

async function sendPrompt(page, text) {
  const input = page.locator('[data-testid="chat-composer-input"]');
  const messages = page.locator('[data-testid="chat-message"]');
  const initial = await messages.count();
  await expect(input).toBeEnabled({ timeout: 5_000 });
  await input.fill(text);
  await page.locator('[data-testid="chat-composer-submit"]').click();
  await expect
    .poll(async () => messages.count(), { timeout: 20_000 })
    .toBeGreaterThan(initial);
}

test.describe('Issue #541 (R9): grant-all CTA evaluates the pending task', () => {
  test.beforeEach(async ({ page }) => {
    await bootIssue541Permissions(page);
  });

  test('panel CTA without a pending task offers Grant + mode flip', async ({ page }) => {
    // No terminal command was issued yet, so the panel's button copy must NOT
    // mention "run pending task" — that wording would be a lie.
    await page.locator('[data-testid="mode-option-agent"]').click();
    await expect(
      page.locator('[data-testid="desktop-permission-panel-message"]'),
    ).toBeVisible();

    const cta = page.locator(
      '[data-testid="desktop-permission-panel-message-grant-all"]',
    );
    await expect(cta).toBeVisible();
    await expect(cta).toHaveAttribute('data-has-pending-task', 'false');
    await expect(cta).toHaveText('Grant all and switch to Agent mode');

    await cta.click();

    // Every desktop tool now reports Granted, and the mode toggle stays on
    // Agent. The bridge received the grants too.
    for (const tool of ['shell', 'http_fetch', 'url_navigate', 'eval_js', 'read_local_file', 'code_exec']) {
      await expect(
        page.locator(
          `[data-testid="desktop-permission-panel-sidebar-state-${tool}"]`,
        ),
      ).toHaveText('Granted');
    }
    await expect.poll(() => page.evaluate(() => window.__toolGrants.shell)).toBe(true);
    await expect.poll(() => page.evaluate(() => window.__toolGrants.code_exec)).toBe(true);
    // The mode toggle is a radiogroup, so the active option uses aria-checked
    // rather than aria-pressed (which is reserved for toggle buttons).
    await expect(
      page.locator('[data-testid="mode-option-agent"]'),
    ).toHaveAttribute('aria-checked', 'true');
  });

  test('panel CTA with a pending shell task runs that task on click', async ({ page }) => {
    // Stay in Chat mode. The worker should treat "run `ls ~` in terminal" as a
    // terminal command and emit `terminal:command:ls ~` evidence, which
    // requestTerminalCommandExecution converts into a pendingAgentTask.
    await sendPrompt(page, 'run `ls ~` in terminal');

    // The permission panel appears inside the agent_permission_onboarding
    // system message — that's what the issue calls "the message for granting
    // permissions". The button copy now reads as a queue replay.
    const cta = page.locator(
      '[data-testid="desktop-permission-panel-message-grant-all"]',
    );
    await expect(cta).toBeVisible({ timeout: 10_000 });
    await expect(cta).toHaveAttribute('data-has-pending-task', 'true');
    await expect(cta).toHaveText(
      'Grant all, switch to Agent mode, and run pending task',
    );

    // No desktop boundary calls have happened yet — the user hasn't clicked
    // anything that could authorize execution.
    expect(await page.evaluate(() => window.__agentProviderCalls.length)).toBe(0);

    // Click the CTA. The renderer:
    //   * mirrors all six DESKTOP_TOOL_OPTIONS to true (sync ref + async state)
    //   * flips mode to agent (sync ref + async state)
    //   * replays the captured command via executeTerminalCommand
    await cta.click();

    // The bridge sees the replayed shell command. executeTerminalCommand asks
    // the desktop side for a provider answer first (runAgentProvider); that
    // single call is the canonical evidence the queue replay reached the
    // boundary with the right command, tool, mode, and grants. In production
    // a configured provider returns the executed result, ending the chain
    // there. Our mock returns a non-executed unavailable response so the UI
    // surfaces a graceful refusal message instead of crashing.
    await expect
      .poll(() => page.evaluate(() => window.__agentProviderCalls.length), {
        timeout: 10_000,
      })
      .toBeGreaterThanOrEqual(1);

    const lastProviderCall = await page.evaluate(
      () => window.__agentProviderCalls[window.__agentProviderCalls.length - 1],
    );
    expect(lastProviderCall.command).toBe('ls ~');
    expect(lastProviderCall.tool).toBe('shell');
    expect(lastProviderCall.mode).toBe('agent');
    expect(lastProviderCall.grants && lastProviderCall.grants.shell).toBe(true);

    // The mode toggle reflects the new state. (radiogroup → aria-checked)
    await expect(
      page.locator('[data-testid="mode-option-agent"]'),
    ).toHaveAttribute('aria-checked', 'true');

    // After the replay, the queue is empty — the panel's CTA copy reverts to
    // the no-pending-task wording so it does not falsely claim there is still
    // a task to run.
    await expect(cta).toHaveAttribute('data-has-pending-task', 'false');
    await expect(cta).toHaveText('Grant all and switch to Agent mode');
  });
});
