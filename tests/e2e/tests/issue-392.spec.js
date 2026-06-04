// @ts-check
//
// Issue #392: conversation-level "copy as Markdown" must work under the same
// activation-sensitive browser constraints as the code-block and message copy
// actions. These tests do not grant broad clipboard permissions; instead they
// provide a small clipboard shim that only accepts writes during the immediate
// click task, matching browsers that reject clipboard writes after async work.
const { test, expect } = require('@playwright/test');

const RUST_PROMPT =
  'Write me a Rust program that lists files in the current directory';

const UI_LANGUAGE_CASES = [
  { language: 'en', name: 'English', userHeading: '### You' },
  { language: 'ru', name: 'Russian', userHeading: '### Вы' },
  { language: 'hi', name: 'Hindi', userHeading: '### आप' },
  { language: 'zh', name: 'Chinese', userHeading: '### 你' },
];

async function installActivationBoundClipboard(page, uiLanguage) {
  await page.addInitScript((language) => {
    const state = {
      writes: [],
      events: [],
      active: false,
    };

    const activateForCurrentTask = () => {
      state.active = true;
      setTimeout(() => {
        state.active = false;
      }, 0);
    };

    window.addEventListener('pointerdown', activateForCurrentTask, true);
    window.addEventListener('keydown', activateForCurrentTask, true);

    Object.defineProperty(window, '__issue392Clipboard', {
      value: state,
      configurable: true,
    });

    Object.defineProperty(navigator, 'clipboard', {
      value: {
        writeText: async (text) => {
          state.events.push({
            method: 'writeText',
            active: state.active,
            text: String(text),
          });
          if (!state.active) {
            throw new DOMException(
              'Clipboard write requires transient activation',
              'NotAllowedError',
            );
          }
          state.writes.push(String(text));
        },
        readText: async () => state.writes[state.writes.length - 1] || '',
      },
      configurable: true,
    });

    const originalExecCommand =
      typeof document.execCommand === 'function'
        ? document.execCommand.bind(document)
        : null;
    document.execCommand = (command, ...args) => {
      if (String(command).toLowerCase() !== 'copy') {
        return originalExecCommand
          ? originalExecCommand(command, ...args)
          : false;
      }
      const active = state.active;
      const focused = document.activeElement;
      const text = focused && 'value' in focused ? String(focused.value) : '';
      state.events.push({ method: 'execCommand', active, text });
      if (!active) return false;
      state.writes.push(text);
      return true;
    };

    window.localStorage.setItem(
      'formal-ai.preferences.v1',
      [
        'demo_preferences',
        '  demoMode "off"',
        '  diagnosticsMode "on"',
        '  greetingVariations "off"',
        `  uiLanguage "${language}"`,
      ].join('\n'),
    );
  }, uiLanguage);
}

async function openApp(page, uiLanguage) {
  await installActivationBoundClipboard(page, uiLanguage);
  await page.goto('./');
  await expect(page.locator('.app')).toBeVisible({ timeout: 15_000 });
  await expect(page.locator('[data-testid="demo-status"]')).toBeVisible();
  await expect(page.locator('[data-testid="setting-ui-language"]')).toHaveValue(
    uiLanguage,
  );
  await expect(page.locator('html')).toHaveAttribute('lang', uiLanguage);
  await expect(page.locator('.status')).toBeVisible();
}

async function sendPrompt(page, text = RUST_PROMPT) {
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

async function clipboardWrites(page) {
  return page.evaluate(() => window.__issue392Clipboard.writes);
}

test.describe('Issue #392 - activation-safe copy actions', () => {
  for (const { language, name, userHeading } of UI_LANGUAGE_CASES) {
    test(`copies code blocks, messages, and conversations in ${name}`, async ({
      page,
    }) => {
      await openApp(page, language);

      const message = await sendPrompt(page);

      const codeCopy = message
        .locator('.markdown-body .code-block [data-testid="code-copy-button"]')
        .first();
      await expect(codeCopy).toBeVisible();
      await codeCopy.click();
      await expect(codeCopy).toHaveAttribute('data-copied', 'true');

      let writes = await clipboardWrites(page);
      expect(writes.at(-1)).toContain('fn main');
      expect(writes.at(-1)).not.toContain('```');

      const messageCopy = message.locator(
        '[data-testid="copy-markdown-button"]',
      );
      await messageCopy.click();
      await expect(messageCopy).toHaveAttribute('data-copied', 'true');

      writes = await clipboardWrites(page);
      expect(writes.at(-1)).toContain('```rust');
      expect(writes.at(-1)).toContain('fn main');

      const conversationCopy = page
        .locator('[data-testid="conversation-copy"]')
        .first();
      await expect(conversationCopy).toBeVisible();
      await conversationCopy.click();
      await expect(conversationCopy).toHaveAttribute('data-copied', 'true');

      writes = await clipboardWrites(page);
      expect(writes.at(-1)).toContain('# Write me a Rust program');
      expect(writes.at(-1)).toContain(userHeading);
      expect(writes.at(-1)).toContain(RUST_PROMPT);
      expect(writes.at(-1)).toContain('### formal-ai');
      expect(writes.at(-1)).toContain('```rust');
    });
  }
});
