// @ts-check
const { test, expect } = require('@playwright/test');

const UNKNOWN_ANSWER_MARKER = 'cannot answer that from local links rules';

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
  await expect(page.locator('[data-testid="tool-entry"]').first()).toBeVisible({
    timeout: 10_000,
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

test.describe('multilingual chat surface', () => {
  test.beforeEach(async ({ page }) => {
    await disableGreetingVariations(page);
    await page.goto('./');
    await expect(page.locator('.app')).toBeVisible({ timeout: 15_000 });
    await switchToManualMode(page);
  });

  test('Russian greeting replies in Russian', async ({ page }) => {
    const last = await sendPrompt(page, 'Привет');
    await expect(last).toHaveClass(/assistant/);
    await expect(last).toContainText(/Здравствуйте|Привет/);
  });

  test('how-are-you small talk replies as a greeting across languages', async ({ page }) => {
    const cases = [
      { prompt: 'How are you?', answer: /Hi|Hello|Hey/ },
      { prompt: 'Как твои дела?', answer: /Здравствуйте|Привет/ },
      { prompt: 'आप कैसे हैं?', answer: /नमस्ते|नमस्कार/ },
      { prompt: '你好吗?', answer: /你好|您好/ },
    ];

    for (const { prompt, answer } of cases) {
      const last = await sendPrompt(page, prompt);
      await expect(last).toHaveClass(/assistant/);
      await expect(last).toContainText(answer);
      await expect(last).not.toContainText(UNKNOWN_ANSWER_MARKER);
    }
  });

  test('Russian combined greeting and identity question replies with identity', async ({ page }) => {
    for (const prompt of ['Привет. ты кто?', 'Привет давай знакомиться!']) {
      const last = await sendPrompt(page, prompt);
      await expect(last).toHaveClass(/assistant/);
      await expect(last).toContainText('formal-ai');
      await expect(last).toContainText(/символьный|детерминированный/);
    }
  });

  test('behavior-rule list possessive phrasing shows rules across supported languages', async ({ page }) => {
    const cases = [
      'Show rules',
      'Show list of your rules',
      'Покажи правила',
      'Покажи список своих правил',
      'नियम दिखाओ',
      'अपने नियमों की सूची दिखाओ',
      '显示规则',
      '显示你的规则列表',
    ];

    for (const prompt of cases) {
      const last = await sendPrompt(page, prompt);
      await expect(last).toHaveClass(/assistant/);
      await expect(last).toContainText('rule_greeting');
      await expect(last).toContainText('rule_unknown');
      await expect(last).not.toContainText(UNKNOWN_ANSWER_MARKER);
    }
  });

  test('reported Russian behavior-rule list is localized and markdown-safe', async ({ page }) => {
    const last = await sendPrompt(page, 'Перечисли свои правила');
    const body = last.locator('.markdown-body');
    await expect(last).toHaveClass(/assistant/);
    await expect(body).toContainText('Правила поведения, которые я могу показать');
    await expect(body).toContainText('rule_greeting');
    await expect(body).toContainText('rule_unknown');
    await expect(body).not.toContainText('Behavior rules I can inspect');
    await expect(body.locator('h1')).toHaveCount(0);

    const text = (await body.textContent()) || '';
    expect(text).not.toContain('\\`');
  });

  test('Hindi greeting replies in Hindi', async ({ page }) => {
    const last = await sendPrompt(page, 'नमस्ते');
    await expect(last).toHaveClass(/assistant/);
    await expect(last).toContainText('नमस्ते');
  });

  test('Chinese identity question replies in Chinese', async ({ page }) => {
    const last = await sendPrompt(page, '你是谁?');
    await expect(last).toHaveClass(/assistant/);
    await expect(last).toContainText('formal-ai');
    await expect(last).toContainText(/符号|确定性/);
  });

  test('single-variable equations resolve as calculations', async ({ page }) => {
    const last = await sendPrompt(page, 'x*2 = 123');
    await expect(last).toHaveClass(/assistant/);
    await expect(last).toContainText('x*2 = 123 => x = 61.5');
  });

  test('placeholder unknown equations resolve as calculations across supported languages', async ({ page }) => {
    const cases = [
      { language: 'en', prompt: '?+2=4', expected: '?+2=4 => ? = 2' },
      { language: 'ru', prompt: '?+2=4', expected: '?+2=4 => ? = 2' },
      { language: 'hi', prompt: '*+2=4', expected: '*+2=4 => * = 2' },
      { language: 'zh', prompt: '*+2=4', expected: '*+2=4 => * = 2' },
    ];

    for (const { language, prompt, expected } of cases) {
      const last = await sendPrompt(page, prompt);
      await expect(last, language).toHaveClass(/assistant/);
      await expect(last, language).toContainText(expected);
    }
  });

  test('symbolic and polynomial equations resolve as calculations', async ({ page }) => {
    const cases = [
      {
        prompt: '2 * x + 3 * y = 12',
        expected: '2 * x + 3 * y = 12 => x = 6 - 1.5*y',
      },
      { prompt: 'x + ? = 4', expected: 'x + ? = 4 => ? = 4 - x' },
      { prompt: 'x^2 = 4', expected: 'x^2 = 4 => x = -2 or x = 2' },
      {
        prompt: 'x^2 - 5 * x + 6 = 0',
        expected: 'x^2 - 5 * x + 6 = 0 => x = 2 or x = 3',
      },
      { prompt: '? * ? = 4', expected: '? * ? = 4 => ? = -2 or ? = 2' },
      { prompt: '* * * = 4', expected: '* * * = 4 => * = -2 or * = 2' },
    ];

    for (const { prompt, expected } of cases) {
      const last = await sendPrompt(page, prompt);
      await expect(last, prompt).toHaveClass(/assistant/);
      await expect(last, prompt).toContainText(expected);
    }
  });

  test('polite arithmetic action resolves as a calculation', async ({ page }) => {
    const last = await sendPrompt(page, 'Can you calculate 2 + 2?');
    await expect(last).toHaveClass(/assistant/);
    await expect(last).toContainText('2 + 2 = 4');
    await expect(last).not.toContainText('arithmetic is available');
  });

  test('misspelled calculate action resolves as a calculation with interpretation', async ({ page }) => {
    await page.locator('.diagnostics-toggle').click();

    const last = await sendPrompt(page, 'Calcualte 2+5050');
    await expect(last).toHaveClass(/assistant/);
    await expect(last).toContainText('Interpreted "Calcualte" as "calculate".');
    await expect(last).toContainText('2+5050 = 5052');
    await expect(last).not.toContainText('could not evaluate');
    await last.evaluate((node) => {
      for (const det of node.querySelectorAll('details.diagnostics-detail')) {
        det.open = true;
      }
    });
    const formalization = last.locator('[data-testid="formalization"]').first();
    await expect(formalization).toContainText('OP:compute');

    const second = await sendPrompt(page, 'Calcuate 2+5050');
    await expect(second).toHaveClass(/assistant/);
    await expect(second).toContainText('Interpreted "Calcuate" as "calculate".');
    await expect(second).toContainText('2+5050 = 5052');
    await expect(second).not.toContainText('could not evaluate');
  });

  test('Russian word-number arithmetic resolves as a calculation', async ({ page }) => {
    const last = await sendPrompt(page, 'Сколько будет два плюс два?');
    await expect(last).toHaveClass(/assistant/);
    await expect(last).toContainText('два плюс два = 4');
  });

  test('embedded calculation requests resolve across supported languages', async ({ page }) => {
    const cases = [
      {
        language: 'en',
        prompt: 'I want to know what is 2+2',
        expected: '2+2 = 4',
      },
      {
        language: 'ru',
        prompt: 'хочу понять сколько будет 2+2',
        expected: '2+2 = 4',
      },
      {
        language: 'hi',
        prompt: 'मुझे बताओ गणना करें 8 / 2',
        expected: '8 / 2 = 4',
      },
      {
        language: 'zh',
        prompt: '我想知道计算 2 + 2',
        expected: '2 + 2 = 4',
      },
    ];

    for (const { prompt, expected } of cases) {
      const last = await sendPrompt(page, prompt);
      await expect(last).toHaveClass(/assistant/);
      await expect(last).toContainText(expected);
      await expect(last).not.toContainText(UNKNOWN_ANSWER_MARKER);
    }
  });

  test('Russian currency conversion resolves as a calculation', async ({ page }) => {
    const last = await sendPrompt(page, 'Посчитай 1000 рублей в долларах');
    await expect(last).toHaveClass(/assistant/);
    await expect(last).toContainText('1000 рублей в долларах = 11.1731843575 USD');
    await expect(last).not.toContainText(UNKNOWN_ANSWER_MARKER);
  });

  test('exchange-rate basis prompts resolve as calculations across supported languages', async ({ page }) => {
    const cases = [
      {
        language: 'en',
        prompt: 'what dollar exchange rate do you use for calculations?',
      },
      { language: 'ru', prompt: 'какой курс долора у тебя при расчетах?' },
      {
        language: 'hi',
        prompt: 'गणना में आप डॉलर का कौन सा विनिमय दर उपयोग करते हैं?',
      },
      { language: 'zh', prompt: '你计算时使用什么美元汇率?' },
    ];

    for (const { language, prompt } of cases) {
      const last = await sendPrompt(page, prompt);
      await expect(last, `${language} reply`).toHaveClass(/assistant/);
      await expect(last, `${language} calculator`).toContainText('link-calculator');
      await expect(last, `${language} rate`).toContainText('1 USD in RUB = 89.5 RUB');
      await expect(last, `${language} detail`).toContainText('Exchange rate: 1 USD = 89.5 RUB');
      await expect(last, `${language} unknown`).not.toContainText(UNKNOWN_ANSWER_MARKER);
    }
  });

  test('Russian weekday relation resolves through calendar reasoning', async ({ page }) => {
    const last = await sendPrompt(page, 'какой день недели наступает после вторника');
    await expect(last).toHaveClass(/assistant/);
    await expect(last).toContainText('среда');
    await expect(last).toContainText('семидневном календарном цикле');
  });

  test('current-day questions resolve through calendar reasoning across supported languages', async ({ page }) => {
    const cases = [
      { prompt: 'What day is today?', locale: 'en-US', today: 'Today is' },
      { prompt: 'Какой сегодня день?', locale: 'ru-RU', today: 'Сегодня' },
      { prompt: 'आज कौन सा दिन है?', locale: 'hi-IN', today: 'आज' },
      { prompt: '今天是星期几?', locale: 'zh-CN', today: '今天' },
    ];

    for (const { prompt, locale, today } of cases) {
      const expectedWeekday = await page.evaluate(
        (nextLocale) =>
          new Intl.DateTimeFormat(nextLocale, { weekday: 'long' }).format(
            new Date(),
          ),
        locale,
      );
      const last = await sendPrompt(page, prompt);
      await expect(last).toHaveClass(/assistant/);
      await expect(last).toContainText(today);
      await expect(last).toContainText(expectedWeekday);
      await expect(last).not.toContainText(UNKNOWN_ANSWER_MARKER);
    }
  });

  test('calendar create event from natural language (issue #404)', async ({ page }) => {
    // Russian exact prompt from the bug report + English fallback.
    // The worker (edited for parity) should now return non-unknown calendar_create_event
    // with a confirmation-style proposal instead of falling through.
    // Every environment returns a real, portable calendar artifact: an RFC 5545
    // VEVENT (.ics) the user can import anywhere plus a no-login Google Calendar
    // render URL. We assert those appear for all four supported languages, with
    // the Russian timezone alias ("по грузии") resolved to IANA Asia/Tbilisi.
    const cases = [
      {
        prompt: 'Забей мне 18 число в 17:00 по грузии на встречу с Леваном',
        locale: 'ru-RU',
        mustContain: [
          'событие',
          '18',
          '17:00',
          'Asia/Tbilisi',
          'BEGIN:VCALENDAR',
          'calendar.google.com',
          'да',
        ],
      },
      {
        prompt: 'schedule meeting with Levan on the 18th at 5pm Georgia time',
        locale: 'en-US',
        mustContain: [
          'Create event',
          '18',
          '17:00',
          'Asia/Tbilisi',
          'BEGIN:VCALENDAR',
          'calendar.google.com',
          'yes',
        ],
      },
      {
        prompt: '18 तारीख को शाम 5 बजे लेवान के साथ मीटिंग शेड्यूल करें',
        locale: 'hi-IN',
        mustContain: ['मीटिंग', '18', '17:00', 'BEGIN:VCALENDAR', 'calendar.google.com', 'हाँ'],
      },
      {
        prompt: '18号下午5点和Levan安排会议',
        locale: 'zh-CN',
        mustContain: ['会议', '18', '17:00', 'BEGIN:VCALENDAR', 'calendar.google.com', '是'],
      },
    ];

    for (const { prompt, mustContain } of cases) {
      const last = await sendPrompt(page, prompt);
      await expect(last).toHaveClass(/assistant/);
      for (const needle of mustContain) {
        await expect(last).toContainText(needle);
      }
      await expect(last).not.toContainText(UNKNOWN_ANSWER_MARKER);
    }
  });

  test('percentage-of-currency prompt resolves as a calculation before Wikipedia fallback', async ({ page }) => {
    let wikipediaRequests = 0;
    await page.route('https://en.wikipedia.org/**', async (route) => {
      wikipediaRequests += 1;
      const url = route.request().url();
      if (url.includes('/w/rest.php/v1/search/page')) {
        await route.fulfill({
          status: 200,
          contentType: 'application/json',
          body: JSON.stringify({
            pages: [
              {
                id: 1,
                key: 'Douglas_DC-8',
                title: 'Douglas DC-8',
                excerpt: 'The Douglas DC-8 is an early long-range narrow-body jetliner.',
                description: 'Jet airliner',
              },
            ],
          }),
        });
        return;
      }
      await route.fulfill({
        status: 200,
        contentType: 'application/json',
        body: JSON.stringify({
          title: 'Douglas DC-8',
          extract: 'The Douglas DC-8 is an early long-range narrow-body jetliner.',
          content_urls: {
            desktop: { page: 'https://en.wikipedia.org/wiki/Douglas_DC-8' },
          },
        }),
      });
    });

    const last = await sendPrompt(page, 'What is 8% of $50?');
    await expect(last).toHaveClass(/assistant/);
    await expect(last).toContainText('8% of $50 = 4 USD');
    await expect(last).not.toContainText('Douglas DC-8');
    expect(wikipediaRequests).toBe(0);
  });

  test('Russian "What is X?" returns the offline concept summary', async ({ page }) => {
    const last = await sendPrompt(page, 'Что такое Википедия?');
    await expect(last).toHaveClass(/assistant/);
    await expect(last).toContainText(/Wikipedia|encyclopedia/i);
  });

  test('Issue #184: OpenStreerMap typo resolves through Wikipedia fuzzy search across supported languages', async ({
    page,
  }) => {
    const apiCalls = [];

    await page.route('**/api/rest_v1/page/summary/**', async (route) => {
      const url = route.request().url();
      const slug = decodeURIComponent(url.split('/').pop() || '');
      apiCalls.push({ kind: 'summary', slug, url });
      if (slug === 'OpenStreetMap') {
        await route.fulfill({
          status: 200,
          contentType: 'application/json',
          body: JSON.stringify({
            title: 'OpenStreetMap',
            type: 'standard',
            extract:
              'OpenStreetMap is a free collaborative map database maintained by volunteers.',
            content_urls: {
              desktop: { page: 'https://en.wikipedia.org/wiki/OpenStreetMap' },
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

    await page.route('**/w/rest.php/v1/search/page**', async (route) => {
      const url = route.request().url();
      apiCalls.push({ kind: 'search', url });
      await route.fulfill({
        status: 200,
        contentType: 'application/json',
        body: JSON.stringify({
          pages: [
            {
              key: 'OpenStreetMap',
              title: 'OpenStreetMap',
              excerpt:
                'OpenStreetMap is a free collaborative map database maintained by volunteers.',
              description: 'collaborative map database',
            },
          ],
        }),
      });
    });

    await page.route('**://*.wikidata.org/w/api.php**', async (route) => {
      await route.fulfill({
        status: 200,
        contentType: 'application/json',
        body: JSON.stringify({ search: [] }),
      });
    });

    await page.route('**://*.wiktionary.org/w/api.php**', async (route) => {
      await route.fulfill({
        status: 200,
        contentType: 'application/json',
        body: JSON.stringify(['OpenStreerMap', [], [], []]),
      });
    });

    const cases = [
      'what is OpenStreerMap',
      'что такое OpenStreerMap',
      'OpenStreerMap क्या है',
      'OpenStreerMap是什么',
    ];

    for (const prompt of cases) {
      const before = apiCalls.length;
      const last = await sendPrompt(page, prompt);
      const calls = apiCalls.slice(before);
      await expect(last).toHaveClass(/assistant/);
      await expect(last).toContainText('OpenStreetMap');
      await expect(last).toContainText('collaborative map database');
      await expect(last).toContainText('wikipedia.org');
      await expect(last).toContainText('Closest match from Wikipedia search');
      await expect(last).not.toContainText(UNKNOWN_ANSWER_MARKER);
      expect(calls.some((call) => call.kind === 'search')).toBeTruthy();
      expect(
        calls.some(
          (call) => call.kind === 'summary' && call.slug === 'OpenStreetMap',
        ),
      ).toBeTruthy();
    }
  });

  test('Issue #182: BSD ports prompts across supported languages do not fall through to OpenBSD', async ({
    page,
  }) => {
    let wikipediaRequests = 0;
    await page.route('**/w/rest.php/v1/search/page**', async (route) => {
      wikipediaRequests += 1;
      await route.fulfill({
        status: 200,
        contentType: 'application/json',
        body: JSON.stringify({
          pages: [
            {
              id: 12,
              key: 'OpenBSD',
              title: 'OpenBSD',
              excerpt: 'OpenBSD is a security-focused operating system.',
              description: 'BSD operating system',
            },
          ],
        }),
      });
    });
    await page.route('**/api/rest_v1/page/summary/**', async (route) => {
      wikipediaRequests += 1;
      await route.fulfill({
        status: 200,
        contentType: 'application/json',
        body: JSON.stringify({
          title: 'OpenBSD',
          extract:
            'OpenBSD is a security-focused operating system based on the Berkeley Software Distribution.',
          type: 'standard',
          content_urls: {
            desktop: { page: 'https://ru.wikipedia.org/wiki/OpenBSD' },
          },
        }),
      });
    });

    const cases = [
      {
        prompt: 'what is ports in BSD?',
        term: /BSD ports/i,
        explanation: /package|source-based/i,
      },
      {
        prompt: 'что такое порты в bsd',
        term: /Порты BSD|BSD ports/i,
        explanation: /пакет|package|приложен/i,
      },
      {
        prompt: 'BSD में पोर्ट्स क्या है?',
        term: /BSD पोर्ट्स|BSD ports/i,
        explanation: /पैकेज|package|source/i,
      },
      {
        prompt: 'BSD中的端口集合是什么?',
        term: /BSD Ports|BSD ports/i,
        explanation: /源代码|package|包管理/i,
      },
    ];

    for (const entry of cases) {
      const last = await sendPrompt(page, entry.prompt);
      await expect(last).toHaveClass(/assistant/);
      await expect(last).toContainText(entry.term);
      await expect(last).toContainText(entry.explanation);
      await expect(last).not.toContainText('OpenBSD:');
    }

    expect(wikipediaRequests).toBe(0);
  });

  test('Issue #182: context Wikipedia search rejects a title that only matches the context', async ({ page }) => {
    await page.route('**/w/rest.php/v1/search/page**', async (route) => {
      await route.fulfill({
        status: 200,
        contentType: 'application/json',
        body: JSON.stringify({
          pages: [
            {
              id: 12,
              key: 'OpenBSD',
              title: 'OpenBSD',
              excerpt: 'OpenBSD is a security-focused operating system.',
              description: 'BSD operating system',
            },
          ],
        }),
      });
    });
    await page.route('**/api/rest_v1/page/summary/**', async (route) => {
      const slug = decodeURIComponent(route.request().url().split('/').pop() || '');
      if (slug === 'OpenBSD') {
        await route.fulfill({
          status: 200,
          contentType: 'application/json',
          body: JSON.stringify({
            title: 'OpenBSD',
            extract:
              'OpenBSD is a security-focused operating system based on the Berkeley Software Distribution.',
            type: 'standard',
            content_urls: {
              desktop: { page: 'https://ru.wikipedia.org/wiki/OpenBSD' },
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
    await page.route('**://*.wikidata.org/w/api.php**', async (route) => {
      await route.fulfill({
        status: 200,
        contentType: 'application/json',
        body: JSON.stringify({ search: [] }),
      });
    });
    await page.route('**://*.wiktionary.org/w/api.php**', async (route) => {
      await route.fulfill({
        status: 200,
        contentType: 'application/json',
        body: JSON.stringify(['зупфы', [], [], []]),
      });
    });

    const last = await sendPrompt(page, 'что такое зупфы в bsd');
    await expect(last).toHaveClass(/assistant/);
    await expect(last).toContainText(/не могу ответить|cannot answer/i);
    await expect(last).not.toContainText('OpenBSD');
  });

  test('Issue #159: Russian Hive Mind prompt prefers link-assistant project and still searches the web', async ({ page }) => {
    await page.route('**://api.duckduckgo.com/**', async (route) => {
      await route.fulfill({
        status: 200,
        contentType: 'application/json',
        body: JSON.stringify({
          Heading: 'Hive mind',
          AbstractText: 'A hive mind is a collective intelligence concept.',
          AbstractURL: 'https://example.com/hive-mind-overview',
          RelatedTopics: [],
        }),
      });
    });
    await page.route('**/w/rest.php/v1/search/page**', async (route) => {
      await route.fulfill({
        status: 200,
        contentType: 'application/json',
        body: JSON.stringify({
          pages: [
            {
              id: 42,
              key: 'LOIC',
              title: 'LOIC',
              excerpt: 'LOIC has a Hive Mind mode, but it is not the preferred project match.',
              description: 'network stress-testing software',
            },
          ],
        }),
      });
    });
    await page.route('**/wikidata.org/w/api.php**', async (route) => {
      await route.fulfill({
        status: 200,
        contentType: 'application/json',
        body: JSON.stringify({
          search: [
            {
              id: 'Q188641',
              label: 'Hive mind',
              description: 'collective consciousness or group intelligence concept',
              concepturi: 'https://www.wikidata.org/wiki/Q188641',
            },
          ],
        }),
      });
    });
    await page.route('**/api/rest_v1/page/summary/**', async (route) => {
      const url = route.request().url();
      if (url.endsWith('/LOIC')) {
        await route.fulfill({
          status: 200,
          contentType: 'application/json',
          body: JSON.stringify({
            title: 'LOIC',
            extract: 'LOIC is open-source software with a Hive Mind mode.',
            type: 'standard',
            content_urls: {
              desktop: { page: 'https://ru.wikipedia.org/wiki/LOIC' },
            },
          }),
        });
        return;
      }
      await route.fulfill({
        status: 404,
        contentType: 'application/json',
        body: JSON.stringify({ title: 'Not found' }),
      });
    });

    const last = await sendPrompt(page, 'Что такое Hive Mind?');
    await expect(last).toHaveClass(/assistant/);
    await expect(last).toContainText('link-assistant/hive-mind');
    await expect(last).toContainText(/ИИ|AI that controls AIs/);
    await expect(last).toContainText('Результаты поиска для');
    await expect(last).toContainText('LOIC');
    await expect(last).not.toContainText('Ближайшее совпадение по поиску Wikipedia');
  });

  test('Chinese "X 是什么?" returns the offline concept summary', async ({ page }) => {
    const last = await sendPrompt(page, '维基百科是什么?');
    await expect(last).toHaveClass(/assistant/);
    await expect(last).toContainText(/Wikipedia|encyclopedia/i);
  });

  test('Russian capital-of-Russia prompt returns the seeded fact answer', async ({ page }) => {
    const last = await sendPrompt(page, 'столица россии');
    await expect(last).toHaveClass(/assistant/);
    await expect(last).toContainText('Москва');
    await expect(last).not.toContainText(UNKNOWN_ANSWER_MARKER);
  });

  // Issue #127: the fact-query reasoning pipeline pre-warms the cache from
  // `data/seed/facts.lino` records that carry a `relation` field. The seeded
  // matrix covers Russia, Japan, France, Germany, China, India, USA, UK, and
  // Brazil — every country resolves offline in every supported language.
  const FACT_QUERY_CASES = [
    { prompt: 'What is the capital of France?', expected: 'Paris' },
    { prompt: 'What is the capital of Germany?', expected: 'Berlin' },
    { prompt: 'What is the capital of China?', expected: 'Beijing' },
    { prompt: 'What is the capital of India?', expected: 'New Delhi' },
    { prompt: 'What is the capital of the United States?', expected: 'Washington' },
    { prompt: 'What is the capital of the UK?', expected: 'London' },
    { prompt: 'What is the capital of Brazil?', expected: 'Bras' },
    { prompt: 'Столица Германии', expected: 'Берлин' },
    { prompt: 'Столица Франции', expected: 'Париж' },
    { prompt: '中国的首都是什么?', expected: '北京' },
    { prompt: 'भारत की राजधानी क्या है?', expected: 'दिल्ली' },
  ];

  for (const { prompt, expected } of FACT_QUERY_CASES) {
    test(`fact-query pipeline resolves: ${prompt}`, async ({ page }) => {
      const last = await sendPrompt(page, prompt);
      await expect(last).toHaveClass(/assistant/);
      await expect(last).toContainText(expected);
      await expect(last).not.toContainText(UNKNOWN_ANSWER_MARKER);
    });
  }

  test('merged Wikipedia definitions combine localized seed summaries', async ({ page }) => {
    const last = await sendPrompt(page, 'Merge Wikipedia definitions of IIR');
    await expect(last).toHaveClass(/assistant/);
    await expect(last).toContainText('Merged definition of infinite impulse response (IIR)');
    await expect(last).toContainText('Source languages: en, ru, hi, zh');
    await expect(last).toContainText('recursive digital filter');
    await expect(last).toContainText('Фильтр с бесконечной импульсной характеристикой');
  });

  test('definition fusion setting merges plain definition prompts', async ({ page }) => {
    await page.locator('[data-testid="setting-definition-fusion"]').selectOption('auto');
    const last = await sendPrompt(page, 'What is IIR?');
    await expect(last).toHaveClass(/assistant/);
    await expect(last).toContainText('Merged definition of infinite impulse response (IIR)');
    await expect(last).toContainText('Source languages: en, ru, hi, zh');
  });

  test('punctuation-only prompt asks for clarification', async ({ page }) => {
    const last = await sendPrompt(page, '.');
    await expect(last).toHaveClass(/assistant/);
    await expect(last).toContainText(/only punctuation/i);
    await expect(last).toContainText(/What would you like/i);
  });

  // Issue #31: "что такое Kiss в рамках програмирования" was returning the
  // rock band KISS instead of the software design principle because the
  // wikipedia_lookup intent ignored the context clause.
  test('Russian "what is KISS in programming" returns the design principle, not the band', async ({ page }) => {
    const last = await sendPrompt(page, 'что такое Kiss в рамках програмирования');
    await expect(last).toHaveClass(/assistant/);
    // Must mention the design principle, not the rock band.
    await expect(last).toContainText(/принцип|principle|KISS|simple/i);
    await expect(last).not.toContainText(/рок-группа|rock band|american.*rock|глэм/i);
  });

  test('GitHub navigation suggests an external link without iframe preview', async ({ page }) => {
    const framePolicyRequests = await routeFramePolicy(page, {
      'x-frame-options': 'deny',
      'content-security-policy': "frame-ancestors 'none'",
    });
    const githubRequestTypes = [];
    await page.route(/https:\/\/github\.com\/?.*/, async (route) => {
      githubRequestTypes.push(route.request().resourceType());
      await route.abort('blockedbyclient');
    });

    const last = await sendPrompt(page, 'Navigate to github.com');
    await expect(last).toHaveClass(/assistant/);
    await expect(last).toContainText('https://github.com');
    await expect(last).toContainText('I suggest opening this in a new tab');
    await expect(last).toContainText("I checked the page's frame policy");
    await expect(last).toContainText('does not allow embedding');
    await expect(last).toContainText('X-Frame-Options: DENY');
    await expect(last).toContainText("CSP frame-ancestors 'none'");
    await expect(last).toContainText('Browser JavaScript');
    await expect(last).not.toContainText('Could not fetch');
    await expect(last).not.toContainText('cannot reliably confirm');
    await expect(last).not.toContainText('URL requested for');
    await expect(last).not.toContainText('Open this');
    await expect(last).not.toContainText('demo');
    await expect(last).not.toContainText('iframe');
    await expect(last).not.toContainText('preview below');
    await expect(last).toContainText(/new tab/i);
    await expect(last).not.toContainText(UNKNOWN_ANSWER_MARKER);
    const link = last.locator('.markdown-body a.external-link').filter({
      hasText: 'https://github.com',
    });
    await expect(link).toHaveAttribute(
      'href',
      /https:\/\/github\.com\/?/,
    );
    await expect(link).toHaveAttribute('target', '_blank');
    await expect(link).toHaveAttribute('rel', /noopener/);
    await expect(last.locator('.external-link-icon')).toBeVisible();
    await expect(last.locator('[data-testid="fetch-iframe-container"]')).toHaveCount(0);
    expect(framePolicyRequests).toHaveLength(1);
    expect(framePolicyRequests[0]).toContain('url=https%3A%2F%2Fgithub.com');
    expect(githubRequestTypes).not.toContain('fetch');
    expect(githubRequestTypes).not.toContain('document');
  });

  test('Navigation previews URLs when frame policy allows embedding', async ({ page }) => {
    const framePolicyRequests = await routeFramePolicy(page, {});
    const exampleRequestTypes = [];
    await page.route(/https:\/\/example\.com\/?.*/, async (route) => {
      exampleRequestTypes.push(route.request().resourceType());
      await route.fulfill({
        status: 200,
        contentType: 'text/html',
        body: '<!doctype html><title>Example preview</title><p>Example preview</p>',
      });
    });

    const last = await sendPrompt(page, 'Navigate to example.com');
    await expect(last).toHaveClass(/assistant/);
    await expect(last).toContainText('https://example.com');
    await expect(last).toContainText("I checked the page's frame policy");
    await expect(last).toContainText('Direct link');
    await expect(last).not.toContainText(UNKNOWN_ANSWER_MARKER);
    const frameContainer = last.locator('[data-testid="fetch-iframe-container"]');
    await expect(frameContainer).toContainText(/https:\/\/example\.com\/?/);
    await expect(frameContainer.locator('[data-testid="fetch-iframe"]')).toHaveAttribute(
      'src',
      /https:\/\/example\.com\/?/,
    );
    expect(framePolicyRequests).toHaveLength(1);
    expect(framePolicyRequests[0]).toContain('url=https%3A%2F%2Fexample.com');
    expect(exampleRequestTypes).not.toContain('fetch');
    expect(exampleRequestTypes).toContain('document');
  });

  // Issue #125 follow-up: "Make a request to X" must still attempt an HTTP
  // fetch (with frame-policy checked CORS fallback), while "Navigate to X" must not.
  test('Make a request to X attempts a fetch and falls back to the iframe', async ({ page }) => {
    const framePolicyRequests = await routeFramePolicy(page, {});
    const fetchAttempts = [];
    await page.route(/https:\/\/example\.com\/?.*/, async (route) => {
      fetchAttempts.push(route.request().resourceType());
      await route.abort('blockedbyclient');
    });

    const last = await sendPrompt(page, 'Make a request to example.com');
    await expect(last).toHaveClass(/assistant/);
    await expect(last).toContainText('https://example.com');
    await expect(last).not.toContainText(UNKNOWN_ANSWER_MARKER);
    await expect(last).toContainText("I checked the page's frame policy");
    // The browser worker must call fetch() before falling back to the iframe.
    expect(fetchAttempts).toContain('fetch');
    expect(framePolicyRequests).toHaveLength(1);
    const frameContainer = last.locator('[data-testid="fetch-iframe-container"]');
    await expect(frameContainer).toContainText(/https:\/\/example\.com\/?/);
  });

  test('explicit web search fuses DuckDuckGo, Wikipedia, and Wikidata results', async ({
    page,
  }) => {
    await page.locator('.diagnostics-toggle').click();

    // Issue #133: the worker now fans out to DuckDuckGo (default), Wikipedia,
    // and Wikidata in parallel and fuses results with reciprocal rank fusion.
    // Mock all three so the test is deterministic.
    await page.route('**://api.duckduckgo.com/**', async (route) => {
      await route.fulfill({
        status: 200,
        contentType: 'application/json',
        body: JSON.stringify({
          Heading: 'Nikola Tesla',
          AbstractText: 'Nikola Tesla was a Serbian-American inventor.',
          AbstractURL: 'https://duckduckgo.com/Nikola_Tesla',
          RelatedTopics: [],
        }),
      });
    });
    await page.route('**/w/rest.php/v1/search/page**', async (route) => {
      await route.fulfill({
        status: 200,
        contentType: 'application/json',
        body: JSON.stringify({
          pages: [
            {
              id: 123,
              key: 'Nikola_Tesla',
              title: 'Nikola Tesla',
              excerpt: 'Nikola Tesla was a Serbian-American inventor.',
              description: 'inventor and electrical engineer',
            },
          ],
        }),
      });
    });
    await page.route('**://*.wikidata.org/w/api.php**', async (route) => {
      await route.fulfill({
        status: 200,
        contentType: 'application/json',
        body: JSON.stringify({
          search: [
            {
              id: 'Q9036',
              label: 'Nikola Tesla',
              description: 'Serbian-American inventor and electrical engineer',
              concepturi: 'https://www.wikidata.org/wiki/Q9036',
            },
          ],
        }),
      });
    });

    const last = await sendPrompt(page, 'Search the web for Nikola Tesla');
    await expect(last).toHaveClass(/assistant/);
    await expect(last).toContainText('Search results for');
    await expect(last).toContainText('Nikola Tesla');
    await expect(last).toContainText('Serbian-American inventor');
    await expect(last.locator('.evidence-list')).toContainText('web_search:provider:duckduckgo');
    await expect(last.locator('.evidence-list')).toContainText('web_search:provider:wikipedia');
    await expect(last.locator('.evidence-list')).toContainText('web_search:combined:rrf:k=60');
    await expect(last).not.toContainText(UNKNOWN_ANSWER_MARKER);
  });
});

test.describe('Wikipedia REST fallback', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('./');
    await expect(page.locator('.app')).toBeVisible({ timeout: 15_000 });
    await switchToManualMode(page);
  });

  test('"What is X?" for an out-of-corpus term fetches a Wikipedia summary', async ({ page }) => {
    // Stub the Wikipedia REST endpoint so the test is hermetic and does not depend
    // on external network availability or rate limiting.
    await page.route('**/api/rest_v1/page/summary/**', async (route) => {
      const json = {
        title: 'Albert Einstein',
        extract: 'Albert Einstein was a German-born theoretical physicist...',
        type: 'standard',
        content_urls: {
          desktop: { page: 'https://en.wikipedia.org/wiki/Albert_Einstein' },
        },
      };
      await route.fulfill({
        status: 200,
        contentType: 'application/json',
        body: JSON.stringify(json),
      });
    });

    const last = await sendPrompt(page, 'What is Albert Einstein?');
    await expect(last).toHaveClass(/assistant/);
    await expect(last).toContainText('Albert Einstein');
    await expect(last).toContainText('theoretical physicist');
    await expect(last).toContainText('en.wikipedia.org');
  });

  test('"Tell me, who is X" resolves through Wikipedia lookup', async ({ page }) => {
    await page.route('**/api/rest_v1/page/summary/**', async (route) => {
      const json = {
        title: 'Donald Trump',
        extract:
          'Donald John Trump is an American politician, media personality, and businessman.',
        type: 'standard',
        content_urls: {
          desktop: { page: 'https://en.wikipedia.org/wiki/Donald_Trump' },
        },
      };
      await route.fulfill({
        status: 200,
        contentType: 'application/json',
        body: JSON.stringify(json),
      });
    });

    const last = await sendPrompt(page, 'Tell me, who is Trump');
    await expect(last).toHaveClass(/assistant/);
    await expect(last).toContainText('Donald Trump');
    await expect(last).toContainText('politician');
    await expect(last).not.toContainText(UNKNOWN_ANSWER_MARKER);
  });

  test('"Who X is" resolves through Wikipedia lookup', async ({ page }) => {
    await page.route('**/api/rest_v1/page/summary/**', async (route) => {
      const json = {
        title: 'Donald Trump',
        extract:
          'Donald John Trump is an American politician, media personality, and businessman.',
        type: 'standard',
        content_urls: {
          desktop: { page: 'https://en.wikipedia.org/wiki/Donald_Trump' },
        },
      };
      await route.fulfill({
        status: 200,
        contentType: 'application/json',
        body: JSON.stringify(json),
      });
    });

    const last = await sendPrompt(page, 'Who Trump is');
    await expect(last).toHaveClass(/assistant/);
    await expect(last).toContainText('Donald Trump');
    await expect(last).toContainText('politician');
    await expect(last).not.toContainText(UNKNOWN_ANSWER_MARKER);
  });

  test('Issue #183: Russian "как устроен X" resolves through Wikipedia lookup', async ({ page }) => {
    const requestedSlugs = [];
    await page.route('**/api/rest_v1/page/summary/**', async (route) => {
      const url = route.request().url();
      const slug = decodeURIComponent(url.split('/').pop() || '');
      requestedSlugs.push(slug);
      if (slug.toLowerCase() === 'aur') {
        const json = {
          title: 'Arch User Repository',
          extract:
            'The Arch User Repository is a community-driven repository for Arch Linux users.',
          type: 'standard',
          content_urls: {
            desktop: {
              page: 'https://en.wikipedia.org/wiki/Arch_User_Repository',
            },
          },
        };
        await route.fulfill({
          status: 200,
          contentType: 'application/json',
          body: JSON.stringify(json),
        });
        return;
      }
      await route.fulfill({
        status: 404,
        contentType: 'application/json',
        body: JSON.stringify({ httpCode: 404, httpReason: 'Not Found' }),
      });
    });

    const last = await sendPrompt(page, 'как устроен AUR');
    await expect(last).toHaveClass(/assistant/);
    await expect(last).toContainText('Arch User Repository');
    await expect(last).toContainText('community-driven repository');
    await expect(last).toContainText('en.wikipedia.org');
    await expect(last).not.toContainText(UNKNOWN_ANSWER_MARKER);
    expect(requestedSlugs.some((slug) => slug.toLowerCase() === 'aur')).toBe(true);
  });

  // Issue #21: Wikipedia returns percent-encoded URLs for non-ASCII titles.
  // The chat must display the readable Cyrillic form while the underlying
  // link still points at the canonical (encoded) URL.
  test('Russian Wikipedia summary displays decoded Cyrillic URL with encoded href', async ({ page }) => {
    const encodedUrl =
      'https://ru.wikipedia.org/wiki/%D0%98%D0%B7%D1%83%D0%BC%D1%80%D1%83%D0%B4';
    const humanUrl = 'https://ru.wikipedia.org/wiki/Изумруд';
    await page.route('**/api/rest_v1/page/summary/**', async (route) => {
      const json = {
        title: 'Изумруд',
        extract: 'Изумруд — драгоценный камень берилловой группы.',
        type: 'standard',
        content_urls: { desktop: { page: encodedUrl } },
      };
      await route.fulfill({
        status: 200,
        contentType: 'application/json',
        body: JSON.stringify(json),
      });
    });

    const last = await sendPrompt(page, 'Что такое изумруд?');
    await expect(last).toHaveClass(/assistant/);
    // Display text is the readable IRI form.
    await expect(last).toContainText(humanUrl);
    // And the percent-encoded form must not leak into the visible message.
    await expect(last).not.toContainText(
      '%D0%98%D0%B7%D1%83%D0%BC%D1%80%D1%83%D0%B4',
    );
    // The anchor's href stays the canonical encoded URL so clicking it still resolves.
    const anchor = last.locator(`a[href="${encodedUrl}"]`);
    await expect(anchor).toHaveCount(1);
    await expect(anchor).toHaveText(humanUrl);
  });

  // Issue #27: ru.wikipedia.org biographies use the "Surname, Given names"
  // form, so `Илон_Маск` 404s while `Маск,_Илон` resolves. The worker must
  // try the swapped variant for two-word terms.
  test('Кто такой Илон Маск? resolves via surname-first variant', async ({ page }) => {
    const requestedSlugs = [];
    await page.route('**/api/rest_v1/page/summary/**', async (route) => {
      const url = route.request().url();
      const slug = decodeURIComponent(url.split('/').pop());
      requestedSlugs.push(slug);
      if (slug === 'Маск,_Илон') {
        const json = {
          title: 'Маск, Илон',
          extract:
            'И́лон Рив Маск — американский и южноафриканский предприниматель, инженер и миллиардер.',
          type: 'standard',
          content_urls: {
            desktop: {
              page:
                'https://ru.wikipedia.org/wiki/%D0%9C%D0%B0%D1%81%D0%BA%2C_%D0%98%D0%BB%D0%BE%D0%BD',
            },
          },
        };
        await route.fulfill({
          status: 200,
          contentType: 'application/json',
          body: JSON.stringify(json),
        });
        return;
      }
      await route.fulfill({
        status: 404,
        contentType: 'application/json',
        body: JSON.stringify({ httpCode: 404, httpReason: 'Not Found' }),
      });
    });

    const last = await sendPrompt(page, 'Кто такой Илон Маск?');
    await expect(last).toHaveClass(/assistant/);
    await expect(last).toContainText('Маск, Илон');
    await expect(last).toContainText('предприниматель');
    expect(requestedSlugs).toContain('Маск,_Илон');
  });

  // Issue #70: terms whose Wikipedia title is a disambiguation page (e.g.
  // "Tesla") were returning "unknown intent" because the bare-slug loop skipped
  // disambiguation results without falling back to the search endpoint.
  test('"what is tesla" resolves via search fallback when direct slug is disambiguation', async ({ page }) => {
    await page.route('**/api/rest_v1/page/summary/**', async (route) => {
      // Every direct slug attempt returns a disambiguation page.
      const json = {
        title: 'Tesla',
        type: 'disambiguation',
        extract: 'Tesla may refer to: Nikola Tesla or Tesla, Inc.',
        content_urls: { desktop: { page: 'https://en.wikipedia.org/wiki/Tesla' } },
      };
      await route.fulfill({
        status: 200,
        contentType: 'application/json',
        body: JSON.stringify(json),
      });
    });

    await page.route('**/rest.php/v1/search/page**', async (route) => {
      // Search returns the company as the top result.
      const json = {
        pages: [{ key: 'Tesla,_Inc.', title: 'Tesla, Inc.' }],
      };
      await route.fulfill({
        status: 200,
        contentType: 'application/json',
        body: JSON.stringify(json),
      });
    });

    // After stubbing search results, the second fetch for Tesla,_Inc. must
    // return a standard article, not the disambiguation stub. Override the
    // summary route so that the Tesla,_Inc. slug gets a real response while all
    // other slugs remain disambiguation pages.
    await page.route('**/api/rest_v1/page/summary/Tesla%2C_Inc.**', async (route) => {
      const json = {
        title: 'Tesla, Inc.',
        type: 'standard',
        extract: 'Tesla, Inc. is an American multinational automotive and clean energy company.',
        content_urls: { desktop: { page: 'https://en.wikipedia.org/wiki/Tesla,_Inc.' } },
      };
      await route.fulfill({
        status: 200,
        contentType: 'application/json',
        body: JSON.stringify(json),
      });
    });

    const last = await sendPrompt(page, 'what is tesla');
    await expect(last).toHaveClass(/assistant/);
    await expect(last).toContainText('Tesla');
    await expect(last).not.toContainText(UNKNOWN_ANSWER_MARKER);
    await expect(last).toContainText('en.wikipedia.org');
  });

  for (const testCase of definitionDisambiguationCases) {
    test(`Issue #232: definition-style disambiguation page outranks Wikidata alias fallback (${testCase.language})`, async ({
      page,
    }) => {
      await routeDefinitionDisambiguationCase(page, testCase);

      const last = await sendPrompt(page, testCase.prompt);
      await expect(last).toHaveClass(/assistant/);
      await expect(last).toContainText(testCase.title);
      for (const entry of testCase.entries) {
        await expect(last).toContainText(entry);
      }
      await expect(last).toContainText(testCase.sourceUrl);
      await expect(last).not.toContainText(testCase.rejectedWikidata.label);
      await expect(last).not.toContainText('wikidata.org');
      await expect(last).not.toContainText(UNKNOWN_ANSWER_MARKER);
    });
  }

  test('Russian typo resolves to the closest Wikipedia match when guessing is preferred', async ({ page }) => {
    await page.route('**/api/rest_v1/page/summary/**', async (route) => {
      const slug = decodeURIComponent(route.request().url().split('/').pop() || '');
      if (slug === 'Грамматика') {
        await route.fulfill({
          status: 200,
          contentType: 'application/json',
          body: JSON.stringify({
            title: 'Грамматика',
            type: 'standard',
            extract: 'Грамматика — раздел лингвистики, изучающий грамматический строй языка.',
            content_urls: {
              desktop: { page: 'https://ru.wikipedia.org/wiki/Грамматика' },
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

    await page.route('**/rest.php/v1/search/page**', async (route) => {
      await route.fulfill({
        status: 200,
        contentType: 'application/json',
        body: JSON.stringify({
          pages: [{ key: 'Грамматика', title: 'Грамматика' }],
        }),
      });
    });

    const last = await sendPrompt(page, 'что такое граматика');
    await expect(last).toHaveClass(/assistant/);
    await expect(last).toContainText('Грамматика');
    await expect(last).toContainText('раздел лингвистики');
    await expect(last).toContainText(/closest match|ближайшее совпадение/i);
    await expect(last).not.toContainText(UNKNOWN_ANSWER_MARKER);
  });

  test('Issue #226: Wikipedia article-existence questions cover supported languages', async ({ page }) => {
    const articleSummaries = {
      'Agreement_(linguistics)': {
        language: 'en',
        title: 'Agreement (linguistics)',
        extract: 'Agreement is a grammatical phenomenon where words change form to match one another.',
        url: 'https://en.wikipedia.org/wiki/Agreement_(linguistics)',
      },
      'Согласование_(грамматика)': {
        language: 'ru',
        title: 'Согласование (грамматика)',
        extract: 'Согласование — одна из трёх основных разновидностей подчинительной синтаксической связи.',
        url: 'https://ru.wikipedia.org/wiki/Согласование_(грамматика)',
      },
      'व्याकरणिक_सहमति': {
        language: 'hi',
        title: 'व्याकरणिक सहमति',
        extract: 'व्याकरणिक सहमति वह संबंध है जिसमें शब्द व्याकरणिक रूप से मेल खाते हैं.',
        url: 'https://hi.wikipedia.org/wiki/व्याकरणिक_सहमति',
      },
      '一致_(语言学)': {
        language: 'zh',
        title: '一致 (语言学)',
        extract: '一致是语法中一个词的形式与另一个词相配合的现象。',
        url: 'https://zh.wikipedia.org/wiki/一致_(语言学)',
      },
    };
    const slugByLanguage = {
      en: 'Agreement_(linguistics)',
      ru: 'Согласование_(грамматика)',
      hi: 'व्याकरणिक_सहमति',
      zh: '一致_(语言学)',
    };
    const requestedSlugs = [];
    const searchQueries = [];

    await page.route('**/api/rest_v1/page/summary/**', async (route) => {
      const slug = decodeURIComponent(route.request().url().split('/').pop() || '');
      const language = new URL(route.request().url()).hostname.split('.')[0];
      requestedSlugs.push({ language, slug });
      const summary = articleSummaries[slug];
      if (summary) {
        await route.fulfill({
          status: 200,
          contentType: 'application/json',
          body: JSON.stringify({
            title: summary.title,
            type: 'standard',
            extract: summary.extract,
            content_urls: {
              desktop: {
                page: summary.url,
              },
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

    await page.route('**/rest.php/v1/search/page**', async (route) => {
      const url = new URL(route.request().url());
      const language = url.hostname.split('.')[0];
      const query = url.searchParams.get('q') || '';
      searchQueries.push({ language, query });
      const summary = articleSummaries[slugByLanguage[language] || slugByLanguage.en];
      await route.fulfill({
        status: 200,
        contentType: 'application/json',
        body: JSON.stringify({
          pages: [
            {
              key: slugByLanguage[summary.language],
              title: summary.title,
              excerpt: summary.extract,
              description: 'grammar article',
            },
          ],
        }),
      });
    });

    const exactCases = [
      {
        language: 'en',
        prompt: 'does wikipedia have an article about Agreement (linguistics)',
        title: 'Agreement (linguistics)',
        slug: 'Agreement_(linguistics)',
        marker: /has an article titled/i,
      },
      {
        language: 'ru',
        prompt: 'есть ли в википедии статья о Согласование (грамматика)',
        title: 'Согласование (грамматика)',
        slug: 'Согласование_(грамматика)',
        marker: /есть статья/,
      },
      {
        language: 'hi',
        prompt: 'क्या विकिपीडिया पर व्याकरणिक सहमति लेख है',
        title: 'व्याकरणिक सहमति',
        slug: 'व्याकरणिक_सहमति',
        marker: /लेख है/,
      },
      {
        language: 'zh',
        prompt: '维基百科有一致 (语言学)条目吗',
        title: '一致 (语言学)',
        slug: '一致_(语言学)',
        marker: /有一篇/,
      },
    ];

    for (const { language, prompt, title, slug, marker } of exactCases) {
      const searchCountBefore = searchQueries.length;
      const last = await sendPrompt(page, prompt);
      await expect(last).toHaveClass(/assistant/);
      await expect(last).toContainText(marker);
      await expect(last).toContainText(title);
      await expect(last).toContainText(`${language}.wikipedia.org`);
      await expect(last).not.toContainText(UNKNOWN_ANSWER_MARKER);
      expect(requestedSlugs).toContainEqual({ language, slug });
      expect(searchQueries.slice(searchCountBefore)).toEqual([]);
    }

    const closestCases = [
      {
        language: 'en',
        prompt: 'agreement in a sentence - is there a wikipedia article?',
        title: 'Agreement (linguistics)',
        slug: 'Agreement_(linguistics)',
        context: 'grammar',
        marker: /did not find an exact/i,
      },
      {
        language: 'ru',
        prompt: 'согласованность в предложении - есть такая статья в википедии?',
        title: 'Согласование (грамматика)',
        slug: 'Согласование_(грамматика)',
        context: 'граммат',
        marker: /не нашёл отдельной статьи/,
      },
      {
        language: 'hi',
        prompt: 'वाक्य में सहमति - क्या विकिपीडिया पर ऐसा लेख है?',
        title: 'व्याकरणिक सहमति',
        slug: 'व्याकरणिक_सहमति',
        context: 'व्याकरण',
        marker: /शीर्षक वाला अलग लेख नहीं मिला/,
      },
      {
        language: 'zh',
        prompt: '句子中的一致 - 维基百科有这样的条目吗?',
        title: '一致 (语言学)',
        slug: '一致_(语言学)',
        context: '语法',
        marker: /没有找到标题为/,
      },
    ];

    for (const { language, prompt, title, slug, context, marker } of closestCases) {
      const searchCountBefore = searchQueries.length;
      const last = await sendPrompt(page, prompt);
      await expect(last).toHaveClass(/assistant/);
      await expect(last).toContainText(marker);
      await expect(last).toContainText(title);
      await expect(last).toContainText(`${language}.wikipedia.org`);
      await expect(last).not.toContainText(UNKNOWN_ANSWER_MARKER);
      expect(requestedSlugs).toContainEqual({ language, slug });
      expect(
        searchQueries
          .slice(searchCountBefore)
          .some((entry) => entry.language === language && entry.query.includes(context)),
      ).toBe(true);
    }
  });

  // Issue #163: a short word query like "что такое что" should not accept an
  // unrelated full-text Wikipedia hit ("Знак ударения"). If direct Wikipedia
  // lookup misses and the search title is not a plausible term match, the
  // worker should fall back to Wikidata before rendering the fuzzy result.
  test('Russian word lookup falls back to Wikidata before unrelated Wikipedia search hits', async ({ page }) => {
    await page.route('**/api/rest_v1/page/summary/**', async (route) => {
      const slug = decodeURIComponent(route.request().url().split('/').pop() || '');
      if (slug === 'Знак_ударения') {
        await route.fulfill({
          status: 200,
          contentType: 'application/json',
          body: JSON.stringify({
            title: 'Знак ударения',
            type: 'standard',
            extract: 'Знак ударения — небуквенный орфографический знак.',
            content_urls: {
              desktop: { page: 'https://ru.wikipedia.org/wiki/Знак_ударения' },
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

    await page.route('**/rest.php/v1/search/page**', async (route) => {
      await route.fulfill({
        status: 200,
        contentType: 'application/json',
        body: JSON.stringify({
          pages: [{ key: 'Знак_ударения', title: 'Знак ударения' }],
        }),
      });
    });

    await page.route('**://*.wikidata.org/w/api.php**', async (route) => {
      await route.fulfill({
        status: 200,
        contentType: 'application/json',
        body: JSON.stringify({
          search: [
            {
              id: 'Q12892367',
              label: 'what',
              description: 'interrogative pronoun or question',
              concepturi: 'https://www.wikidata.org/wiki/Q12892367',
              match: { type: 'label', language: 'ru', text: 'что' },
              aliases: ['что'],
            },
          ],
        }),
      });
    });

    const last = await sendPrompt(page, 'что такое что');
    await expect(last).toHaveClass(/assistant/);
    await expect(last).toContainText('what');
    await expect(last).toContainText('interrogative pronoun');
    await expect(last).toContainText('wikidata.org');
    await expect(last).not.toContainText('Знак ударения');
    await expect(last).not.toContainText(UNKNOWN_ANSWER_MARKER);
  });

  test('unrelated Wikipedia search hits are rejected when exact term fallbacks miss', async ({ page }) => {
    await page.route('**/api/rest_v1/page/summary/**', async (route) => {
      const slug = decodeURIComponent(route.request().url().split('/').pop() || '');
      if (slug === 'Знак_ударения') {
        await route.fulfill({
          status: 200,
          contentType: 'application/json',
          body: JSON.stringify({
            title: 'Знак ударения',
            type: 'standard',
            extract: 'Знак ударения — небуквенный орфографический знак.',
            content_urls: {
              desktop: { page: 'https://ru.wikipedia.org/wiki/Знак_ударения' },
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

    await page.route('**/rest.php/v1/search/page**', async (route) => {
      await route.fulfill({
        status: 200,
        contentType: 'application/json',
        body: JSON.stringify({
          pages: [{ key: 'Знак_ударения', title: 'Знак ударения' }],
        }),
      });
    });
    await page.route('**://*.wikidata.org/w/api.php**', async (route) => {
      await route.fulfill({
        status: 200,
        contentType: 'application/json',
        body: JSON.stringify({ search: [] }),
      });
    });
    await page.route('**://*.wiktionary.org/w/api.php**', async (route) => {
      await route.fulfill({
        status: 200,
        contentType: 'application/json',
        body: JSON.stringify(['что', [], [], []]),
      });
    });

    const last = await sendPrompt(page, 'что такое что');
    await expect(last).toHaveClass(/assistant/);
    await expect(last).toContainText(/не могу ответить|cannot answer/i);
    await expect(last).not.toContainText('Знак ударения');
  });

  test('word lookup falls back to Wiktionary when Wikipedia and Wikidata miss', async ({ page }) => {
    await page.route('**/api/rest_v1/page/summary/**', async (route) => {
      await route.fulfill({
        status: 404,
        contentType: 'application/json',
        body: JSON.stringify({ httpCode: 404, httpReason: 'Not Found' }),
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
        body: JSON.stringify({ search: [] }),
      });
    });
    await page.route('**://*.wiktionary.org/w/api.php**', async (route) => {
      await route.fulfill({
        status: 200,
        contentType: 'application/json',
        body: JSON.stringify([
          'flibbertigibbet',
          ['flibbertigibbet'],
          ['a frivolous, flighty person'],
          ['https://en.wiktionary.org/wiki/flibbertigibbet'],
        ]),
      });
    });

    const last = await sendPrompt(page, 'what is flibbertigibbet');
    await expect(last).toHaveClass(/assistant/);
    await expect(last).toContainText('flibbertigibbet');
    await expect(last).toContainText('frivolous');
    await expect(last).toContainText('wiktionary.org');
    await expect(last).not.toContainText(UNKNOWN_ANSWER_MARKER);
  });

});

test.describe('Issue #82: assistant behavior settings', () => {
  test.beforeEach(async ({ page }) => {
    await disableGreetingVariations(page);
    await page.goto('./');
    await expect(page.locator('.app')).toBeVisible({ timeout: 15_000 });
    await switchToManualMode(page);
  });

  test('settings sidebar exposes ambiguity, temperature, language, skins, and location controls', async ({ page }) => {
    const settings = page.locator('[data-testid="sidebar-settings"]');
    await expect(settings).toBeVisible();
    await expect(page.locator('[data-testid="setting-guess-probability"]')).toBeVisible();
    await expect(page.locator('[data-testid="setting-follow-up-probability"]')).toBeVisible();
    await expect(page.locator('[data-testid="setting-temperature"]')).toBeVisible();
    await expect(page.locator('[data-testid="setting-definition-fusion"]')).toBeVisible();
    await expect(page.locator('[data-testid="setting-ui-language"]')).toBeVisible();
    await expect(page.locator('[data-testid="setting-theme"]')).toBeVisible();
    await expect(page.locator('[data-testid="setting-ui-skin"]')).toBeVisible();
    await expect(page.locator('[data-testid="setting-chat-style"]')).toBeVisible();
    await expect(page.locator('[data-testid="setting-assistant-name"]')).toBeVisible();
    await expect(page.locator('[data-testid="setting-location"]')).toBeVisible();

    await setRangeValue(page, 'setting-temperature', 0);
    await setRangeValue(page, 'setting-follow-up-probability', 0);
    await page.locator('[data-testid="setting-definition-fusion"]').selectOption('auto');
    await page.locator('[data-testid="setting-assistant-name"]').fill('Astra');
    await page.locator('[data-testid="setting-location"]').fill('Berlin');
    await page.locator('[data-testid="setting-theme"]').selectOption('dark');

    await expect.poll(() =>
      page.evaluate(() => window.localStorage.getItem('formal-ai.preferences.v1') || ''),
    ).toContain('theme "dark"');
    const stored = await page.evaluate(() =>
      window.localStorage.getItem('formal-ai.preferences.v1') || '',
    );
    expect(stored).toContain('temperature "0"');
    expect(stored).toContain('followUpProbability "0"');
    expect(stored).toContain('definitionFusion "auto"');
    expect(stored).toContain('assistantName "Astra"');
    expect(stored).toContain('location "Berlin"');
    await expect(page.locator('html')).toHaveAttribute('data-theme', 'dark');
  });

  test('low ambiguity guessing asks before using a fuzzy Wikipedia match', async ({ page }) => {
    await setRangeValue(page, 'setting-guess-probability', 0);

    await page.route('**/api/rest_v1/page/summary/**', async (route) => {
      const slug = decodeURIComponent(route.request().url().split('/').pop() || '');
      if (slug === 'Грамматика') {
        await route.fulfill({
          status: 200,
          contentType: 'application/json',
          body: JSON.stringify({
            title: 'Грамматика',
            type: 'standard',
            extract: 'Грамматика — раздел лингвистики, изучающий грамматический строй языка.',
            content_urls: {
              desktop: { page: 'https://ru.wikipedia.org/wiki/Грамматика' },
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

    await page.route('**/rest.php/v1/search/page**', async (route) => {
      await route.fulfill({
        status: 200,
        contentType: 'application/json',
        body: JSON.stringify({
          pages: [{ key: 'Грамматика', title: 'Грамматика' }],
        }),
      });
    });

    const last = await sendPrompt(page, 'что такое граматика');
    await expect(last).toHaveClass(/assistant/);
    await expect(last).toContainText(/Грамматика/);
    await expect(last).toContainText(/уточните|Did you mean/i);
    await expect(last).not.toContainText('раздел лингвистики');
  });
});

test.describe('memory export/import', () => {
  test.beforeEach(async ({ page }) => {
    await disableGreetingVariations(page);
    await page.goto('./');
    await expect(page.locator('.app')).toBeVisible({ timeout: 15_000 });
    await switchToManualMode(page);
  });

  test('Export memory and Import memory buttons are present', async ({ page }) => {
    await expect(page.locator('[data-testid="memory-export"]')).toBeVisible();
    await expect(page.locator('[data-testid="memory-import"]')).toBeVisible();
  });

  test('Export memory downloads a full formal_ai_bundle by default (R109)', async ({ page }) => {
    // Send one message so there is at least one event in the log.
    await sendPrompt(page, 'Hi');

    const [download] = await Promise.all([
      page.waitForEvent('download'),
      page.locator('[data-testid="memory-export"]').click(),
    ]);

    expect(download.suggestedFilename()).toBe('formal-ai-memory.lino');

    const path = await download.path();
    expect(path).toBeTruthy();
    const fs = require('node:fs');
    const text = fs.readFileSync(path, 'utf8');
    // R109: the default export is now the full self-contained bundle —
    // seed files + UI preferences + environment metadata + the embedded
    // demo_memory log. The user must not have to click a second button to
    // get the full state.
    expect(text.startsWith('formal_ai_bundle\n')).toBe(true);
    expect(text).toContain('seed_files');
    expect(text).toContain('seed/agent-info.lino');
    expect(text).toContain('preferences');
    expect(text).toContain('demo_memory');
    expect(text).toContain('role "user"');
    expect(text).toContain('content "Hi"');
    // Status indicator should reflect the full-memory shape.
    await expect(page.locator('[data-testid="memory-status"]')).toContainText(/Exported full memory:/);
  });

  test('Import memory accepts a Links Notation file', async ({ page }) => {
    const importInput = page.locator('[data-testid="memory-import-input"]');
    const lino = [
      'demo_memory',
      '  event "1"',
      '    role "user"',
      '    content "Imported greeting"',
      '    sentAt "2026-05-15T12:00:00.000Z"',
      '  event "2"',
      '    role "assistant"',
      '    intent "greeting"',
      '    content "Hi, how may I help you?"',
      '    sentAt "2026-05-15T12:00:01.000Z"',
      '',
    ].join('\n');
    await importInput.setInputFiles({
      name: 'memory.lino',
      mimeType: 'text/plain',
      buffer: Buffer.from(lino, 'utf8'),
    });
    // R110: legacy demo_memory imports must still succeed. R111: importing a
    // legacy log surfaces a migration suggestion because no seed metadata is
    // attached, so the status indicator reports "Migration: ..." alongside
    // the import count.
    await expect(page.locator('[data-testid="memory-status"]')).toContainText('Imported 2 events');
    await expect(page.locator('[data-testid="memory-status"]')).toContainText(/Migration:.*legacy demo_memory/);
  });

  test('Import memory accepts a formal_ai_bundle and reports seed migrations (R110, R111)', async ({ page }) => {
    const importInput = page.locator('[data-testid="memory-import-input"]');
    const bundle = [
      'formal_ai_bundle',
      '  exported_at "2026-05-15T12:00:00.000Z"',
      '  version "0.0.1"',
      '  seed_files',
      '    file "seed/agent-info.lino"',
      '      agent_info',
      '        field "version"',
      '          value "0.0.1"',
      '  preferences',
      '    demo_mode "off"',
      '  demo_memory',
      '    event "1"',
      '      role "user"',
      '      content "Imported via bundle"',
      '      sentAt "2026-05-15T12:00:00.000Z"',
      '',
    ].join('\n');
    await importInput.setInputFiles({
      name: 'bundle.lino',
      mimeType: 'text/plain',
      buffer: Buffer.from(bundle, 'utf8'),
    });
    await expect(page.locator('[data-testid="memory-status"]')).toContainText('Imported 1 event(s) from full bundle');
    await expect(page.locator('[data-testid="memory-status"]')).toContainText(/Migration: Seed version 0\.0\.1 →/);
  });

  test('Memory module exposes explicit destructive operations only', async ({ page }) => {
    const api = await page.evaluate(() => Object.keys(window.FormalAiMemory || {}));
    expect(api).toContain('appendEvent');
    expect(api).toContain('listEvents');
    expect(api).toContain('importEvents');
    expect(api).toContain('exportLinksNotation');
    expect(api).toContain('exportBundle');
    // R109/R110/R111: full-memory export, header-agnostic import, and
    // migration suggestions must all be reachable from the public API.
    expect(api).toContain('exportFullMemory');
    expect(api).toContain('importFullMemory');
    expect(api).toContain('suggestMigrations');
    expect(api).toContain('purgeDeletedConversations');
    expect(api).toContain('deleteEventsByConversationId');
    expect(api).toContain('clearEvents');
    expect(api).not.toContain('delete');
    expect(api).not.toContain('deleteEvent');
    expect(api).not.toContain('forget');
    expect(api).not.toContain('clear');
    expect(api).not.toContain('remove');
  });

  test('Issue #27: Download bundle button is removed (duplicate of Export memory)', async ({ page }) => {
    await expect(page.locator('[data-testid="memory-bundle"]')).toHaveCount(0);
    // The underlying exportBundle helper must remain on the public API for
    // Rust/CLI parity; only the redundant UI button is gone.
    const api = await page.evaluate(() => Object.keys(window.FormalAiMemory || {}));
    expect(api).toContain('exportBundle');
  });

  test('Issue #27: Export memory does not surface a "Bundled N events + seed" label', async ({ page }) => {
    await sendPrompt(page, 'Hi');
    const [download] = await Promise.all([
      page.waitForEvent('download'),
      page.locator('[data-testid="memory-export"]').click(),
    ]);
    expect(download.suggestedFilename()).toBe('formal-ai-memory.lino');
    const status = await page.locator('[data-testid="memory-status"]').innerText();
    expect(status).not.toMatch(/bundled\s+\d+\s+events\s+\+\s+seed/i);
  });

  test('Issue #27: typing "Export memory" triggers the export button', async ({ page }) => {
    const input = page.locator('[data-testid="chat-composer-input"]');
    await expect(input).toBeEnabled({ timeout: 5_000 });
    await input.fill('Export memory');
    const [download] = await Promise.all([
      page.waitForEvent('download'),
      page.locator('[data-testid="chat-composer-submit"]').click(),
    ]);
    expect(download.suggestedFilename()).toBe('formal-ai-memory.lino');
    const messages = page.locator('[data-testid="chat-message"]');
    await expect(messages.last()).toContainText('Triggered Export memory');
  });

  test('Issue #27: typing "Export your memory" also triggers the export button', async ({ page }) => {
    const input = page.locator('[data-testid="chat-composer-input"]');
    await expect(input).toBeEnabled({ timeout: 5_000 });
    await input.fill('Export your memory');
    const [download] = await Promise.all([
      page.waitForEvent('download'),
      page.locator('[data-testid="chat-composer-submit"]').click(),
    ]);
    expect(download.suggestedFilename()).toBe('formal-ai-memory.lino');
  });

  test('Issue #27: typing "Import memory" opens the file picker', async ({ page }) => {
    const input = page.locator('[data-testid="chat-composer-input"]');
    await expect(input).toBeEnabled({ timeout: 5_000 });
    // We cannot programmatically observe a native file dialog opening, but we
    // can confirm the assistant acknowledges the trigger and the file input
    // remains in the DOM ready to accept a file.
    await input.fill('Import memory');
    await page.locator('[data-testid="chat-composer-submit"]').click();
    const messages = page.locator('[data-testid="chat-message"]');
    await expect(messages.last()).toContainText('Triggered Import memory');
    await expect(page.locator('[data-testid="memory-import-input"]')).toHaveCount(1);
  });

  test('Issue #196: reset memory phrases are recognised in every supported language', async ({ page }) => {
    const resetPromptCases = [
      { language: 'en', phrase: 'Reset memory' },
      { language: 'ru', phrase: 'сбросить память' },
      { language: 'hi', phrase: 'स्मृति रीसेट करें' },
      { language: 'zh', phrase: '重置记忆' },
    ];
    const dialogs = [];
    page.on('dialog', async (dialog) => {
      dialogs.push(dialog.message());
      if (dialogs.length % 2 === 1) {
        await dialog.dismiss();
      } else {
        await dialog.accept();
      }
    });

    for (const { language, phrase } of resetPromptCases) {
      await sendPrompt(page, `Memory reset seed ${language}`);
      const input = page.locator('[data-testid="chat-composer-input"]');
      await expect(input).toBeEnabled({ timeout: 5_000 });
      await input.fill(phrase);
      await page.locator('[data-testid="chat-composer-submit"]').click();
      await expect(page.locator('[data-testid="chat-message"]')).toHaveCount(0);
      await expect.poll(() =>
        page.evaluate(() =>
          window.FormalAiMemory.listEvents().then((events) => events.length),
        ),
      ).toBe(0);
    }

    expect(dialogs.length).toBe(resetPromptCases.length * 2);
  });

  test('Report issue link is present in the topbar and links to the upload-memory guide (R112 + issue #78)', async ({ page }) => {
    const reportLink = page.locator('[data-testid="report-issue"]');
    await expect(reportLink).toBeVisible();
    const href = await reportLink.getAttribute('href');
    expect(href).toBeTruthy();
    const url = new URL(href);
    expect(url.origin + url.pathname).toBe('https://github.com/link-assistant/formal-ai/issues/new');
    const body = url.searchParams.get('body') || '';
    // Issue #78: the prefilled body must stay short. It still mentions the
    // export filename, the Export memory action, .zip / Gist upload paths, and
    // redaction — but only in one line that links to docs/upload-memory.md for
    // the full walkthrough (R112).
    expect(body).toContain('formal-ai-memory.lino');
    expect(body).toContain('Export memory');
    expect(body).toMatch(/\.zip/);
    expect(body).toMatch(/redact/i);
    expect(body).toContain('docs/upload-memory.md');
    // The long block of per-OS zip instructions that used to live in the body
    // must be gone (it has moved into docs/upload-memory.md so a single link
    // is enough).
    expect(body).not.toMatch(/Send to.*Compressed/);
    expect(body).not.toMatch(/right-click.*Compress/);
  });

  test('Tool registry surfaces seed-loaded tools with mode badges', async ({ page }) => {
    const registry = page.locator('[data-testid="tool-registry"]');
    await expect(registry).toBeVisible({ timeout: 10_000 });
    const entries = page.locator('[data-testid="tool-entry"]');
    await expect(entries.first()).toBeVisible();
    const count = await entries.count();
    expect(count).toBeGreaterThan(0);
    const modes = await entries.evaluateAll((nodes) =>
      nodes.map((node) => node.getAttribute('data-tool-mode')),
    );
    expect(modes).toContain('thinking');
    await expect(registry).toContainText('calculator');
  });

  test('Issue #112: tool registry includes all supported tools and localizes descriptions', async ({ page }) => {
    await page.locator('[data-testid="setting-ui-language"]').selectOption('ru');
    const entries = page.locator('[data-testid="tool-entry"]');
    const toolIds = await entries.evaluateAll((nodes) =>
      nodes.map((node) => node.getAttribute('data-tool-id')),
    );
    expect(toolIds).toEqual(expect.arrayContaining([
      'tool_http_fetch',
      'tool_url_navigate',
      'tool_web_search',
      'tool_wikipedia_lookup',
      'tool_calculator',
      'tool_eval_js',
      'tool_read_local_file',
      'tool_append_memory',
      'tool_export_memory',
      'tool_import_memory',
      'tool_conversation_recall',
      'tool_concept_lookup',
      'tool_write_program',
      'tool_intent_routing',
      'tool_fact_lookup',
      'tool_summarize_conversation',
      'tool_brainstorm',
      'tool_coreference',
      'tool_roleplay',
    ]));
    await expect(page.locator('[data-tool-id="tool_calculator"] .tool-desc')).toContainText(/Вычисляет|математические/);
    await expect(page.locator('[data-tool-id="tool_web_search"] .tool-desc')).not.toContainText('Search the open web');
  });

  test('Reasoning steps and tool calls land in the append-only log', async ({ page }) => {
    await sendPrompt(page, 'Hi');
    const events = await page.evaluate(async () => {
      const list = await window.FormalAiMemory.listEvents();
      return list.map((event) => ({ kind: event.kind, role: event.role }));
    });
    const kinds = new Set(events.map((event) => event.kind).filter(Boolean));
    expect(kinds.has('message')).toBe(true);
    expect(kinds.has('reasoning')).toBe(true);
  });
});

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
    await expect(input).toBeEnabled({ timeout: 5_000 });
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
    await page.locator('[data-testid="conversation-new"]').click();
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
    const allSections = page.locator(
      '[data-testid="context-panel"] .sidebar-section',
    );
    const initialExpanded = await page
      .locator('[data-testid="context-panel"] .sidebar-section.is-expanded')
      .count();
    if (initialExpanded < 2) test.skip();

    const initialOther = await allSections
      .nth(1)
      .locator('.sidebar-section-body')
      .boundingBox();

    // Collapse the first section by clicking its header button.
    await allSections.first().locator('.sidebar-section-header').click();
    await expect(allSections.first()).toHaveAttribute('data-collapsed', 'true');

    const grownOther = await allSections
      .nth(1)
      .locator('.sidebar-section-body')
      .boundingBox();
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
    await page.locator('[data-testid="conversation-new"]').click();
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
});
