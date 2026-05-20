// @ts-check
const { test, expect } = require('@playwright/test');

// Issue #153 — search UX, formalization SVO view, cross-source dedupe, and
// localized search-result templates. Each test mocks the three providers so
// the assertions are hermetic.

const UNKNOWN_ANSWER_MARKER = 'cannot answer that from local Links Notation rules';

async function disableGreetingVariations(page) {
  await page.addInitScript(() => {
    try {
      window.localStorage.setItem(
        'formal-ai.preferences.v1',
        'demo_preferences\n  greetingVariations "off"',
      );
    } catch (_error) {
      // ignore — localStorage may be unavailable in some sandboxes.
    }
  });
}

async function switchToManualMode(page) {
  const demoToggle = page.locator('.mode-toggle');
  // Wait for i18n hydration to finish before reading the label — on a slow
  // CI worker the catalog is still resolving when the locator first appears,
  // and we briefly see `buttons.demoOn` instead of the translated text.
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

async function mockAppleSearch(page) {
  await page.route('**://api.duckduckgo.com/**', async (route) => {
    await route.fulfill({
      status: 200,
      contentType: 'application/json',
      body: JSON.stringify({
        Heading: 'Apple',
        AbstractText: 'Apple is a fruit produced by an apple tree.',
        AbstractURL: 'https://duckduckgo.com/Apple',
        RelatedTopics: [],
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
            id: 856,
            key: 'Apple',
            title: 'Apple',
            excerpt: 'Apple is the edible fruit of an apple tree.',
            description: 'fruit and species of plant',
          },
        ],
      }),
    });
  });
  // Wikidata: respond with the same single-entity payload regardless of which
  // call path (fact_query or web_search) triggered the request. Both code
  // paths use `action=wbsearchentities`; the web_search path additionally
  // asks for `props=sitelinks/urls`, but we always include the sitelink so
  // the dedupe logic has the data it needs even when fact_query is the
  // first caller.
  await page.route('**://www.wikidata.org/w/api.php**', async (route) => {
    await route.fulfill({
      status: 200,
      contentType: 'application/json',
      body: JSON.stringify({
        search: [
          {
            id: 'Q89',
            label: 'Apple',
            description: 'fruit of the apple tree',
            concepturi: 'https://www.wikidata.org/wiki/Q89',
            sitelinks: {
              enwiki: {
                site: 'enwiki',
                title: 'Apple',
                url: 'https://en.wikipedia.org/wiki/Apple',
              },
            },
          },
        ],
      }),
    });
  });
}

test.describe('Issue #153 — search UX, formalization, and dedupe', () => {
  test.beforeEach(async ({ page }) => {
    await disableGreetingVariations(page);
    await page.goto('./');
    await expect(page.locator('.app')).toBeVisible({ timeout: 15_000 });
    await switchToManualMode(page);
  });

  test('top menu uses a lab emoji for the diagnostics toggle', async ({ page }) => {
    const diagnosticsToggle = page.locator('.diagnostics-toggle');
    await expect(diagnosticsToggle).toBeVisible();
    await expect(diagnosticsToggle.locator('.btn-icon')).toHaveText('🧪');
  });

  test('top menu links to the GitHub source code', async ({ page }) => {
    const link = page.locator('[data-testid="source-code"]');
    await expect(link).toBeVisible();
    await expect(link).toHaveAttribute(
      'href',
      'https://github.com/link-assistant/formal-ai',
    );
  });

  test('"New conversation" button is disabled until the chat has content', async ({
    page,
  }) => {
    const newBtn = page.locator('[data-testid="conversation-new"]');
    // Reach the canonical zero-state. Demo mode may have added a greeting
    // turn before `switchToManualMode` flipped it off; clicking
    // "New conversation" is exactly the operation that clears it.
    if (await newBtn.isEnabled().catch(() => false)) {
      await newBtn.click();
    }
    await expect(newBtn).toBeDisabled();
    await sendPrompt(page, 'Hi');
    await expect(newBtn).toBeEnabled();
  });

  test('sidebar can be collapsed and expanded', async ({ page }) => {
    await page.setViewportSize({ width: 1440, height: 900 });
    const sidebarToggle = page.locator('[data-testid="sidebar-toggle"]');
    await expect(sidebarToggle).toBeVisible();
    const workspace = page.locator('.workspace');
    await sidebarToggle.click();
    await expect(workspace).toHaveClass(/sidebar-collapsed/);
    const collapsedLayout = await page.evaluate(() => {
      const box = (selector) => {
        const rect = document.querySelector(selector).getBoundingClientRect();
        return {
          left: Math.round(rect.left),
          right: Math.round(rect.right),
          width: Math.round(rect.width),
        };
      };
      return {
        viewportWidth: window.innerWidth,
        chat: box('.chat-panel'),
        composer: box('.composer'),
      };
    });
    expect(collapsedLayout.chat.left).toBeLessThanOrEqual(1);
    expect(collapsedLayout.chat.right).toBeGreaterThanOrEqual(
      collapsedLayout.viewportWidth - 1,
    );
    expect(collapsedLayout.composer.left).toBeLessThanOrEqual(1);
    expect(collapsedLayout.composer.right).toBeGreaterThanOrEqual(
      collapsedLayout.viewportWidth - 1,
    );
    await sidebarToggle.click();
    await expect(workspace).not.toHaveClass(/sidebar-collapsed/);
  });

  test('left menu actions section can be collapsed like the other sidebar sections', async ({
    page,
  }) => {
    await page.setViewportSize({ width: 1440, height: 900 });
    const menu = page.locator('[data-testid="drawer-menu-actions"]');
    await expect(menu).toBeVisible();
    await expect(menu).toHaveAttribute('data-collapsed', 'false');
    await expect(menu.locator('[data-testid="drawer-source-code"]')).toBeVisible();

    await menu.locator('.sidebar-section-header').click();

    await expect(menu).toHaveAttribute('data-collapsed', 'true');
    await expect(menu.locator('[data-testid="drawer-source-code"]')).toHaveCount(0);
  });

  test('diagnostics shows the SVO formalization view with @USER + OP:* + Q-id', async ({
    page,
  }) => {
    await mockAppleSearch(page);
    await page.locator('.diagnostics-toggle').click();

    const last = await sendPrompt(page, 'Search the web for Apple');
    // Diagnostic steps render each formalization view inside a collapsed
    // <details>. Open them all so the SVO slots are visible to the assertions.
    await last.evaluate((node) => {
      for (const det of node.querySelectorAll('details.diagnostics-detail')) {
        det.open = true;
      }
    });
    const formalizationViews = last.locator('[data-testid="formalization"]');
    await expect(formalizationViews.first()).toBeVisible({ timeout: 10_000 });

    // The placeholder formalize step uses the bare query as the object slot.
    const first = formalizationViews.first();
    await expect(first).toContainText('@USER');
    await expect(first).toContainText('OP:search');
    await expect(first.locator('.formalization-slot').nth(0)).toHaveText('S');
    await expect(first.locator('.formalization-slot').nth(1)).toHaveText('V');
    await expect(first.locator('.formalization-slot').nth(2)).toHaveText('O');

    // The resolved formalize step must fold the Wikidata Q-id back into the
    // SVO tuple alongside the original placeholder.
    const resolved = formalizationViews.nth(1);
    await expect(resolved).toBeVisible();
    await expect(resolved).toContainText('Q89');
    await expect(resolved.locator('.formalization-tuple')).toContainText(
      '(@USER OP:search Q89)',
    );
  });

  test('search results dedupe cross-provider entries by Wikidata sitelinks', async ({
    page,
  }) => {
    await mockAppleSearch(page);
    await page.locator('.diagnostics-toggle').click();

    const last = await sendPrompt(page, 'Search the web for Apple');

    await expect(last).toContainText('Search results for');
    // Wikidata returned a `Q89` Q-id and the Wikipedia provider returned the
    // same entity via `enwiki: Apple` — they collapse into one bullet.
    await expect(last).toContainText('Q89');
    await expect(last).toContainText('Other sources');

    // Evidence trail must list the deduplication and the formalized id.
    const evidence = last.locator('.evidence-list');
    await expect(evidence).toContainText('web_search:dedupe:Q:Q89');
    await expect(evidence).toContainText('web_search:formal:1:Q89');
  });

  test('DuckDuckGo provider contributes results (regression: signature mismatch)', async ({
    page,
  }) => {
    // Issue #153: searchDuckDuckGo was declared as (query, limit) but the
    // dispatcher calls every provider as (query, language, providerLimit).
    // Verify the fix by mocking only DDG and asserting it shows up in the
    // ranked output.
    await page.route('**://api.duckduckgo.com/**', async (route) => {
      await route.fulfill({
        status: 200,
        contentType: 'application/json',
        body: JSON.stringify({
          Heading: 'Geometric Tesseract',
          AbstractText: 'A tesseract is the four-dimensional analogue of a cube.',
          AbstractURL: 'https://duckduckgo.com/Tesseract',
          RelatedTopics: [
            {
              FirstURL: 'https://duckduckgo.com/Hypercube',
              Text: 'Hypercube - generalised cube',
            },
          ],
        }),
      });
    });
    await page.route('**://*.wikipedia.org/w/rest.php/v1/search/page**', async (route) => {
      await route.fulfill({
        status: 200,
        contentType: 'application/json',
        body: JSON.stringify({ pages: [] }),
      });
    });
    await page.route('**://www.wikidata.org/w/api.php**', async (route) => {
      await route.fulfill({
        status: 200,
        contentType: 'application/json',
        body: JSON.stringify({ search: [] }),
      });
    });

    await page.locator('.diagnostics-toggle').click();
    const last = await sendPrompt(page, 'Search the web for Geometric Tesseract');

    await expect(last).toContainText('Geometric Tesseract');
    await expect(last).toContainText('duckduckgo');
    await expect(last.locator('.evidence-list')).toContainText(
      'web_search:provider:duckduckgo:count:2',
    );
  });

  test('priority-based topbar overflow never clips actions and keeps Bug reporting visible at 375px', async ({
    page,
  }) => {
    // Issue #153: every .topbar-actions button must carry a data-menu-priority
    // and lower-priority actions must leave the topbar before any button is
    // clipped. The full action list must still be reachable from the sidebar
    // Menu on desktop and via the hamburger drawer on mobile.
    const expectVisible = async (selector) => {
      await expect(page.locator(selector)).toBeVisible();
    };
    const expectHidden = async (selector) => {
      // `display: none !important` removes the element from layout; Playwright
      // reports `not visible` (not `not attached`).
      await expect(page.locator(selector)).toBeHidden();
    };
    const requireAttr = async (selector, value) => {
      await expect(page.locator(selector).first()).toHaveAttribute(
        'data-menu-priority',
        value,
      );
    };

    // Sanity-check every priority assignment at desktop width first.
    await requireAttr('.topbar-actions [data-testid="report-issue"]', '1');
    await requireAttr('.topbar-actions .diagnostics-toggle', '2');
    await requireAttr('.topbar-actions .mode-toggle', '3');
    await requireAttr('.topbar-actions [data-testid="agent-toggle"]', '4');
    await requireAttr('.topbar-actions [data-testid="source-code"]', '5');
    await requireAttr('.topbar-actions [data-testid="memory-export"]', '6');
    await requireAttr('.topbar-actions [data-testid="memory-import"]', '6');

    const assertTopbarFits = async (width, expectedHiddenPriorities = []) => {
      await page.setViewportSize({
        width,
        height: width <= 500 ? 720 : 900,
      });
      await page.waitForFunction(() => {
        const cssWidth = Number.parseFloat(
          getComputedStyle(document.documentElement).getPropertyValue(
            '--formal-ai-viewport-width',
          ),
        );
        return Math.abs(cssWidth - window.innerWidth) <= 1;
      });
      const layout = await page.evaluate(() => {
        const viewportWidth = window.innerWidth;
        const topbarRect = document.querySelector('.topbar').getBoundingClientRect();
        const visibleActions = [...document.querySelectorAll('.topbar-actions > *')]
          .filter((element) => getComputedStyle(element).display !== 'none')
          .map((element) => {
            const rect = element.getBoundingClientRect();
            return {
              id:
                element.getAttribute('data-testid') ||
                element.className ||
                element.tagName,
              priority: element.getAttribute('data-menu-priority'),
              left: rect.left,
              right: rect.right,
            };
          });
        return {
          viewportWidth,
          topbarLeft: topbarRect.left,
          topbarRight: topbarRect.right,
          visiblePriorities: visibleActions.map((item) => item.priority),
          visibleActions,
        };
      });
      expect(layout.topbarLeft, `topbar left at ${width}px`).toBeGreaterThanOrEqual(-1);
      expect(layout.topbarRight, `topbar right at ${width}px`).toBeLessThanOrEqual(
        layout.viewportWidth + 1,
      );
      for (const item of layout.visibleActions) {
        expect(item.left, `${item.id} left at ${width}px`).toBeGreaterThanOrEqual(-1);
        expect(item.right, `${item.id} right at ${width}px`).toBeLessThanOrEqual(
          layout.viewportWidth + 1,
        );
      }
      for (const priority of expectedHiddenPriorities) {
        expect(layout.visiblePriorities).not.toContain(priority);
      }
    };

    await assertTopbarFits(1440);
    await assertTopbarFits(1320);
    await assertTopbarFits(1180);
    await assertTopbarFits(1040);
    await assertTopbarFits(980);

    const sidebarMenu = page.locator('[data-testid="drawer-menu-actions"]');
    await expect(sidebarMenu).toBeVisible();
    await expect(sidebarMenu).toContainText(/Source code|Исходный код/);
    await expect(sidebarMenu).toContainText(/Export memory|Экспорт памяти/);
    await expect(sidebarMenu).toContainText(/Import memory|Импорт памяти/);

    // Mobile 375 — only priorities 1-3 should remain in the topbar.
    await page.setViewportSize({ width: 375, height: 720 });
    await expectVisible('.topbar-actions [data-testid="report-issue"]');
    await expectVisible('.topbar-actions .diagnostics-toggle');
    await expectVisible('.topbar-actions .mode-toggle');
    await expectHidden('.topbar-actions [data-testid="agent-toggle"]');
    await expectHidden('.topbar-actions [data-testid="source-code"]');
    await expectHidden('.topbar-actions [data-testid="memory-export"]');
    await expectHidden('.topbar-actions [data-testid="memory-import"]');

    // Hamburger drawer continues to expose the full action list.
    await page.locator('[data-testid="mobile-menu-toggle"]').click();
    const drawer = page.locator('.drawer-menu-section');
    await expect(drawer).toBeVisible();
    await expect(drawer).toContainText(/Source code|Исходный код/);
    await expect(drawer).toContainText(/Export memory|Экспорт памяти/);
    await expect(drawer).toContainText(/Import memory|Импорт памяти/);
    await expect(drawer).toContainText(/Chat|Agent|Чат|Агент/);
  });

  test('search header is localized when the UI language is Russian', async ({
    page,
  }) => {
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
    // Re-load to apply the language override (`beforeEach` already navigated).
    await page.goto('./');
    await expect(page.locator('.app')).toBeVisible({ timeout: 15_000 });
    await switchToManualMode(page);
    await mockAppleSearch(page);

    const last = await sendPrompt(page, 'Найди в интернете яблоко');
    await expect(last).toContainText('Результаты поиска для');
    await expect(last).toContainText('Apple');
    await expect(last).not.toContainText(UNKNOWN_ANSWER_MARKER);
  });

  test('Russian information-search phrasing routes to web search', async ({
    page,
  }) => {
    await page.locator('.diagnostics-toggle').click();
    await mockAppleSearch(page);

    const last = await sendPrompt(
      page,
      'Найди информацию о Rust программировании',
    );
    await expect(last).toContainText('Результаты поиска для');
    await expect(last).toContainText('Rust программировании');
    await expect(last.locator('.evidence-list')).toContainText(
      'web_search:request:Rust программировании',
    );
    await expect(last).not.toContainText(UNKNOWN_ANSWER_MARKER);
  });

  test('multilingual information-search phrasings route to web search', async ({
    page,
  }) => {
    await page.locator('.diagnostics-toggle').click();
    await mockAppleSearch(page);

    const cases = [
      {
        prompt: 'Find detailed information about Rust programming',
        query: 'Rust programming',
      },
      {
        prompt: 'Rust programming के बारे में जानकारी खोजो',
        query: 'Rust programming',
      },
      {
        prompt: '查找关于 Rust 编程的信息',
        query: 'Rust 编程',
      },
    ];

    for (const item of cases) {
      const last = await sendPrompt(page, item.prompt);
      await expect(last.locator('.evidence-list')).toContainText(
        `web_search:request:${item.query}`,
      );
      await expect(last).not.toContainText(UNKNOWN_ANSWER_MARKER);
    }
  });
});
