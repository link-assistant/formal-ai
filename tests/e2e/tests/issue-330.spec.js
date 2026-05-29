// @ts-check
//
// Issue #330: the chat UI must syntax-highlight code blocks, expose a copy
// button on each code block, and offer a "copy the whole message as Markdown"
// button. These end-to-end tests drive the real browser bundle to prove the
// feature works against a freshly built `src/web` tree.
const { test, expect } = require('@playwright/test');

// A prompt that resolves to `write_program(rust, list_files)` and therefore
// returns a fenced ```rust code block — the canonical surface for this feature.
const RUST_PROMPT =
  'Напиши мне программу на Rust, которая выдаёт список файлов в текущей директории';

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

test.describe('Issue #330 — code highlighting and copy buttons', () => {
  test.beforeEach(async ({ context, page }) => {
    await context.grantPermissions(['clipboard-read', 'clipboard-write']);
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

  test('renders a syntax-highlighted, copyable code block', async ({ page }) => {
    const message = await sendPrompt(page, RUST_PROMPT);

    // The marked-rendered fence is wrapped in a `.code-block` shell.
    const codeBlock = message.locator('.markdown-body .code-block').first();
    await expect(codeBlock).toBeVisible();

    // The language label reflects the resolved grammar.
    await expect(codeBlock.locator('.code-block-lang')).toHaveText('rust');

    // The code element carries the `hljs` class and at least one token span,
    // proving the highlighter ran (it should colour `fn`, `let`, `use`, ...).
    const code = codeBlock.locator('code.hljs');
    await expect(code).toBeVisible();
    await expect(code.locator('.hljs-keyword').first()).toBeVisible();
    await expect(code).toContainText('fn main');

    // The per-block copy button copies the raw source to the clipboard.
    const copyButton = codeBlock.locator('[data-testid="code-copy-button"]');
    await expect(copyButton).toBeVisible();
    await copyButton.click();
    await expect(copyButton).toHaveAttribute('data-copied', 'true');

    const clipboard = await page.evaluate(() => navigator.clipboard.readText());
    expect(clipboard).toContain('fn main');
    expect(clipboard).toContain('read_dir');
    // The copied artefact is the raw code, not the surrounding markdown fences.
    expect(clipboard).not.toContain('```');
  });

  test('copies the whole message as Markdown', async ({ page }) => {
    const message = await sendPrompt(page, RUST_PROMPT);

    const copyMarkdown = message.locator('[data-testid="copy-markdown-button"]');
    await expect(copyMarkdown).toBeVisible();
    await copyMarkdown.click();
    await expect(copyMarkdown).toHaveAttribute('data-copied', 'true');

    const clipboard = await page.evaluate(() => navigator.clipboard.readText());
    // The whole-message copy preserves the Markdown fences and prose.
    expect(clipboard).toContain('```rust');
    expect(clipboard).toContain('fn main');
  });

  test('highlights every seeded program language', async ({ page }) => {
    const cases = [
      { prompt: 'Write me a Python program that lists files in the current directory', lang: 'python' },
      { prompt: 'Write me a Go program that lists files in the current directory', lang: 'go' },
      { prompt: 'Write me a Ruby program that lists files in the current directory', lang: 'ruby' },
    ];
    for (const { prompt, lang } of cases) {
      const message = await sendPrompt(page, prompt);
      const codeBlock = message.locator('.markdown-body .code-block').first();
      await expect(codeBlock.locator('.code-block-lang'), lang).toHaveText(lang);
      await expect(codeBlock.locator('code.hljs'), lang).toBeVisible();
      await expect(
        codeBlock.locator('.hljs-keyword, .hljs-title, .hljs-type').first(),
        lang,
      ).toBeVisible();
    }
  });

  // The highlighting + copy chrome is language-agnostic: a Rust `list_files`
  // program asked for in any supported UI language must render the same
  // `.code-block` shell with a highlighted, copyable `code.hljs`. This pins the
  // behavior for every supported language (en, ru, hi, zh), not just Russian.
  test('highlights and offers copy across every supported language', async ({ page }) => {
    const cases = [
      {
        language: 'en',
        prompt: 'Write me a Rust program that lists files in the current directory',
      },
      {
        language: 'ru',
        prompt:
          'Напиши мне программу на Rust, которая выдаёт список файлов в текущей директории',
      },
      {
        language: 'hi',
        prompt: 'Rust में वर्तमान निर्देशिका की फ़ाइलों की सूची देने वाला प्रोग्राम लिखो',
      },
      {
        language: 'zh',
        prompt: '用 Rust 写一个列出当前目录中的文件的程序',
      },
    ];

    for (const { language, prompt } of cases) {
      const message = await sendPrompt(page, prompt);

      const codeBlock = message.locator('.markdown-body .code-block').first();
      await expect(codeBlock, language).toBeVisible();
      await expect(codeBlock.locator('.code-block-lang'), language).toHaveText('rust');

      const code = codeBlock.locator('code.hljs');
      await expect(code, language).toBeVisible();
      await expect(code.locator('.hljs-keyword').first(), language).toBeVisible();

      const copyButton = codeBlock.locator('[data-testid="code-copy-button"]');
      await copyButton.click();
      await expect(copyButton, language).toHaveAttribute('data-copied', 'true');

      const clipboard = await page.evaluate(() => navigator.clipboard.readText());
      expect(clipboard, language).toContain('fn main');
      expect(clipboard, language).not.toContain('```');
    }
  });
});
