// @ts-check
// Issue #672 (F5): "Mode-flip-on-grant in tests covering raw IPC".
//
// Issue #541 (R9) gave the permission panel a single CTA that grants every
// tool, flips Chat → Agent, and replays the command the user already asked for.
// The R9 spec proves that with a bridge mocked *inside the page*: every
// `window.FormalAiDesktop` method is a closure in the renderer's own realm, so
// the assertions can never see a value that failed to survive the trip to the
// desktop side. A regression where the replayed request stopped being
// structured-cloneable, or where the grants were mutated after the boundary
// call, would still pass there.
//
// This spec closes that gap the way issue #511's cold-start spec does: the
// bridge's `runAgentProvider` is a `page.exposeFunction` binding, so the
// request really leaves the browser context, is serialized, and is answered by
// a provider running in the test's Node process against a hermetic home
// directory. What the user reads back in the chat is a listing this file wrote
// to disk — it cannot be faked by the renderer.
//
// The mode flip is asserted on both sides of that boundary: the renderer's
// radiogroup (`aria-checked`) and the `mode` field of the request Node
// actually received.

const { test, expect } = require('@playwright/test');
const fs = require('node:fs');
const os = require('node:os');
const path = require('node:path');

const PREF_KEY = 'formal-ai.preferences.v1';
const COMMAND = 'ls ~';
const PROMPT = 'run `ls ~` in terminal';
const MARKER = 'issue-541-cold-start-marker.txt';

const DESKTOP_TOOLS = [
  'shell',
  'write_file',
  'edit_file',
  'multi_edit',
  'eval_js',
  'code_exec',
];

function preferences() {
  return [
    'demo_preferences',
    '  demoMode "off"',
    '  greetingVariations "off"',
    '  diagnosticsMode "on"',
    '  uiLanguage "en"',
  ].join('\n');
}

/** A throwaway home whose listing is known to this file and to nothing else. */
function makeHermeticHome() {
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), 'formal-ai-issue-541-home-'));
  fs.mkdirSync(path.join(dir, 'Desktop'));
  fs.mkdirSync(path.join(dir, 'Documents'));
  fs.writeFileSync(path.join(dir, MARKER), 'issue 541 cold start marker\n');
  return dir;
}

function lsListing(homeDir) {
  return `${fs
    .readdirSync(homeDir)
    .sort((left, right) => left.localeCompare(right))
    .join('\n')}\n`;
}

/**
 * The provider that answers the replayed task. It runs in Node, on the far side
 * of the exposeFunction boundary, and refuses anything but the exact command
 * the user typed — so a replay that garbles the command produces a visible
 * failure instead of a listing.
 */
function makeProvider(homeDir, received) {
  return async (request) => {
    received.push(request);
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
      answer: {
        intent: 'agent_cli_turn',
        source: 'agent_provider',
        confidence: 'medium',
        content: listing.trim(),
        evidence: ['agent_provider:ndjson', 'agent_events:1', 'provider:in-process', 'status:ok'],
        steps: [
          { step: 'agent_tool_start', detail: `shell: ${command}` },
          { step: 'agent_tool_result', detail: `shell: ${command} (exit 0)` },
        ],
        toolCalls: [
          {
            tool: 'shell',
            inputs: { command },
            outputs: { stdout: listing, stderr: '', exitCode: 0, status: 'completed' },
          },
        ],
      },
    };
  };
}

/**
 * The bridge itself stays in the page — that is what `preload.cjs` exposes —
 * but every method that carries a payload across the boundary forwards to the
 * Node binding, so the payload is serialized exactly as `ipcRenderer.invoke`
 * would serialize it.
 */
async function installBridge(page) {
  await page.addInitScript(
    ({ prefKey, prefs }) => {
      try {
        window.localStorage.clear();
        window.localStorage.setItem(prefKey, prefs);
      } catch (_error) {
        // localStorage can be unavailable in hardened browser contexts.
      }

      window.__ensureAgentServerCalls = 0;
      const status = () => ({
        shell: 'Electron',
        apiBase: '',
        staticBase: '',
        graphUrl: '',
        traceUrl: '',
        memory: 'formal_ai_bundle',
        agentModeDefault: false,
        toolCallPolicy: 'explicit-permission',
        apiReady: false,
        agentExecutionProvider: { type: 'in-process' },
        agentProvider: null,
      });

      window.FormalAiDesktop = {
        getStatus: async () => status(),
        ensureAgentServer: async () => {
          window.__ensureAgentServerCalls += 1;
          return status();
        },
        setToolGrants: async (grants) => window.__issue541SetToolGrants(grants || {}),
        invokeTool: async (request) => ({
          ok: false,
          tool: request && request.tool,
          status: 'refused',
          executed: false,
          reason: 'issue 541 cold start expects the agent provider path',
        }),
        runAgentProvider: async (request) => window.__issue541RunAgentProvider(request),
      };
    },
    { prefKey: PREF_KEY, prefs: preferences() },
  );
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
  await expect
    .poll(async () => messages.count(), { timeout: 20_000 })
    .toBeGreaterThan(initial);
}

test.describe('Issue #541 (R9) over a raw IPC-shaped boundary', () => {
  /** @type {string} */
  let homeDir;
  /** @type {Array<any>} */
  let providerRequests;
  /** @type {Array<any>} */
  let grantRequests;

  test.beforeEach(async ({ page }) => {
    homeDir = makeHermeticHome();
    providerRequests = [];
    grantRequests = [];
    await page.exposeFunction('__issue541RunAgentProvider', makeProvider(homeDir, providerRequests));
    await page.exposeFunction('__issue541SetToolGrants', async (grants) => {
      // The desktop side keeps its own copy, exactly like main.cjs does. The
      // renderer never sees this object again, so a later assertion on it is
      // an assertion about what actually crossed the boundary.
      grantRequests.push({ ...grants });
      return { ...grants };
    });
    await installBridge(page);
  });

  test.afterEach(() => {
    fs.rmSync(homeDir, { recursive: true, force: true });
  });

  test('the grant-all CTA flips the mode and replays the task across the boundary', async ({
    page,
  }) => {
    // Chat mode. The worker recognizes a terminal command and queues it instead
    // of running it, which is what puts the CTA into its "pending task" wording.
    await sendPrompt(page, PROMPT);
    const cta = page.locator('[data-testid="desktop-permission-panel-message-grant-all"]');
    await expect(cta).toBeVisible({ timeout: 10_000 });
    await expect(cta).toHaveAttribute('data-has-pending-task', 'true');

    // Nothing has been *authorized* yet. The renderer does mirror its initial
    // state to the desktop side at boot, so the assertion is on the content of
    // those snapshots — every tool still false — rather than on their absence.
    for (const grants of grantRequests) {
      for (const tool of DESKTOP_TOOLS) {
        expect(grants[tool]).toBe(false);
      }
    }
    expect(providerRequests).toEqual([]);

    await cta.click();

    // The mode flip as the renderer shows it…
    await expect(page.locator('[data-testid="mode-option-agent"]')).toHaveAttribute(
      'aria-checked',
      'true',
    );

    // …and as Node received it. This is the assertion the in-page mock cannot
    // make: `mode`, `tool`, `command` and `grants` are read from the object
    // that was serialized out of the browser context.
    await expect.poll(() => providerRequests.length, { timeout: 15_000 }).toBe(1);
    const request = providerRequests[0];
    expect(request.mode).toBe('agent');
    expect(request.tool).toBe('shell');
    expect(request.command).toBe(COMMAND);
    for (const tool of DESKTOP_TOOLS) {
      expect(request.grants[tool]).toBe(true);
    }

    // The grants reached the desktop side too, as one all-true snapshot.
    expect(grantRequests.length).toBeGreaterThanOrEqual(1);
    const lastGrants = grantRequests[grantRequests.length - 1];
    for (const tool of DESKTOP_TOOLS) {
      expect(lastGrants[tool]).toBe(true);
    }

    // And the user reads back the real listing of the hermetic home — proof the
    // round trip completed in both directions, not just outbound.
    const answer = page.locator('[data-testid="chat-message"].assistant').last();
    await expect(answer).toContainText(MARKER, { timeout: 15_000 });
    await expect(answer).toContainText('Desktop');
    await expect(answer).toContainText('Documents');
    await expect(answer.locator('[data-testid="diagnostics-tools"]')).toContainText(COMMAND);

    // The queue is empty afterwards, so the CTA stops promising a replay.
    await expect(cta).toHaveAttribute('data-has-pending-task', 'false');
  });

  test('flipping the mode back and forth after the replay does not re-run the task', async ({
    page,
  }) => {
    // A replay that stayed queued would fire again on the next flip — from the
    // user's side an unrequested shell command. The boundary record is the only
    // place that can be checked without trusting the renderer's own state.
    await sendPrompt(page, PROMPT);
    await page.locator('[data-testid="desktop-permission-panel-message-grant-all"]').click();
    await expect.poll(() => providerRequests.length, { timeout: 15_000 }).toBe(1);

    for (const mode of ['chat', 'agent', 'fullAuto', 'agent']) {
      await page.locator(`[data-testid="mode-option-${mode}"]`).click();
      await expect(page.locator(`[data-testid="mode-option-${mode}"]`)).toHaveAttribute(
        'aria-checked',
        'true',
      );
    }

    expect(providerRequests.length).toBe(1);
    // The grants survive the flips: the tool list still reads Granted, and the
    // desktop side was never told to revoke anything.
    for (const tool of DESKTOP_TOOLS) {
      await expect(
        page.locator(`[data-testid="desktop-permission-panel-sidebar-state-${tool}"]`),
      ).toHaveText('Granted');
      expect(grantRequests[grantRequests.length - 1][tool]).toBe(true);
    }
  });

  test('a task queued before any grant still crosses the boundary intact', async ({ page }) => {
    // The queue is written in Chat mode and read after the flip, so the command
    // text lives across a mode change and a grant round trip. Backticks and the
    // `~` are exactly the characters a sloppy serializer would mangle.
    await sendPrompt(page, PROMPT);
    await page.locator('[data-testid="mode-option-agent"]').click();
    await page
      .locator('[data-testid="desktop-permission-panel-sidebar-grant-shell"]')
      .click();
    await expect(
      page.locator('[data-testid="desktop-permission-panel-sidebar-state-shell"]'),
    ).toHaveText('Granted');

    // Granting one tool by hand is not the CTA, so the queued task is untouched
    // and the CTA still offers the replay.
    expect(providerRequests).toEqual([]);
    const cta = page.locator('[data-testid="desktop-permission-panel-message-grant-all"]');
    await expect(cta).toHaveAttribute('data-has-pending-task', 'true');

    await cta.click();
    await expect.poll(() => providerRequests.length, { timeout: 15_000 }).toBe(1);
    expect(providerRequests[0].command).toBe(COMMAND);
    await expect(page.locator('[data-testid="chat-message"].assistant').last()).toContainText(
      MARKER,
      { timeout: 15_000 },
    );
  });
});
