// @ts-check
const { test, expect } = require('@playwright/test');

const UNKNOWN_ANSWER_MARKER = 'That one is new to me';
const RUSSIAN_UNKNOWN_ANSWER_MARKER = 'Я тебя не понял';
const SAMPLE_TEXT = [
  'Formal AI issue 535 sample text for originality checking.',
  'The variation-tech-model manual explains a deterministic links model.',
  'This unique sentence should be visible to the solver as attachment text.',
].join('\n');

async function switchToManualMode(page) {
  const demoToggle = page.locator('.mode-toggle');
  await expect(demoToggle).toContainText(/Demo on|Demo off|Демо/, { timeout: 10_000 });
  await demoToggle.click();
  await expect(page.locator('[data-testid="demo-status"]')).toHaveText('Manual mode');
  await expect(page.locator('[data-testid="chat-composer-input"]')).toBeEnabled({
    timeout: 5_000,
  });
}

async function sendPrompt(page, text) {
  const input = page.locator('[data-testid="chat-composer-input"]');
  await expect(input).toBeEnabled({ timeout: 5_000 });
  await input.fill(text);
  const messages = page.locator('[data-testid="chat-message"]');
  const initialAssistant = await page.locator('[data-testid="chat-message"].assistant').count();
  await page.locator('[data-testid="chat-composer-submit"]').click();
  await expect
    .poll(async () => page.locator('[data-testid="chat-message"].assistant').count(), {
      timeout: 20_000,
    })
    .toBeGreaterThan(initialAssistant);
  await expect(messages.last()).toHaveClass(/assistant/);
  return page.locator('[data-testid="chat-message"].assistant').last();
}

test.describe('Issue #535 - text attachment originality checks', () => {
  test.beforeEach(async ({ page }) => {
    await page.addInitScript(() => {
      try {
        window.localStorage.setItem(
          'formal-ai.preferences.v1',
          'demo_preferences\n  diagnosticsMode "on"\n  greetingVariations "off"',
        );
      } catch (_error) {
        // localStorage may be unavailable in restricted browser contexts.
      }
    });
    await page.goto('./');
    await expect(page.locator('.app')).toBeVisible({ timeout: 15_000 });
    await switchToManualMode(page);
  });

  test('routes a Russian plagiarism request with a text attachment instead of unknown', async ({
    page,
  }) => {
    await page.locator('[data-testid="composer-attachment-input"]').setInputFiles({
      name: 'variation-tech-model-manual.txt',
      mimeType: 'text/plain',
      buffer: Buffer.from(SAMPLE_TEXT, 'utf8'),
    });
    await expect(page.locator('[data-testid="composer-attachment-status"]')).toContainText(
      /1 attached/i,
    );

    const last = await sendPrompt(page, 'Проверь данный текст на уникальность и на плагиат');

    await expect(last.locator('.intent')).toContainText('intent:document_originality_check');
    await expect(last).toContainText(/уникаль/i);
    await expect(last).toContainText(/плагиат/i);
    await expect(last).not.toContainText(UNKNOWN_ANSWER_MARKER);
    await expect(last).not.toContainText(RUSSIAN_UNKNOWN_ANSWER_MARKER);

    const evidence = last.locator('.evidence-list');
    await expect(evidence).toContainText(
      'document_originality_check:attachment:variation-tech-model-manual.txt',
    );
    await expect(evidence).toContainText(
      'read_local_file:request:variation-tech-model-manual.txt',
    );
    await expect(evidence).toContainText('document_originality_check:text_sample:present');
    await expect(evidence).toContainText('web_search:query_kind:document_originality_check');

    // Issue #535 comment 4754747438: every statement is weighed with
    // relative-meta-logic — assumed true, raised by trusted original-first
    // sources, reposts ignored. The web worker replays the same deterministic
    // offline plan the Rust engine records, so these links must appear too.
    await expect(evidence).toContainText('relative_meta_logic:assumed_prior:0.600000');
    await expect(evidence).toContainText(
      'relative_meta_logic:trusted_source_tier:original_first_party:weight=1.000000',
    );
    await expect(evidence).toContainText(
      'relative_meta_logic:trusted_source_tier:original_journalism:weight=0.850000',
    );
    await expect(evidence).toContainText('relative_meta_logic:ignored_source_tier:unoriginal');
    await expect(evidence).toContainText('statement_verification:statement_count:');
    await expect(evidence).toContainText('statement_verification:query:');
    await expect(evidence).toContainText(
      'statement_verification:assessment:prior=0.600000 support=0.000000 contradiction=0.000000 posterior=0.600000 ignored=0',
    );
  });
});
