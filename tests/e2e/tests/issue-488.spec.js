// @ts-check
const { test, expect } = require('@playwright/test');

const LANGUAGE_CASES = [
  {
    language: 'en',
    name: 'English',
    thinking: 'Thinking',
    expand: 'Expand',
    collapse: 'Collapse',
    currentLabel: 'Current thinking step',
    contextPrefix: 'Applied available context:',
    firstStep: 'Received the user request.',
    thinkingDetailLabel: 'Thinking detail',
    thinkingDetailOptions: ['Brief', 'Standard', 'Detailed'],
  },
  {
    language: 'ru',
    name: 'Russian',
    thinking: 'Мышление',
    expand: 'Развернуть',
    collapse: 'Свернуть',
    currentLabel: 'Текущий шаг мышления',
    contextPrefix: 'Применен доступный контекст:',
    firstStep: 'Получен запрос пользователя.',
    thinkingDetailLabel: 'Детализация мышления',
    thinkingDetailOptions: ['Кратко', 'Стандартно', 'Подробно'],
  },
  {
    language: 'hi',
    name: 'Hindi',
    thinking: 'सोच',
    expand: 'फैलाएं',
    collapse: 'समेटें',
    currentLabel: 'मौजूदा सोच चरण',
    contextPrefix: 'उपलब्ध context लागू किया:',
    firstStep: 'उपयोगकर्ता का अनुरोध मिला.',
    thinkingDetailLabel: 'सोच का विवरण',
    thinkingDetailOptions: ['संक्षिप्त', 'मानक', 'विस्तृत'],
  },
  {
    language: 'zh',
    name: 'Chinese',
    thinking: '思考',
    expand: '展开',
    collapse: '折叠',
    currentLabel: '当前思考步骤',
    contextPrefix: '已应用可用上下文',
    firstStep: '已接收用户请求。',
    thinkingDetailLabel: '思考详细程度',
    thinkingDetailOptions: ['简略', '标准', '详细'],
  },
];

async function bootManualChat(page) {
  await page.addInitScript(() => {
    try {
      window.localStorage.setItem(
        'formal-ai.preferences.v1',
        [
          'demo_preferences',
          '  theme "light"',
          '  demoMode "off"',
          '  diagnosticsMode "off"',
          '  greetingVariations "off"',
        ].join('\n'),
      );
    } catch (_error) {
      // localStorage may be unavailable in hardened browser contexts.
    }
  });
  await page.goto('./');
  await expect(page.locator('.app')).toBeVisible({ timeout: 15_000 });
  await expect(page.locator('[data-testid="demo-status"]')).toHaveText(
    'Manual mode',
  );
}

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
  return assistantMessage;
}

test.describe('Issue #488 - visible thinking preview', () => {
  test('shows collapsed human-readable thinking by default and expands details', async ({
    page,
  }) => {
    await bootManualChat(page);

    const assistantMessage = await sendPrompt(page, 'Hi');

    await expect(assistantMessage.locator('.thinking-steps')).toHaveCount(0);
    const preview = assistantMessage.locator('[data-testid="thinking-preview"]');
    await expect(preview).toBeVisible();

    const toggle = preview.locator('[data-testid="thinking-preview-toggle"]');
    await expect(toggle).toHaveAttribute('aria-expanded', 'false');
    await expect(
      preview.locator('[data-testid="thinking-preview-previous"]'),
    ).toBeVisible();
    await expect(
      preview.locator('[data-testid="thinking-preview-current"]'),
    ).toContainText('Applied available context:');
    await expect(preview).not.toContainText(
      /match_rule|dispatch_handler|deformalize|formalize/i,
    );

    await toggle.click();
    await expect(toggle).toHaveAttribute('aria-expanded', 'true');

    const expandedList = preview.locator(
      '[data-testid="thinking-expanded-list"]',
    );
    await expect(expandedList).toBeVisible();
    expect(await expandedList.locator('li').count()).toBeGreaterThanOrEqual(6);
    await expect(expandedList).toContainText('Received the user request.');
    await expect(expandedList).toContainText('Matched the greeting rule.');
    await expect(expandedList).toContainText(
      'Prepared the answer in readable text.',
    );
    await expect(expandedList).not.toContainText(
      /match_rule|dispatch_handler|deformalize|formalize/i,
    );

    await page
      .locator('[data-testid="setting-thinking-detail"]')
      .selectOption('brief');
    await expect(expandedList.locator('li')).toHaveCount(1);
    await expect(expandedList).toContainText('Applied available context:');
    await expect(
      page.locator('[data-testid="settings-reset-thinkingDetailLevel"]'),
    ).toBeVisible();
  });

  test('localizes thinking preview and detail settings across supported languages', async ({
    page,
  }) => {
    await bootManualChat(page);

    for (const locale of LANGUAGE_CASES) {
      await page
        .locator('[data-testid="setting-ui-language"]')
        .selectOption(locale.language);
      await expect(page.locator('html')).toHaveAttribute(
        'lang',
        locale.language,
      );

      const detailSelect = page.locator(
        '[data-testid="setting-thinking-detail"]',
      );
      await expect(
        detailSelect.locator('xpath=ancestor::label[1]'),
      ).toContainText(locale.thinkingDetailLabel);
      await expect(detailSelect.locator('option')).toHaveText(
        locale.thinkingDetailOptions,
      );

      const assistantMessage = await sendPrompt(page, 'Hi');
      const preview = assistantMessage.locator(
        '[data-testid="thinking-preview"]',
      );
      await expect(preview).toHaveAttribute('aria-label', locale.thinking);

      const toggle = preview.locator('[data-testid="thinking-preview-toggle"]');
      await expect(toggle).toHaveText(locale.expand);
      await expect(
        preview.locator('[data-testid="thinking-preview-current"]'),
      ).toHaveAttribute('aria-label', locale.currentLabel);
      await expect(
        preview.locator('[data-testid="thinking-preview-current"]'),
      ).toContainText(locale.contextPrefix);

      await toggle.click();
      await expect(toggle).toHaveText(locale.collapse);
      await expect(
        preview.locator('[data-testid="thinking-expanded-list"]'),
      ).toContainText(locale.firstStep);
    }
  });
});
