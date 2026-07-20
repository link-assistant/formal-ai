// @ts-check
// Issue #672 (F4): "Reasoning-step hierarchy editing for power users".
//
// The solver labels each reasoning step `high` (a phase of the universal
// algorithm) or `detailed` (an internal move), and the default "standard"
// thinking detail shows only the phases plus the conclusion. That split is a
// good default and a bad fit for anyone who lives in one part of the trace: a
// user debugging tool dispatch wants `invoke_tool` promoted to a phase, and a
// user who never cares about language detection wants it out of the way.
//
// F4 asked for that to be editable from the UI. The edit is recorded as an
// append-only event and the visible hierarchy is a *projection* of the event
// log over the solver's own labels — the message the worker produced is never
// rewritten. This spec pins both halves: the projection changes what the
// preview shows, and the underlying diagnostics trace stays exactly as the
// solver reported it.

const { test, expect } = require('@playwright/test');

const PREF_KEY = 'formal-ai.preferences.v1';

const PREFERENCES = [
  'demo_preferences',
  '  demoMode "off"',
  '  greetingVariations "off"',
  '  diagnosticsMode "on"',
  '  uiLanguage "en"',
  '  minMessageAnimationMs "0"',
].join('\n');

// The arithmetic prompt produces a stable trace with both levels present:
// `impulse` / `detect_language` / `dispatch_handler` / `deformalize` /
// `user_context` are phases, `formalize` and `invoke_tool` are internals.
const PROMPT = 'What is 2 + 2?';
// Each is matched two ways: by step id in the diagnostics trace, and by the
// naturalized sentence the thinking preview renders for it.
const A_PHASE = { step: 'detect_language', text: 'Detect the request language' };
const AN_INTERNAL = { step: 'formalize', text: 'Formalize the request' };

async function seed(page, preferences) {
  await page.addInitScript(
    ({ prefKey, value }) => {
      try {
        window.localStorage.setItem(prefKey, value);
      } catch (_error) {
        // localStorage can be unavailable in hardened browser contexts.
      }
    },
    { prefKey: PREF_KEY, value: preferences },
  );
  await page.goto('./');
  await expect(page.locator('.app')).toBeVisible({ timeout: 15_000 });
  await expect(page.locator('[data-testid="chat-composer-input"]')).toBeEnabled({
    timeout: 10_000,
  });
}

async function send(page, text) {
  const messages = page.locator('[data-testid="chat-message"]');
  const initial = await messages.count();
  await page.locator('[data-testid="chat-composer-input"]').fill(text);
  await page.locator('[data-testid="chat-composer-submit"]').click();
  await expect(messages).toHaveCount(initial + 2, { timeout: 20_000 });
  return messages.last();
}

async function sendPrompt(page, text) {
  const answer = await send(page, text);
  await expect(answer.locator('[data-testid="diagnostics-step"]').first()).toBeVisible({
    timeout: 20_000,
  });
  return answer;
}

/** The reasoning steps the user actually sees, at the current hierarchy. */
async function visibleSteps(answer) {
  const toggle = answer.locator('[data-testid="thinking-preview-toggle"]');
  if ((await toggle.getAttribute('aria-expanded')) !== 'true') {
    await toggle.click();
  }
  return answer.locator('[data-testid="thinking-expanded-list"] li').allTextContents();
}

function step(answer, target) {
  return answer
    .locator(`[data-testid="diagnostics-step"][data-step="${target.step}"]`)
    .first();
}

async function openMenu(page, answer, target) {
  await step(answer, target).locator('summary').click({ button: 'right' });
  const menu = page.locator('[data-testid="step-hierarchy-menu"]');
  await expect(menu).toBeVisible();
  await expect(menu).toHaveAttribute('data-step', target.step);
  return menu;
}

function mentions(steps, target) {
  return steps.some((text) => text.includes(target.text));
}

test.describe('Issue #672 (F4): reasoning-step hierarchy editing', () => {
  test('promoting an internal step makes it a visible phase', async ({ page }) => {
    await seed(page, PREFERENCES);
    const answer = await sendPrompt(page, PROMPT);

    // Baseline: the solver labelled `formalize` an internal move, so the
    // standard detail level hides it. Without this the assertion below could
    // pass on a build where the step happened to be visible all along.
    await expect(step(answer, AN_INTERNAL)).toHaveAttribute('data-level', 'detailed');
    expect(mentions(await visibleSteps(answer), AN_INTERNAL)).toBe(false);

    const menu = await openMenu(page, answer, AN_INTERNAL);
    await menu.locator('[data-testid="step-hierarchy-bump"]').click();

    await expect(menu).toHaveCount(0);
    await expect(step(answer, AN_INTERNAL)).toHaveAttribute(
      'data-level-override',
      'high',
    );
    expect(mentions(await visibleSteps(answer), AN_INTERNAL)).toBe(true);
  });

  test('demoting a phase takes it out of the standard view', async ({ page }) => {
    await seed(page, PREFERENCES);
    const answer = await sendPrompt(page, PROMPT);
    expect(mentions(await visibleSteps(answer), A_PHASE)).toBe(true);

    const menu = await openMenu(page, answer, A_PHASE);
    await menu.locator('[data-testid="step-hierarchy-demote"]').click();

    await expect(step(answer, A_PHASE)).toHaveAttribute(
      'data-level-override',
      'detailed',
    );
    expect(mentions(await visibleSteps(answer), A_PHASE)).toBe(false);
  });

  test('the edit never rewrites the trace the solver reported', async ({ page }) => {
    // This is the whole point of projecting instead of mutating: whatever the
    // user does to their view, the diagnostics panel — the audit surface —
    // still shows every step the worker emitted, in the order it emitted them.
    await seed(page, PREFERENCES);
    const answer = await sendPrompt(page, PROMPT);
    const steps = answer.locator('[data-testid="diagnostics-step"]');
    const before = await steps.evaluateAll((nodes) =>
      nodes.map((node) => `${node.getAttribute('data-step')}`),
    );

    // Edit in both directions, so the assertion below covers a promotion and a
    // demotion rather than one kind of event twice.
    await (await openMenu(page, answer, AN_INTERNAL))
      .locator('[data-testid="step-hierarchy-bump"]')
      .click();
    await (await openMenu(page, answer, A_PHASE))
      .locator('[data-testid="step-hierarchy-demote"]')
      .click();

    const after = await steps.evaluateAll((nodes) =>
      nodes.map((node) => `${node.getAttribute('data-step')}`),
    );
    expect(after).toEqual(before);
    await expect(step(answer, A_PHASE)).toHaveAttribute(
      'data-level-override',
      'detailed',
    );
    await expect(step(answer, AN_INTERNAL)).toHaveAttribute(
      'data-level-override',
      'high',
    );
    // The solver's own labels are still on the nodes, untouched by either edit.
    await expect(step(answer, A_PHASE)).toHaveAttribute('data-solver-level', 'high');
    await expect(step(answer, AN_INTERNAL)).toHaveAttribute(
      'data-solver-level',
      'detailed',
    );
  });

  test('resetting replays the log back to the solver level', async ({ page }) => {
    await seed(page, PREFERENCES);
    const answer = await sendPrompt(page, PROMPT);

    const menu = await openMenu(page, answer, A_PHASE);
    // Reset is offered only once there is something to reset.
    await expect(menu.locator('[data-testid="step-hierarchy-reset"]')).toHaveCount(0);
    await menu.locator('[data-testid="step-hierarchy-demote"]').click();
    expect(mentions(await visibleSteps(answer), A_PHASE)).toBe(false);

    const second = await openMenu(page, answer, A_PHASE);
    await second.locator('[data-testid="step-hierarchy-reset"]').click();

    await expect(step(answer, A_PHASE)).not.toHaveAttribute(
      'data-level-override',
      /.*/,
    );
    expect(mentions(await visibleSteps(answer), A_PHASE)).toBe(true);
  });

  test('an edit applies to the same step in later answers too', async ({ page }) => {
    // The override is keyed by step id, not by message: a user who decided
    // `formalize` is a phase means it for the trace, not for one answer.
    await seed(page, PREFERENCES);
    const first = await sendPrompt(page, PROMPT);
    await (await openMenu(page, first, AN_INTERNAL))
      .locator('[data-testid="step-hierarchy-bump"]')
      .click();

    const second = await sendPrompt(page, 'What is 3 + 5?');
    await expect(step(second, AN_INTERNAL)).toHaveAttribute(
      'data-level-override',
      'high',
    );
    expect(mentions(await visibleSteps(second), AN_INTERNAL)).toBe(true);
  });

  test('the menu is dismissible without making an edit', async ({ page }) => {
    await seed(page, PREFERENCES);
    const answer = await sendPrompt(page, PROMPT);
    const menu = await openMenu(page, answer, A_PHASE);
    await page.keyboard.press('Escape');
    await expect(menu).toHaveCount(0);
    await expect(step(answer, A_PHASE)).not.toHaveAttribute(
      'data-level-override',
      /.*/,
    );
  });

  test('the affordance is confined to Diagnostics mode', async ({ page }) => {
    // Outside Diagnostics there is no step list to right-click at all, so the
    // menu cannot be reached — asserted here so a future refactor that renders
    // the steps elsewhere does not quietly expose it to every user.
    await seed(
      page,
      PREFERENCES.replace('diagnosticsMode "on"', 'diagnosticsMode "off"'),
    );
    await send(page, PROMPT);
    await expect(page.locator('[data-testid="diagnostics-step"]')).toHaveCount(0);
    await expect(page.locator('[data-testid="step-hierarchy-menu"]')).toHaveCount(0);
  });
});
