// @ts-check
// Issue #327: Browser text synthesis matches the Rust parity fixture and accepts supported-language wrappers.
const { test, expect } = require('@playwright/test');
const { WORKER_READY_TIMEOUT_MS } = require('./support/worker-ready');
// Plan 16 L1 moved this spec to rust/tests/e2e/tests/, so the repo root is
// four levels up, not three.
const parityCases = require('../../../../data/parity/cross-runtime-synthesis.json');

async function sendPrompt(page, text) {
  const input = page.locator('[data-testid="chat-composer-input"]');
  await expect(input).toBeEnabled({ timeout: WORKER_READY_TIMEOUT_MS });
  await input.fill(text);

  const messages = page.locator('[data-testid="chat-message"]');
  const initialCount = await messages.count();
  await page.locator('[data-testid="chat-composer-submit"]').click();
  await expect(messages).toHaveCount(initialCount + 2, { timeout: 20_000 });

  const assistantMessage = messages.last();
  await expect(assistantMessage).toHaveClass(/assistant/);
  await expect(assistantMessage.locator('.markdown-body')).toBeVisible();
  return assistantMessage;
}

test.describe('Issue #327 cross-runtime synthesis parity', () => {
  test.beforeEach(async ({ page }) => {
    await page.addInitScript(() => {
      window.localStorage.setItem(
        'formal-ai.preferences.v1',
        'demo_preferences\n  demoMode "off"\n  diagnosticsMode "on"\n  greetingVariations "off"',
      );
    });
    await page.goto('./');
    await expect(page.locator('.app')).toBeVisible({ timeout: 15_000 });
    await expect(page.locator('[data-testid="demo-status"]')).toHaveText('Manual mode');
    await expect(page.locator('.status')).toContainText('wasm worker');
  });

  for (const item of parityCases) {
    test(`${item.id} matches the Rust parity fixture`, async ({ page }) => {
      // Cases whose handler the browser worker does not implement yet carry
      // `browserNotImplemented`; they are pinned native-only (worker-parity
      // lane) until the worker gains the handlers.
      test.skip(
        item.browserNotImplemented === true,
        'the browser worker does not route this handler yet (worker-parity lane)',
      );
      const message = await sendPrompt(page, item.prompt);
      const body = message.locator('.markdown-body');
      const evidence = message.locator('.evidence-list');

      await expect(message).toContainText(`intent:${item.expectedIntent}`);
      const expectedAnswerFragments =
        item.browserExpectedAnswerFragments || item.expectedAnswerFragments;
      const expectedEvidencePrefixes =
        item.browserExpectedEvidencePrefixes || item.expectedEvidencePrefixes;
      for (const expected of expectedAnswerFragments) {
        if (expected.startsWith('```')) continue;
        // Playwright observes rendered text, so inline Markdown delimiters are
        // not present even though the parity fixture deliberately documents
        // the source Markdown returned by the worker.
        await expect(body).toContainText(expected.replaceAll('`', ''));
      }
      for (const forbidden of item.forbiddenAnswerFragments) {
        await expect(body).not.toContainText(forbidden);
      }
      // A forbidden answer fragment must not reach the evidence either,
      // unless the case names what its evidence may not hold: a summary's
      // evidence lists the statements it dropped, so only keeping the
      // forbidden statement is wrong there.
      const forbiddenEvidence =
        item.forbiddenEvidenceFragments || item.forbiddenAnswerFragments;
      for (const forbidden of forbiddenEvidence) {
        if (/[A-Za-z_]/.test(forbidden)) {
          await expect(evidence).not.toContainText(forbidden);
        }
      }
      for (const prefix of expectedEvidencePrefixes) {
        await expect(evidence, `${item.id} evidence should include ${prefix}`).toContainText(
          prefix,
        );
      }
    });
  }

  test('text synthesis route accepts supported-language wrappers', async ({ page }) => {
    for (const { language, prompt } of [
      {
        language: 'en',
        prompt: "English request: Uppercase and reverse words: 'links notation rules'",
      },
      {
        language: 'ru',
        prompt: "Русский запрос: Uppercase and reverse words: 'links notation rules'",
      },
      {
        language: 'hi',
        prompt: "हिंदी अनुरोध: Uppercase and reverse words: 'links notation rules'",
      },
      {
        language: 'zh',
        prompt: "中文请求: Uppercase and reverse words: 'links notation rules'",
      },
    ]) {
      const message = await sendPrompt(page, prompt);
      await expect(message, language).toContainText('intent:text_manipulation');
      await expect(message.locator('.markdown-body'), language).toContainText(
        'RULES NOTATION LINKS',
      );
      await expect(message.locator('.evidence-list'), language).toContainText(
        'text_operation:reverse_words',
      );
    }
  });
});
