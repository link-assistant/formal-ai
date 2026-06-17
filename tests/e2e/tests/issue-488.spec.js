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
    firstStep: 'Read the request:',
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
    firstStep: 'Прочитать запрос:',
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
    firstStep: 'अनुरोध पढ़ें:',
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
    firstStep: '读取请求',
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
    // The naturalized preview must never leak raw snake_case meta-language step
    // identifiers (issue #488 pipeline stage 2 humanizes every step).
    await expect(preview).not.toContainText(
      /match_rule|dispatch_handler|invoke_tool|detect_language/i,
    );

    await toggle.click();
    await expect(toggle).toHaveAttribute('aria-expanded', 'true');

    const expandedList = preview.locator(
      '[data-testid="thinking-expanded-list"]',
    );
    await expect(expandedList).toBeVisible();
    // Default detail level is "detailed": every concrete reasoning step shows,
    // each surfacing real content (the prompt, the matched rule, the composed
    // answer) instead of a generic category label.
    expect(await expandedList.locator('li').count()).toBeGreaterThanOrEqual(6);
    await expect(expandedList).toContainText('Read the request:');
    await expect(expandedList).toContainText('Match the greeting rule.');
    await expect(expandedList).toContainText('Compose the answer:');
    await expect(expandedList).not.toContainText(
      /match_rule|dispatch_handler|invoke_tool|detect_language/i,
    );

    // Configurable granularity (issue #488): "standard" recursively folds the
    // low-level / symbolic children (the formalize tuple, the tool probe) out of
    // view and keeps only the high-level universal-algorithm phases plus the
    // final step.
    await page
      .locator('[data-testid="setting-thinking-detail"]')
      .selectOption('standard');
    await expect(expandedList.locator('li')).toHaveCount(5);
    await expect(expandedList).toContainText('Read the request:');
    await expect(expandedList).toContainText('Match the greeting rule.');
    await expect(expandedList).toContainText('Applied available context:');
    await expect(expandedList).not.toContainText(/symbolic form/i);

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
