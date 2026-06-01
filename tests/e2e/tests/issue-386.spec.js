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
    for (let i = 0; i < 12; i += 1) {
      const count = await messages.count();
      await input.fill(
        `Подскажи пожалуйста, как мне сделать шаг номер ${i} ` +
          'в моей довольно длинной задаче с множеством деталей и контекста ' +
          'который занимает достаточно много места в итоговом отчёте об ошибке.',
      );
      await page.locator('[data-testid="chat-composer-submit"]').click();
      await expect(messages).toHaveCount(count + 2, { timeout: 20_000 });
    }

    const { body, href } = await reportBody(page);
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
