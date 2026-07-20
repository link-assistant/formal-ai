// @ts-check
// Issue #672 (F3): "Animation budget per-message override".
//
// Issue #541 (R5/R6) paces a freshly produced answer: the reasoning steps fill
// in first and the answer body is withheld (`.markdown-body.is-revealing` is
// `display: none`) until the whole animation budget elapses. The budget is a
// single global preference, so a user who generally likes the paced reveal but
// wants THIS answer now had only one option: go to Settings and drag the slider
// to zero, changing the behaviour for every future message too.
//
// F3 asked for a one-shot per-message affordance instead. This spec covers it:
// the control appears only while the reveal is actually withholding the answer,
// clicking it shows the body immediately, and — the part that is easy to get
// wrong — it is scoped to the message it belongs to and does not touch the
// preference, so the next answer still animates.
//
// Note on the test environment: `playwright.local.config.js` sets
// `reducedMotion: 'reduce'` globally, which short-circuits the reveal to "show
// everything at once" so the rest of the suite can read answer text without
// racing the animation. This spec is about the animation itself, so it opts
// back in with `no-preference` — and the last test asserts that the
// reduced-motion path still wins on its own, without any click.

const fs = require('node:fs');
const path = require('node:path');
const { test, expect } = require('@playwright/test');

const PREF_KEY = 'formal-ai.preferences.v1';
const SCREENSHOT_DIR = path.resolve(__dirname, '../../../docs/screenshots/issue-672');

// The maximum the settings slider allows. A long budget makes the
// still-revealing window comfortably observable instead of a race.
const LONG_BUDGET_MS = 6000;

function preferences(budgetMs) {
  return [
    'demo_preferences',
    '  demoMode "off"',
    '  greetingVariations "off"',
    '  diagnosticsMode "off"',
    '  uiLanguage "en"',
    `  minMessageAnimationMs "${budgetMs}"`,
  ].join('\n');
}

async function boot(page, { budgetMs = LONG_BUDGET_MS, reducedMotion = 'no-preference' } = {}) {
  await page.emulateMedia({ reducedMotion });
  await page.addInitScript(
    ({ prefKey, value }) => {
      try {
        window.localStorage.setItem(prefKey, value);
      } catch (_error) {
        // localStorage can be unavailable in hardened browser contexts.
      }
    },
    { prefKey: PREF_KEY, value: preferences(budgetMs) },
  );
  await page.goto('./');
  await expect(page.locator('.app')).toBeVisible({ timeout: 15_000 });
  await expect(page.locator('[data-testid="chat-composer-input"]')).toBeEnabled({
    timeout: 10_000,
  });
}

async function sendPrompt(page, text) {
  const messages = page.locator('[data-testid="chat-message"]');
  const initial = await messages.count();
  await page.locator('[data-testid="chat-composer-input"]').fill(text);
  await page.locator('[data-testid="chat-composer-submit"]').click();
  await expect(messages).toHaveCount(initial + 2, { timeout: 20_000 });
  return messages.last();
}

test.describe('Issue #672 (F3): per-message animation budget override', () => {
  test('the override shows the withheld answer immediately', async ({ page }) => {
    await boot(page);
    const answer = await sendPrompt(page, 'Hello');

    // Precondition: the reveal really is withholding the body. Without this the
    // test below would pass vacuously on a build where the animation never ran.
    const body = answer.locator('[data-testid="message-markdown-body"]');
    await expect(body).toHaveClass(/is-revealing/);
    await expect(body).toBeHidden();
    await expect(answer).toHaveAttribute('data-skip-animation', 'available');

    const skip = answer.locator('[data-testid="message-skip-animation"]');
    await expect(skip).toBeVisible();
    await skip.click();

    // Immediately — well inside the 6 s budget the animation would otherwise
    // take. `expect` polls, so a slow machine cannot make this flaky, but the
    // 2 s ceiling is a third of the budget and would fail if the click were a
    // no-op and we were merely waiting the animation out.
    await expect(body).not.toHaveClass(/is-revealing/, { timeout: 2_000 });
    await expect(body).toBeVisible();
    await expect(body).not.toHaveText('');
  });

  test('the affordance disappears once the answer is shown', async ({ page }) => {
    await boot(page);
    const answer = await sendPrompt(page, 'Hello');
    const skip = answer.locator('[data-testid="message-skip-animation"]');
    await expect(skip).toBeVisible();
    await skip.click();
    // Dead chrome on a settled message is worse than no affordance at all.
    await expect(skip).toHaveCount(0);
    await expect(answer).not.toHaveAttribute('data-skip-animation', 'available');
  });

  test('the override is one-shot: it does not change the preference', async ({ page }) => {
    await boot(page);
    const first = await sendPrompt(page, 'Hello');
    await first.locator('[data-testid="message-skip-animation"]').click();
    await expect(
      first.locator('[data-testid="message-markdown-body"]'),
    ).toBeVisible();

    // The next answer must still animate — this is the whole difference between
    // a per-message override and dragging the global slider to zero.
    const second = await sendPrompt(page, 'What is formal-ai?');
    await expect(
      second.locator('[data-testid="message-markdown-body"]'),
    ).toHaveClass(/is-revealing/);
    await expect(second.locator('[data-testid="message-skip-animation"]')).toBeVisible();

    // And the stored budget is untouched.
    const settings = page.getByRole('button', { name: 'Settings', exact: true });
    if ((await settings.getAttribute('aria-expanded')) !== 'true') {
      await settings.click();
    }
    const slider = page.locator('[data-testid="setting-min-message-animation"]');
    await expect(slider).toHaveValue(String(LONG_BUDGET_MS));
  });

  test('skipping one message leaves the others alone', async ({ page }) => {
    await boot(page);
    await sendPrompt(page, 'Hello');
    const second = await sendPrompt(page, 'What is formal-ai?');
    const first = page.locator('[data-testid="chat-message"]').nth(1);

    // Both are mid-reveal; skipping the newer one must not settle the older one.
    await expect(
      first.locator('[data-testid="message-markdown-body"]'),
    ).toHaveClass(/is-revealing/);
    await second.locator('[data-testid="message-skip-animation"]').click();
    await expect(
      second.locator('[data-testid="message-markdown-body"]'),
    ).toBeVisible();
    await expect(
      first.locator('[data-testid="message-markdown-body"]'),
    ).toHaveClass(/is-revealing/);
  });

  test('reduced motion still suppresses the animation without any click', async ({
    page,
  }) => {
    // F3 must not turn an accessibility preference into a manual chore: a user
    // who asked the OS for reduced motion sees the answer at once, and there is
    // no override control to click because there is nothing to override.
    await boot(page, { reducedMotion: 'reduce' });
    const answer = await sendPrompt(page, 'Hello');
    const body = answer.locator('[data-testid="message-markdown-body"]');
    await expect(body).toBeVisible();
    await expect(body).not.toHaveClass(/is-revealing/);
    await expect(answer.locator('[data-testid="message-skip-animation"]')).toHaveCount(0);
  });

  // Human-review artefacts for the pull request, regenerated on every run. Not
  // asserted beyond the affordance being where the screenshot claims it is —
  // the behaviour is pinned by the tests above.
  test('writes before/after review screenshots', async ({ page }) => {
    fs.mkdirSync(SCREENSHOT_DIR, { recursive: true });
    await boot(page);
    const answer = await sendPrompt(page, 'Hello');
    const skip = answer.locator('[data-testid="message-skip-animation"]');
    await expect(skip).toBeVisible();
    await page.screenshot({ path: path.join(SCREENSHOT_DIR, 'f3-before-skip.png') });
    await skip.click();
    await expect(
      answer.locator('[data-testid="message-markdown-body"]'),
    ).toBeVisible();
    await page.screenshot({ path: path.join(SCREENSHOT_DIR, 'f3-after-skip.png') });
  });

  test('a zero budget keeps the immediate path, with no affordance', async ({ page }) => {
    await boot(page, { budgetMs: 0 });
    const answer = await sendPrompt(page, 'Hello');
    await expect(
      answer.locator('[data-testid="message-markdown-body"]'),
    ).toBeVisible();
    await expect(answer.locator('[data-testid="message-skip-animation"]')).toHaveCount(0);
  });
});
