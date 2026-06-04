// @ts-check
const { test, expect } = require('@playwright/test');

async function pinPreferences(page, overrides = '') {
  await page.addInitScript((extra) => {
    try {
      window.localStorage.setItem(
        'formal-ai.preferences.v1',
        `demo_preferences
  demoMode "off"
  greetingVariations "off"${extra}`,
      );
    } catch (_error) {
      // localStorage may be unavailable in hardened browser contexts.
    }
  }, overrides);
}

async function boot(page, overrides = '') {
  await pinPreferences(page, overrides);
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
  const initial = await messages.count();
  await page.locator('[data-testid="chat-composer-submit"]').click();
  await expect(messages).toHaveCount(initial + 2, { timeout: 20_000 });
  return messages.last();
}

async function openAllDetails(messageLocator) {
  await messageLocator.evaluate((node) => {
    for (const detail of node.querySelectorAll('details.diagnostics-detail')) {
      detail.open = true;
    }
  });
}

function parseRgb(value) {
  const match = value.match(/rgba?\(([^)]+)\)/);
  if (!match) return null;
  const normalized = match[1].trim().replace(/\s*\/\s*/, ' ');
  const channels = normalized.includes(',')
    ? match[1].split(',')
    : normalized.split(/\s+/);
  const [r, g, b, a = 1] = channels
    .map((part) => Number.parseFloat(part.trim()));
  if (![r, g, b, a].every(Number.isFinite)) return null;
  return { r, g, b, a };
}

function luminance({ r, g, b }) {
  const channel = (value) => {
    const normalized = value / 255;
    return normalized <= 0.03928
      ? normalized / 12.92
      : ((normalized + 0.055) / 1.055) ** 2.4;
  };
  return 0.2126 * channel(r) + 0.7152 * channel(g) + 0.0722 * channel(b);
}

async function visibleLightSurfaces(page, selectors) {
  return await page.evaluate(
    ({ targetSelectors }) => {
      const parse = (value) => {
        const match = value.match(/rgba?\(([^)]+)\)/);
        if (!match) return null;
        const normalized = match[1].trim().replace(/\s*\/\s*/, ' ');
        const channels = normalized.includes(',')
          ? match[1].split(',')
          : normalized.split(/\s+/);
        const [r, g, b, a = 1] = channels
          .map((part) => Number.parseFloat(part.trim()));
        if (![r, g, b, a].every(Number.isFinite) || a === 0) return null;
        return { r, g, b };
      };
      const luma = ({ r, g, b }) => {
        const channel = (value) => {
          const normalized = value / 255;
          return normalized <= 0.03928
            ? normalized / 12.92
            : ((normalized + 0.055) / 1.055) ** 2.4;
        };
        return 0.2126 * channel(r) + 0.7152 * channel(g) + 0.0722 * channel(b);
      };
      return targetSelectors.flatMap((selector) =>
        Array.from(document.querySelectorAll(selector))
          .filter((node) => {
            const rect = node.getBoundingClientRect();
            const style = window.getComputedStyle(node);
            return (
              rect.width > 0 &&
              rect.height > 0 &&
              style.visibility !== 'hidden' &&
              style.display !== 'none'
            );
          })
          .map((node) => {
            const color = parse(window.getComputedStyle(node).backgroundColor);
            return {
              selector,
              className: node.className || node.tagName.toLowerCase(),
              background: window.getComputedStyle(node).backgroundColor,
              luminance: color ? luma(color) : 0,
            };
          })
          .filter((entry) => entry.luminance > 0.55),
      );
    },
    { targetSelectors: selectors },
  );
}

test.describe('Issue #388 - adaptive header actions', () => {
  test('desktop header switches to icon-only controls before the reported width clips', async ({
    page,
  }) => {
    await page.setViewportSize({ width: 1824, height: 1115 });
    await boot(page);

    const labels = page.locator('.topbar-actions .btn-label');
    expect(await labels.count()).toBeGreaterThan(0);
    const visibleLabels = await labels.evaluateAll((nodes) =>
      nodes
        .filter((node) => window.getComputedStyle(node).display !== 'none')
        .map((node) => node.textContent?.trim() || ''),
    );
    expect(visibleLabels).toEqual([]);

    const layout = await page.locator('.topbar').evaluate((topbar) => {
      const actions = topbar.querySelector('.topbar-actions');
      if (!actions) {
        return { fits: false, overflow: ['missing .topbar-actions'] };
      }
      const barRect = topbar.getBoundingClientRect();
      const actionRect = actions.getBoundingClientRect();
      const overflow = Array.from(actions.children)
        .filter((child) => {
          const style = window.getComputedStyle(child);
          const rect = child.getBoundingClientRect();
          return (
            style.display !== 'none' &&
            style.visibility !== 'hidden' &&
            rect.width > 0 &&
            rect.height > 0 &&
            (rect.left < barRect.left ||
              rect.right > barRect.right ||
              rect.left < actionRect.left - 0.5 ||
              rect.right > actionRect.right + 0.5)
          );
        })
        .map((child) => child.getAttribute('data-testid') || child.className);
      return {
        fits:
          overflow.length === 0 &&
          actions.scrollWidth <= actions.clientWidth + 1,
        overflow,
      };
    });

    expect(layout).toEqual({ fits: true, overflow: [] });

    const targets = await page
      .locator('.topbar-actions a, .topbar-actions button')
      .evaluateAll((nodes) =>
        nodes
          .filter((node) => window.getComputedStyle(node).display !== 'none')
          .map((node) => {
            const rect = node.getBoundingClientRect();
            return {
              label: node.getAttribute('aria-label') || node.textContent || '',
              width: rect.width,
              height: rect.height,
            };
          }),
      );

    for (const target of targets) {
      expect(target.label.trim()).not.toBe('');
      expect(target.width).toBeGreaterThanOrEqual(24);
      expect(target.height).toBeGreaterThanOrEqual(24);
    }
  });
});

async function assertDarkSurfaceParity(page) {
  const last = await sendPrompt(page, 'hi');
  await openAllDetails(last);
  await expect(last.locator('[data-testid="diagnostics-step"]').first()).toBeVisible();

  const lightSurfaces = await visibleLightSurfaces(page, [
    '.topbar',
    '.context-panel',
    '.sidebar-section-header',
    '.sidebar-section-body',
    '.settings-reset',
    '.settings-reset-all',
    '.settings-reset-one',
    '.conversation-new',
    '.conversation-copy',
    '.conversation-entry-button',
    '.conversation-delete',
    '.chat-panel',
    '.message-body',
    '.diagnostics-detail',
    '.diagnostics-detail-body',
    '.diagnostics-payload',
    '.diagnostics-tool-reasoning',
    '.composer',
    '.composer textarea',
    '.composer-action-button',
    '.send-button',
    '.setting-row input[type="text"]',
    '.setting-row select',
  ]);

  expect(lightSurfaces).toEqual([]);

  const textContrastTargets = await page
    .locator('.diagnostics-detail, .diagnostics-payload, .settings-reset')
    .evaluateAll((nodes) =>
      nodes.map((node) => ({
        selector: node.className || node.tagName.toLowerCase(),
        color: window.getComputedStyle(node).color,
        backgroundColor: window.getComputedStyle(node).backgroundColor,
      })),
    );

  for (const target of textContrastTargets) {
    const foreground = parseRgb(target.color);
    const background = parseRgb(target.backgroundColor);
    expect(foreground, target.selector).toBeTruthy();
    expect(background, target.selector).toBeTruthy();
    const bright = Math.max(luminance(foreground), luminance(background));
    const dark = Math.min(luminance(foreground), luminance(background));
    expect((bright + 0.05) / (dark + 0.05)).toBeGreaterThanOrEqual(3);
  }
}

test.describe('Issue #388 - dark theme surface parity', () => {
  for (const mode of [
    {
      name: 'explicit dark theme',
      overrides: `
  diagnosticsMode "on"
  theme "dark"`,
    },
    {
      name: 'system dark theme',
      colorScheme: 'dark',
      overrides: `
  diagnosticsMode "on"
  theme "system"`,
    },
  ]) {
    test(`${mode.name} avoids light-only diagnostics, settings, and sidebar surfaces`, async ({
      page,
    }) => {
      if (mode.colorScheme) {
        await page.emulateMedia({ colorScheme: mode.colorScheme });
      }
      await page.setViewportSize({ width: 1440, height: 1000 });
      await boot(page, mode.overrides);
      await assertDarkSurfaceParity(page);
    });
  }
});
