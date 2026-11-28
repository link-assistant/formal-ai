// Source-transport replay for the four factual routing probes. No canned
// answer enters FACTS: production parsing must consume the publisher claims.
import { wikidataFixture } from './fact-fixtures.mjs';

export const routingFactExpectations = new Map([
  ['p109', ['Q142', 'Q90', 'Paris']],
  ['p110', ['Q30', 'Q61', 'Washington']],
  ['p111', ['Q159', 'Q649', 'Москва']],
  ['p112', ['Q575650', 'Q28937563', 'Graydon Hoare']],
]);

export function routingFactReplay(fallback, requests) {
  const subjects = new Map([['france', 'Q142'], ['usa', 'Q30'], ['the usa', 'Q30'], ['россии', 'Q159'], ['rust', 'Q575650']]);
  return async (url, init) => {
    const target = new URL(String(url), 'http://localhost');
    if (target.hostname !== 'www.wikidata.org') return fallback(url, init);
    requests.push(target.href);
    const term = target.searchParams.get('search');
    let body;
    if (term) body = subjects.has(term.toLowerCase())
      ? { search: [{id: subjects.get(term.toLowerCase()), label: term}] } : { search: [] };
    else {
      const id = target.searchParams.get('ids') || target.pathname.split('/').at(-1).split('.')[0];
      body = wikidataFixture(target, null, ['Q575650', 'Q28937563'].includes(id) ? 'live-entities' : 'entities');
    }
    return { ok: Boolean(body), status: body ? 200 : 404, text: async () => JSON.stringify(body) };
  };
}
