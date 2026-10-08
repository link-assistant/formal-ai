// @ts-check
// Issue #27: greeting variations, summarize skill, agent mode, mobile layout,
// conversations sidebar, demo prompts, sidebar accordion, and
// cross-conversation recall.
// (Split from the former multilingual.spec.js; shared helpers live in
// ./support/multilingual.js.)
const { test, expect } = require('@playwright/test');
const {
  UNKNOWN_ANSWER_MARKER,
  switchToManualMode,
  disableGreetingVariations,
  sendPrompt,
  emulateReducedMotion,
} = require('./support/multilingual');

// Issue #541 (R5/R6): emulate prefers-reduced-motion so answers render at
// once instead of racing the paced reveal (see support/multilingual.js).
emulateReducedMotion(test);

test.describe('Issue #27: random greeting variations', () => {
  test.beforeEach(async ({ page }) => {
    // Default-on: do NOT call disableGreetingVariations — the seed-driven
    // randomisation must be observable when the user accepts the defaults.
    await page.goto('./');
    await expect(page.locator('.app')).toBeVisible({ timeout: 15_000 });
    await switchToManualMode(page);
  });

  test('English greeting falls within the seeded variant list', async ({ page }) => {
    const last = await sendPrompt(page, 'Hi');
    const text = (await last.innerText()).trim();
    const variants = [
      'Hi, how may I help you?',
      'Hello! How can I assist you today?',
      'Hi there! What can I do for you?',
      'Hey, how can I help?',
      'Hello — what would you like to explore?',
    ];
    expect(variants.some((variant) => text.includes(variant))).toBe(true);
  });

  test('disabling variations pins the canonical English greeting', async ({ page, context }) => {
    await context.addInitScript(() => {
      try {
        window.localStorage.setItem(
          'formal-ai.preferences.v1',
          'demo_preferences\n  greetingVariations "off"',
        );
      } catch (_error) {}
    });
    await page.goto('./');
    await expect(page.locator('.app')).toBeVisible({ timeout: 15_000 });
    await switchToManualMode(page);
    for (let attempt = 0; attempt < 3; attempt += 1) {
      const last = await sendPrompt(page, 'Hi');
      await expect(last).toContainText('Hi, how may I help you?');
    }
  });
});

test.describe('Issue #27: summarize skill', () => {
  test.beforeEach(async ({ page }) => {
    await disableGreetingVariations(page);
    await page.goto('./');
    await expect(page.locator('.app')).toBeVisible({ timeout: 15_000 });
    await switchToManualMode(page);
  });

  test('"summarize this conversation" returns a structured report', async ({ page }) => {
    await sendPrompt(page, 'Hi');
    await sendPrompt(page, 'What is 2 + 2?');
    const last = await sendPrompt(page, 'Summarize this conversation');
    await expect(last).toHaveClass(/assistant/);
    await expect(last).toContainText('Conversation summary');
    await expect(last).toContainText('user');
    await expect(last).toContainText('assistant');
    await expect(last).toContainText('greeting');
    await expect(last).toContainText('calculation');
    await expect(last).toContainText('2 + 2 = 4');
  });

  test('single-word "summarize" triggers the skill', async ({ page }) => {
    await sendPrompt(page, 'Hi');
    const last = await sendPrompt(page, 'Summarize');
    await expect(last).toContainText('Conversation summary');
  });

  test('Russian "резюме беседы" triggers the skill', async ({ page }) => {
    await sendPrompt(page, 'Привет');
    const last = await sendPrompt(page, 'Резюме беседы');
    await expect(last).toContainText('Conversation summary');
  });

  test('Chinese "总结" triggers the skill', async ({ page }) => {
    await sendPrompt(page, '你好');
    const last = await sendPrompt(page, '总结');
    await expect(last).toContainText('Conversation summary');
  });
});

test.describe('Issue #27: agent mode', () => {
  test.beforeEach(async ({ page }) => {
    await disableGreetingVariations(page);
    await page.goto('./');
    await expect(page.locator('.app')).toBeVisible({ timeout: 15_000 });
    await switchToManualMode(page);
  });

  test('Chat/Agent/Full-Auto radio is present and starts in Chat', async ({ page }) => {
    // Issue #513: the binary agent toggle became a three-way radio group.
    const group = page.locator('[data-testid="mode-radio"]');
    await expect(group).toBeVisible();
    await expect(page.locator('[data-testid="mode-option-chat"]')).toHaveAttribute(
      'aria-checked',
      'true',
    );
    await expect(page.locator('[data-testid="mode-option-fullAuto"]')).toBeVisible();
    await page.locator('[data-testid="mode-option-agent"]').click();
    await expect(page.locator('[data-testid="mode-option-agent"]')).toHaveAttribute(
      'aria-checked',
      'true',
    );
    await expect(page.locator('[data-testid="mode-status"]')).toContainText('Agent');
  });

  test('Agent mode decomposes a multi-step task and runs each step', async ({ page }) => {
    await page.locator('[data-testid="mode-option-agent"]').click();
    const last = await sendPrompt(
      page,
      'Hi; then what is 2 + 2; then who are you',
    );
    await expect(last).toHaveClass(/assistant/);
    await expect(last).toContainText('Agent plan (3 steps)');
    await expect(last).toContainText('Step 1: Hi');
    await expect(last).toContainText('Step 2: what is 2 + 2');
    await expect(last).toContainText('Step 3: who are you');
    // Step 1 greeting, step 2 calculation, step 3 identity.
    await expect(last).toContainText('Hi, how may I help you?');
    await expect(last).toContainText('2 + 2 = 4');
    await expect(last).toContainText('formal-ai');
  });

  test('Agent mode preserves single-step prompts as plain Q&A', async ({ page }) => {
    await page.locator('[data-testid="mode-option-agent"]').click();
    const last = await sendPrompt(page, 'Hi');
    // No "; then …" — should run as a single step (chat-style answer).
    await expect(last).toContainText('Hi, how may I help you?');
    await expect(last).not.toContainText('Agent plan');
  });
});

// Issue #27: phone-sized viewport asserts that the topbar collapses to
// icon-only buttons, the sidebar hides behind a hamburger drawer, and the
// chat surface keeps the full message viewport.
test.describe('Issue #27: mobile layout', () => {
  test.use({ viewport: { width: 390, height: 780 } });

  test.beforeEach(async ({ page }) => {
    await disableGreetingVariations(page);
    await page.goto('./');
    await expect(page.locator('.app')).toBeVisible({ timeout: 15_000 });
  });

  test('topbar buttons collapse to icons on mobile', async ({ page }) => {
    const demoToggle = page.locator('.mode-toggle');
    await expect(demoToggle).toBeVisible();
    // The label span is hidden via CSS on the mobile breakpoint…
    await expect(demoToggle.locator('.btn-label')).toBeHidden();
    // …but the icon stays visible so the action is still recognisable.
    await expect(demoToggle.locator('.btn-icon')).toBeVisible();
    // The aria-label still announces the action for screen readers.
    await expect(demoToggle).toHaveAttribute('aria-label', /Demo/);
  });

  test('hamburger toggle opens the full-width sidebar drawer and the close button dismisses it', async ({ page }) => {
    const hamburger = page.locator('[data-testid="mobile-menu-toggle"]');
    await expect(hamburger).toBeVisible();
    const sidebar = page.locator('[data-testid="context-panel"]');

    // Off-canvas: translated out of view so the chat fills the viewport.
    const boxBefore = await sidebar.boundingBox();
    expect(boxBefore).toBeTruthy();
    expect(boxBefore && boxBefore.x).toBeLessThan(0);

    await hamburger.click();
    await expect(sidebar).toHaveClass(/is-mobile-open/);
    // The drawer translates in over 200ms — wait for the transform to settle
    // before sampling the bounding box.
    await page.waitForTimeout(300);
    const boxAfter = await sidebar.boundingBox();
    expect(boxAfter).toBeTruthy();
    expect(boxAfter && boxAfter.x).toBeGreaterThanOrEqual(0);
    expect(boxAfter && boxAfter.width).toBeGreaterThanOrEqual(389);

    await page.locator('[data-testid="drawer-close"]').click();
    await expect(sidebar).not.toHaveClass(/is-mobile-open/);
  });

  test('Issue #112: mobile drawer lists topbar actions before conversations', async ({ page }) => {
    await page.locator('[data-testid="mobile-menu-toggle"]').click();
    const drawerActions = page.locator('[data-testid="drawer-menu-actions"]');
    const conversations = page.locator('[data-testid="sidebar-conversations"]');
    await expect(drawerActions).toBeVisible();
    if ((await drawerActions.getAttribute('data-collapsed')) === 'true') {
      await drawerActions.locator('.sidebar-section-header').click();
    }
    await expect(drawerActions).toContainText('Report issue');
    await expect(drawerActions).toContainText('Export memory');
    await expect(drawerActions).toContainText('Import memory');
    await expect(drawerActions).toContainText('Diagnostics');
    await expect(drawerActions).toContainText(/Chat|Agent/);
    await expect(drawerActions).toContainText(/Demo/);

    const actionsBox = await drawerActions.boundingBox();
    const conversationsBox = await conversations.boundingBox();
    expect(actionsBox).toBeTruthy();
    expect(conversationsBox).toBeTruthy();
    expect(actionsBox && conversationsBox && actionsBox.y).toBeLessThan(conversationsBox.y);
  });

  test('Issue #112: focused composer grows to content with equal padding and a half-panel cap', async ({ page }) => {
    await page.locator('.mode-toggle').click();
    const input = page.locator('[data-testid="chat-composer-input"]');
    await expect(input).toBeEnabled({ timeout: 15_000 });
    await input.fill('line one\nline two\nline three\nline four');

    const metrics = await input.evaluate((node) => {
      const style = getComputedStyle(node);
      const composer = node.closest('.composer');
      const chatPanel = document.querySelector('.chat-panel');
      return {
        clientHeight: node.clientHeight,
        scrollHeight: node.scrollHeight,
        boxHeight: node.getBoundingClientRect().height,
        paddingTop: style.paddingTop,
        paddingRight: style.paddingRight,
        paddingBottom: style.paddingBottom,
        paddingLeft: style.paddingLeft,
        composerHeight: composer ? composer.getBoundingClientRect().height : 0,
        chatPanelHeight: chatPanel ? chatPanel.getBoundingClientRect().height : 0,
      };
    });

    expect(metrics.boxHeight).toBeGreaterThan(42);
    expect(metrics.scrollHeight - metrics.clientHeight).toBeLessThanOrEqual(1);
    expect(metrics.paddingTop).toBe(metrics.paddingRight);
    expect(metrics.paddingRight).toBe(metrics.paddingBottom);
    expect(metrics.paddingBottom).toBe(metrics.paddingLeft);
    expect(metrics.composerHeight).toBeLessThanOrEqual(metrics.chatPanelHeight * 0.5 + 1);
  });

  test('chat surface keeps the full viewport when the menu is closed', async ({ page }) => {
    const sidebar = page.locator('[data-testid="context-panel"]');
    const sidebarBox = await sidebar.boundingBox();
    expect(sidebarBox && sidebarBox.x).toBeLessThan(0);

    const composer = page.locator('.composer-grid');
    const composerBox = await composer.boundingBox();
    expect(composerBox).toBeTruthy();
    // The full composer row spans most of the viewport width. Issue #108 adds
    // compact action/send buttons inside that row, so the textarea itself no
    // longer owns the entire row width.
    expect(composerBox && composerBox.width).toBeGreaterThan(300);

    const inputBox = await page.locator('[data-testid="chat-composer-input"]').boundingBox();
    expect(inputBox).toBeTruthy();
    expect(inputBox && inputBox.width).toBeGreaterThan(250);
  });
});

test.describe('Issue #27: conversations sidebar', () => {
  test.beforeEach(async ({ page }) => {
    await disableGreetingVariations(page);
    // Reset the IndexedDB event log so each test starts with no prior
    // conversations. The init script runs before every navigation (including
    // page.reload()), so we use a sessionStorage sentinel to delete only on
    // the first navigation of the test and preserve the DB on subsequent
    // reloads (otherwise the restore-after-reload test always sees an empty
    // log).
    await page.addInitScript(() => {
      try {
        if (typeof indexedDB === 'undefined') return;
        if (window.sessionStorage.getItem('formal-ai-test-reset') === '1') {
          return;
        }
        window.sessionStorage.setItem('formal-ai-test-reset', '1');
        indexedDB.deleteDatabase('formal-ai-demo');
      } catch (_error) {}
    });
    await page.goto('./');
    await expect(page.locator('.app')).toBeVisible({ timeout: 15_000 });
    await switchToManualMode(page);
    // Clear any demo dialog messages so each test starts with a fresh thread.
    // Issue #541 (R4): demo isolation now keeps the demo conversation session-
    // scoped and hidden from the sidebar, so `switchToManualMode` may already
    // land on an empty user conversation — leaving the "+ New conversation"
    // button disabled by design (see search-menu-and-deduplication.spec.js:140). Skip the click
    // when the button is disabled; the zero-state assertion below is the
    // source of truth either way.
    const newBtn = page.locator('[data-testid="conversation-new"]');
    if (await newBtn.isEnabled().catch(() => false)) {
      await newBtn.click();
    }
    await expect(page.locator('[data-testid="chat-message"]')).toHaveCount(0, {
      timeout: 5_000,
    });
  });

  test('sending a prompt adds an entry to the conversation list', async ({ page }) => {
    const entries = page.locator('[data-testid="conversation-entries"] li');

    await sendPrompt(page, 'Hello');

    // The new conversation now shows up titled by its first user message.
    await expect(entries.first()).toContainText('Hello', { timeout: 5_000 });
  });

  test('"+ New conversation" clears the transcript and starts a fresh thread', async ({ page }) => {
    const messages = page.locator('[data-testid="chat-message"]');
    await sendPrompt(page, 'Hello');
    await expect(messages).toHaveCount(2);

    await page.locator('[data-testid="conversation-new"]').click();
    await expect(messages).toHaveCount(0);

    await sendPrompt(page, 'Who are you?');

    const entries = page.locator('[data-testid="conversation-entries"] li');
    await expect(entries.first()).toContainText('Who are you', { timeout: 5_000 });
    await expect(entries.nth(1)).toContainText('Hello');
  });

  test('the last conversation is restored after reloading the page', async ({ page }) => {
    const messages = page.locator('[data-testid="chat-message"]');
    await sendPrompt(page, 'Hello');
    await expect(messages).toHaveCount(2);

    // Rendering the assistant turn and committing it to IndexedDB are separate
    // asynchronous operations. Wait for the persistence contract itself before
    // navigating so the reload cannot interrupt the second append.
    await expect
      .poll(
        () =>
          page.evaluate(async () => {
            const events = await window.FormalAiMemory.listEvents();
            return events.filter(
              (event) =>
                event.kind === 'message' &&
                !event.isDemo &&
                (event.role === 'user' || event.role === 'assistant'),
            ).length;
          }),
        { timeout: 5_000 },
      )
      .toBe(2);

    await page.reload();
    await expect(page.locator('.app')).toBeVisible({ timeout: 15_000 });
    // The transcript should be re-populated from IndexedDB; the active
    // conversation is the one persisted in preferences.
    const restored = page.locator('[data-testid="chat-message"]');
    await expect(restored).toHaveCount(2, { timeout: 15_000 });
    await expect(restored.first()).toContainText('Hello');
  });

  test('Issue #112: deleting a conversation soft-hides it behind the deleted view', async ({ page }) => {
    const messages = page.locator('[data-testid="chat-message"]');
    await sendPrompt(page, 'Hello to delete');
    await expect(messages).toHaveCount(2);

    await page.locator('[data-testid="conversation-delete"]').first().click();
    await expect(messages).toHaveCount(0);
    await expect(
      page.locator('[data-testid="conversation-entries"] li', {
        hasText: 'Hello to delete',
      }),
    ).toHaveCount(0);

    const showDeleted = page.locator('[data-testid="conversation-show-deleted"]');
    await expect(showDeleted).toBeVisible();
    await showDeleted.check();

    const deletedEntry = page.locator('[data-testid="conversation-entries"] li').first();
    await expect(deletedEntry).toContainText('Hello to delete');
    await expect(deletedEntry).toHaveClass(/is-deleted/);

    await deletedEntry.locator('.conversation-entry-button').click();
    await expect(messages).toHaveCount(2, { timeout: 5_000 });
    await expect(messages.first()).toContainText('Hello to delete');
  });

  test('Issue #196: deleted conversations can be permanently removed after export warning and confirmation', async ({ page }) => {
    const messages = page.locator('[data-testid="chat-message"]');
    await sendPrompt(page, 'Hello to purge');
    await expect(messages).toHaveCount(2);

    await page.locator('[data-testid="conversation-delete"]').first().click();
    await expect(messages).toHaveCount(0);

    const showDeleted = page.locator('[data-testid="conversation-show-deleted"]');
    await showDeleted.check();
    const deletedEntry = page.locator('[data-testid="conversation-entries"] li', {
      hasText: 'Hello to purge',
    });
    await expect(deletedEntry).toBeVisible();
    const purgedConversationId = await deletedEntry
      .locator('.conversation-entry-button')
      .getAttribute('data-conversation-id');
    expect(purgedConversationId).toBeTruthy();

    const dialogs = [];
    page.on('dialog', async (dialog) => {
      dialogs.push(dialog.message());
      if (dialogs.length === 1) {
        await dialog.dismiss();
      } else {
        await dialog.accept();
      }
    });
    await page.locator('[data-testid="conversation-purge-deleted"]').click();
    await expect(page.locator('[data-testid="memory-status"]')).toContainText(
      'Permanently deleted',
    );

    expect(dialogs.length).toBe(2);
    expect(dialogs[0]).toContain('Export memory first');
    expect(dialogs[1]).toContain('irreversible');

    await expect(
      page.locator('[data-testid="conversation-entries"] li', {
        hasText: 'Hello to purge',
      }),
    ).toHaveCount(0);

    const remainingEvents = await page.evaluate(
      (conversationId) =>
        window.FormalAiMemory.listEvents().then((events) =>
          events.filter((event) => event.conversationId === conversationId),
        ),
      purgedConversationId,
    );
    expect(remainingEvents).toEqual([]);
  });

  test('Issue #196: reset memory clears all browser events after export warning and confirmation', async ({ page }) => {
    await sendPrompt(page, 'Hello before reset');
    await expect(page.locator('[data-testid="conversation-entries"] li').first()).toContainText(
      'Hello before reset',
    );

    let dialogCount = 0;
    page.on('dialog', async (dialog) => {
      dialogCount += 1;
      if (dialogCount === 1) {
        await dialog.dismiss();
      } else {
        await dialog.accept();
      }
    });
    await page.locator('[data-testid="memory-reset"]').click();

    await expect(page.locator('[data-testid="memory-status"]')).toContainText(
      'Reset memory: deleted',
    );
    expect(dialogCount).toBe(2);
    await expect(page.locator('[data-testid="chat-message"]')).toHaveCount(0);
    await expect(page.locator('[data-testid="conversation-entries"] li')).toHaveCount(0);

    const eventCount = await page.evaluate(() =>
      window.FormalAiMemory.listEvents().then((events) => events.length),
    );
    expect(eventCount).toBe(0);
  });
});

// Issue #27 R5: the demo cycle pulls turns from the same Example prompts
// list that the sidebar shows, so users discover every feature in demo mode.
test.describe('Issue #27: demo iterates Example prompts', () => {
  test.beforeEach(async ({ page }) => {
    await disableGreetingVariations(page);
    await page.goto('./');
    await expect(page.locator('.app')).toBeVisible({ timeout: 15_000 });
  });

  test('demo messages carry a label that matches an Example prompts entry', async ({ page }) => {
    // Collect labels from the sidebar (the visible Example prompts list).
    const sidebarLabels = await page
      .locator('.prompt-list button')
      .evaluateAll((nodes) =>
        nodes.map((n) => n.getAttribute('data-prompt-label')).filter(Boolean),
      );
    expect(sidebarLabels.length).toBeGreaterThan(0);

    // Wait for the first demo user message to appear.
    const userMessages = page.locator(
      '[data-testid="chat-message"].user[data-demo-label]',
    );
    await expect(userMessages.first()).toBeVisible({ timeout: 15_000 });

    const demoLabels = await userMessages.evaluateAll((nodes) =>
      nodes.map((n) => n.getAttribute('data-demo-label')).filter(Boolean),
    );
    expect(demoLabels.length).toBeGreaterThan(0);
    for (const label of demoLabels) {
      expect(sidebarLabels).toContain(label);
    }
  });

  test('Issue #112: Example prompts cover every supported prompt family', async ({ page }) => {
    const labels = await page
      .locator('.prompt-list button')
      .evaluateAll((nodes) =>
        nodes.map((node) => node.getAttribute('data-prompt-label') || ''),
      );
    expect(labels).toEqual(expect.arrayContaining([
      'Greeting (en)',
      'Farewell (en)',
      'Identity (hi)',
      'Clarification (ru)',
      'Capabilities (en)',
      'Calculation (en)',
      'Concept (hi)',
      'Summarization',
      'Brainstorming',
      'Fact Q&A (zh)',
      'Navigate URL',
      'Fetch URL',
      'Web search',
      'Coreference',
      'Roleplay',
      'Recall (cross-conv)',
      'Export memory',
      'Import memory',
    ]));
  });
});

// Issue #27 R3: sidebar sections behave like VS Code's accordion — expanded
// sections flex to share the remaining height equally and each section body
// scrolls independently.
test.describe('Issue #27: sidebar accordion', () => {
  test.beforeEach(async ({ page }) => {
    await disableGreetingVariations(page);
    await page.goto('./');
    await expect(page.locator('.app')).toBeVisible({ timeout: 15_000 });
    await switchToManualMode(page);
  });

  test('expanded sidebar sections share the available height equally', async ({ page }) => {
    const sections = page.locator(
      '[data-testid="context-panel"] .sidebar-section.is-expanded',
    );
    const count = await sections.count();
    expect(count).toBeGreaterThanOrEqual(2);
    const heights = [];
    for (let i = 0; i < count; i++) {
      const box = await sections.nth(i).boundingBox();
      expect(box).toBeTruthy();
      heights.push(box.height);
    }
    const min = Math.min(...heights);
    const max = Math.max(...heights);
    // Equal-share flex: heights should differ by no more than 4px (header
    // rounding tolerance).
    expect(max - min).toBeLessThanOrEqual(4);
  });

  test('each section body scrolls independently when content overflows', async ({ page }) => {
    const bodies = page.locator(
      '[data-testid="context-panel"] .sidebar-section.is-expanded .sidebar-section-body',
    );
    const count = await bodies.count();
    expect(count).toBeGreaterThanOrEqual(2);
    for (let i = 0; i < count; i++) {
      const overflow = await bodies.nth(i).evaluate(
        (el) => getComputedStyle(el).overflowY,
      );
      // `auto` (scroll when needed) or `scroll` (always) both satisfy the
      // independent-scroll requirement.
      expect(['auto', 'scroll']).toContain(overflow);
    }
  });

  test('collapsing a section gives its space to the remaining expanded sections', async ({ page }) => {
    const expandedSections = page.locator(
      '[data-testid="context-panel"] .sidebar-section.is-expanded',
    );
    const initialExpanded = await expandedSections.count();
    if (initialExpanded < 2) test.skip();
    const targetSectionId = await expandedSections.first().getAttribute('data-testid');
    const otherSectionId = await expandedSections.nth(1).getAttribute('data-testid');
    const targetSection = page.locator(`[data-testid="${targetSectionId}"]`);
    const otherSection = page.locator(`[data-testid="${otherSectionId}"]`);

    const initialOther = await otherSection.locator('.sidebar-section-body').boundingBox();

    // Collapse the first section by clicking its header button.
    await targetSection.locator('.sidebar-section-header').click();
    await expect(targetSection).toHaveAttribute('data-collapsed', 'true');

    const grownOther = await otherSection.locator('.sidebar-section-body').boundingBox();
    expect(grownOther.height).toBeGreaterThan(initialOther.height);
  });
});

// Issue #27 R11: natural-language cross-conversation recall. The user types
// something like "when did I ask about Rust" / "find Donald Trump in another
// conversation" and the assistant returns a Markdown report grouping matching
// events by conversation.
test.describe('Issue #27: cross-conversation recall', () => {
  test.beforeEach(async ({ page }) => {
    await disableGreetingVariations(page);
    // Wipe IndexedDB so each test starts from an empty event log.
    await page.addInitScript(() => {
      try {
        if (typeof indexedDB === 'undefined') return;
        if (window.sessionStorage.getItem('formal-ai-recall-reset') === '1') return;
        window.sessionStorage.setItem('formal-ai-recall-reset', '1');
        indexedDB.deleteDatabase('formal-ai-demo');
      } catch (_error) {}
    });
    await page.goto('./');
    await expect(page.locator('.app')).toBeVisible({ timeout: 15_000 });
    await switchToManualMode(page);
    // Issue #541 (R4): demo isolation keeps the demo conversation session-
    // scoped + sidebar-hidden, so the user may already be on an empty fresh
    // conversation here — and the "+ New conversation" button is disabled by
    // design until there is content to clear. Skip the click in that case.
    const newBtn = page.locator('[data-testid="conversation-new"]');
    if (await newBtn.isEnabled().catch(() => false)) {
      await newBtn.click();
    }
    await expect(page.locator('[data-testid="chat-message"]')).toHaveCount(0, { timeout: 5_000 });
  });

  test('"When did I ask about X" lists matches grouped by conversation', async ({ page }) => {
    // Conversation 1: ask about Rust.
    await sendPrompt(page, 'What is Rust?');
    // Switch to a fresh conversation.
    await page.locator('[data-testid="conversation-new"]').click();
    await expect(page.locator('[data-testid="chat-message"]')).toHaveCount(0, { timeout: 5_000 });
    // Conversation 2: ask about Wikipedia.
    await sendPrompt(page, 'What is Wikipedia?');
    // Conversation 2 (continued): trigger recall.
    const last = await sendPrompt(page, 'When did I ask about Rust?');
    await expect(last).toHaveClass(/assistant/);
    await expect(last).toContainText('mention');
    await expect(last).toContainText('Rust');
    // The matching conversation header should appear.
    await expect(last).toContainText('What is Rust');
  });

  test('"find X in another conversation" excludes the current conversation', async ({ page }) => {
    await sendPrompt(page, 'What is Rust?');
    await page.locator('[data-testid="conversation-new"]').click();
    await expect(page.locator('[data-testid="chat-message"]')).toHaveCount(0, { timeout: 5_000 });
    // The current conversation also mentions Rust; "in another conversation"
    // must filter it out and only surface the earlier one.
    await sendPrompt(page, 'Tell me more about Rust');
    const last = await sendPrompt(page, 'find Rust in another conversation');
    await expect(last).toHaveClass(/assistant/);
    const text = await last.innerText();
    // The earlier conversation must be surfaced.
    expect(text).toContain('What is Rust');
    // …and the current conversation's "Tell me more" turn must NOT appear in
    // the report (scope='other' filters out the active thread).
    expect(text).not.toContain('Tell me more');
  });

  test('recall with no matches reports a clear "no mentions" message', async ({ page }) => {
    await sendPrompt(page, 'Hi');
    const last = await sendPrompt(page, 'When did I ask about Haskell?');
    await expect(last).toHaveClass(/assistant/);
    await expect(last).toContainText(/No mentions of "Haskell"/);
  });

  test('Russian phrasing "Когда я спрашивал про X" triggers the recall skill', async ({ page }) => {
    await sendPrompt(page, 'Что такое Википедия?');
    const last = await sendPrompt(page, 'Когда я спрашивал про Википедия?');
    await expect(last).toHaveClass(/assistant/);
    await expect(last).toContainText('Википедия');
    // The earlier conversation header should be present.
    await expect(last).toContainText('Что такое');
  });

  test('Russian "what did I ask" recalls the previous user request', async ({ page }) => {
    await sendPrompt(page, 'Поставь мне встречу с мамукой на 10:00');
    const firstRecall = await sendPrompt(page, 'Что я спрашивал в прошлом сообщении?');
    await expect(firstRecall).toHaveClass(/assistant/);
    await expect(firstRecall).toContainText('Поставь мне встречу с мамукой на 10:00');
    await expect(firstRecall).not.toContainText(UNKNOWN_ANSWER_MARKER);

    const followup = await sendPrompt(page, 'а я что спрашивал?');
    await expect(followup).toHaveClass(/assistant/);
    await expect(followup).toContainText('Поставь мне встречу с мамукой на 10:00');
    await expect(followup).not.toContainText('Что я спрашивал в прошлом сообщении?');
  });
});
