// @ts-check
const { test, expect } = require('@playwright/test');

const SECTION_IDS = [
  'drawer-menu-actions',
  'sidebar-desktop',
  'sidebar-services',
  'sidebar-conversations',
  'sidebar-settings',
  'sidebar-prompts',
  'sidebar-tools',
  'sidebar-trace',
];

async function loadFreshApp(page) {
  await page.addInitScript(() => {
    try {
      window.localStorage.removeItem('formal-ai.preferences.v1');
    } catch (_error) {
      // localStorage may be unavailable; the app falls back to shipped defaults.
    }
  });
  await page.goto('./');
  await expect(page.locator('.app')).toBeVisible({ timeout: 15_000 });
  await expect(page.locator('[data-testid="sidebar-conversations"]')).toBeVisible();
  await expect(page.locator('[data-testid="sidebar-prompts"]')).toBeVisible();
}

async function sidebarStates(page) {
  return page.evaluate((ids) => {
    const states = {};
    for (const id of ids) {
      const node = document.querySelector(`[data-testid="${id}"]`);
      if (node) {
        states[id] = node.getAttribute('data-collapsed');
      }
    }
    return states;
  }, SECTION_IDS);
}

async function expectOnlyExpanded(page, expandedIds) {
  const expanded = new Set(expandedIds);
  const states = await sidebarStates(page);
  for (const [id, collapsed] of Object.entries(states)) {
    expect(collapsed, id).toBe(expanded.has(id) ? 'false' : 'true');
  }
}

test.describe('Issue #476 - sidebar section isolation', () => {
  test('first run expands only conversations and example prompts', async ({ page }) => {
    await loadFreshApp(page);

    await expectOnlyExpanded(page, ['sidebar-conversations', 'sidebar-prompts']);
  });

  test('normal header clicks toggle only the clicked section', async ({ page }) => {
    await loadFreshApp(page);

    const menu = page.locator('[data-testid="drawer-menu-actions"]');
    await menu.locator('.sidebar-section-header').click();

    await expect(menu).toHaveAttribute('data-collapsed', 'false');
    await expect(page.locator('[data-testid="sidebar-conversations"]')).toHaveAttribute(
      'data-collapsed',
      'false',
    );
    await expect(page.locator('[data-testid="sidebar-prompts"]')).toHaveAttribute(
      'data-collapsed',
      'false',
    );
  });

  test('right-side action expands only that section and collapses the others', async ({
    page,
  }) => {
    await loadFreshApp(page);

    const settings = page.locator('[data-testid="sidebar-settings"]');
    await settings.locator('[data-testid="sidebar-section-isolate"]').click();

    await expectOnlyExpanded(page, ['sidebar-settings']);
  });

  test('shift-clicking a section title expands only that section', async ({ page }) => {
    await loadFreshApp(page);

    const settingsHeader = page.locator(
      '[data-testid="sidebar-settings"] .sidebar-section-header',
    );
    await settingsHeader.click({ modifiers: ['Shift'] });

    await expectOnlyExpanded(page, ['sidebar-settings']);
  });
});
