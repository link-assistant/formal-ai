// @ts-check
// Issue #435: the Russian prompt "Можешь поставить мне созвон в кальндарь на
// завтра?" carries no day number and no clock time — only a relative-date word
// ("на завтра") and an event noun ("созвон"). It used to fall through to the
// unknown intent. The browser worker must now recognize the relative date,
// resolve "завтра" to tomorrow, and draft an importable calendar event titled
// from the event noun instead of pretending it cannot help.
const { test, expect } = require('@playwright/test');

async function disableGreetingVariations(page) {
  await page.addInitScript(() => {
    try {
      window.localStorage.setItem(
        'formal-ai.preferences.v1',
        'demo_preferences\n  greetingVariations "off"',
      );
    } catch (_error) {
      // localStorage may be unavailable in some sandboxes.
    }
  });
}

async function switchToManualMode(page) {
  const demoToggle = page.locator('.mode-toggle');
  await expect(demoToggle).toContainText(/Demo on|Demo off|Демо/, {
    timeout: 10_000,
  });
  await demoToggle.click();
  await expect(page.locator('[data-testid="demo-status"]')).toHaveText(
    'Manual mode',
  );
  await expect(page.locator('[data-testid="chat-composer-input"]')).toBeEnabled({
    timeout: 5_000,
  });
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

test.describe('Issue #435 - relative-date calendar request', () => {
  test.beforeEach(async ({ page }) => {
    await disableGreetingVariations(page);
    await page.goto('./');
    await expect(page.locator('.app')).toBeVisible({ timeout: 15_000 });
    await expect(page.locator('[data-testid="demo-status"]')).toBeVisible();
    await switchToManualMode(page);
  });

  test('reported "на завтра" prompt drafts a tomorrow event instead of unknown', async ({
    page,
  }) => {
    const answer = await sendPrompt(
      page,
      'Можешь поставить мне созвон в кальндарь на завтра?',
    );
    const body = answer.locator('.markdown-body');

    await expect(body).not.toContainText('Я тебя не понял');
    await expect(body).not.toContainText('Я пока не знаю');
    // Title is derived from the event noun «созвон».
    await expect(body).toContainText('Созвон');
    await expect(body).toContainText('.ics');
    await expect(body).toContainText('BEGIN:VEVENT');
    await expect(body).toContainText('calendar.google.com/calendar/render');
    await expect(body).toContainText('Ответьте «да», чтобы подтвердить');

    // The resolved date must be tomorrow (today UTC + 1 day), formatted YYYY-MM-DD.
    const now = new Date();
    const tomorrow = new Date(
      Date.UTC(now.getUTCFullYear(), now.getUTCMonth(), now.getUTCDate() + 1),
    );
    const iso = `${tomorrow.getUTCFullYear()}-${String(
      tomorrow.getUTCMonth() + 1,
    ).padStart(2, '0')}-${String(tomorrow.getUTCDate()).padStart(2, '0')}`;
    await expect(body).toContainText(iso);
  });
});
