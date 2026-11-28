import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { readFileSync } from 'node:fs';

import { REPLAY_PATH } from '../../../../scripts/generate-coding-documentation-captures.mjs';
import { childValue, parseLino } from '../../../../js/server/lino.mjs';

const root = new URL('../../../../', import.meta.url);

export function codingDocumentationReplay() {
  const manifest = parseLino(readFileSync(new URL(REPLAY_PATH, root), 'utf8'));
  const pages = new Map();
  for (const row of manifest.children.filter((node) => node.name === 'capture')) {
    const fixture = childValue(row, 'fixture');
    const bytes = readFileSync(new URL(fixture, root));
    assert.equal(createHash('sha256').update(bytes).digest('hex'), childValue(row, 'sha256'), fixture);
    pages.set(childValue(row, 'url'), bytes.toString('utf8'));
  }
  const requests = [];
  return {
    requests,
    networkFetch(url) {
      url = String(url);
      requests.push(url);
      const text = pages.get(url);
      return Promise.resolve({ url, ok: text !== undefined, status: text === undefined ? 404 : 200,
        text: () => Promise.resolve(text ?? '') });
    },
  };
}
