// @ts-check
const { test, expect } = require('@playwright/test');
const path = require('node:path');
const { pathToFileURL } = require('node:url');

const PREF_KEY = 'formal-ai.preferences.v1';
const SCREENSHOT_PATH = process.env.ISSUE_864_SCREENSHOT_PATH;

test('failure detection uses semantic signals instead of error-like prose', async () => {
  const moduleUrl = pathToFileURL(
    path.resolve(__dirname, '../../../src/web/app/detected-failure.js'),
  );
  const { answerHasDetectedFailure } = await import(moduleUrl.href);

  expect(answerHasDetectedFailure({
    intent: 'answer',
    content: 'This documentation explains the words error and failed.',
  })).toBe(false);
  expect(answerHasDetectedFailure({
    intent: 'answer',
    toolCalls: [{ outputs: { ok: false, error: 'provider unavailable' } }],
  })).toBe(true);
  expect(answerHasDetectedFailure({
    intent: 'answer',
    toolCalls: [{ outputs: { ok: false, status: 'awaiting_approval' } }],
  })).toBe(false);
});

test('detected provider failures proactively offer a contextual issue report', async ({ page }) => {
  await page.addInitScript(({ prefKey }) => {
    window.localStorage.setItem(
      prefKey,
      [
        'demo_preferences',
        '  demoMode "off"',
        '  greetingVariations "off"',
        '  diagnosticsMode "off"',
        '  uiLanguage "en"',
      ].join('\n'),
    );
    const status = {
      shell: 'Electron',
      apiBase: '',
      graphUrl: '',
      memory: 'formal_ai_bundle',
      apiReady: false,
      activeEngine: 'agent',
      engines: [
        { id: 'out-of-box', label: 'Out of the box', type: 'native', available: true },
        { id: 'agent', label: 'Agent', type: 'passthrough', available: true },
      ],
    };
    window.FormalAiDesktop = {
      getStatus: async () => status,
      ensureAgentServer: async () => status,
      setToolGrants: async () => ({}),
      syncMemory: async () => ({ ok: true }),
      runAgentProvider: async () => ({
        ok: false,
        provider: 'agent',
        status: 'error',
        executed: false,
        reason: 'provider exited before returning an answer',
      }),
    };
  }, { prefKey: PREF_KEY });

  await page.goto('./');
  const input = page.locator('[data-testid="chat-composer-input"]');
  await expect(input).toBeEnabled({ timeout: 10_000 });
  await input.fill('inspect this workspace');
  await page.locator('[data-testid="chat-composer-submit"]').click();

  const answer = page.locator('[data-testid="chat-message"].assistant').last();
  await expect(answer).toContainText('provider exited before returning an answer');
  const skipAnimation = answer.locator('[data-testid="message-skip-animation"]');
  if (await skipAnimation.isVisible()) await skipAnimation.click();
  if (SCREENSHOT_PATH) {
    await page.screenshot({ path: SCREENSHOT_PATH, fullPage: true });
  }

  const invitation = answer.locator('[data-testid="detected-failure-report"]');
  await expect(invitation).toContainText(
    'I detected a failure while working on this request. Would you like to report it?',
  );
  const report = invitation.locator('a');
  await expect(report).toHaveText('Report issue');
  await expect(report).toHaveAttribute(
    'href',
    /github\.com\/link-assistant\/formal-ai\/issues\/new\?.*labels=bug/,
  );
  const href = await report.getAttribute('href');
  if (!href) throw new Error('the proactive report action has no URL');
  const reportUrl = new URL(href);
  const reportBody = reportUrl.searchParams.get('body');
  for (const section of [
    '## Environment',
    '## User Context',
    '## Reproduction of dialog',
    '## Reasoning Trace',
    '## Description',
    '## Attach full memory (optional)',
  ]) {
    expect(reportBody).toContain(section);
  }
  expect(reportBody).toContain('inspect this workspace');
  expect(reportBody).toContain('provider exited before returning an answer');
});
