// @ts-check
const { test, expect } = require('@playwright/test');

async function expandSidebarSection(page, testId) {
  const section = page.locator(`[data-testid="${testId}"]`);
  await expect(section).toBeVisible();
  if ((await section.getAttribute('data-collapsed')) === 'true') {
    await section.locator('.sidebar-section-header').click();
  }
  await expect(section).toHaveAttribute('data-collapsed', 'false');
}

async function loadedModelScripts(page) {
  return page.evaluate(() =>
    Array.from(document.scripts)
      .map((script) => script.src)
      .filter((src) => /webllm|formalization-model|model-fallback/i.test(src)),
  );
}

const FORMALIZATION_MODEL_LANGUAGE_CASES = [
  {
    language: 'en',
    label: 'Small-model formalization fallback',
    warning: 'No model is bundled',
    modelLabel: 'Formalization model',
  },
  {
    language: 'ru',
    label: 'Резервная формализация малой моделью',
    warning: 'Модель не входит в сборку',
    modelLabel: 'Модель формализации',
  },
  {
    language: 'hi',
    label: 'Small-model formalization fallback',
    warning: 'कोई model bundle नहीं है',
    modelLabel: 'Formalization model',
  },
  {
    language: 'zh',
    label: '小模型形式化回退',
    warning: '应用不内置模型',
    modelLabel: '形式化模型',
  },
];

test.describe('Issue #483 experimental formalization model fallback', () => {
  test('is off by default and does not load a model runtime on initial page load', async ({
    page,
  }) => {
    await page.goto('./');
    await expect(page.locator('.app')).toBeVisible({ timeout: 15_000 });

    expect(await loadedModelScripts(page)).toEqual([]);

    await expandSidebarSection(page, 'sidebar-settings');
    const toggle = page.locator(
      '[data-testid="setting-experimental-formalization-model-fallback"]',
    );
    await expect(toggle).toBeVisible();
    await expect(toggle).not.toBeChecked();
    await expect(page.locator('[data-testid="setting-formalization-model-id"]'))
      .toBeDisabled();
    await expect(page.locator('[data-testid="setting-experimental-formalization-model-warning"]'))
      .toContainText(/No model is bundled/i);
  });

  test('shows only fitting WebGPU models sorted by public rating after opt-in', async ({
    page,
  }) => {
    await page.addInitScript(() => {
      Object.defineProperty(navigator, 'gpu', {
        configurable: true,
        value: {},
      });
      Object.defineProperty(navigator, 'deviceMemory', {
        configurable: true,
        value: 1,
      });
    });

    await page.goto('./');
    await expect(page.locator('.app')).toBeVisible({ timeout: 15_000 });
    await expandSidebarSection(page, 'sidebar-settings');

    await page.locator(
      '[data-testid="setting-experimental-formalization-model-fallback"]',
    ).check();

    const select = page.locator('[data-testid="setting-formalization-model-id"]');
    await expect(select).toBeEnabled();
    const options = await select.locator('option').evaluateAll((nodes) =>
      nodes.map((node) => ({
        value: node.value,
        rating: Number(node.getAttribute('data-public-rating') || '0'),
        vram: Number(node.getAttribute('data-vram-required-mb') || '0'),
      })),
    );

    expect(options.map((option) => option.value)).toEqual([
      'auto',
      'SmolLM2-360M-Instruct-q4f16_1-MLC',
      'Qwen2.5-0.5B-Instruct-q4f16_1-MLC',
    ]);
    const modelOptions = options.slice(1);
    expect(modelOptions.every((option) => option.vram <= 1024)).toBe(true);
    expect(modelOptions[0].rating).toBeGreaterThanOrEqual(modelOptions[1].rating);
    expect(await loadedModelScripts(page)).toEqual([]);
  });

  test('localizes the formalization fallback setting across supported languages', async ({
    page,
  }) => {
    await page.goto('./');
    await expect(page.locator('.app')).toBeVisible({ timeout: 15_000 });
    await expandSidebarSection(page, 'sidebar-settings');

    for (const { language, label, warning, modelLabel } of FORMALIZATION_MODEL_LANGUAGE_CASES) {
      await page.locator('[data-testid="setting-ui-language"]').selectOption(language);
      const setting = page.locator('[data-testid="settings-formalization-model"]');
      await expect(setting).toContainText(label);
      await expect(setting).toContainText(modelLabel);
      await expect(page.locator('[data-testid="setting-experimental-formalization-model-warning"]'))
        .toContainText(warning);
    }
  });
});
