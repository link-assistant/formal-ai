// @ts-check
// Issue #1963 (P1, P3, P4, P5): UI/UX polish — one continuous reasoning
// scroll-fade, a stable pending-bubble width, dark-themed services/update
// panels, and a uniform topbar hover/focus affordance.
//
// hive-mind #1963 reported five UI/UX defects in the desktop app. P2 ("Thinking
// steps are not fully written, some parts are omitted.") is a Rust-side
// truncation cap raised from 120 to 600 chars and is pinned by the
// `tests/unit/issue_1963.rs` unit test (the JS `thinkingDetailText` helper is a
// documented mirror of the same constant). The remaining four are pure
// CSS/cascade defects in `src/web/styles.css`, and this spec guards them.
//
// The desktop SERVICES + UPDATE panels only render when the Electron desktop
// bridge supplies a `serviceStatus`, so they never mount in the plain web build
// Playwright loads. Rather than mock the bridge, this spec loads the real app
// (so the shipped `styles.css` and the full cascade apply exactly as they ship)
// and injects the precise DOM the React render functions emit, then asserts the
// COMPUTED styles. Every assertion below FAILS against the pre-fix CSS and
// PASSES against the fix:
//
//   - P1 "Animation gradient does not span 2 paragraphs/steps of thinking":
//     a single container-level mask now spans both collapsed steps; the former
//     per-line mask on `.thinking-preview-previous` is gone, and a lone first
//     step (no previous) is never masked.
//   - P3 "the width of message broken ... sudden jump in width when message
//     starts displaying": the pending (thinking-only) bubble keeps the normal
//     message-body width instead of the old 116px clamp.
//   - P4 "We still have theming issues in all `services` box": the services +
//     update panels honor the dark theme instead of staying white (the exact
//     failure mode #541 R1 fixed for other widgets).
//   - P5 "Not all buttons on top menu are reacting to hover ... it is partial":
//     every topbar control shares one hover treatment (and one focus ring).

const { test, expect } = require('@playwright/test');

const PREF_KEY = 'formal-ai.preferences.v1';
const FIXTURE_ID = 'ui1963-fixture';

// Mirrors the #541 dark-boot preference blob: pin theme "dark" so the boot
// effect mirrors `data-theme="dark"` onto <html> and the dark cascade applies.
function preferencesWithDarkTheme() {
  return [
    'demo_preferences',
    '  demoMode "off"',
    '  greetingVariations "off"',
    '  diagnosticsMode "off"',
    '  uiLanguage "en"',
    '  theme "dark"',
  ].join('\n');
}

function parseRgb(color) {
  const match = /rgba?\(\s*(\d+)\s*,\s*(\d+)\s*,\s*(\d+)/.exec(color);
  if (!match) return null;
  return { r: Number(match[1]), g: Number(match[2]), b: Number(match[3]) };
}

// A background is an acceptable dark surface when every channel is under 70
// (the fix uses #1d201f / rgb(29,32,31) and #242826 / rgb(36,40,38)). The bug
// we guard against is the light #ffffff / rgb(255,255,255) that bled through.
function isDarkSurface(rgb) {
  if (!rgb) return false;
  return rgb.r < 70 && rgb.g < 70 && rgb.b < 70;
}

// A foreground is acceptable for dark mode when it is light+muted (>= 170 on
// every channel — the fix uses #ece7df / rgb(236,231,223)). The bug we guard
// against is the light theme's dark-grey #24333d / rgb(36,51,61), which sits
// far below 170 on every channel.
function isMutedDarkText(rgb) {
  if (!rgb) return false;
  return rgb.r >= 170 && rgb.g >= 170 && rgb.b >= 170;
}

function rgbEquals(rgb, r, g, b) {
  return !!rgb && rgb.r === r && rgb.g === g && rgb.b === b;
}

// Inject a fixture host pinned on top of the app (max z-index) so the real
// shipped CSS styles it and topbar buttons are the topmost target for hover.
// One host per test (Playwright gives each test a fresh page), so no cleanup.
async function injectFixture(page, innerHtml) {
  await page.evaluate(
    ({ id, html }) => {
      const existing = document.querySelector(`[data-testid="${id}"]`);
      if (existing) existing.remove();
      const host = document.createElement('div');
      host.setAttribute('data-testid', id);
      host.style.cssText =
        'position:fixed;top:0;left:0;z-index:2147483647;padding:16px;';
      host.innerHTML = html;
      document.body.appendChild(host);
    },
    { id: FIXTURE_ID, html: innerHtml },
  );
  return page.locator(`[data-testid="${FIXTURE_ID}"]`);
}

async function bootDefaultTheme(page) {
  await page.goto('./');
  await expect(page.locator('.app')).toBeVisible({ timeout: 15_000 });
}

async function bootDarkTheme(page) {
  await page.addInitScript(
    ({ prefKey, preferences }) => {
      try {
        window.localStorage.setItem(prefKey, preferences);
      } catch (_error) {
        // localStorage can be unavailable in hardened browser contexts.
      }
    },
    { prefKey: PREF_KEY, preferences: preferencesWithDarkTheme() },
  );
  await page.goto('./');
  await expect(page.locator('.app')).toBeVisible({ timeout: 15_000 });
  await expect(page.locator('html')).toHaveAttribute('data-theme', 'dark', {
    timeout: 10_000,
  });
}

// --- P1: one continuous scroll-fade across the collapsed reasoning ----------
test.describe('Issue #1963 (P1): the reasoning scroll-fade is one continuous gradient', () => {
  test.beforeEach(async ({ page }) => {
    await bootDefaultTheme(page);
  });

  test('the collapsed container masks across both steps, not per line', async ({
    page,
  }) => {
    const fixture = await injectFixture(
      page,
      `<div class="thinking-preview-collapsed" data-testid="ui1963-collapsed">
         <p class="thinking-preview-previous" data-testid="ui1963-previous">Match the greeting rule.</p>
         <p class="thinking-preview-current">Applied available context: reply in English.</p>
       </div>`,
    );

    const collapsed = fixture.locator('[data-testid="ui1963-collapsed"]');
    const previous = fixture.locator('[data-testid="ui1963-previous"]');

    // The CONTAINER carries the single gradient mask (fix). Pre-fix the
    // container had `mask-image: none`, so this fails.
    const containerMask = await collapsed.evaluate((node) => {
      const style = getComputedStyle(node);
      return style.maskImage !== 'none' && style.maskImage
        ? style.maskImage
        : style.webkitMaskImage;
    });
    expect(containerMask, 'container mask-image').toContain('gradient');

    // The previous LINE no longer carries its own mask (that per-line fade was
    // the P1 bug). Pre-fix this was a `linear-gradient`, so this fails.
    const previousMask = await previous.evaluate((node) => {
      const style = getComputedStyle(node);
      return {
        standard: style.maskImage,
        webkit: style.webkitMaskImage,
      };
    });
    expect(previousMask.standard, 'previous-line mask-image (standard)').toBe(
      'none',
    );
    expect(previousMask.webkit, 'previous-line mask-image (webkit)').toBe(
      'none',
    );
  });

  test('a lone first step (no previous) is never masked', async ({ page }) => {
    const fixture = await injectFixture(
      page,
      `<div class="thinking-preview-collapsed" data-testid="ui1963-collapsed-lone">
         <p class="thinking-preview-current">Read the request: "Hi".</p>
       </div>`,
    );
    const lone = fixture.locator('[data-testid="ui1963-collapsed-lone"]');
    // `:has(.thinking-preview-previous)` does not match, so no mask is applied —
    // a single opening step must not look dimmed/faded.
    const mask = await lone.evaluate((node) => {
      const style = getComputedStyle(node);
      return { standard: style.maskImage, webkit: style.webkitMaskImage };
    });
    expect(mask.standard).toBe('none');
    expect(mask.webkit).toBe('none');
  });
});

// --- P3: the pending (thinking-only) bubble keeps the full message width -----
test.describe('Issue #1963 (P3): the pending bubble does not jump in width', () => {
  test.beforeEach(async ({ page }) => {
    await bootDefaultTheme(page);
  });

  test('pending body matches the settled body width (no 116px clamp)', async ({
    page,
  }) => {
    // Both messages live in the same fixed-width column, so after the fix the
    // pending body and the settled body resolve to the same grid-cell width.
    const fixture = await injectFixture(
      page,
      `<div style="width:640px">
         <article class="message assistant pending">
           <div class="avatar" aria-hidden="true">FA</div>
           <div class="message-body" data-testid="ui1963-pending-body">
             <section class="thinking-preview is-collapsed is-pending">
               <div class="thinking-preview-collapsed">
                 <p class="thinking-preview-current">Applied available context.</p>
               </div>
             </section>
           </div>
         </article>
         <article class="message assistant">
           <div class="avatar" aria-hidden="true">FA</div>
           <div class="message-body" data-testid="ui1963-settled-body">
             <p>Hello! How can I help you today?</p>
           </div>
         </article>
       </div>`,
    );

    const pendingWidth = await fixture
      .locator('[data-testid="ui1963-pending-body"]')
      .evaluate((node) => node.getBoundingClientRect().width);
    const settledWidth = await fixture
      .locator('[data-testid="ui1963-settled-body"]')
      .evaluate((node) => node.getBoundingClientRect().width);

    // Pre-fix the pending body was clamped to a fixed 116px while the settled
    // body filled the column — so they differed and the bubble visibly jumped.
    expect(
      Math.round(pendingWidth),
      'pending body must not keep the old 116px clamp',
    ).not.toBe(116);
    expect(
      Math.abs(pendingWidth - settledWidth),
      `pending (${pendingWidth}) vs settled (${settledWidth}) body width`,
    ).toBeLessThanOrEqual(1);
    // Sanity: the column is wide, so the pending body is far wider than 116px.
    expect(pendingWidth).toBeGreaterThan(300);
  });
});

// --- P4: the services + update panels honor the dark theme ------------------
test.describe('Issue #1963 (P4): the services box is dark-themed in dark mode', () => {
  test.beforeEach(async ({ page }) => {
    await bootDarkTheme(page);
  });

  test('service cards, action buttons, and update state go dark', async ({
    page,
  }) => {
    const fixture = await injectFixture(
      page,
      `<div class="desktop-services-panel">
         <div class="desktop-update-panel" data-state="available">
           <span class="desktop-update-state" role="status" data-testid="ui1963-update-state">Update available: v0.215.0</span>
           <div class="desktop-update-actions">
             <button type="button" data-testid="ui1963-update-button">Check for updates</button>
           </div>
         </div>
         <div class="desktop-service" data-state="stopped" data-testid="ui1963-service">
           <div class="desktop-service-head">
             <span class="desktop-service-dot"></span>
             <span class="desktop-service-label">Telegram bot</span>
             <span class="desktop-service-state" data-testid="ui1963-service-state">Stopped</span>
           </div>
           <div class="desktop-service-actions">
             <button type="button" data-testid="ui1963-service-button">Start</button>
           </div>
         </div>
       </div>`,
    );

    const serviceBg = await fixture
      .locator('[data-testid="ui1963-service"]')
      .evaluate((node) => getComputedStyle(node).backgroundColor);
    // Pre-fix the card had the light base #ffffff and never went dark.
    expect(
      isDarkSurface(parseRgb(serviceBg)),
      `service card background ${serviceBg} must be a dark surface`,
    ).toBe(true);

    const actionBg = await fixture
      .locator('[data-testid="ui1963-service-button"]')
      .evaluate((node) => getComputedStyle(node).backgroundColor);
    expect(
      isDarkSurface(parseRgb(actionBg)),
      `service action button background ${actionBg} must be a dark surface`,
    ).toBe(true);

    const updateColor = await fixture
      .locator('[data-testid="ui1963-update-state"]')
      .evaluate((node) => getComputedStyle(node).color);
    // Pre-fix the update state text kept the light #24333d and read as a seam.
    expect(
      isMutedDarkText(parseRgb(updateColor)),
      `update-state color ${updateColor} must be a muted dark-mode text`,
    ).toBe(true);
  });
});

// --- P5: every topbar control reacts to hover (and shows a focus ring) -------
test.describe('Issue #1963 (P5): every topbar control reacts to hover', () => {
  test.beforeEach(async ({ page }) => {
    await bootDefaultTheme(page);
  });

  // Each of these had NO light-mode :hover before the fix, so hovering left the
  // base white background and felt "partial". After the fix they share the
  // single #edf7f3 / rgb(237,247,243) hover tint.
  const HOVER_BUTTONS = [
    { cls: 'report-button', label: 'Report' },
    { cls: 'source-code-button', label: 'Source' },
    { cls: 'download-button', label: 'Download' },
    { cls: 'memory-button', label: 'Memory' },
    { cls: 'sidebar-toggle', label: 'Sidebar' },
  ];

  for (const button of HOVER_BUTTONS) {
    test(`.${button.cls} adopts the shared hover background`, async ({
      page,
    }) => {
      const fixture = await injectFixture(
        page,
        `<header class="topbar">
           <button type="button" class="${button.cls}" data-testid="ui1963-hover-target">${button.label}</button>
         </header>`,
      );
      const target = fixture.locator('[data-testid="ui1963-hover-target"]');

      const before = await target.evaluate(
        (node) => getComputedStyle(node).backgroundColor,
      );
      // Base state is the white #ffffff every topbar button starts from.
      expect(rgbEquals(parseRgb(before), 255, 255, 255)).toBe(true);

      await target.hover();

      // Web-first poll: the shared hover tint must apply once the cursor lands.
      await expect
        .poll(
          () =>
            target.evaluate((node) => getComputedStyle(node).backgroundColor),
          { message: `hover background for .${button.cls}` },
        )
        .toBe('rgb(237, 247, 243)');
    });
  }

  test('topbar controls expose a visible focus ring for keyboard users', async ({
    page,
  }) => {
    const fixture = await injectFixture(
      page,
      `<header class="topbar">
         <button type="button" class="report-button" data-testid="ui1963-focus-target">Report</button>
       </header>`,
    );
    const target = fixture.locator('[data-testid="ui1963-focus-target"]');
    await target.focus();
    const outlineWidth = await target.evaluate(
      (node) => getComputedStyle(node).outlineWidth,
    );
    // The fix adds `outline: 2px solid #175f4f` on :focus-visible; pre-fix the
    // button had the UA default (0px in this engine) and no shared ring.
    expect(parseInt(outlineWidth, 10)).toBeGreaterThanOrEqual(2);
  });
});
