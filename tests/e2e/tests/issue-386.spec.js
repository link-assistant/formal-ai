// @ts-check
// Issue #386: the prefilled "Report issue" body wasted space on settings that
// were already at their default. This suite locks in the trimmed report:
//   * the worker is folded into the version (`<version> (wasm)`);
//   * Mode/Status (manual), Diagnostics (off), and any default-valued User
//     Context field (Theme=auto, default sliders, inference-only location) are
//     omitted from a fresh-default report;
//   * the Reasoning Trace is dropped once earlier turns are trimmed to fit
//     GitHub's URL cap, since the dialog is then no longer complete;
//   * the "Attach full memory" section is a short pointer to the docs.
const { test, expect } = require('@playwright/test');

async function disableGreetingVariations(page) {
  await page.addInitScript(() => {
    try {
      window.localStorage.setItem(
        'formal-ai.preferences.v1',
        'demo_preferences\n  greetingVariations "off"',
      );
    } catch (_error) {
      // localStorage may be unavailable in some sandboxes.
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

async function reportBody(page, locator = '[data-testid="report-issue"]') {
  const href = await page.locator(locator).getAttribute('href');
  expect(href).toBeTruthy();
  return { body: new URL(href || '').searchParams.get('body') || '', href };
}

async function setRangeValue(page, testId, value) {
  await page.locator(`[data-testid="${testId}"]`).evaluate((node, nextValue) => {
    const valueSetter = Object.getOwnPropertyDescriptor(
      Object.getPrototypeOf(node),
      'value',
    )?.set;
    valueSetter.call(node, String(nextValue));
    node.dispatchEvent(new Event('input', { bubbles: true }));
    node.dispatchEvent(new Event('change', { bubbles: true }));
  }, value);
}

test.describe('Issue #386 - trimmed issue report', () => {
  test.beforeEach(async ({ page }) => {
    await disableGreetingVariations(page);
    await page.goto('./');
    await expect(page.locator('.app')).toBeVisible({ timeout: 15_000 });
    await switchToManualMode(page);
  });

  test('a fresh-default report omits default settings and folds the worker into the version', async ({
    page,
  }) => {
    const { body } = await reportBody(page);

    // The worker moves into the version line; the standalone Worker line is gone.
    expect(body).toMatch(/\*\*Version\*\*: .*\(wasm\)/);
    expect(body).not.toContain('**Worker**');

    // Manual mode is the interactive default, so Mode/Status are not reported.
    expect(body).not.toContain('**Mode**');
    expect(body).not.toContain('**Status**');

    // Diagnostics defaults to off and is only reported when enabled.
    expect(body).not.toContain('**Diagnostics**');

    // Default-valued User Context fields are omitted.
    expect(body).not.toContain('**Theme**');
    expect(body).not.toContain('**Guess probability**');
    expect(body).not.toContain('**Temperature**');
    expect(body).not.toContain('**Follow-up probability**');
    expect(body).not.toContain('**Location**');

    // The interesting, non-default context is still present.
    expect(body).toContain('## Environment');
    expect(body).toContain('**Version**');
    expect(body).toContain('## User Context');
    expect(body).toMatch(/\*\*UI\*\*: .* browser/);
  });

  test('the attach-memory section is a short pointer to the docs', async ({
    page,
  }) => {
    const { body } = await reportBody(page);

    expect(body).toContain('## Attach full memory (optional)');
    expect(body).toContain('docs/upload-memory.md');
    // The old multi-clause walkthrough (zip / Gist wording) is gone.
    expect(body).not.toContain('wrap it in a `.zip`');
    expect(body).not.toContain('GitHub Gist');
  });

  test('the reasoning trace is dropped once earlier turns are trimmed to fit', async ({
    page,
  }) => {
    const input = page.locator('[data-testid="chat-composer-input"]');
    const messages = page.locator('[data-testid="chat-message"]');

    // A long, Cyrillic-heavy dialog forces the URL fitter to drop earlier
    // turns, which marks the dialog as incomplete.
    let report = { body: '', href: '' };
    for (let i = 0; i < 12; i += 1) {
      const count = await messages.count();
      await input.fill(
        `Подскажи пожалуйста, как мне сделать шаг номер ${i} ` +
          'в моей довольно длинной задаче с множеством деталей и контекста ' +
          'который занимает достаточно много места в итоговом отчёте об ошибке.',
      );
      await page.locator('[data-testid="chat-composer-submit"]').click();
      await expect(messages).toHaveCount(count + 2, { timeout: 20_000 });

      report = await reportBody(page);
      if (report.body.includes('omitted')) break;
    }

    const { body, href } = report;
    expect(href.length).toBeLessThanOrEqual(8192);
    // Earlier turns were omitted, so the trace must not appear.
    expect(body).toContain('omitted');
    expect(body).not.toContain('## Reasoning Trace');
  });
});

test.describe('Issue #386 - reset settings to default', () => {
  test.beforeEach(async ({ page }) => {
    await disableGreetingVariations(page);
    await page.goto('./');
    await expect(page.locator('.app')).toBeVisible({ timeout: 15_000 });
    await switchToManualMode(page);
  });

  test('a fresh, default panel reports nothing to reset', async ({ page }) => {
    const reset = page.locator('[data-testid="settings-reset"]');
    await expect(reset).toBeVisible();
    // "greetingVariations off" is the only seeded non-default; everything else
    // is at its default, so only that one row may appear. The empty-state and
    // the disabled all-button track whatever is left modified.
    const empty = page.locator('[data-testid="settings-reset-empty"]');
    const greeting = page.locator(
      '[data-testid="settings-reset-greetingVariations"]',
    );
    await expect(greeting).toBeVisible();
    await expect(empty).toHaveCount(0);

    // Reset the one seeded change; the panel then declares everything default.
    await greeting.click();
    await expect(empty).toBeVisible();
    await expect(
      page.locator('[data-testid="settings-reset-all"]'),
    ).toBeDisabled();
  });

  test('a changed setting can be reset individually and in bulk', async ({
    page,
  }) => {
    // Clear the seeded greeting change so we start from a clean default panel.
    await page.locator('[data-testid="settings-reset-greetingVariations"]').click();
    await expect(
      page.locator('[data-testid="settings-reset-empty"]'),
    ).toBeVisible();

    // Change three independent settings of different shapes (slider, select,
    // text) and confirm each shows up as resettable.
    await setRangeValue(page, 'setting-temperature', 0);
    await page.locator('[data-testid="setting-theme"]').selectOption('dark');
    await page.locator('[data-testid="setting-assistant-name"]').fill('Astra');

    await expect(
      page.locator('[data-testid="settings-reset-temperature"]'),
    ).toBeVisible();
    await expect(
      page.locator('[data-testid="settings-reset-theme"]'),
    ).toBeVisible();
    await expect(
      page.locator('[data-testid="settings-reset-assistantName"]'),
    ).toBeVisible();

    // Reset just the theme; the control returns to "auto" and the row vanishes.
    await page.locator('[data-testid="settings-reset-theme"]').click();
    await expect(page.locator('[data-testid="setting-theme"]')).toHaveValue(
      'auto',
    );
    await expect(
      page.locator('[data-testid="settings-reset-theme"]'),
    ).toHaveCount(0);
    // The other two changes survive a single-setting reset.
    await expect(
      page.locator('[data-testid="settings-reset-temperature"]'),
    ).toBeVisible();
    await expect(
      page.locator('[data-testid="settings-reset-assistantName"]'),
    ).toBeVisible();

    // Reset-all clears everything that is left.
    await page.locator('[data-testid="settings-reset-all"]').click();
    await expect(
      page.locator('[data-testid="settings-reset-empty"]'),
    ).toBeVisible();
    await expect(
      page.locator('[data-testid="setting-assistant-name"]'),
    ).toHaveValue('');
    await expect(
      page.locator('[data-testid="settings-reset-all"]'),
    ).toBeDisabled();
  });
});

test.describe('Issue #386 - copy a conversation as Markdown', () => {
  async function sendPrompt(page, text) {
    const input = page.locator('[data-testid="chat-composer-input"]');
    await expect(input).toBeEnabled({ timeout: 5_000 });
    await input.fill(text);
    const messages = page.locator('[data-testid="chat-message"]');
    const count = await messages.count();
    await page.locator('[data-testid="chat-composer-submit"]').click();
    await expect(messages).toHaveCount(count + 2, { timeout: 20_000 });
  }

  test('the sidebar copies the full dialog, folding in reasoning under diagnostics', async ({
    context,
    page,
  }) => {
    await context.grantPermissions(['clipboard-read', 'clipboard-write']);
    // Diagnostics on so the export must fold reasoning in after the AI turn.
    await page.addInitScript(() => {
      window.localStorage.setItem(
        'formal-ai.preferences.v1',
        'demo_preferences\n  demoMode "off"\n  diagnosticsMode "on"\n  greetingVariations "off"',
      );
    });
    await page.goto('./');
    await expect(page.locator('.app')).toBeVisible({ timeout: 15_000 });
    await expect(page.locator('[data-testid="demo-status"]')).toHaveText(
      'Manual mode',
    );

    await sendPrompt(page, 'What is recursion?');

    const copy = page.locator('[data-testid="conversation-copy"]').first();
    await expect(copy).toBeVisible();
    await copy.click();
    await expect(copy).toHaveAttribute('data-copied', 'true');

    const clipboard = await page.evaluate(() =>
      navigator.clipboard.readText(),
    );
    // The whole dialog is present: a user turn and an assistant turn.
    expect(clipboard).toContain('### You');
    expect(clipboard).toContain('What is recursion?');
    // Diagnostics is on, so a reasoning section trails the AI message with a
    // numbered step list.
    expect(clipboard).toContain('#### ');
    expect(clipboard).toMatch(/\n1\. /);
  });

  test('without diagnostics the copy omits the reasoning section', async ({
    context,
    page,
  }) => {
    await context.grantPermissions(['clipboard-read', 'clipboard-write']);
    await page.addInitScript(() => {
      window.localStorage.setItem(
        'formal-ai.preferences.v1',
        'demo_preferences\n  demoMode "off"\n  diagnosticsMode "off"\n  greetingVariations "off"',
      );
    });
    await page.goto('./');
    await expect(page.locator('.app')).toBeVisible({ timeout: 15_000 });
    await expect(page.locator('[data-testid="demo-status"]')).toHaveText(
      'Manual mode',
    );

    await sendPrompt(page, 'What is recursion?');

    const copy = page.locator('[data-testid="conversation-copy"]').first();
    await copy.click();
    await expect(copy).toHaveAttribute('data-copied', 'true');

    const clipboard = await page.evaluate(() =>
      navigator.clipboard.readText(),
    );
    expect(clipboard).toContain('### You');
    expect(clipboard).toContain('What is recursion?');
    // Diagnostics is off, so no reasoning subsection is appended.
    expect(clipboard).not.toContain('#### ');
  });
});

test.describe('Issue #386 - cancel a program modification', () => {
  // The original bug: after building a reverse-sorted, path-argument file
  // lister, "Отмени сортировку" ("cancel the sorting") returned intent:unknown.
  // The cancel verb must be the data-derived inverse of reverse_sort, restoring
  // the ascending program while keeping the path argument.
  const FIRST_PROMPT =
    'Напиши мне программу на Rust, которая выдаёт список файлов в текущей директории';
  const PATH_ARG_PROMPT = 'Сделай так, чтобы программа принимала путь как аргумент';
  const REVERSE_SORT_PROMPT = 'Сделай сортировку результатов в обратном порядке';
  const CANCEL_SORT_PROMPT = 'Отмени сортировку';

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
      .evaluateAll((nodes) =>
        nodes.map((node) => node.getAttribute('data-step') || ''),
      );
  }

  test.beforeEach(async ({ page }) => {
    await disableGreetingVariations(page);
    await page.goto('./');
    await expect(page.locator('.app')).toBeVisible({ timeout: 15_000 });
    await switchToManualMode(page);
  });

  test('the cancel follow-up restores the ascending program (was: unknown)', async ({
    page,
  }) => {
    await sendPrompt(page, FIRST_PROMPT);
    await sendPrompt(page, PATH_ARG_PROMPT);
    const reversed = await sendPrompt(page, REVERSE_SORT_PROMPT);
    await expect(reversed.locator('.markdown-body')).toContainText('b.cmp(a)');

    const cancelled = await sendPrompt(page, CANCEL_SORT_PROMPT);

    // The follow-up answers with the program — never the "unknown" fallback
    // ("Unknown prompt: Отмени сортировку"), which has no code block at all.
    const body = cancelled.locator('.markdown-body');
    await expect(body).not.toContainText('Unknown prompt');
    // The reverse sort is removed: ascending sort, no descending comparator,
    // no .rev(); the path argument survives the cancel.
    await expect(body).toContainText('names.sort();');
    await expect(body).not.toContainText('b.cmp(a)');
    await expect(body).not.toContainText('.rev()');
    await expect(body).toContainText('env::args');
  });

  test('the cancel follow-up exposes the inverse-rule reasoning chain', async ({
    page,
  }) => {
    await page.locator('.diagnostics-toggle').click();

    await sendPrompt(page, FIRST_PROMPT);
    await sendPrompt(page, PATH_ARG_PROMPT);
    await sendPrompt(page, REVERSE_SORT_PROMPT);
    const cancelled = await sendPrompt(page, CANCEL_SORT_PROMPT);

    await expect(cancelled.locator('.intent')).toContainText(
      'intent:write_program',
    );
    await openAllDetails(cancelled);

    const stepNames = await readStepNames(cancelled);
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
      cancelled.locator(
        '[data-testid="diagnostics-step"][data-step="route_attempt"]',
      ),
    ).toContainText('selected_rule initial unknown reason no_seed_route');
    // The active artifact is the accumulated reverse-sorted variant.
    await expect(
      cancelled.locator(
        '[data-testid="diagnostics-step"][data-step="coreference_binding"]',
      ),
    ).toContainText(
      'referent=active_program_artifact task=list_files_arg_reverse_sort language=rust',
    );
    // The cancel verb is decomposed as the inverse of reverse_sort.
    await expect(
      cancelled.locator(
        '[data-testid="diagnostics-step"][data-step="modifier_detection"]',
      ),
    ).toContainText('cancel_reverse_sort');
    await expect(
      cancelled.locator(
        '[data-testid="diagnostics-step"][data-step="rule_construction"]',
      ),
    ).toContainText('cancel_reverse_sort__reverse_sort_list_files_arg');
    await expect(
      cancelled.locator(
        '[data-testid="diagnostics-step"][data-step="rule_verification"]',
      ),
    ).toContainText('status passed');
    // The plan lowers back to the unsorted path variant.
    await expect(
      cancelled.locator(
        '[data-testid="diagnostics-step"][data-step="program_plan"]',
      ),
    ).toContainText('resolved_task list_files_arg');
  });
});
