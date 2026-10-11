// @ts-check
// Wikipedia REST fallback: offline + Wikipedia `What is X?` resolution and
// definition disambiguation across languages.
// (Split from the former multilingual.spec.js; shared helpers live in
// ./support/multilingual.js.)
const { test, expect } = require('@playwright/test');
const {
  UNKNOWN_ANSWER_MARKER,
  definitionDisambiguationCases,
  routeDefinitionDisambiguationCase,
  switchToManualMode,
  sendPrompt,
  emulateReducedMotion,
} = require('./support/multilingual');

// Issue #541 (R5/R6): emulate prefers-reduced-motion so answers render at
// once instead of racing the paced reveal (see support/multilingual.js).
emulateReducedMotion(test);

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
