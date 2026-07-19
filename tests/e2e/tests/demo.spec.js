// @ts-check
const { test, expect } = require('@playwright/test');

const UNKNOWN_ANSWER_MARKER = 'cannot answer that from local links rules';

async function switchToManualMode(page) {
  const demoToggle = page.locator('.mode-toggle');
  await expect(demoToggle).toContainText(/Demo on|Demo off|Демо/, {
    timeout: 10_000,
  });
  await demoToggle.click();
  await expect(page.locator('[data-testid="demo-status"]')).toHaveText('Manual mode');
  await expect(page.locator('[data-testid="chat-composer-input"]')).toBeEnabled({
    timeout: 5_000,
  });
}

// Issue #27: greeting randomisation defaults to ON in production. Tests pin
// the canonical greeting text so they assert deterministic output; new tests
// that actually exercise the randomisation flip the preference back on.
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

async function sendPrompt(page, text) {
  const input = page.locator('[data-testid="chat-composer-input"]');
  await expect(input).toBeEnabled({ timeout: 5_000 });
  await input.fill(text);
  return submitCurrentPrompt(page);
}

async function submitCurrentPrompt(page) {
  const messages = page.locator('[data-testid="chat-message"]');
  const initialCount = await messages.count();
  await page.locator('[data-testid="chat-composer-submit"]').click();
  await expect(messages).toHaveCount(initialCount + 2, { timeout: 20_000 });
  const lastMsg = messages.last();
  await expect(lastMsg).toHaveClass(/assistant/);
  return lastMsg;
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

// Issue #557 / #108: assert that a control's bounding box sits inside the
// composer field surface, proving the buttons are embedded into the text field
// rather than living in a separate bordered box. A small tolerance absorbs
// sub-pixel rounding and the focus ring.
function assertContained(childBox, parentBox, tolerance = 2) {
  expect(childBox).toBeTruthy();
  expect(parentBox).toBeTruthy();
  expect(childBox.x).toBeGreaterThanOrEqual(parentBox.x - tolerance);
  expect(childBox.y).toBeGreaterThanOrEqual(parentBox.y - tolerance);
  expect(childBox.x + childBox.width).toBeLessThanOrEqual(
    parentBox.x + parentBox.width + tolerance,
  );
  expect(childBox.y + childBox.height).toBeLessThanOrEqual(
    parentBox.y + parentBox.height + tolerance,
  );
}

test.describe('formal-ai demo UI', () => {
  test.beforeEach(async ({ page }) => {
    await disableGreetingVariations(page);
    await page.goto('./');
    // Wait for React to mount and the app shell to be visible
    await expect(page.locator('.app')).toBeVisible({ timeout: 15_000 });
  });

  test('page title is formal-ai', async ({ page }) => {
    await expect(page).toHaveTitle('formal-ai');
  });

  test('brand header is visible', async ({ page }) => {
    await expect(page.locator('.brand')).toBeVisible();
    await expect(page.locator('.brand strong')).toContainText('formal-ai');
  });

  test('demo mode starts automatically and exposes a live countdown', async ({ page }) => {
    const demoToggle = page.locator('.mode-toggle');
    await expect(demoToggle).toBeVisible();
    await expect(demoToggle).toContainText('Demo on');

    const input = page.locator('[data-testid="chat-composer-input"]');
    await expect(input).toBeDisabled();

    const demoStatus = page.locator('[data-testid="demo-status"]');
    await expect(demoStatus).toBeVisible();
    await expect(demoStatus).toContainText(/Demo playing|Next dialog in/);
    await expect(demoStatus).toContainText(/Next dialog in \d+s/, { timeout: 15_000 });

    const firstCountdown = await demoStatus.textContent();
    await page.waitForTimeout(1200);
    await expect(demoStatus).not.toHaveText(firstCountdown || '');
  });

  test('automatic demo renders a chat exchange', async ({ page }) => {
    const messageList = page.locator('[data-testid="message-list"]');
    await expect(messageList).toBeVisible();

    const messages = page.locator('[data-testid="chat-message"]');
    await expect(messages.first()).toBeVisible({ timeout: 15_000 });

    await expect(page.locator('[data-testid="chat-message"].user').first()).toBeVisible();
    await expect(page.locator('[data-testid="chat-message"].assistant').first()).toBeVisible({
      timeout: 15_000,
    });
  });

  test('quick prompts sidebar is visible with expected prompts', async ({ page }) => {
    const promptList = page.locator('.prompt-list');
    await expect(promptList).toBeVisible();

    const buttons = promptList.locator('button');
    await expect(buttons.first()).toContainText('Hi');
    // Issue #27: the list now includes multilingual greetings before the Rust
    // hello-world entry, so match by label instead of by absolute index.
    await expect(promptList.locator('button[data-prompt-label*="Rust"]').first()).toBeVisible();
  });

  test('chat input and send button are present', async ({ page }) => {
    await switchToManualMode(page);

    const input = page.locator('[data-testid="chat-composer-input"]');
    await expect(input).toBeVisible();
    await expect(input).toBeEnabled();

    const sendBtn = page.locator('[data-testid="chat-composer-submit"]');
    await expect(sendBtn).toBeVisible();
  });

  test('send button is disabled when input is empty', async ({ page }) => {
    await switchToManualMode(page);

    const sendBtn = page.locator('[data-testid="chat-composer-submit"]');
    await expect(sendBtn).toBeDisabled();
  });

  test('clicking a quick prompt populates the input', async ({ page }) => {
    const hiButton = page.locator('.prompt-list button').first();
    await hiButton.click();

    const input = page.locator('[data-testid="chat-composer-input"]');
    await expect(input).toBeEnabled({ timeout: 5_000 });
    await expect(input).toHaveValue('Hi');
  });

  test('sending a greeting produces an assistant reply', async ({ page }) => {
    await switchToManualMode(page);

    const input = page.locator('[data-testid="chat-composer-input"]');
    await input.fill('Hello');

    const messages = page.locator('[data-testid="chat-message"]');
    const initialCount = await messages.count();

    const sendBtn = page.locator('[data-testid="chat-composer-submit"]');
    await sendBtn.click();

    // Wait for the assistant response to appear (worker or fallback)
    await expect(messages).toHaveCount(initialCount + 2, { timeout: 15_000 });

    const lastMsg = messages.last();
    await expect(lastMsg).toHaveClass(/assistant/);
    await expect(lastMsg).toContainText('Hi');
  });

  test('known assistant dialogs keep report issue in the topbar only', async ({ page }) => {
    await switchToManualMode(page);

    const input = page.locator('[data-testid="chat-composer-input"]');
    await input.fill('Hello');

    const messages = page.locator('[data-testid="chat-message"]');
    const initialCount = await messages.count();

    await page.locator('[data-testid="chat-composer-submit"]').click();
    await expect(messages).toHaveCount(initialCount + 2, { timeout: 15_000 });

    const lastMsg = messages.last();
    await expect(lastMsg).toHaveClass(/assistant/);
    await expect(lastMsg.locator('.message-actions a')).toHaveCount(0);

    const reportLink = page.locator('[data-testid="report-issue"]');
    await expect(reportLink).toHaveText(/Report issue/);

    const href = await reportLink.getAttribute('href');
    expect(href).toBeTruthy();

    const url = new URL(href || '');
    const body = url.searchParams.get('body') || '';

    expect(`${url.origin}${url.pathname}`).toBe(
      'https://github.com/link-assistant/formal-ai/issues/new',
    );
    expect(url.searchParams.get('labels')).toBe('bug');
    expect(body).toContain('## Environment');
    expect(body).toContain('## Reproduction of dialog');
    expect(body).not.toContain('## Dialog');
    expect(body).not.toContain('## Reproduction Steps');
    // Issue #78: the dialog is now a single fenced code block with `U:` /
    // `A:` line prefixes instead of one Markdown subsection per message.
    expect(body).toContain('Legend: `U` = user, `A` = agent.');
    expect(body).toContain('U: Hello');
    expect(body).toContain('A: Hi');
    expect(body).not.toContain('reported');
    // The verbose per-message subsections must be gone.
    expect(body).not.toMatch(/^### \d+\. /m);
    expect(body).not.toContain('- **Role**:');
  });

  test('known TypeScript hello world response does not surface a response-level report link', async ({ page }) => {
    await switchToManualMode(page);

    const input = page.locator('[data-testid="chat-composer-input"]');
    await input.fill('Write hello world in TypeScript');

    const messages = page.locator('[data-testid="chat-message"]');
    const initialCount = await messages.count();

    await page.locator('[data-testid="chat-composer-submit"]').click();
    await expect(messages).toHaveCount(initialCount + 2, { timeout: 15_000 });

    const lastMsg = messages.last();
    await expect(lastMsg).toHaveClass(/assistant/);
    await expect(lastMsg).toContainText('typescript');

    await expect(lastMsg.locator('.message-actions a')).toHaveCount(0);

    const reportLink = page.locator('[data-testid="report-issue"]');
    const href = await reportLink.getAttribute('href');
    expect(href).toBeTruthy();

    const url = new URL(href || '');
    const body = url.searchParams.get('body') || '';

    expect(url.searchParams.get('title')).toContain('Issue with dialog');
    expect(url.searchParams.get('title')).toContain('TypeScript');
    expect(body).toContain('U: Write hello world in TypeScript');
    expect(body).not.toContain('reported');
    // The dialog must not accidentally annotate user messages.
    expect(body).not.toMatch(/U \(.*reported.*\):/);
  });

  test('asking who are you produces an identity response', async ({ page }) => {
    await switchToManualMode(page);

    const input = page.locator('[data-testid="chat-composer-input"]');
    await input.fill('Who are you?');

    const messages = page.locator('[data-testid="chat-message"]');
    const initialCount = await messages.count();

    await page.locator('[data-testid="chat-composer-submit"]').click();
    await expect(messages).toHaveCount(initialCount + 2, { timeout: 15_000 });

    const lastMsg = messages.last();
    await expect(lastMsg).toHaveClass(/assistant/);
    await expect(lastMsg).toContainText('formal-ai');
    await expect(lastMsg).not.toContainText(UNKNOWN_ANSWER_MARKER);
  });

  test('assistant name prompts answer and respect the configured name', async ({ page }) => {
    await switchToManualMode(page);

    let lastMsg = await sendPrompt(page, 'Как твое имя?');
    await expect(lastMsg).toContainText('formal AI');
    await expect(lastMsg).not.toContainText(UNKNOWN_ANSWER_MARKER);

    await page.locator('[data-testid="setting-assistant-name"]').fill('Astra');
    await expect.poll(() =>
      page.evaluate(() => window.localStorage.getItem('formal-ai.preferences.v1') || ''),
    ).toContain('assistantName "Astra"');

    lastMsg = await sendPrompt(page, 'What is your name?');
    await expect(lastMsg).toContainText('Astra');
    await expect(lastMsg).not.toContainText(UNKNOWN_ANSWER_MARKER);

    lastMsg = await sendPrompt(page, 'call yourself Vega');
    await expect(lastMsg).toContainText('Assistant name');
    await expect(lastMsg).toContainText('Vega');

    lastMsg = await sendPrompt(page, 'What is your name?');
    await expect(lastMsg).toContainText('Vega');
    await expect(lastMsg).not.toContainText(UNKNOWN_ANSWER_MARKER);

    lastMsg = await sendPrompt(page, 'Теперь тебя зовут Алексей');
    await expect(lastMsg).toContainText('Assistant name');
    await expect(lastMsg).toContainText('Алексей');
    await expect(lastMsg).not.toContainText(UNKNOWN_ANSWER_MARKER);

    lastMsg = await sendPrompt(page, 'Как тебя зовут?');
    await expect(lastMsg).toContainText('Алексей');
    await expect(lastMsg).not.toContainText(UNKNOWN_ANSWER_MARKER);
  });

  test('polite small-talk follow-up does not fall through to unknown', async ({ page }) => {
    await switchToManualMode(page);
    await setRangeValue(page, 'setting-temperature', 0);
    await setRangeValue(page, 'setting-follow-up-probability', 1);

    const lastMsg = await sendPrompt(page, 'I am fine, thank you');

    await expect(lastMsg).toContainText('Glad to hear it');
    await expect(lastMsg).toContainText('What would you like to do next?');
    await expect(lastMsg).not.toContainText(UNKNOWN_ANSWER_MARKER);
  });

  test('courtesy response can leave initiative with the user', async ({ page }) => {
    await switchToManualMode(page);
    await setRangeValue(page, 'setting-temperature', 0);
    await setRangeValue(page, 'setting-follow-up-probability', 0);

    const lastMsg = await sendPrompt(page, 'I am fine, thank you');

    await expect(lastMsg).toContainText('Glad to hear it.');
    await expect(lastMsg).not.toContainText('What would you like to do next?');
    await expect(lastMsg).not.toContainText(UNKNOWN_ANSWER_MARKER);
  });

  test('reported prompt examples resolve through the browser worker', async ({ page }) => {
    await switchToManualMode(page);

    await page.route('**/w/rest.php/v1/search/page**', async (route) => {
      await route.fulfill({
        status: 200,
        contentType: 'application/json',
        body: JSON.stringify({
          pages: [
            {
              id: 123,
              key: 'Genshin_Impact',
              title: 'Genshin Impact',
              excerpt: 'Genshin Impact is an action role-playing game.',
              description: 'action role-playing game',
            },
          ],
        }),
      });
    });

    const testStatus = await sendPrompt(page, 'Test');
    await expect(testStatus).toContainText('Test passed');
    await expect(testStatus).toContainText("I'm here");
    await expect(testStatus).not.toContainText(UNKNOWN_ANSWER_MARKER);

    for (const { prompt, expected } of [
      { prompt: 'тест пройден', expected: 'Тест пройден' },
      { prompt: 'परीक्षण सफल रहा', expected: 'परीक्षण सफल रहा' },
      { prompt: '测试通过', expected: '测试通过' },
    ]) {
      const localizedStatus = await sendPrompt(page, prompt);
      await expect(localizedStatus).toContainText(expected);
      await expect(localizedStatus).not.toContainText(UNKNOWN_ANSWER_MARKER);
    }

    const capabilities = await sendPrompt(page, 'What you can do?');
    await expect(capabilities).toContainText('Here is what I can do');
    await expect(capabilities).toContainText('Hello World');
    await expect(capabilities).not.toContainText(UNKNOWN_ANSWER_MARKER);

    const webSearchCapability = await sendPrompt(page, 'Ты можешь искать в интернете?');
    await expect(webSearchCapability).toContainText('Да');
    await expect(webSearchCapability).toContainText('DuckDuckGo');
    await expect(webSearchCapability).not.toContainText(UNKNOWN_ANSWER_MARKER);

    const moreCapabilities = await sendPrompt(page, 'Что ещё ты умеешь?');
    await expect(moreCapabilities).toContainText('Арифметика');
    await expect(moreCapabilities).toContainText('Перевод');
    await expect(moreCapabilities).not.toContainText('DuckDuckGo');
    await expect(moreCapabilities).not.toContainText('интернете');
    await expect(moreCapabilities).not.toContainText(UNKNOWN_ANSWER_MARKER);

    const reportedCapabilities = await sendPrompt(page, 'А в чём ты можешь быть полезен');
    await expect(reportedCapabilities).toContainText('Я formal-ai');
    await expect(reportedCapabilities).toContainText('Вот что я умею');
    await expect(reportedCapabilities).not.toContainText(UNKNOWN_ANSWER_MARKER);

    const arithmeticCapability = await sendPrompt(page, 'Can you do arithmetic?');
    await expect(arithmeticCapability).toContainText('Yes');
    await expect(arithmeticCapability).toContainText('arithmetic');
    await expect(arithmeticCapability).not.toContainText(UNKNOWN_ANSWER_MARKER);

    const translation = await sendPrompt(page, 'Переведи "как у тебя дела?" на английский.');
    await expect(translation).toContainText('how are you?');
    await expect(translation).not.toContainText(UNKNOWN_ANSWER_MARKER);

    const search = await sendPrompt(page, 'Search online for Genshin Impact');
    await expect(search).toContainText('Search results for');
    await expect(search).toContainText('Genshin Impact');
    await expect(search).not.toContainText(UNKNOWN_ANSWER_MARKER);

    const roleplay = await sendPrompt(
      page,
      'Pretend you are Albert Einstein and explain relativity to a teenager.',
    );
    await expect(roleplay).toContainText('Roleplay frame recorded for Albert Einstein');
    await expect(roleplay).toContainText('relativity');
    await expect(roleplay).not.toContainText(UNKNOWN_ANSWER_MARKER);

    await page.locator('button[data-prompt-label="Idiom (ru)"]').click();
    await expect(page.locator('[data-testid="chat-composer-input"]')).toHaveValue('Купи слона');
    const idiom = await submitCurrentPrompt(page);
    await expect(idiom).toContainText('У всех есть слон');
    await expect(idiom).not.toContainText(UNKNOWN_ANSWER_MARKER);
  });

  test('Owlbear extension request returns a software project plan', async ({ page }) => {
    await switchToManualMode(page);

    const prompt = [
      'Hi, can you write for me extension for owlbear?',
      'I am currently leading some dnd games and i want to try wargame.',
      'I need extensions that can track hp for different units,',
      'track Protection and Resistance stacks, reduce damage count on those stats,',
      'and track cooldown of some abilities.',
    ].join(' ');

    const input = page.locator('[data-testid="chat-composer-input"]');
    await input.fill(prompt);

    const messages = page.locator('[data-testid="chat-message"]');
    const initialCount = await messages.count();

    await page.locator('[data-testid="chat-composer-submit"]').click();
    await expect(messages).toHaveCount(initialCount + 2, { timeout: 15_000 });

    const lastMsg = messages.last();
    await expect(lastMsg).toHaveClass(/assistant/);
    await expect(lastMsg).toContainText('Implementation plan');
    await expect(lastMsg).toContainText('Formalized meaning');
    await expect(lastMsg).toContainText('software_project_request');
    await expect(lastMsg).toContainText('Owlbear');
    await expect(lastMsg).toContainText('Protection');
    await expect(lastMsg).toContainText('approve plan');
    await expect(lastMsg).not.toContainText('mitigateDamage');
    await expect(lastMsg).not.toContainText(UNKNOWN_ANSWER_MARKER);
  });

  test('unknown prompts include a prefilled missing-rule issue link', async ({ page }) => {
    await switchToManualMode(page);

    // Pick a phrase no Wikipedia article will match so the unknown-intent
    // fallback path is exercised (the Wikipedia REST API answers many
    // plausible-looking questions like "What is the capital of France?").
    const prompt = 'Quxblort fnordwarble plimsy gabble what?';
    const input = page.locator('[data-testid="chat-composer-input"]');
    await input.fill(prompt);

    const messages = page.locator('[data-testid="chat-message"]');
    const initialCount = await messages.count();

    await page.locator('[data-testid="chat-composer-submit"]').click();
    await expect(messages).toHaveCount(initialCount + 2, { timeout: 15_000 });

    const lastMsg = messages.last();
    await expect(lastMsg).toHaveClass(/assistant/);
    await expect(lastMsg).toContainText(UNKNOWN_ANSWER_MARKER);

    const reportLink = lastMsg.locator('.message-actions a');
    await expect(reportLink).toHaveText('Report missing rule');

    const href = await reportLink.getAttribute('href');
    expect(href).toBeTruthy();

    const url = new URL(href || '');
    const body = url.searchParams.get('body') || '';

    expect(url.searchParams.get('title')).toContain('Unknown prompt');
    expect(url.searchParams.get('labels')).toBe('bug');
    expect(body).toContain('## Environment');
    expect(body).toContain('**Version**');
    // Issue #140: the user agent is now folded into the combined **UI** field
    // of User Context (e.g. "1536x730 viewport, ... Chrome/... browser, ...").
    expect(body).toMatch(/\*\*UI\*\*:.* browser/);
    expect(body).toContain('## Reproduction of dialog');
    expect(body).toContain(prompt);
    expect(body).toContain('## Reasoning Trace');
    expect(body).toContain('diagnostics_steps:');
    // Issue #78: intent now appears inline next to the assistant turn marker
    // ("A (intent: unknown, reported): ...") inside the dialog code block.
    expect(body).toMatch(/A \(intent: unknown[^)]*\):/);
    expect(body).not.toContain('## Reproduction Steps');
    // Issue #140: the prefilled URL must stay below GitHub's 8192-byte cap.
    expect(href.length).toBeLessThanOrEqual(8192);
  });

  test('behavior rules can be listed, inspected, and updated through chat', async ({ page }) => {
    await switchToManualMode(page);

    let lastMsg = await sendPrompt(page, 'List behavior rules');
    await expect(lastMsg).toContainText('rule_greeting');
    await expect(lastMsg).toContainText('rule_unknown');

    lastMsg = await sendPrompt(page, 'Show behavior rule unknown');
    await expect(lastMsg).toContainText('rule_unknown');
    await expect(lastMsg).toContainText('When I say');

    lastMsg = await sendPrompt(
      page,
      'When I say `Какая у тебя модель личности?`, answer `У меня символьная модель личности.`',
    );
    await expect(lastMsg).toContainText('behavior_rule_runtime');

    lastMsg = await sendPrompt(page, 'Какая у тебя модель личности?');
    await expect(lastMsg).toContainText('У меня символьная модель личности.');

    lastMsg = await sendPrompt(page, 'List all facts you know about yourself');
    await expect(lastMsg).toContainText('self_fact_model');
    await expect(lastMsg).toContainText('local links rules');

    lastMsg = await sendPrompt(page, 'какие факты ты знаешь?');
    await expect(lastMsg).toContainText(/Интернет|Internet/);
    await expect(lastMsg).toContainText(/Память|memory/i);
  });

  test('sending a hello world request produces a code block', async ({ page }) => {
    await switchToManualMode(page);

    const input = page.locator('[data-testid="chat-composer-input"]');
    await input.fill('Write me hello world program in Rust');

    const messages = page.locator('[data-testid="chat-message"]');
    const initialCount = await messages.count();

    const sendBtn = page.locator('[data-testid="chat-composer-submit"]');
    await sendBtn.click();

    // Wait for assistant message with Rust code
    await expect(messages).toHaveCount(initialCount + 2, { timeout: 15_000 });

    const lastMsg = messages.last();
    await expect(lastMsg).toHaveClass(/assistant/);
    await expect(lastMsg).toContainText('Rust hello world');
    await expect(lastMsg).toContainText('Execution status:');
  });

  // Issue #330 (R11): the catalog supports a wider range of coding tasks than
  // hello-world, and the JS worker mirrors the Rust engine — including the R9
  // novice guidance. A FizzBuzz request must return the code, its verified
  // output, and the localized "How it works" / "How to test" sections.
  test('sending a FizzBuzz request returns code, output and novice guidance', async ({
    page,
  }) => {
    await switchToManualMode(page);

    const input = page.locator('[data-testid="chat-composer-input"]');
    await input.fill('Write me a FizzBuzz program in Rust');

    const messages = page.locator('[data-testid="chat-message"]');
    const initialCount = await messages.count();

    const sendBtn = page.locator('[data-testid="chat-composer-submit"]');
    await sendBtn.click();

    await expect(messages).toHaveCount(initialCount + 2, { timeout: 15_000 });

    const lastMsg = messages.last();
    await expect(lastMsg).toHaveClass(/assistant/);
    await expect(lastMsg).toContainText('Rust FizzBuzz');
    // The verified deterministic output is shown for the novice to compare.
    await expect(lastMsg).toContainText('FizzBuzz');
    // R9 novice guidance mirrored from the Rust engine.
    await expect(lastMsg).toContainText('How it works:');
    await expect(lastMsg).toContainText('How to test it yourself:');
  });

  test('pressing Enter submits the message', async ({ page }) => {
    await switchToManualMode(page);

    const input = page.locator('[data-testid="chat-composer-input"]');
    await input.fill('Hi');

    const messages = page.locator('[data-testid="chat-message"]');
    const initialCount = await messages.count();

    await input.press('Enter');

    await expect(messages).toHaveCount(initialCount + 2, { timeout: 15_000 });
  });

  test('demo mode toggle button is present', async ({ page }) => {
    const demoToggle = page.locator('.mode-toggle');
    await expect(demoToggle).toBeVisible();
    await expect(demoToggle).toContainText('Demo on');

    await demoToggle.click();
    await expect(demoToggle).toContainText('Demo');
  });

  test('toggling demo mode disables the input', async ({ page }) => {
    const demoToggle = page.locator('.mode-toggle');

    const input = page.locator('[data-testid="chat-composer-input"]');
    await expect(input).toBeDisabled({ timeout: 5_000 });

    await demoToggle.click();
    await expect(input).toBeEnabled({ timeout: 5_000 });
  });

  test('diagnostics are hidden by default', async ({ page }) => {
    await expect(page.locator('.trace-list')).toHaveCount(0);
    await expect(page.locator('.intent')).toHaveCount(0);
    await expect(page.locator('.evidence-list')).toHaveCount(0);
    await expect(page.locator('.thinking-steps')).toHaveCount(0);
  });

  test('diagnostics toggle shows trace, intent, evidence, and thinking steps', async ({ page }) => {
    await expect(page.locator('[data-testid="chat-message"].assistant').first()).toBeVisible({
      timeout: 15_000,
    });

    const diagnosticsToggle = page.locator('.diagnostics-toggle');
    await expect(diagnosticsToggle).toBeVisible();
    await expect(diagnosticsToggle).toContainText('Diagnostics');
    await diagnosticsToggle.click();

    const traceList = page.locator('.trace-list');
    await expect(traceList).toBeVisible();
    await expect(traceList).toContainText('formal-ai');
    await expect(traceList).toContainText('Intent');

    const assistantMessage = page.locator('[data-testid="chat-message"].assistant').first();
    await expect(assistantMessage.locator('.intent')).toContainText(/intent:/);
    await expect(assistantMessage.locator('.evidence-list')).toContainText(/source:/);
    await expect(assistantMessage.locator('.thinking-steps')).toContainText(/match_rule|dispatch_handler|fallback/);
  });

  test('message commands configure UI controls', async ({ page }) => {
    await switchToManualMode(page);

    const diagnostics = await sendPrompt(page, 'Turn on diagnostics');
    await expect(diagnostics).toContainText('Diagnostics is now on');
    await expect(page.locator('.diagnostics-toggle')).toHaveAttribute('aria-pressed', 'true');

    const theme = await sendPrompt(page, 'Switch to dark theme');
    await expect(theme).toContainText('Theme is now dark');
    await expect(page.locator('html')).toHaveAttribute('data-theme', 'dark');
    await expect(page.locator('[data-testid="setting-theme"]')).toHaveValue('dark');
  });

  test('Issue #226: quoted language prose is not a UI language command across supported languages', async ({ page }) => {
    await switchToManualMode(page);

    const quotedPrompts = [
      [
        'Gemini answered that English Wikipedia does not have a separate article',
        'titled "agreement in a sentence", but it has an article about Russian grammar.',
      ].join(' '),
      [
        'Google Gemini ответил мне так -',
        'В русскоязычной Википедии нет отдельной статьи с названием «Согласованность в предложении»,',
        'однако есть статья «Согласование (грамматика)» о грамматике русского языка.',
        'Можешь научиться делать так же?',
      ].join(' '),
      'हिन्दी विकिपीडिया में रूसी भाषा पर लेख है। क्या तुम ऐसा उत्तर सीख सकते हो?',
      '中文维基百科有一篇关于俄语语言的条目。可以学习这种回答吗？',
    ];

    for (const prompt of quotedPrompts) {
      const quotedAnswer = await sendPrompt(page, prompt);
      await expect(quotedAnswer).not.toContainText('Done. UI language is now');
      await expect(page.locator('[data-testid="setting-ui-language"]')).toHaveValue('auto');
    }

    const commands = [
      { prompt: 'set ui language to english', value: 'en' },
      { prompt: 'переключи язык на русский', value: 'ru' },
      { prompt: 'भाषा हिंदी सेट करें', value: 'hi' },
      { prompt: '设置界面语言为中文', value: 'zh' },
    ];

    for (const { prompt, value } of commands) {
      const explicitCommand = await sendPrompt(page, prompt);
      await expect(explicitCommand).toContainText(`UI language is now ${value}`);
      await expect(page.locator('[data-testid="setting-ui-language"]')).toHaveValue(value);
      await expect(page.locator('html')).toHaveAttribute('lang', value);
    }
  });

  test('composer does not expose an unused preview control', async ({ page }) => {
    await switchToManualMode(page);

    await expect(page.locator('.preview-toggle')).toHaveCount(0);
    await expect(page.locator('.composer-preview')).toHaveCount(0);
  });

  test('demo hint is shown in demo mode and hidden in manual mode', async ({ page }) => {
    const hint = page.locator('[data-testid="composer-demo-hint"]');
    await expect(hint).toBeVisible({ timeout: 5_000 });
    await expect(hint).toContainText('Demo is running');

    await page.locator('.mode-toggle').click();
    await expect(hint).toHaveCount(0);
  });
});

test.describe('Issue #94: theme, localization, and report context', () => {
  test('honors dark color-scheme preference automatically', async ({ page }) => {
    await disableGreetingVariations(page);
    await page.emulateMedia({ colorScheme: 'dark' });
    await page.goto('./');
    await expect(page.locator('.app')).toBeVisible({ timeout: 15_000 });

    const colors = await page.locator('.topbar').evaluate((node) => {
      const styles = window.getComputedStyle(node);
      return {
        background: styles.backgroundColor,
        color: styles.color,
      };
    });

    expect(colors.background).not.toBe('rgb(251, 252, 253)');
    expect(colors.color).not.toBe('rgb(30, 37, 43)');
  });

  test('auto-detects Russian UI language from browser preferences', async ({ page }) => {
    await page.addInitScript(() => {
      Object.defineProperty(window.navigator, 'language', {
        configurable: true,
        get: () => 'ru-RU',
      });
      Object.defineProperty(window.navigator, 'languages', {
        configurable: true,
        get: () => ['ru-RU', 'en-US'],
      });
    });
    await disableGreetingVariations(page);
    await page.goto('./');
    await expect(page.locator('.app')).toBeVisible({ timeout: 15_000 });

    await expect(page.locator('[data-testid="demo-status"]')).toContainText('Демо');
    await page.locator('.mode-toggle').click();
    await expect(page.locator('[data-testid="demo-status"]')).toHaveText('Ручной режим');
    await expect(page.locator('[data-testid="report-issue"]')).toContainText(
      'Сообщить о проблеме',
    );
    await expect(page.locator('[data-testid="report-issue"]')).toHaveAttribute(
      'title',
      /Сообщить о проблеме/,
    );
    await expect(page.locator('.diagnostics-toggle')).toHaveAttribute(
      'title',
      /Показать диагностическую трассировку/,
    );
    await expect(page.locator('[data-testid="chat-composer-input"]')).toHaveAttribute(
      'placeholder',
      'Сообщение formal-ai',
    );
    await expect(page.locator('html')).toHaveAttribute('lang', 'ru');
  });

  test('ships UI dictionaries for all required languages', async ({ page }) => {
    await disableGreetingVariations(page);
    await page.goto('./');
    await expect(page.locator('.app')).toBeVisible({ timeout: 15_000 });

    const labels = await page.evaluate(async () => {
      await window.FormalAiI18n.ready;
      return {
        en: window.FormalAiI18n.t('buttons.reportIssue', 'en'),
        ru: window.FormalAiI18n.t('buttons.reportIssue', 'ru'),
        zh: window.FormalAiI18n.t('buttons.reportIssue', 'zh'),
        hi: window.FormalAiI18n.t('buttons.reportIssue', 'hi'),
      };
    });

    expect(labels).toEqual({
      en: 'Report issue',
      ru: 'Сообщить о проблеме',
      zh: '报告问题',
      hi: 'समस्या रिपोर्ट करें',
    });
  });

  test('loads the published lino-i18n runtime for UI translations', async ({ page }) => {
    await disableGreetingVariations(page);
    await page.goto('./');
    await expect(page.locator('.app')).toBeVisible({ timeout: 15_000 });

    const runtime = await page.evaluate(async () => {
      await window.FormalAiI18n.ready;
      return {
        engine: window.FormalAiI18n.ENGINE_SOURCE,
        russian: window.FormalAiI18n.t('buttons.reportIssue', 'ru'),
        fallback: window.FormalAiI18n.t('buttons.reportIssue', 'zz'),
      };
    });

    expect(runtime).toEqual({
      engine: 'lino-i18n@0.1.1',
      russian: 'Сообщить о проблеме',
      fallback: 'Report issue',
    });
  });

  test('loads the nested Links Notation catalog with generated parent labels', async ({ page }) => {
    await disableGreetingVariations(page);
    await page.goto('./');
    await expect(page.locator('.app')).toBeVisible({ timeout: 15_000 });

    const catalog = await page.evaluate(async () => {
      await window.FormalAiI18n.ready;
      return {
        source: window.FormalAiI18n.ENGINE_SOURCE,
        reportTitle: window.FormalAiI18n.t('titles.reportIssue', 'en'),
        settingsLanguage: window.FormalAiI18n.t('settings.language', 'en'),
        timedStatus: window.FormalAiI18n.t('status.nextDialogIn', 'en', {
          seconds: 8,
        }),
        catalogUrl: window.FormalAiI18n.CATALOG_URL,
      };
    });

    expect(catalog).toMatchObject({
      source: 'lino-i18n@0.1.1',
      settingsLanguage: 'Language',
      timedStatus: 'Next dialog in 8s',
      catalogUrl: 'i18n-catalog.lino',
    });
    expect(catalog.reportTitle).toContain('pre-filled GitHub issue');
    expect(catalog.reportTitle).toContain('docs/upload-memory.md');
  });

  test('issue reports include UI, browser, and coarse location context', async ({ page }) => {
    await disableGreetingVariations(page);
    await page.emulateMedia({ colorScheme: 'dark' });
    await page.goto('./');
    await expect(page.locator('.app')).toBeVisible({ timeout: 15_000 });
    await switchToManualMode(page);

    const href = await page.locator('[data-testid="report-issue"]').getAttribute('href');
    expect(href).toBeTruthy();

    const body = new URL(href || '').searchParams.get('body') || '';
    expect(body).toContain('## User Context');
    // Issue #140: UI languages, theme, UI, locale, and location are now
    // emitted on combined single lines so prefilled URLs stay below GitHub's
    // 8192-byte cap. Browser languages appear inside **UI languages**, and
    // the user agent / viewport / screen / platform inside **UI**.
    expect(body).toMatch(/\*\*UI languages\*\*: \*?[^*]+\*?(?:, [^,\n]+)*/);
    expect(body).toMatch(/\*\*UI\*\*: .*viewport, .*screen, .* browser/);
    expect(body).toMatch(/\*\*Locale\*\*: .* \([^)]+\)/);
    // Issue #386: settings that sit at their default are omitted to save room
    // for the dialog. The default theme ("auto") and the inference-only
    // location are therefore no longer emitted in a fresh-default report.
    expect(body).not.toContain('**Theme**');
    expect(body).not.toContain('**Location**');
    // The verbose per-field labels from the old layout must be gone so the
    // prefilled URL stays below GitHub's 8192-byte cap.
    expect(body).not.toContain('**UI Language**');
    expect(body).not.toContain('**Browser Languages**');
    expect(body).not.toContain('**Color Scheme**');
    expect(body).not.toContain('**Time Zone**');
    expect(body).not.toContain('**Location Inference**');
    expect(body).not.toContain('**Online**');
    expect(body).not.toContain('**Viewport**');
    expect(body).not.toContain('**Screen**');
    expect(body).not.toContain('**Platform**');
    expect(href.length).toBeLessThanOrEqual(8192);
  });

  // Issue #140: GitHub rejects prefilled URLs longer than ~8192 bytes with
  // "Whoa there!". The Report issue link must therefore stay under that cap
  // even when the dialog has many turns and Cyrillic content. The fitter is
  // expected to drop earlier messages and, if needed, truncate the last two.
  test('prefilled issue URL stays below GitHub 8KB cap with a long dialog', async ({ page }) => {
    await disableGreetingVariations(page);
    await page.goto('./');
    await expect(page.locator('.app')).toBeVisible({ timeout: 15_000 });
    await switchToManualMode(page);

    const input = page.locator('[data-testid="chat-composer-input"]');
    const messages = page.locator('[data-testid="chat-message"]');

    for (let i = 0; i < 12; i += 1) {
      const baseline = await messages.count();
      // Keep this as a multi-token unknown prompt so issue #500's bare-term
      // web-search fallback does not turn the report-link fixture into search.
      await input.fill('ва ва');
      await page.locator('[data-testid="chat-composer-submit"]').click();
      await expect(messages).toHaveCount(baseline + 2, { timeout: 15_000 });
    }

    const reportLink = messages.last().locator('.message-actions a');
    await expect(reportLink).toHaveText(/Report missing rule|Report issue/);

    const href = await reportLink.getAttribute('href');
    expect(href).toBeTruthy();
    expect(href.length).toBeLessThanOrEqual(8192);

    const url = new URL(href || '');
    const body = url.searchParams.get('body') || '';
    expect(body).toContain('## Environment');
    expect(body).toContain('## Reproduction of dialog');
    // The fitter should keep the final exchange and as much earlier dialog as
    // will fit. When the boundary turn is too large, it should be represented
    // by a truncation marker rather than dropping everything before the last
    // two messages.
    expect(body).toMatch(/omitted \d+ (earlier (message|messages)|lines|characters)/);
    const dialogTurnCount = (body.match(/^(U|A)( \([^)]*\))?: /gm) || []).length;
    expect(dialogTurnCount).toBeGreaterThanOrEqual(3);
  });

  test('toolbar labels switch to icon-only before controls wrap', async ({ page }) => {
    await disableGreetingVariations(page);
    await page.setViewportSize({ width: 980, height: 760 });
    await page.goto('./');
    await expect(page.locator('.app')).toBeVisible({ timeout: 15_000 });

    const labelDisplays = await page
      .locator('.topbar-actions .btn-label')
      .evaluateAll((nodes) => nodes.map((node) => window.getComputedStyle(node).display));
    expect(labelDisplays.length).toBeGreaterThan(0);
    expect(labelDisplays.every((display) => display === 'none')).toBe(true);

    const actionBoxes = await page.locator('.topbar-actions > *').evaluateAll((nodes) =>
      nodes.map((node) => {
        const rect = node.getBoundingClientRect();
        return { top: Math.round(rect.top), height: Math.round(rect.height) };
      }),
    );
    const rows = new Set(actionBoxes.filter((box) => box.height > 0).map((box) => box.top));
    expect(rows.size).toBe(1);
  });
});

test.describe('Issue #108: mobile composer and configurable input UI', () => {
  test('mobile uses a left menu, hides the wordmark, and exposes brand details inside the drawer', async ({ page }) => {
    await disableGreetingVariations(page);
    await page.setViewportSize({ width: 390, height: 780 });
    await page.goto('./');
    await expect(page.locator('.app')).toBeVisible({ timeout: 15_000 });

    const menu = page.locator('[data-testid="mobile-menu-toggle"]');
    await expect(menu).toBeVisible();
    const menuBox = await menu.boundingBox();
    expect(menuBox).toBeTruthy();
    expect(menuBox && menuBox.x).toBeLessThan(20);

    await expect(page.locator('.topbar .brand')).toBeHidden();

    await menu.click();
    const drawerBrand = page.locator('[data-testid="drawer-brand"]');
    await expect(drawerBrand).toBeVisible();
    await expect(drawerBrand).toContainText('formal-ai');
    await expect(drawerBrand).toContainText(/v(dev|\d+\.\d+\.\d+)/);
  });

  test('focused mobile composer embeds its controls inside the field and keeps the top menu reachable', async ({
    page,
  }) => {
    await disableGreetingVariations(page);
    await page.setViewportSize({ width: 390, height: 780 });
    await page.goto('./');
    await expect(page.locator('.app')).toBeVisible({ timeout: 15_000 });

    await page.locator('.mode-toggle').click();
    await expect(page.locator('[data-testid="chat-composer-input"]')).toBeEnabled({
      timeout: 5_000,
    });

    const input = page.locator('[data-testid="chat-composer-input"]');
    await input.focus();

    const topbarBox = await page.locator('.topbar').boundingBox();
    const menuBox = await page.locator('[data-testid="mobile-menu-toggle"]').boundingBox();
    const gridBox = await page.locator('.composer-grid').boundingBox();
    const actionBox = await page.locator('[data-testid="composer-menu-toggle"]').boundingBox();
    const inputBox = await input.boundingBox();
    const sendBox = await page.locator('[data-testid="chat-composer-submit"]').boundingBox();
    const viewport = page.viewportSize();

    expect(topbarBox).toBeTruthy();
    expect(menuBox).toBeTruthy();
    expect(gridBox).toBeTruthy();
    expect(actionBox).toBeTruthy();
    expect(inputBox).toBeTruthy();
    expect(sendBox).toBeTruthy();
    expect(viewport).toBeTruthy();

    expect(topbarBox && topbarBox.y).toBeGreaterThanOrEqual(0);
    expect(menuBox && menuBox.y).toBeGreaterThanOrEqual(0);

    // Issue #557 / #108: the input, action, and send controls all live inside the
    // single composer field surface (buttons embedded into the text field) rather
    // than in separate bordered boxes. Assert every control is contained within
    // the `.composer-grid` bounds instead of a shared baseline.
    assertContained(inputBox, gridBox);
    assertContained(actionBox, gridBox);
    assertContained(sendBox, gridBox);

    // The controls sit in the bottom toolbar, below the text, with the action on
    // the left and the send on the right of that same row.
    expect(actionBox && inputBox && actionBox.y).toBeGreaterThanOrEqual(inputBox.y - 1);
    expect(sendBox && actionBox && Math.abs(sendBox.y - actionBox.y)).toBeLessThan(4);
    expect(sendBox && actionBox && sendBox.x).toBeGreaterThan(actionBox.x);
    expect(inputBox && viewport && inputBox.width).toBeGreaterThan(viewport.width * 0.55);
  });

  test('UI skin, chat style, composer style, and action button are configurable and persisted', async ({ page }) => {
    await disableGreetingVariations(page);
    await page.goto('./');
    await expect(page.locator('.app')).toBeVisible({ timeout: 15_000 });

    await page.locator('[data-testid="setting-ui-skin"]').selectOption('glass');
    await page.locator('[data-testid="setting-chat-style"]').selectOption('bubbles');
    await page.locator('[data-testid="setting-composer-style"]').selectOption('glass-clear');
    await page.locator('[data-testid="setting-composer-action"]').selectOption('plus');

    await expect(page.locator('.app')).toHaveClass(/ui-skin-glass/);
    await expect(page.locator('.app')).toHaveClass(/chat-style-bubbles/);
    await expect(page.locator('.app')).toHaveClass(/composer-style-glass-clear/);
    await expect(page.locator('[data-testid="composer-menu-toggle"]')).toContainText('+');

    const stored = await page.evaluate(
      () => window.localStorage.getItem('formal-ai.preferences.v1') || '',
    );
    expect(stored).toContain('uiSkin "glass"');
    expect(stored).toContain('chatStyle "bubbles"');
    expect(stored).toContain('composerStyle "glass-clear"');
    expect(stored).toContain('composerAction "plus"');
  });

  test('composer menu exposes attachment and memory actions at the input', async ({ page }) => {
    await disableGreetingVariations(page);
    await page.goto('./');
    await expect(page.locator('.app')).toBeVisible({ timeout: 15_000 });

    await page.locator('[data-testid="composer-menu-toggle"]').click();
    const menu = page.locator('[data-testid="composer-menu"]');
    await expect(menu).toBeVisible();
    await expect(menu).toContainText('Attach files');
    await expect(menu).toContainText('Export memory');
    await expect(menu).toContainText('Import memory');
    await expect(menu).toContainText('Report issue');
  });
});

test.describe('Issue #557: adaptive composer skins', () => {
  test('desktop composer embeds its controls inside a single field surface', async ({ page }) => {
    await disableGreetingVariations(page);
    await page.setViewportSize({ width: 1280, height: 860 });
    await page.goto('./');
    await expect(page.locator('.app')).toBeVisible({ timeout: 15_000 });

    await page.locator('.mode-toggle').click();
    const input = page.locator('[data-testid="chat-composer-input"]');
    await expect(input).toBeEnabled({ timeout: 5_000 });
    await input.focus();

    const gridBox = await page.locator('.composer-grid').boundingBox();
    const inputBox = await input.boundingBox();
    const actionBox = await page.locator('[data-testid="composer-menu-toggle"]').boundingBox();
    const sendBox = await page.locator('[data-testid="chat-composer-submit"]').boundingBox();

    // Buttons embedded into the text field: input, action, and send controls are
    // all contained within the one rounded composer surface.
    assertContained(inputBox, gridBox);
    assertContained(actionBox, gridBox);
    assertContained(sendBox, gridBox);

    // The embedded toolbar sits below the text with the send anchored to the right.
    expect(actionBox && inputBox && actionBox.y).toBeGreaterThanOrEqual(inputBox.y - 1);
    expect(sendBox && actionBox && sendBox.x).toBeGreaterThan(actionBox.x);

    // The send control is rendered as a compact embedded button, not a wide bar.
    expect(sendBox && sendBox.width).toBeLessThan(80);
  });

  test('settings expose Material skin and a glass transparency slider', async ({ page }) => {
    await disableGreetingVariations(page);
    await page.goto('./');
    await expect(page.locator('.app')).toBeVisible({ timeout: 15_000 });

    const skin = page.locator('[data-testid="setting-ui-skin"]');
    await skin.selectOption('material');
    await expect(page.locator('.app')).toHaveClass(/ui-skin-material/);

    await expect(page.locator('[data-testid="setting-glass-opacity"]')).toHaveCount(0);

    await skin.selectOption('glass');
    const glassOpacity = page.locator('[data-testid="setting-glass-opacity"]');
    await expect(glassOpacity).toBeVisible();
    await setRangeValue(page, 'setting-glass-opacity', 0.45);

    const glassAlpha = await page.locator('.app').evaluate((node) =>
      getComputedStyle(node).getPropertyValue('--fa-glass-alpha').trim(),
    );
    expect(glassAlpha).toBe('0.45');

    const stored = await page.evaluate(
      () => window.localStorage.getItem('formal-ai.preferences.v1') || '',
    );
    expect(stored).toContain('uiSkin "glass"');
    expect(stored).toContain('glassOpacity "0.45"');
  });

  // Issue #557: the adaptive skin picker (Flat / Glass / Material / Contrast)
  // and the glass transparency slider are language-facing UI, so every supported
  // locale must render the translated skin label, options, and slider caption.
  // Covering all languages here prevents a single-language regression from
  // shipping a skin control that is only translated for English.
  const skinLocaleCases = [
    { language: 'en', skinLabel: 'UI skin', glass: 'Glass', opacity: 'Glass opacity' },
    { language: 'ru', skinLabel: 'Скин UI', glass: 'Стекло', opacity: 'Прозрачность стекла' },
    { language: 'hi', skinLabel: 'UI स्किन', glass: 'ग्लास', opacity: 'ग्लास अपारदर्शिता' },
    { language: 'zh', skinLabel: '界面皮肤', glass: '玻璃', opacity: '玻璃不透明度' },
  ];

  for (const { language, skinLabel, glass, opacity } of skinLocaleCases) {
    test(`skin picker and glass slider are translated for ${language}`, async ({ page }) => {
      // Seed the full preference snapshot (glass skin + UI language) before the
      // first load. A single init script avoids the greeting-off helper, which
      // re-writes preferences on every navigation and would otherwise clobber
      // the selected uiLanguage on reload.
      await page.addInitScript((lang) => {
        try {
          window.localStorage.setItem(
            'formal-ai.preferences.v1',
            `demo_preferences\n  demoMode "off"\n  greetingVariations "off"\n  uiSkin "glass"\n  uiLanguage "${lang}"`,
          );
        } catch (_error) {
          // localStorage can be unavailable in hardened browser contexts.
        }
      }, language);
      await page.goto('./');
      await expect(page.locator('.app')).toBeVisible({ timeout: 15_000 });
      await expect(page.locator('html')).toHaveAttribute('lang', language);

      const panel = page.locator('.settings-panel');
      // The Material option keeps its brand name across locales, while Flat,
      // Glass, and Contrast are localized; assert the localized Glass label plus
      // the always-present Material brand render in the picker.
      const skin = page.locator('[data-testid="setting-ui-skin"]');
      await expect(skin.locator('option', { hasText: glass })).toHaveCount(1);
      await expect(skin.locator('option', { hasText: 'Material' })).toHaveCount(1);
      await expect(panel).toContainText(skinLabel);
      // With the glass skin active the transparency slider caption is visible.
      await expect(page.locator('[data-testid="setting-glass-opacity"]')).toBeVisible();
      await expect(panel).toContainText(opacity);
    });
  }
});

test.describe('Issue #136: desktop sidebar sizing', () => {
  test.beforeEach(async ({ page }) => {
    await disableGreetingVariations(page);
    await page.setViewportSize({ width: 1536, height: 730 });
    await page.goto('./');
    await expect(page.locator('.app')).toBeVisible({ timeout: 15_000 });
  });

  test('tool cards fit inside the sidebar without horizontal overflow', async ({ page }) => {
    const settings = page.locator('[data-testid="sidebar-settings"]');
    if ((await settings.getAttribute('data-collapsed')) === 'true') {
      await settings.locator('.sidebar-section-header').click();
    }
    await page.locator('[data-testid="setting-ui-language"]').selectOption('ru');
    const tools = page.locator('[data-testid="sidebar-tools"]');
    await expect(tools).toBeVisible({ timeout: 10_000 });
    if ((await tools.getAttribute('data-collapsed')) === 'true') {
      await tools.locator('.sidebar-section-header').click();
    }
    const registry = page.locator('[data-testid="tool-registry"]');
    await expect(registry).toBeVisible({ timeout: 10_000 });

    const metrics = await page.evaluate(() => {
      const body = document.querySelector(
        '[data-testid="sidebar-tools"] .sidebar-section-body',
      );
      const nodes = Array.from(
        document.querySelectorAll(
          [
            '[data-testid="sidebar-tools"] [data-testid="tool-entry"]',
            '[data-testid="sidebar-tools"] .tool-head',
            '[data-testid="sidebar-tools"] .tool-head strong',
            '[data-testid="sidebar-tools"] .tool-mode',
            '[data-testid="sidebar-tools"] .tool-desc',
          ].join(','),
        ),
      );
      if (!body) {
        return null;
      }
      const bodyRect = body.getBoundingClientRect();
      const overflows = nodes.map((node) => {
        const rect = node.getBoundingClientRect();
        return Math.max(0, rect.right - bodyRect.right, bodyRect.left - rect.left);
      });
      return {
        bodyClientWidth: body.clientWidth,
        bodyScrollWidth: body.scrollWidth,
        maxChildOverflow: Math.ceil(Math.max(0, ...overflows)),
        overflowX: getComputedStyle(body).overflowX,
      };
    });

    expect(metrics).toBeTruthy();
    expect(metrics.bodyScrollWidth).toBeLessThanOrEqual(metrics.bodyClientWidth + 1);
    expect(metrics.maxChildOverflow).toBeLessThanOrEqual(1);
    expect(metrics.overflowX).toBe('hidden');
  });

  test('desktop sidebar can be resized with the separator', async ({ page }) => {
    const panel = page.locator('[data-testid="context-panel"]');
    const resizer = page.locator('[data-testid="context-resizer"]');
    await expect(resizer).toBeVisible();

    const before = await panel.boundingBox();
    const handle = await resizer.boundingBox();
    expect(before).toBeTruthy();
    expect(handle).toBeTruthy();

    await page.mouse.move(handle.x + handle.width / 2, handle.y + handle.height / 2);
    await page.mouse.down();
    await page.mouse.move(handle.x + handle.width / 2 + 120, handle.y + handle.height / 2);
    await page.mouse.up();

    const after = await panel.boundingBox();
    expect(after).toBeTruthy();
    expect(after.width).toBeGreaterThan(before.width + 90);

    const stored = await page.evaluate(
      () => window.localStorage.getItem('formal-ai.preferences.v1') || '',
    );
    expect(stored).toMatch(/contextPanelWidth "\d+"/);
  });
});

test.describe('Issue #110: mobile keyboard viewport handling', () => {
  test('focused mobile input pins the app shell to the visual viewport offset', async ({ page }) => {
    await disableGreetingVariations(page);
    await page.addInitScript(() => {
      const listeners = new Map();
      const viewport = {
        width: 390,
        height: 780,
        offsetTop: 0,
        offsetLeft: 0,
        pageTop: 0,
        pageLeft: 0,
        scale: 1,
        addEventListener(type, listener) {
          const current = listeners.get(type) || [];
          current.push(listener);
          listeners.set(type, current);
        },
        removeEventListener(type, listener) {
          const current = listeners.get(type) || [];
          listeners.set(
            type,
            current.filter((entry) => entry !== listener),
          );
        },
        dispatchEvent(event) {
          for (const listener of listeners.get(event.type) || []) {
            listener.call(viewport, event);
          }
          return true;
        },
        __set(next) {
          Object.assign(viewport, next);
          viewport.dispatchEvent(new Event('resize'));
          viewport.dispatchEvent(new Event('scroll'));
        },
      };
      Object.defineProperty(window, 'visualViewport', {
        configurable: true,
        value: viewport,
      });
    });
    await page.setViewportSize({ width: 390, height: 780 });
    await page.goto('./');
    await expect(page.locator('.app')).toBeVisible({ timeout: 15_000 });

    await page.locator('.mode-toggle').click();
    const input = page.locator('[data-testid="chat-composer-input"]');
    await expect(input).toBeEnabled({ timeout: 5_000 });
    await input.focus();

    const viewportState = await page.evaluate(() => {
      window.visualViewport.__set({
        height: 520,
        offsetTop: 180,
      });
      return {
        height: window.visualViewport.height,
        offsetTop: window.visualViewport.offsetTop,
      };
    });

    const topbarBox = await page.locator('.topbar').boundingBox();
    const composerBox = await page.locator('.composer').boundingBox();

    expect(topbarBox).toBeTruthy();
    expect(composerBox).toBeTruthy();
    expect(Math.round(topbarBox.y)).toBe(viewportState.offsetTop);
    expect(
      Math.round(composerBox.y + composerBox.height),
    ).toBeLessThanOrEqual(viewportState.offsetTop + viewportState.height + 1);
  });
});
