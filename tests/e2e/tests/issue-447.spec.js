// Issue #447: the sidebar splitter looked like a scrollbar and made users try
// to scroll it. Keep the full-height resize target, but render only a thin sash
// whose hover/drag state does not paint the entire hit area.
const { test, expect } = require('@playwright/test');

test.describe('Issue #447: unambiguous sidebar splitter', () => {
  test.beforeEach(async ({ page }) => {
    await page.addInitScript(() => {
      window.localStorage.setItem('formal-ai.preferences.v1', 'greetingVariations false');
    });
    await page.setViewportSize({ width: 1280, height: 565 });
    await page.goto('./');
    await expect(page.locator('.app')).toBeVisible({ timeout: 15_000 });
  });

  test('uses a full-height thin line instead of a scrollbar-like thumb', async ({ page }) => {
    const metrics = await page.locator('[data-testid="context-resizer"]').evaluate((resizer) => {
      const style = getComputedStyle(resizer);
      const line = getComputedStyle(resizer, '::before');
      return {
        hitWidth: resizer.getBoundingClientRect().width,
        background: style.backgroundColor,
        lineWidth: Number.parseFloat(line.width),
        lineHeight: Number.parseFloat(line.height),
        resizerHeight: resizer.getBoundingClientRect().height,
      };
    });

    expect(metrics.hitWidth).toBeGreaterThanOrEqual(8);
    expect(metrics.background).toBe('rgba(0, 0, 0, 0)');
    expect(metrics.lineWidth).toBeLessThanOrEqual(2);
    expect(metrics.lineHeight).toBeGreaterThanOrEqual(metrics.resizerHeight - 1);
  });

  test('highlights only the sash while preserving resize and sidebar scrolling', async ({ page }) => {
    const panel = page.locator('[data-testid="context-panel"]');
    const resizer = page.locator('[data-testid="context-resizer"]');
    const scrollBody = page.locator(
      '[data-testid="sidebar-conversations"] .sidebar-section-body',
    );
    const before = await panel.boundingBox();
    const handle = await resizer.boundingBox();
    expect(before).toBeTruthy();
    expect(handle).toBeTruthy();

    await page.mouse.move(handle.x + handle.width / 2, handle.y + handle.height / 2);
    const hoverStyles = await resizer.evaluate((node) => ({
      track: getComputedStyle(node).backgroundColor,
      line: getComputedStyle(node, '::before').backgroundColor,
      cursor: getComputedStyle(node).cursor,
    }));
    expect(hoverStyles.track).toBe('rgba(0, 0, 0, 0)');
    expect(hoverStyles.line).not.toBe('rgba(0, 0, 0, 0)');
    expect(hoverStyles.cursor).toMatch(/col-resize|ew-resize/);

    await page.mouse.down();
    await page.mouse.move(handle.x + handle.width / 2 + 80, handle.y + handle.height / 2);
    await page.mouse.up();
    const after = await panel.boundingBox();
    expect(after.width).toBeGreaterThan(before.width + 60);
    const stored = await page.evaluate(
      () => window.localStorage.getItem('formal-ai.preferences.v1') || '',
    );
    expect(stored).toMatch(/contextPanelWidth "\d+"/);

    await page.evaluate(() => {
      const list = document.querySelector('[data-testid="conversation-list"]');
      if (!list) throw new Error('Conversation list was not rendered');
      for (let index = 0; index < 40; index += 1) {
        const item = document.createElement('li');
        item.textContent = `Regression scroll row ${index}`;
        list.appendChild(item);
      }
    });
    const scrollMetrics = await scrollBody.evaluate((node) => {
      node.scrollTop = 48;
      return {
        overflowY: getComputedStyle(node).overflowY,
        scrollTop: node.scrollTop,
        scrollHeight: node.scrollHeight,
        clientHeight: node.clientHeight,
      };
    });
    expect(scrollMetrics.overflowY).toBe('auto');
    expect(scrollMetrics.scrollHeight).toBeGreaterThan(scrollMetrics.clientHeight);
    expect(scrollMetrics.scrollTop).toBeGreaterThan(0);
  });

  test('retains keyboard splitter semantics and controls', async ({ page }) => {
    const resizer = page.locator('[data-testid="context-resizer"]');
    await expect(resizer).toHaveAttribute('role', 'separator');
    await expect(resizer).toHaveAttribute('aria-orientation', 'vertical');
    await resizer.focus();
    const before = Number(await resizer.getAttribute('aria-valuenow'));
    await page.keyboard.press('ArrowRight');
    const after = Number(await resizer.getAttribute('aria-valuenow'));
    expect(after).toBeGreaterThan(before);
  });
});
