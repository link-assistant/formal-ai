// @ts-check
const { test, expect } = require('@playwright/test');

// Issue #360: the Diagnostics toggle must expose the full write_program
// reasoning chain for the issue #349 turn-5 follow-up, including the recovered
// active program, modifier detection, synthesized rule, verification, and plan.

const FIRST_PROMPT =
  'Напиши мне программу на Rust, которая выдаёт список файлов в текущей директории';
const PATH_ARG_PROMPT = 'Сделай так, чтобы программа принимала путь как аргумент';
const REVERSE_SORT_PROMPT = 'Сделай сортировку результатов в обратном порядке';

async function disableGreetingVariations(page) {
  await page.addInitScript(() => {
    try {
      window.localStorage.setItem(
        'formal-ai.preferences.v1',
        'demo_preferences\n  greetingVariations "off"',
      );
    } catch (_error) {
      // ignore - localStorage may be unavailable in some sandboxes.
    }
  });
}

async function switchToManualMode(page) {
  const demoToggle = page.locator('.mode-toggle');
  await expect(demoToggle).toContainText(/Demo on|Demo off|Демо/, {
    timeout: 10_000,
  });
  await demoToggle.click();
  await expect(page.locator('[data-testid="chat-composer-input"]')).toBeEnabled({
    timeout: 5_000,
  });
}

async function sendPrompt(page, text) {
  const input = page.locator('[data-testid="chat-composer-input"]');
  await expect(input).toBeEnabled({ timeout: 5_000 });
  await input.fill(text);
  const messages = page.locator('[data-testid="chat-message"]');
  const initial = await messages.count();
  await page.locator('[data-testid="chat-composer-submit"]').click();
  await expect(messages).toHaveCount(initial + 2, { timeout: 20_000 });
  return messages.last();
}

async function openAllDetails(messageLocator) {
  await messageLocator.evaluate((node) => {
    for (const det of node.querySelectorAll('details.diagnostics-detail')) {
      det.open = true;
    }
  });
}

async function readStepNames(messageLocator) {
  return await messageLocator
    .locator('[data-testid="diagnostics-step"]')
    .evaluateAll((nodes) => nodes.map((node) => node.getAttribute('data-step') || ''));
}

test.describe('Issue #360 - write_program diagnostics chain', () => {
  test.beforeEach(async ({ page }) => {
    await disableGreetingVariations(page);
    await page.goto('./');
    await expect(page.locator('.app')).toBeVisible({ timeout: 15_000 });
    await switchToManualMode(page);
  });

  test('turn-5 reverse-sort follow-up exposes synthesized reasoning steps', async ({
    page,
  }) => {
    await page.locator('.diagnostics-toggle').click();

    await sendPrompt(page, FIRST_PROMPT);
    await sendPrompt(page, PATH_ARG_PROMPT);
    const last = await sendPrompt(page, REVERSE_SORT_PROMPT);

    await expect(last.locator('.intent')).toContainText('intent:write_program');
    await expect(last.locator('.markdown-body')).toContainText(
      'names.sort_by(|a, b| b.cmp(a))',
    );

    await openAllDetails(last);

    const stepNames = await readStepNames(last);
    for (const expected of [
      'route_attempt',
      'coreference_binding',
      'modifier_detection',
      'rule_construction',
      'rule_verification',
      'program_plan',
      'deformalize',
    ]) {
      expect(stepNames).toContain(expected);
    }

    await expect(
      last.locator('[data-testid="diagnostics-step"][data-step="route_attempt"]'),
    ).toContainText('selected_rule initial unknown reason no_seed_route');
    await expect(
      last.locator('[data-testid="diagnostics-step"][data-step="coreference_binding"]'),
    ).toContainText('referent=active_program_artifact task=list_files_arg language=rust');
    await expect(
      last.locator('[data-testid="diagnostics-step"][data-step="modifier_detection"]'),
    ).toContainText('reverse_sort');
    await expect(
      last.locator('[data-testid="diagnostics-step"][data-step="rule_construction"]'),
    ).toContainText('rule_synthesis_candidate');
    await expect(
      last.locator('[data-testid="diagnostics-step"][data-step="rule_verification"]'),
    ).toContainText('status passed');
    await expect(
      last.locator('[data-testid="diagnostics-step"][data-step="program_plan"]'),
    ).toContainText('resolved_task list_files_arg_reverse_sort');
  });
});
