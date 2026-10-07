// Issue #133 (R181-R193): DuckDuckGo-first search across providers, fused by
// reciprocal rank fusion with k = 60, run five at a time, with failing
// providers disabled for the session and every step recorded as evidence.
// The worker runs its real planner (`runWebSearchQuery`) here; only the
// network is replaced, by a fetch that answers two providers and refuses
// the rest, so each request the planner makes is observed.

import { describe, test } from 'node:test';
import assert from 'node:assert/strict';
import { existsSync, readdirSync, readFileSync } from 'node:fs';
import { join } from 'node:path';

import { REPO_ROOT, createWorkerContext, evaluate, plain } from './support/browser-runtime.mjs';

const json = (body) => Promise.resolve({
  ok: true,
  status: 200,
  headers: { get: () => 'application/json' },
  text: () => Promise.resolve(JSON.stringify(body)),
  json: () => Promise.resolve(body),
});

const ANSWERS = {
  'api.duckduckgo.com': {
    Heading: 'Open-source license',
    AbstractURL: 'https://en.wikipedia.org/wiki/Open-source_license',
    AbstractText: 'An open-source license is a type of license.',
    RelatedTopics: [{ FirstURL: 'https://duckduckgo.com/MIT_License', Text: 'MIT License - a permissive license' }],
  },
  'en.wikipedia.org': {
    pages: [
      { key: 'Open-source_license', title: 'Open-source license', excerpt: 'type of license' },
      { key: 'GNU_General_Public_License', title: 'GNU General Public License', excerpt: 'copyleft license' },
    ],
  },
};

/** A worker whose site files load from disk and whose network is scripted. */
async function searchWorker(answers) {
  const requested = [];
  const fetch = (url) => {
    const href = String(url);
    if (href.startsWith('http://localhost')) {
      const relative = new URL(href).pathname.replace(/^\//, '');
      const onDisk = relative.startsWith('seed/') ? join(REPO_ROOT, 'data', relative) : join(REPO_ROOT, 'js', relative);
      if (!existsSync(onDisk)) return Promise.resolve({ ok: false, status: 404, text: () => Promise.resolve('') });
      const text = readFileSync(onDisk, 'utf8');
      return Promise.resolve({ ok: true, status: 200, text: () => Promise.resolve(text) });
    }
    requested.push(href);
    const host = new URL(href).host;
    return host in answers ? json(answers[host]) : Promise.reject(new TypeError('Failed to fetch'));
  };
  const worker = createWorkerContext({ fetch });
  await evaluate(worker, 'loadSeed()');
  return { worker, requested };
}

const search = async (worker, query) => plain(await evaluate(worker, `runWebSearchQuery(${JSON.stringify(query)}, "en", "explicit", {})`));

describe('R181/R190: DuckDuckGo leads, providers return ten results and run five at a time', () => {
  test('the default provider order opens with DuckDuckGo', async () => {
    const { worker } = await searchWorker({});
    const order = plain(evaluate(worker, 'self.FormalAIWebSearchComponent.orderProviders(WEB_SEARCH_PROVIDERS).map((provider) => provider.id)'));
    assert.equal(order[0], 'duckduckgo');
    assert.ok(order.includes('wikipedia') && order.includes('wikidata'), order.join(','));
  });

  test('the planner constants are k = 60, five concurrent providers and ten results each', async () => {
    const { worker } = await searchWorker({});
    assert.deepEqual(plain(evaluate(worker, '[webSearchRrfK(), webSearchConcurrency(), webSearchProviderLimit()]')), [60, 5, 10]);
  });

  test('no more than five provider tasks are ever in flight', async () => {
    const { worker } = await searchWorker({});
    const peak = plain(await evaluate(worker, `(async () => {
      let running = 0; let peak = 0;
      const tasks = Array.from({ length: 12 }, () => async () => {
        running += 1; peak = Math.max(peak, running);
        await new Promise((resolve) => setTimeout(resolve, 5));
        running -= 1; return { ok: true };
      });
      await runWithConcurrencyLimit(tasks, webSearchConcurrency());
      return peak;
    })()`));
    assert.equal(peak, 5);
  });
});

describe('R182/R183/R188/R189: one fused list from live requests, every step recorded', () => {
  test('reciprocal rank fusion sums 1/(k + rank) across providers', async () => {
    const { worker } = await searchWorker({});
    const fused = plain(evaluate(worker, `reciprocalRankFusion([
      { id: "a", results: [{ url: "u1" }, { url: "u2" }] },
      { id: "b", results: [{ url: "u2" }, { url: "u3" }] },
    ], 60, []).map((entry) => [entry.url, Number(entry.score.toFixed(6)), entry.providers.map((p) => p.id + ":" + p.rank).join("+")])`));
    assert.deepEqual(fused, [
      ['u2', Number((1 / 62 + 1 / 61).toFixed(6)), 'a:2+b:1'],
      ['u1', Number((1 / 61).toFixed(6)), 'a:1'],
      ['u3', Number((1 / 62).toFixed(6)), 'b:2'],
    ]);
  });

  test('a search requests the providers, ranks each answer and fuses them', async () => {
    const { worker, requested } = await searchWorker(ANSWERS);
    const { evidence } = await search(worker, 'open source licenses');
    assert.ok(requested.some((url) => url.startsWith('https://api.duckduckgo.com/')), requested.join('\n'));
    assert.ok(requested.some((url) => url.startsWith('https://en.wikipedia.org/')), requested.join('\n'));
    for (const line of [
      'web_search:request:open source licenses',
      'web_search:language:en',
      'web_search:provider:duckduckgo',
      'web_search:combined:rrf:k=60',
      'web_search:rank:duckduckgo:1:https://en.wikipedia.org/wiki/Open-source_license',
      'web_search:rank:wikipedia:1:https://en.wikipedia.org/wiki/Open-source_license',
      'web_search:fused:1:duckduckgo+wikipedia:https://en.wikipedia.org/wiki/Open-source_license',
    ]) {
      assert.ok(evidence.includes(line), `${line}\n${evidence.join('\n')}`);
    }
  });
});

describe('R191: a failing provider is disabled for the rest of the session', () => {
  test('a refused provider is recorded as disabled and never requested again', async () => {
    const { worker, requested } = await searchWorker({});
    const first = await search(worker, 'open source licenses');
    assert.ok(first.evidence.includes('web_search:disabled:duckduckgo'), first.evidence.join('\n'));
    const count = requested.length;
    const second = await search(worker, 'copyleft');
    assert.ok(second.evidence.includes('web_search:disabled:duckduckgo'), second.evidence.join('\n'));
    assert.equal(requested.length, count, 'a disabled provider was requested again');
  });
});

describe('R184-R187: the diagnostics page probes the extra provider families', () => {
  const page = readFileSync(join(REPO_ROOT, 'js/tests/connectivity.js'), 'utf8');
  const byCategory = new Map();
  for (const [, name, category] of page.matchAll(/name:\s*"([^"]+)",[\s\S]*?category:\s*"([^"]+)"/g)) {
    if (!byCategory.has(category)) byCategory.set(category, []);
    byCategory.get(category).push(name);
  }
  const expected = {
    search: ['Yandex Search', 'Ecosia', 'Mojeek', 'Startpage'],
    code: ['GitHub', 'GitLab', 'Codeberg', 'Gitee', 'Bitbucket Cloud'],
    papers: ['arXiv', 'Europe PMC', 'DOAJ'],
    knowledge: ['Wiktionary', 'DBpedia Lookup'],
  };
  for (const [category, names] of Object.entries(expected)) {
    test(`the ${category} category probes ${names.join(', ')}`, () => {
      for (const name of names) assert.ok((byCategory.get(category) || []).includes(name), `${category}: ${name}`);
    });
  }
});

describe('R193: the change was released as a minor version with its changelog entry', () => {
  test('the 0.71.0 section records the issue #133 addition', () => {
    const archive = readFileSync(join(REPO_ROOT, 'docs/changelog/archive-01.md'), 'utf8');
    const section = archive.slice(archive.indexOf('## [0.71.0]'));
    assert.match(section.slice(0, section.indexOf('\n## ', 1)), /### Added[\s\S]*Issue #133: DuckDuckGo Instant Answer is now the default web search engine/);
  });
});

describe('R192: the issue #133 research is kept in the repository', () => {
  test('the case study has a README and raw data', () => {
    assert.ok(readFileSync(join(REPO_ROOT, 'docs/case-studies/issue-133/README.md'), 'utf8').length > 0);
    assert.ok(readdirSync(join(REPO_ROOT, 'docs/case-studies/issue-133/raw-data')).length > 0);
  });
});
