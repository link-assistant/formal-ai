// @ts-check
// Multilingual chat surface: greetings, identity, fact queries, calendar, and
// the rest of the per-language prompt matrix.
// (Split from the former multilingual.spec.js; shared helpers live in
// ./support/multilingual.js.)
const { test, expect } = require('@playwright/test');
const {
  UNKNOWN_ANSWER_MARKER,
  TEN_POW_100,
  switchToManualMode,
  setUiLanguage,
  disableGreetingVariations,
  sendPrompt,
  routeFramePolicy,
  emulateReducedMotion,
} = require('./support/multilingual');

// Issue #541 (R5/R6): emulate prefers-reduced-motion so answers render at
// once instead of racing the paced reveal (see support/multilingual.js).
emulateReducedMotion(test);

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

  // Issue #676: "how are you?" small talk now has its own wellbeing reply,
  // distinct from a bare greeting, across every supported language.
  test('how-are-you small talk replies with wellbeing across languages', async ({ page }) => {
    const cases = [
      { prompt: 'How are you?', answer: /doing great|doing well|All good|ready to help/ },
      { prompt: 'Как твои дела?', answer: /хорошо|отлично|Готов помочь|полезен/ },
      { prompt: 'आप कैसे हैं?', answer: /बढ़िया|ठीक|अच्छा|मदद/ },
      { prompt: '你好吗?', answer: /我很好|我挺好|一切都好|谢谢/ },
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

  test('large integer exponent renders exactly across supported UI languages', async ({
    page,
  }) => {
    const cases = [
      { language: 'en', name: 'English', prompt: '10^100' },
      { language: 'ru', name: 'Russian', prompt: '10^100' },
      { language: 'hi', name: 'Hindi', prompt: '10^100' },
      { language: 'zh', name: 'Chinese', prompt: '10^100' },
    ];

    for (const { language, name, prompt } of cases) {
      await setUiLanguage(page, language);
      const last = await sendPrompt(page, prompt);
      await expect(last, name).toHaveClass(/assistant/);
      await expect(last, name).toContainText(`${prompt} = ${TEN_POW_100}`);
      await expect(last, name).not.toContainText('1e+1');
    }
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
      {
        prompt: 'what is OpenStreerMap',
        closestMatchNote: 'Closest match from Wikipedia search',
      },
      {
        prompt: 'что такое OpenStreerMap',
        closestMatchNote: 'Ближайшее совпадение по поиску Wikipedia',
      },
      {
        prompt: 'OpenStreerMap क्या है',
        closestMatchNote: 'Wikipedia खोज में सबसे नज़दीकी मिलान',
      },
      {
        prompt: 'OpenStreerMap是什么',
        closestMatchNote: 'Wikipedia 搜索的最接近匹配',
      },
    ];

    for (const { prompt, closestMatchNote } of cases) {
      const before = apiCalls.length;
      const last = await sendPrompt(page, prompt);
      const calls = apiCalls.slice(before);
      await expect(last).toHaveClass(/assistant/);
      await expect(last).toContainText('OpenStreetMap');
      await expect(last).toContainText('collaborative map database');
      await expect(last).toContainText('wikipedia.org');
      await expect(last).toContainText(closestMatchNote);
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
