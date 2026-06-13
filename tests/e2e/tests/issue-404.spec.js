// @ts-check
// Issue #404: the Russian calendar-event request fell through to unknown in the
// browser worker. It must now draft a calendar event and expose safe execution
// paths instead of pretending it can write the user's calendar without consent.
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

test.describe('Issue #404 - calendar event request', () => {
  test.beforeEach(async ({ page }) => {
    await disableGreetingVariations(page);
    await page.goto('./');
    await expect(page.locator('.app')).toBeVisible({ timeout: 15_000 });
    await expect(page.locator('[data-testid="demo-status"]')).toBeVisible();
    await switchToManualMode(page);
  });

  test('reported Russian prompt drafts a Georgia-time event instead of unknown', async ({
    page,
  }) => {
    const answer = await sendPrompt(
      page,
      'Забей мне 18 число в 17:00 по грузии на встречу с Леваном',
    );
    const body = answer.locator('.markdown-body');

    await expect(body).not.toContainText('Я тебя не понял');
    await expect(body).not.toContainText('Я пока не знаю');
    // The subject is extracted from "на встречу с Леваном" and capitalized for
    // the .ics SUMMARY ("Встречу с леваном").
    await expect(body).toContainText('Встречу с леваном');
    await expect(body).toContainText('17:00');
    await expect(body).toContainText('Asia/Tbilisi');
    await expect(body).toContainText('.ics');
    await expect(body).toContainText('BEGIN:VEVENT');
    await expect(body).toContainText('calendar.google.com/calendar/render');
    // The draft asks for confirmation instead of silently writing the calendar.
    await expect(body).toContainText('Ответьте «да», чтобы подтвердить');
  });
});
