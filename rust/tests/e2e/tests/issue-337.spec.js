// @ts-check
const { test, expect } = require('@playwright/test');

// Issue #337: this exact production-demo prompt used to become a 2-step agent
// plan. Step 1 was misrouted to the configuration capability because of
// "available tools" + "programming language"; step 2 then fell through as an
// unknown standalone "format this" request.
const ISSUE_PROMPT = [
  'Navigate to github.com/link-assistant/formal-ai. Extract information about:',
  '1. The main programming language used',
  '2. Number of stars',
  '3. Last commit date',
  '4. List all available tools mentioned in the README',
  '',
  'Then format this as a JSON object.',
].join('\n');

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

test.describe('Issue #337 - GitHub repository extraction prompt', () => {
  test.beforeEach(async ({ page }) => {
    const readme = [
      '# formal-ai',
      '',
      '## Agentic AI Tools',
      '',
      '### Codex CLI',
      'Codex custom providers use the Responses wire API.',
      '',
      '### Claude Code',
      'Claude Code talks to the Anthropic Messages API.',
      '',
      '### OpenCode',
      'OpenCode can call formal-ai through its OpenAI-compatible provider package.',
      '',
      '### Link Assistant Agent CLI',
      'The Link Assistant Agent CLI accepts OpenCode-style provider/model selection.',
      '',
      '## Available tools',
      '- `http_fetch`',
      '- `url_navigate`',
      '- `web_search`',
      '- `wikipedia_lookup`',
    ].join('\n');

    await page.route('https://api.github.com/repos/link-assistant/formal-ai', async (route) => {
      await route.fulfill({
        contentType: 'application/json',
        body: JSON.stringify({
          full_name: 'link-assistant/formal-ai',
          html_url: 'https://github.com/link-assistant/formal-ai',
          language: 'Rust',
          stargazers_count: 42,
        }),
      });
    });
    await page.route(
      /^https:\/\/api\.github\.com\/repos\/link-assistant\/formal-ai\/commits/,
      async (route) => {
        await route.fulfill({
          contentType: 'application/json',
          body: JSON.stringify([
            {
              commit: {
                committer: { date: '2026-05-29T12:34:56Z' },
                author: { date: '2026-05-29T12:34:56Z' },
              },
            },
          ]),
        });
      },
    );
    await page.route('https://api.github.com/repos/link-assistant/formal-ai/readme', async (route) => {
      await route.fulfill({
        contentType: 'application/json',
        body: JSON.stringify({
          encoding: 'base64',
          content: Buffer.from(readme, 'utf8').toString('base64'),
        }),
      });
    });

    await page.addInitScript(() => {
      window.localStorage.setItem(
        'formal-ai.preferences.v1',
        'demo_preferences\n  demoMode "off"\n  diagnosticsMode "on"\n  greetingVariations "off"\n  agentMode "on"',
      );
    });
    await page.goto('./');
    await expect(page.locator('.app')).toBeVisible({ timeout: 15_000 });
    await expect(page.locator('[data-testid="demo-status"]')).toHaveText('Manual mode');
    await expect(page.locator('[data-testid="mode-option-agent"]')).toHaveAttribute(
      'aria-checked',
      'true',
    );
  });

  test('returns repository facts as JSON instead of capability and unknown responses', async ({
    page,
  }) => {
    const message = await sendPrompt(page, ISSUE_PROMPT);

    await expect(message).toContainText('"repository": "link-assistant/formal-ai"');
    await expect(message).toContainText('"mainProgrammingLanguage": "Rust"');
    await expect(message).toContainText('"stars": 42');
    await expect(message).toContainText('"lastCommitDate": "2026-05-29T12:34:56Z"');
    await expect(message).toContainText('"http_fetch"');
    await expect(message).toContainText('"url_navigate"');
    await expect(message).toContainText('"web_search"');
    await expect(message).toContainText('"wikipedia_lookup"');
    await expect(message).toContainText('"Codex CLI"');
    await expect(message).toContainText('"Claude Code"');
    await expect(message).toContainText('"OpenCode"');
    await expect(message).toContainText('"Link Assistant Agent CLI"');

    await expect(message).not.toContainText('message-driven configuration');
    await expect(message).not.toContainText('That one is new to me');
    await expect(message).not.toContainText('Agent plan (2 steps)');
  });
});
