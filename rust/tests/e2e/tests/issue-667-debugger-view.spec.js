// @ts-check
// Issue #667 / R383: the debugger view renders in the built app and steps a
// held turn of a real `--debug-session` server, stage by stage.
//
// The node suites pin the protocol (rust/tests/web/server-debug-session.test.mjs)
// and the pane projection (rust/tests/web/debugger-client.test.mjs); neither
// mounts the JSX view in a browser, loads the real Mermaid bundle, or clicks
// its buttons. This spec does. It starts the JavaScript server with
// `--debug-session`, gives the app the desktop bridge the Electron shell gives
// it (apiBase and the session token), and reaches the server through a
// same-origin path that Playwright forwards. The server sends no CORS headers,
// and the desktop shell does not need them. With stepping on, a chat turn is
// held. The question is asked in the app's own composer, so the turn travels
// the desktop path (requestDesktopAnswer) to the server. The turn first waits
// at its `impulse` stage before the worker solves it; "Next stage" lets the
// solve run, and the rest of the turn is revealed stage by stage. The view
// shows the paused stage's Mermaid graph as SVG and the Rust and JavaScript
// code that emits that stage, so the source panes change from stage to stage,
// and the answer reaches the chat only after the last stage. The final view is
// saved as the live session capture in docs/case-studies/issue-667/.
//
// Writing it found three defects: the view did not bind the JSX factory `h`,
// so diagnostics mode crashed the app (rust/tests/web/web-jsx-factory.test.mjs);
// the chat panel's two-row grid let the message list cover the view; and a bun
// older than .bun-version bundles Mermaid's ELK layout with a bare `__require`,
// so every diagram fell back to its source text.
const { test, expect } = require('@playwright/test');
const { spawn } = require('node:child_process');
const fs = require('node:fs');
const os = require('node:os');
const path = require('node:path');

const REPO = path.resolve(__dirname, '../../../..');
const TOKEN = 'issue-667-debugger-view-token';
const SERVER_PORT = Number(process.env.E2E_DEBUG_SERVER_PORT || 3477);
const SERVER = `http://127.0.0.1:${SERVER_PORT}`;
const PROXY = '/debug-api';
const CAPTURE = path.join(REPO, 'docs/case-studies/issue-667/debugger-live-session.png');

let server;
let home;

async function debug(action, body = {}) {
  const response = await fetch(`${SERVER}/v1/debug/${action}`, {
    method: 'POST',
    headers: { 'content-type': 'application/json' },
    body: JSON.stringify({ token: TOKEN, ...body }),
  });
  return response.json();
}

test.beforeAll(async () => {
  home = fs.mkdtempSync(path.join(os.tmpdir(), 'formal-ai-debugger-view-'));
  const env = {
    ...process.env,
    HOME: home,
    FORMAL_AI_MEMORY_PATH: path.join(home, 'memory.lino'),
    FORMAL_AI_DIALOG_LOG_DIR: path.join(home, 'dialogs'),
    FORMAL_AI_RECORD_CHAT: '0',
    FORMAL_AI_DEBUG_SESSION_TOKEN: TOKEN,
  };
  delete env.FORMAL_AI_API_BEARER_TOKEN;
  delete env.FORMAL_AI_HTTP_BEARER_TOKEN;
  delete env.FORMAL_AI_API_TOKEN;
  server = spawn(process.execPath, ['js/server/main.mjs', 'serve', '--debug-session', '--port', String(SERVER_PORT)], {
    cwd: REPO, env, stdio: 'ignore',
  });
  for (let attempt = 0; attempt < 300; attempt += 1) {
    try {
      if ((await debug('session')).object === 'debug.session') return;
    } catch {
      // The server is still starting.
    }
    await new Promise((resolve) => setTimeout(resolve, 100));
  }
  throw new Error('the debug-session server did not start');
});

test.afterAll(() => {
  server?.kill();
  if (home) fs.rmSync(home, { recursive: true, force: true });
});

test('the debugger view steps a held turn in the built app', async ({ page }) => {
  test.setTimeout(120_000);
  await page.route(`**${PROXY}/**`, async (route) => {
    const url = new URL(route.request().url());
    // A held turn answers only after its last stage, so the forward waits.
    const response = await route.fetch({ url: `${SERVER}${url.pathname.slice(PROXY.length)}${url.search}`, timeout: 0 })
      .catch(() => null);
    if (response) await route.fulfill({ response });
    else await route.abort().catch(() => {});
  });
  await page.addInitScript(({ proxy, token }) => {
    window.localStorage.setItem('formal-ai.preferences.v1',
      'demo_preferences\n  demoMode "off"\n  diagnosticsMode "on"\n  greetingVariations "off"');
    window.FormalAiDesktop = {
      getStatus: async () => ({ shell: 'Electron', apiReady: true, apiBase: `${location.origin}${proxy}`, debugToken: token }),
    };
  }, { proxy: PROXY, token: TOKEN });
  await page.goto('./');

  const view = page.getByTestId('debugger-view');
  await expect(view).toBeVisible();
  await expect(page.getByTestId('debugger-pause')).toBeEnabled({ timeout: 30_000 });
  await page.getByTestId('debugger-pause').click();
  await expect(page.getByTestId('debugger-continue')).toBeEnabled();

  const messages = page.getByTestId('chat-message');
  const before = await messages.count();
  const input = page.getByTestId('chat-composer-input');
  await expect(input).toBeEnabled({ timeout: 15_000 });
  await input.fill('What is 2 + 2?');
  await page.getByTestId('chat-composer-submit').click();
  const answer = messages.nth(before + 1);

  const next = page.getByTestId('debugger-next');
  const rustPane = page.getByTestId('debugger-rust-source').locator('p');
  const jsPane = page.getByTestId('debugger-js-source').locator('p');
  await expect(next).toBeEnabled({ timeout: 30_000 });
  // Stage 0 waits before the solve: only the prompt is known.
  await expect(next).toHaveAttribute('title', / 1\/1 impulse$/);
  await expect(rustPane).toContainText(/rust\/src\/solver\.rs:\d+\s*solve_with_history_probability_store_and_intent_cache/);
  await expect(jsPane).toContainText(/js\/worker\/formal_ai_worker_solver_events\.js:\d+\s*solverEventLog/);
  await expect(page.getByTestId('debugger-diagram').locator('svg')).toBeVisible({ timeout: 30_000 });

  let stages = 1;
  const shown = [];
  for (let stage = 0; stage < stages; stage += 1) {
    await expect(next).toHaveAttribute('title', new RegExp(` ${stage + 1}/${stages} `));
    await expect(answer, `held before stage ${stage + 1} is advanced`).toHaveCount(0);
    const { paused } = await debug('session');
    // The panes show the paused stage's own emitters.
    await expect(rustPane).toContainText(`${paused[0].rust_source.split(':')[1]}`);
    await expect(jsPane).toContainText(`${paused[0].js_source.split(':')[1]}`);
    shown.push(`${await rustPane.innerText()} | ${await jsPane.innerText()}`);
    if (stage === stages - 1) {
      fs.mkdirSync(path.dirname(CAPTURE), { recursive: true });
      await view.screenshot({ path: CAPTURE });
    }
    await next.click();
    if (stage === 0) {
      // The advance let the worker solve; the rest of the turn is revealed.
      await expect(next).toHaveAttribute('title', / 2\/\d+ /, { timeout: 30_000 });
      stages = (await debug('session')).paused[0].stages;
      expect(stages).toBeGreaterThan(2);
    }
  }
  // The source panes change from stage to stage (the prelude, the
  // formalization record, the calculator, the finalizer).
  expect(new Set(shown).size).toBeGreaterThanOrEqual(4);
  expect(shown.some((pane) => /try_arithmetic/.test(pane))).toBe(true);
  expect(shown.some((pane) => /finalize_simple/.test(pane))).toBe(true);

  await expect(answer).toContainText('4', { timeout: 30_000 });
  await expect(next).toBeDisabled();
  await page.getByTestId('debugger-continue').click();
  await expect(page.getByTestId('debugger-continue')).toBeDisabled();
});
