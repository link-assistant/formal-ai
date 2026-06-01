// @ts-check
//
// Issue #335: in agent mode, a composite historical research prompt searched
// Wikipedia with the whole instruction sentence as the query:
// `Nikola Tesla" and "Thomas Edison". Compare their number of patents`.
// Wikipedia REST returns no pages for that broad query, so the browser worker
// rendered "No CORS-enabled web search results" instead of useful evidence.
const { test, expect } = require('@playwright/test');

const REPORTED_PROMPT =
  'Search Wikipedia for "Nikola Tesla" and "Thomas Edison". ' +
  'Compare their number of patents. ' +
  'Then search for information about the "War of Currents" and summarize who won and why.';

async function sendPrompt(page, text) {
  const input = page.locator('[data-testid="chat-composer-input"]');
  await expect(input).toBeEnabled({ timeout: 5_000 });
  await input.fill(text);

  const messages = page.locator('[data-testid="chat-message"]');
  const initialCount = await messages.count();
  await page.locator('[data-testid="chat-composer-submit"]').click();
  await expect(messages).toHaveCount(initialCount + 2, { timeout: 20_000 });

  const assistantMessage = messages.last();
  await expect(assistantMessage).toHaveClass(/assistant/);
  await expect(assistantMessage.locator('.markdown-body')).toBeVisible();
  return assistantMessage;
}

function searchFixtures(query) {
  const normalized = String(query || "").toLowerCase().replace(/\s+/g, " ").trim();
  if (!normalized || normalized === "ping") return [];
  if (normalized.includes("compare their number of patents")) return [];
  if (normalized.includes("war of currents")) {
    return [
      {
        title: "War of the currents",
        key: "War_of_the_currents",
        url: "https://en.wikipedia.org/wiki/War_of_the_currents",
        excerpt:
          "The war of the currents was a series of events surrounding competing AC and DC electric power transmission systems.",
        description: "1880s-1890s electric power transmission dispute",
        qid: "Q1503671",
      },
    ];
  }
  if (normalized.includes("nikola tesla") && normalized.includes("thomas edison")) {
    return [
      {
        title: "Thomas Edison",
        key: "Thomas_Edison",
        url: "https://en.wikipedia.org/wiki/Thomas_Edison",
        excerpt:
          "Thomas Edison accumulated 2,332 patents worldwide, including 1,093 United States patents.",
        description: "American inventor and businessman",
        qid: "Q8743",
      },
      {
        title: "List of Nikola Tesla patents",
        key: "List_of_Nikola_Tesla_patents",
        url: "https://en.wikipedia.org/wiki/List_of_Nikola_Tesla_patents",
        excerpt:
          "Nikola Tesla obtained around 300 patents worldwide for his inventions.",
        description: "patent list",
        qid: "Q6645839",
      },
    ];
  }
  if (normalized.includes("nikola tesla")) {
    return [
      {
        title: "List of Nikola Tesla patents",
        key: "List_of_Nikola_Tesla_patents",
        url: "https://en.wikipedia.org/wiki/List_of_Nikola_Tesla_patents",
        excerpt:
          "Nikola Tesla obtained around 300 patents worldwide for his inventions.",
        description: "patent list",
        qid: "Q6645839",
      },
    ];
  }
  if (normalized.includes("thomas edison")) {
    return [
      {
        title: "List of Edison patents",
        key: "List_of_Edison_patents",
        url: "https://en.wikipedia.org/wiki/List_of_Edison_patents",
        excerpt:
          "Thomas Edison accumulated 2,332 patents worldwide. 1,093 of Edison's patents were in the United States.",
        description: "patent list",
        qid: "Q6645844",
      },
    ];
  }
  return [];
}

async function mockHistoricalSearchProviders(page) {
  await page.route('**://api.duckduckgo.com/**', async (route) => {
    const url = new URL(route.request().url());
    const results = searchFixtures(url.searchParams.get('q') || "");
    const primary = results[0];
    await route.fulfill({
      status: 200,
      contentType: 'application/json',
      body: JSON.stringify({
        Heading: primary ? primary.title : "",
        AbstractText: primary ? primary.excerpt : "",
        AbstractURL: primary ? primary.url : "",
        RelatedTopics: [],
      }),
    });
  });

  await page.route('**://archive.org/advancedsearch.php**', async (route) => {
    const url = new URL(route.request().url());
    const results = searchFixtures(url.searchParams.get('q') || "");
    await route.fulfill({
      status: 200,
      contentType: 'application/json',
      body: JSON.stringify({
        response: {
          docs: results.map((item) => ({
            identifier: item.key,
            title: item.title,
            description: item.excerpt,
          })),
        },
      }),
    });
  });

  await page.route('**://*.wikipedia.org/w/rest.php/v1/search/page**', async (route) => {
    const url = new URL(route.request().url());
    const results = searchFixtures(url.searchParams.get('q') || "");
    await route.fulfill({
      status: 200,
      contentType: 'application/json',
      body: JSON.stringify({
        pages: results.map((item, index) => ({
          id: index + 1,
          key: item.key,
          title: item.title,
          excerpt: item.excerpt,
          description: item.description,
        })),
      }),
    });
  });

  await page.route('**://www.wikidata.org/w/api.php**', async (route) => {
    const url = new URL(route.request().url());
    const results = searchFixtures(url.searchParams.get('search') || "");
    await route.fulfill({
      status: 200,
      contentType: 'application/json',
      body: JSON.stringify({
        search: results.map((item) => ({
          id: item.qid,
          label: item.title,
          description: item.description,
          concepturi: `https://www.wikidata.org/wiki/${item.qid}`,
          url: item.url,
        })),
      }),
    });
  });

  await page.route('**://*.wiktionary.org/w/api.php**', async (route) => {
    const url = new URL(route.request().url());
    const results = searchFixtures(url.searchParams.get('search') || "");
    await route.fulfill({
      status: 200,
      contentType: 'application/json',
      body: JSON.stringify([
        url.searchParams.get('search') || "",
        results.map((item) => item.title),
        results.map((item) => item.excerpt),
        results.map((item) => item.url),
      ]),
    });
  });
}

test.describe('Issue #335 - composite Wikipedia research in agent mode', () => {
  test.beforeEach(async ({ page }) => {
    await page.addInitScript(() => {
      window.localStorage.setItem(
        'formal-ai.preferences.v1',
        'demo_preferences\n  demoMode "off"\n  diagnosticsMode "on"\n  greetingVariations "off"',
      );
    });
    await mockHistoricalSearchProviders(page);
    await page.goto('./');
    await expect(page.locator('.app')).toBeVisible({ timeout: 15_000 });
    await expect(page.locator('[data-testid="demo-status"]')).toHaveText('Manual mode');
    await expect(page.locator('.status')).toContainText('wasm worker');
  });

  test('reported prompt uses focused searches instead of one broad no-result query', async ({
    page,
  }) => {
    await page.locator('[data-testid="agent-toggle"]').click();
    const last = await sendPrompt(page, REPORTED_PROMPT);

    await expect(last).toContainText('Agent plan');
    await expect(last).toContainText('List of Nikola Tesla patents');
    await expect(last).toContainText('around 300 patents');
    await expect(last).toContainText('List of Edison patents');
    await expect(last).toContainText('2,332 patents worldwide');
    await expect(last).toContainText('War of the currents');
    await expect(last).not.toContainText('No CORS-enabled web search results');

    const evidence = last.locator('.evidence-list');
    await expect(evidence).toContainText('web_search:request:Nikola Tesla patents');
    await expect(evidence).toContainText('web_search:request:Thomas Edison patents');
    await expect(evidence).toContainText('web_search:request:War of Currents');
    await expect(evidence).not.toContainText('Compare their number of patents');
  });
});
