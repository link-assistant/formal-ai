// @ts-check
//
// Issue #334: the GitHub Pages WASM-worker demo ran a 2-step agent plan that
// both steps failed on. Step 1 ("Write a Python function that calculates the
// Fibonacci sequence recursively.") returned "I didn't understand you" and
// step 2 ("calculate the 10th Fibonacci number and multiply it by 8% of 500.
// Show me the code and the final result.") returned "unparseable".
//
// These end-to-end tests drive the real browser bundle (in "wasm worker" mode,
// exactly as the production demo runs) to prove both the standalone prompts and
// the full agent plan now resolve correctly.
const { test, expect } = require('@playwright/test');

// The exact two-sentence prompt the user pasted into the demo (issue #334).
const FULL_PROMPT =
  'Write a Python function that calculates the Fibonacci sequence recursively. ' +
  'Then calculate the 10th Fibonacci number and multiply it by 8% of 500. ' +
  'Show me the code and the final result.';

async function sendPrompt(page, text) {
  const input = page.locator('[data-testid="chat-composer-input"]');
  await expect(input).toBeEnabled({ timeout: 5_000 });
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

test.describe('Issue #334 — Fibonacci agent plan', () => {
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
    // The production demo evaluates arithmetic through the compiled WASM worker.
    await expect(page.locator('.status')).toContainText('wasm worker');
  });

  test('step 1 generates a recursive Python Fibonacci program', async ({ page }) => {
    const message = await sendPrompt(
      page,
      'Write a Python function that calculates the Fibonacci sequence recursively.',
    );

    const codeBlock = message.locator('.markdown-body .code-block').first();
    await expect(codeBlock).toBeVisible();
    await expect(codeBlock.locator('.code-block-lang')).toHaveText('python');

    const code = codeBlock.locator('code.hljs');
    await expect(code).toContainText('def fibonacci');
    await expect(code).toContainText('fibonacci(n - 1)');
    await expect(code).toContainText('fibonacci(n - 2)');

    // The verified run prints F(10) = 55, and the explanation is rendered.
    await expect(message).toContainText('55');
    await expect(message).toContainText('How it works:');
  });

  test('step 2 reduces the word problem to 55 * 8% of 500 = 2200', async ({ page }) => {
    const message = await sendPrompt(
      page,
      'calculate the 10th Fibonacci number and multiply it by 8% of 500. ' +
        'Show me the code and the final result.',
    );
    await expect(message).toContainText('2200');
  });

  test('agent mode decomposes the full prompt into two working steps', async ({ page }) => {
    await page.locator('[data-testid="mode-option-agent"]').click();
    const last = await sendPrompt(page, FULL_PROMPT);

    await expect(last).toHaveClass(/assistant/);
    await expect(last).toContainText('Agent plan (2 steps)');
    await expect(last).toContainText(
      'Step 1: Write a Python function that calculates the Fibonacci sequence recursively.',
    );
    await expect(last).toContainText('Step 2: calculate the 10th Fibonacci number');

    // Step 1 yields the Python program (F(10) = 55); step 2 yields 2200.
    const code = last.locator('.markdown-body .code-block code.hljs').first();
    await expect(code).toContainText('def fibonacci');
    await expect(last).toContainText('55');
    await expect(last).toContainText('2200');

    // The regression we are guarding against: the old failure strings.
    await expect(last).not.toContainText("I didn't understand you");
    await expect(last).not.toContainText('unparseable');
  });

  // Issue #334 was reported with the UI in Russian (`ru`/`ru-RU`), but the
  // recursive-Fibonacci task and its aliases live in the catalog for every
  // supported language. A "Write a Python Fibonacci function recursively"
  // prompt asked for in any supported UI language must resolve to the same
  // verified program (F(10) = 55), not "I didn't understand you". This pins
  // the fix for every supported language (en, ru, hi, zh), not just Russian.
  test('generates the recursive Fibonacci program across every supported language', async ({
    page,
  }) => {
    const cases = [
      {
        language: 'en',
        prompt:
          'Write a Python function that calculates the Fibonacci sequence recursively.',
      },
      {
        language: 'ru',
        prompt:
          'Напиши на Python функцию, которая вычисляет последовательность Фибоначчи рекурсивно.',
      },
      {
        language: 'hi',
        prompt:
          'Python में फ़िबोनाची अनुक्रम की पुनरावर्ती गणना करने वाला फ़ंक्शन लिखो।',
      },
      {
        language: 'zh',
        prompt: '用 Python 写一个递归计算斐波那契数列的函数。',
      },
    ];

    for (const { language, prompt } of cases) {
      const message = await sendPrompt(page, prompt);

      const codeBlock = message.locator('.markdown-body .code-block').first();
      await expect(codeBlock, language).toBeVisible();
      await expect(codeBlock.locator('.code-block-lang'), language).toHaveText('python');

      const code = codeBlock.locator('code.hljs');
      await expect(code, language).toContainText('def fibonacci');
      await expect(code, language).toContainText('fibonacci(n - 1)');

      // The verified run prints F(10) = 55, regardless of UI language.
      await expect(message, language).toContainText('55');
      await expect(message, language).not.toContainText("I didn't understand you");
    }
  });
});
