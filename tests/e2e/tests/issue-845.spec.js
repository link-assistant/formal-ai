// @ts-check
const { test, expect } = require('@playwright/test');

const factCheckQueries = [
  { language: 'en', query: 'fact-check this dialogue' },
  { language: 'ru', query: 'проверь факты в диалоге' },
  { language: 'hi', query: 'इस संवाद के तथ्यों की जाँच करें' },
  { language: 'zh', query: '核查此对话中的事实' },
];

async function switchToManualMode(page) {
  const demoToggle = page.locator('.mode-toggle');
  await expect(demoToggle).toBeVisible({ timeout: 10_000 });
  if (
    (await page.locator('[data-testid="demo-status"]').textContent()) !==
    'Manual mode'
  ) {
    await demoToggle.click();
  }
  await expect(page.locator('[data-testid="demo-status"]')).toHaveText(
    'Manual mode',
  );
}

async function sendPrompt(page, text) {
  const input = page.locator('[data-testid="chat-composer-input"]');
  await expect(input).toBeEnabled({ timeout: 5_000 });
  await input.fill(text);
  const messages = page.locator('[data-testid="chat-message"]');
  const initialCount = await messages.count();
  await page.locator('[data-testid="chat-composer-submit"]').click();
  await expect(messages).toHaveCount(initialCount + 2, { timeout: 20_000 });
  return messages.last().locator('.markdown-body');
}

for (const { language, query } of factCheckQueries) {
  test(`current-dialogue fact checking is live in the ${language} browser worker`, async ({
    page,
  }) => {
    const externalRequests = [];
    page.on('request', (request) => {
      const url = new URL(request.url());
      if (
        (url.protocol === 'http:' || url.protocol === 'https:') &&
        url.hostname !== 'localhost' &&
        url.hostname !== '127.0.0.1'
      ) {
        externalRequests.push(request.url());
      }
    });

    await page.goto('./');
    await expect(page.locator('.app')).toBeVisible({ timeout: 15_000 });
    await switchToManualMode(page);

    await sendPrompt(page, '1 + 1 = 2');
    await sendPrompt(page, '1 + 1 = 3');
    const reply = await sendPrompt(page, query);

    await expect(reply).toContainText('1 + 1 = 2');
    await expect(reply).toContainText('1 + 1 = 3');
    await expect(reply).toContainText('1.000000');
    await expect(reply).toContainText('0.000000');
    expect(
      externalRequests,
      'a current-dialogue audit must not fetch external evidence',
    ).toEqual([]);
  });
}
