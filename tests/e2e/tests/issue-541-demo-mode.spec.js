// @ts-check
// Issue #541 (R4): demo mode must be non-destructive. Concretely:
//
//   1. While a user has a real conversation open, switching demo on must NOT
//      append demo turns to that conversation, nor list a demo conversation
//      in the sidebar.
//   2. Switching demo off must restore the user's real conversation to the
//      UI exactly as they left it.
//   3. Switching to any non-demo conversation from the sidebar must auto-
//      disable demo mode.
//   4. The demo turns themselves must persist in their own dedicated
//      conversation so the "last example" survives across off/on toggles
//      within a session — but that dedicated conversation must never appear
//      in the sidebar.
//
// These behaviours together implement the verbatim issue text: "Demo mode
// should now NOT delete or overwrite any user conversations, if we are in the
// existing conversation we should create new conversation for demo mode, that
// is overridden when active, so when user switching to any non-demo mode
// conversation we should automatically disable demo mode, and keep last
// example in the newly created demo conversation."
const { test, expect } = require('@playwright/test');

async function disableGreetingVariations(page) {
  await page.addInitScript(() => {
    try {
      window.localStorage.setItem(
        'formal-ai.preferences.v1',
        'demo_preferences\n  greetingVariations "off"',
      );
    } catch (_error) {
      // localStorage may be unavailable; tests will tolerate variant text.
    }
  });
}

async function disableDemoFromStart(page) {
  // Persist demoMode=off so the app does not auto-play on mount. The seeded
  // user conversation can then be created cleanly without racing the demo
  // useEffect.
  await page.addInitScript(() => {
    try {
      const existing = window.localStorage.getItem('formal-ai.preferences.v1') || 'demo_preferences';
      if (!existing.includes('demoMode')) {
        window.localStorage.setItem(
          'formal-ai.preferences.v1',
          `${existing}\n  demoMode "off"`,
        );
      }
    } catch (_error) {
      // tolerated — test will fail on the visible assertion if storage is broken
    }
  });
}

async function waitForReady(page) {
  await expect(page.locator('.app')).toBeVisible({ timeout: 15_000 });
  await expect(page.locator('[data-testid="chat-composer-input"]')).toBeEnabled({
    timeout: 10_000,
  });
}

async function listEvents(page) {
  return page.evaluate(() => window.FormalAiMemory.listEvents());
}

async function sendPrompt(page, text) {
  const input = page.locator('[data-testid="chat-composer-input"]');
  await expect(input).toBeEnabled({ timeout: 5_000 });
  await input.fill(text);
  const messages = page.locator('[data-testid="chat-message"]');
  const initialCount = await messages.count();
  await page.locator('[data-testid="chat-composer-submit"]').click();
  await expect(messages).toHaveCount(initialCount + 2, { timeout: 20_000 });
}

async function clickDemoToggle(page) {
  // The mode toggle is a single button that flips demoMode. It is visible in
  // both demo-on and demo-off states.
  await page.locator('.mode-toggle').click();
}

test.describe('issue #541 R4: demo mode is non-destructive', () => {
  test.beforeEach(async ({ page }) => {
    await disableGreetingVariations(page);
    await disableDemoFromStart(page);
    await page.goto('./');
    await waitForReady(page);
  });

  test('demo turns persist in their own conversation, never in the user thread', async ({ page }) => {
    // Seed a real user conversation.
    await sendPrompt(page, 'My user conversation seed');
    const eventsBefore = await listEvents(page);
    const userMessages = eventsBefore.filter(
      (event) => event && event.kind === 'message' && !event.isDemo,
    );
    expect(userMessages.length).toBeGreaterThanOrEqual(2); // user + assistant
    const userConversationId = userMessages[0].conversationId;
    expect(userConversationId).toBeTruthy();

    // Capture the exact set of messages that belong to the user conversation
    // BEFORE demo runs — we will assert this set is unchanged after demo
    // mode plays a cycle.
    const userMessageContentsBefore = userMessages
      .filter((event) => event.conversationId === userConversationId)
      .map((event) => event.content);

    // Switch demo on. Demo will start playing its scripted cycle.
    await clickDemoToggle(page);
    await expect(page.locator('[data-testid="demo-status"]')).toContainText(
      /Demo playing|Next dialog in/,
      { timeout: 10_000 },
    );

    // Wait for at least one demo turn to land in IndexedDB.
    await expect.poll(async () => {
      const events = await listEvents(page);
      return events.filter((event) => event && event.isDemo === true).length;
    }, { timeout: 20_000 }).toBeGreaterThanOrEqual(1);

    // Critical assertion #1: the user's real conversation is byte-for-byte
    // unchanged. No demo content has leaked into it.
    const eventsAfter = await listEvents(page);
    const userMessagesAfter = eventsAfter
      .filter(
        (event) =>
          event &&
          event.kind === 'message' &&
          !event.isDemo &&
          event.conversationId === userConversationId,
      )
      .map((event) => event.content);
    expect(userMessagesAfter).toEqual(userMessageContentsBefore);

    // Critical assertion #2: the demo conversation has a DIFFERENT id.
    const demoEvents = eventsAfter.filter((event) => event && event.isDemo === true);
    expect(demoEvents.length).toBeGreaterThan(0);
    const demoConversationId = demoEvents[0].conversationId;
    expect(demoConversationId).not.toBe(userConversationId);
    // Every demo event shares the SAME dedicated demo conversation id so the
    // "last example" survives across cycles within a session.
    for (const event of demoEvents) {
      expect(event.conversationId).toBe(demoConversationId);
    }

    // Critical assertion #3: the sidebar never lists the demo conversation.
    const sidebarIds = await page.evaluate(() =>
      Array.from(
        document.querySelectorAll('[data-conversation-id]'),
      ).map((node) => node.getAttribute('data-conversation-id')),
    );
    expect(sidebarIds).not.toContain(demoConversationId);
    // But the user's real conversation IS listed.
    expect(sidebarIds).toContain(userConversationId);
  });

  test('disabling demo restores the user conversation untouched', async ({ page }) => {
    await sendPrompt(page, 'My user conversation seed');
    const messages = page.locator('[data-testid="chat-message"]');
    await expect(messages).toHaveCount(2);
    const userBubbleTextBefore = await messages.nth(0).textContent();

    // Switch demo on — UI clears and demo plays.
    await clickDemoToggle(page);
    await expect(page.locator('[data-testid="chat-composer-input"]')).toBeDisabled();
    // Wait for at least one demo turn to appear so the UI is definitely not
    // showing the user's seed any more.
    await expect.poll(() => messages.count(), { timeout: 20_000 }).toBeGreaterThan(0);

    // Switch demo off. The user's seed conversation must come back exactly.
    await clickDemoToggle(page);
    await expect(page.locator('[data-testid="chat-composer-input"]')).toBeEnabled({
      timeout: 10_000,
    });
    await expect(messages).toHaveCount(2, { timeout: 10_000 });
    const userBubbleTextAfter = await messages.nth(0).textContent();
    expect(userBubbleTextAfter).toBe(userBubbleTextBefore);
  });

  test('switching to a sidebar conversation auto-disables demo mode', async ({ page }) => {
    // Seed a user conversation so there is something to switch to.
    await sendPrompt(page, 'Sidebar target conversation');
    const initialEvents = await listEvents(page);
    const userConversationId = initialEvents.find(
      (event) => event && event.kind === 'message' && !event.isDemo,
    ).conversationId;

    // Start a NEW user conversation so the sidebar has at least two entries
    // and the "switch" action is meaningful.
    await page.locator('[data-testid="conversation-new"]').click();
    await expect(page.locator('[data-testid="chat-message"]')).toHaveCount(0);
    await sendPrompt(page, 'Second user conversation');

    // Turn demo on — composer goes disabled, demo starts.
    await clickDemoToggle(page);
    await expect(page.locator('[data-testid="chat-composer-input"]')).toBeDisabled();
    await expect(page.locator('[data-testid="demo-status"]')).toContainText(
      /Demo playing|Next dialog in/,
      { timeout: 10_000 },
    );

    // Click the first (older) conversation in the sidebar. There are two
    // elements per entry that carry data-conversation-id (the entry button
    // and the "Copy" button); we want the entry button specifically.
    await page
      .locator(
        `.conversation-entry-button[data-conversation-id="${userConversationId}"]`,
      )
      .click();

    // Demo mode must auto-disable: composer enabled, status not "Demo playing".
    await expect(page.locator('[data-testid="chat-composer-input"]')).toBeEnabled({
      timeout: 10_000,
    });
    await expect(page.locator('[data-testid="demo-status"]')).not.toContainText(
      /Demo playing|Next dialog in/,
      { timeout: 5_000 },
    );

    // And the user is now looking at the conversation they clicked, NOT a
    // demo conversation. The "Sidebar target conversation" seed must be in
    // the visible message list.
    await expect(page.locator('[data-testid="message-list"]')).toContainText(
      'Sidebar target conversation',
      { timeout: 10_000 },
    );
  });
});
