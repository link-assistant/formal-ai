// @ts-check
// Shared constants and page helpers for the multilingual-*.spec.js suites
// (split from the former multilingual.spec.js; this file is not itself a spec).
const { expect } = require('@playwright/test');

const UNKNOWN_ANSWER_MARKER = 'cannot answer that from local links rules';
const TEN_POW_100 =
  '10000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000';

const definitionDisambiguationCases = [
  {
    language: 'en',
    prompt: 'What is creature?',
    term: 'creature',
    title: 'Creature',
    wikipediaHost: 'en.wikipedia.org',
    sourceUrl: 'https://en.wikipedia.org/wiki/Creature',
    entries: [
      'Creature — a living being or organism.',
      'Creature — a fictional or legendary being.',
    ],
    rejectedWikidata: {
      id: 'Q729',
      label: 'Animalia',
      description: 'kingdom of multicellular eukaryotic organisms',
      alias: 'creature',
    },
  },
  {
    language: 'ru',
    prompt: 'Что такое существо?',
    term: 'существо',
    title: 'Существо',
    wikipediaHost: 'ru.wikipedia.org',
    sourceUrl: 'https://ru.wikipedia.org/wiki/Существо',
    entries: [
      'Существо — живой организм, живая особь, животное, человек.',
      'Существо — главное, существенное в ком-либо, чем-либо, его суть; сущность.',
      '«Существо» — музыкальный альбом Дельфина (2011).',
      '«Существо» — фильм ужасов (США, 1982).',
    ],
    rejectedWikidata: {
      id: 'Q729',
      label: 'Animalia',
      description: 'kingdom of multicellular eukaryotic organisms',
      alias: 'существо',
    },
  },
  {
    language: 'hi',
    prompt: 'प्राणी क्या है?',
    term: 'प्राणी',
    title: 'प्राणी',
    wikipediaHost: 'hi.wikipedia.org',
    sourceUrl: 'https://hi.wikipedia.org/wiki/प्राणी',
    entries: [
      'प्राणी — जीवित जीव या व्यक्ति।',
      'प्राणी — कथा या लोककथा का कल्पित जीव।',
    ],
    rejectedWikidata: {
      id: 'Q729',
      label: 'Animalia',
      description: 'बहुकोशिकीय यूकैरियोटिक जीवों का जगत',
      alias: 'प्राणी',
    },
  },
  {
    language: 'zh',
    prompt: '生物是什么?',
    term: '生物',
    title: '生物',
    wikipediaHost: 'zh.wikipedia.org',
    sourceUrl: 'https://zh.wikipedia.org/wiki/生物',
    entries: [
      '生物 — 有生命的个体或有机体。',
      '生物 — 小说或传说中的生命体。',
    ],
    rejectedWikidata: {
      id: 'Q729',
      label: 'Animalia',
      description: '多细胞真核生物界',
      alias: '生物',
    },
  },
];

function escapeHtml(value) {
  const replacements = {
    '&': '&amp;',
    '<': '&lt;',
    '>': '&gt;',
    '"': '&quot;',
    "'": '&#39;',
  };
  return String(value).replace(/[&<>"']/g, (char) => replacements[char]);
}

async function routeDefinitionDisambiguationCase(page, testCase) {
  await page.route('**/api/rest_v1/page/summary/**', async (route) => {
    const url = new URL(route.request().url());
    const slug = decodeURIComponent(url.pathname.split('/').pop() || '');
    if (url.hostname === testCase.wikipediaHost && slug === testCase.title) {
      await route.fulfill({
        status: 200,
        contentType: 'application/json',
        body: JSON.stringify({
          title: testCase.title,
          type: 'disambiguation',
          extract: `${testCase.title}:\n${testCase.entries.join('\n')}`,
          content_urls: {
            desktop: { page: testCase.sourceUrl },
          },
        }),
      });
      return;
    }
    await route.fulfill({
      status: 404,
      contentType: 'application/json',
      body: JSON.stringify({ httpCode: 404, httpReason: 'Not Found' }),
    });
  });

  await page.route(`**://${testCase.wikipediaHost}/w/api.php**`, async (route) => {
    const items = testCase.entries
      .map((entry) => `<li>${escapeHtml(entry)}</li>`)
      .join('');
    await route.fulfill({
      status: 200,
      contentType: 'application/json',
      body: JSON.stringify({
        parse: {
          title: testCase.title,
          pageid: 133629,
          text: `<p><b>${escapeHtml(testCase.title)}</b>:</p><ul>${items}</ul>`,
        },
      }),
    });
  });

  await page.route('**/rest.php/v1/search/page**', async (route) => {
    await route.fulfill({
      status: 200,
      contentType: 'application/json',
      body: JSON.stringify({ pages: [] }),
    });
  });

  await page.route('**://*.wikidata.org/w/api.php**', async (route) => {
    await route.fulfill({
      status: 200,
      contentType: 'application/json',
      body: JSON.stringify({
        search: [
          {
            id: testCase.rejectedWikidata.id,
            label: testCase.rejectedWikidata.label,
            description: testCase.rejectedWikidata.description,
            concepturi: `https://www.wikidata.org/wiki/${testCase.rejectedWikidata.id}`,
            match: {
              type: 'alias',
              language: testCase.language,
              text: testCase.rejectedWikidata.alias,
            },
            aliases: [testCase.rejectedWikidata.alias],
          },
        ],
      }),
    });
  });
}

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
  const tools = page.locator('[data-testid="sidebar-tools"]');
  await expect(tools).toBeVisible({ timeout: 10_000 });
  if ((await tools.getAttribute('data-collapsed')) === 'true') {
    await tools.locator('.sidebar-section-header').click();
  }
  await expect(page.locator('[data-testid="tool-entry"]').first()).toBeVisible({
    timeout: 10_000,
  });
}

async function setUiLanguage(page, language) {
  await page.evaluate((nextLanguage) => {
    window.localStorage.setItem(
      'formal-ai.preferences.v1',
      `demo_preferences\n  demoMode "off"\n  greetingVariations "off"\n  uiLanguage "${nextLanguage}"`,
    );
  }, language);
  await page.reload();
  await expect(page.locator('.app')).toBeVisible({ timeout: 15_000 });
  await expect(page.locator('html')).toHaveAttribute('lang', language);
  await expect(page.locator('[data-testid="chat-composer-input"]')).toBeEnabled({
    timeout: 5_000,
  });
}

// Issue #27: greeting randomisation defaults to ON. Tests below pin the
// canonical greeting text, so disable randomisation up-front for stability.
// The script merges into any existing preference snapshot so reload-survival
// tests still see persisted state (e.g. the active conversation id) after the
// init script re-runs.
async function disableGreetingVariations(page) {
  await page.addInitScript(() => {
    try {
      const KEY = 'formal-ai.preferences.v1';
      const existing = window.localStorage.getItem(KEY) || '';
      if (/greetingVariations\s+"/.test(existing)) {
        // Replace whatever value is set with "off"
        const next = existing.replace(
          /greetingVariations\s+"[^"]*"/,
          'greetingVariations "off"',
        );
        window.localStorage.setItem(KEY, next);
      } else if (existing.startsWith('demo_preferences')) {
        window.localStorage.setItem(
          KEY,
          `${existing}\n  greetingVariations "off"`,
        );
      } else {
        window.localStorage.setItem(
          KEY,
          'demo_preferences\n  greetingVariations "off"',
        );
      }
    } catch (_error) {
      // localStorage may be unavailable; tests will tolerate variant text.
    }
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

async function routeFramePolicy(page, headers) {
  const requests = [];
  await page.route('**://api.microlink.io/**', async (route) => {
    requests.push(route.request().url());
    await route.fulfill({
      status: 200,
      contentType: 'application/json',
      headers: { 'access-control-allow-origin': '*' },
      body: JSON.stringify({
        status: 'success',
        statusCode: 200,
        headers,
      }),
    });
  });
  return requests;
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

// Issue #541 (R5/R6): freshly produced assistant answers stage a reasoning-
// then-body reveal that hides the answer body via `.markdown-body.is-revealing
// { display: none }` for the configured budget (default 2 s). Tests that read
// `last.innerText()` immediately after sendPrompt would race the reveal and
// either flake or see only the thinking-preview text — the body div would be
// invisible to innerText while it carries `.is-revealing`. Emulating
// prefers-reduced-motion makes `usePrefersReducedMotion()` return true, which
// short-circuits `useMessageReveal` to "show everything at once" — the same
// behavior users with the reduced-motion preference see, and the same trick
// already in use in issue-347.spec.js for screenshot captures. The matching
// config-level `reducedMotion: 'reduce'` in playwright.local.config.js does not
// reliably propagate through the test fixture in the local Playwright runner;
// emulating per-page here is the belt-and-braces fix.
function emulateReducedMotion(test) {
  test.beforeEach(async ({ page }) => {
    await page.emulateMedia({ reducedMotion: 'reduce' });
  });
}

module.exports = {
  UNKNOWN_ANSWER_MARKER,
  TEN_POW_100,
  definitionDisambiguationCases,
  escapeHtml,
  routeDefinitionDisambiguationCase,
  switchToManualMode,
  setUiLanguage,
  disableGreetingVariations,
  sendPrompt,
  routeFramePolicy,
  setRangeValue,
  emulateReducedMotion,
};
