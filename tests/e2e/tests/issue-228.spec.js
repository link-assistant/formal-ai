// @ts-check
const { test, expect } = require('@playwright/test');

const UNKNOWN_ANSWER_MARKER = 'cannot answer that from local Links Notation rules';

async function disableGreetingVariations(page) {
  await page.addInitScript(() => {
    try {
      window.localStorage.setItem(
        'formal-ai.preferences.v1',
        'demo_preferences\n  greetingVariations "off"',
      );
    } catch (_error) {
      // ignore
    }
  });
}

async function switchToManualMode(page) {
  const demoToggle = page.locator('.mode-toggle');
  await expect(demoToggle).toContainText(/Demo on|Demo off|Демо/, {
    timeout: 10_000,
  });
  await demoToggle.click();
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

async function mockGenshinSearchProviders(page) {
  await page.route('**://api.duckduckgo.com/**', async (route) => {
    await route.fulfill({
      status: 200,
      contentType: 'application/json',
      body: JSON.stringify({
        Heading: 'Genshin off-field damage characters',
        AbstractText:
          'Furina, Yelan, Nahida, Fischl, Xingqiu, Xiangling, Yae Miko, Albedo, and Kaeya are commonly described as off-field damage or sub-DPS options.',
        AbstractURL: 'https://genshin-impact.fandom.com/wiki/Category:Off-Field_Characters',
        RelatedTopics: [
          {
            FirstURL: 'https://genshinteambuilds.gitbook.io/teams/roles/off-field-dps',
            Text:
              'Off-field DPS - Characters who deal damage or apply elements while inactive.',
          },
        ],
      }),
    });
  });

  await page.route('**://archive.org/advancedsearch.php**', async (route) => {
    await route.fulfill({
      status: 200,
      contentType: 'application/json',
      body: JSON.stringify({
        response: {
          docs: [
            {
              identifier: 'genshin-off-field-dps',
              title: 'Genshin Impact off-field DPS references',
              description:
                'Archived pages discussing Genshin Impact characters with off-field damage abilities.',
            },
          ],
        },
      }),
    });
  });

  await page.route('**://*.wikipedia.org/w/rest.php/v1/search/page**', async (route) => {
    await route.fulfill({
      status: 200,
      contentType: 'application/json',
      body: JSON.stringify({
        pages: [
          {
            id: 1,
            key: 'Genshin_Impact',
            title: 'Genshin Impact',
            excerpt: 'Genshin Impact is an action role-playing game with playable characters.',
            description: 'action role-playing game',
          },
        ],
      }),
    });
  });

  await page.route('**://www.wikidata.org/w/api.php**', async (route) => {
    await route.fulfill({
      status: 200,
      contentType: 'application/json',
      body: JSON.stringify({
        search: [
          {
            id: 'Q85852365',
            label: 'Genshin Impact',
            description: 'action role-playing game',
            concepturi: 'https://www.wikidata.org/wiki/Q85852365',
          },
        ],
      }),
    });
  });

  await page.route('**://*.wiktionary.org/w/api.php**', async (route) => {
    await route.fulfill({
      status: 200,
      contentType: 'application/json',
      body: JSON.stringify([
        'damage',
        ['damage'],
        ['Injury or harm; loss of value or usefulness.'],
        ['https://en.wiktionary.org/wiki/damage'],
      ]),
    });
  });
}

test.describe('Issue #228 — enumeration research requests use web search', () => {
  test.beforeEach(async ({ page }) => {
    await disableGreetingVariations(page);
    await mockGenshinSearchProviders(page);
    await page.goto('./');
    await expect(page.locator('.app')).toBeVisible({ timeout: 15_000 });
    await switchToManualMode(page);
  });

  test('list-all Genshin off-field damage prompt routes to web_search', async ({
    page,
  }) => {
    await page.locator('.diagnostics-toggle').click();

    const last = await sendPrompt(
      page,
      'list all genshin characters with off-field DMG',
    );

    await expect(last).toContainText('Search results for');
    await expect(last).toContainText('Furina');
    await expect(last).not.toContainText(UNKNOWN_ANSWER_MARKER);
    await expect(last.locator('.intent')).toContainText('intent:web_search');
    await expect(last.locator('.evidence-list')).toContainText(
      'web_search:request:genshin characters with off-field DMG',
    );
    await expect(last.locator('.evidence-list')).toContainText(
      'web_search:query_kind:enumeration_research_request',
    );
  });
});
