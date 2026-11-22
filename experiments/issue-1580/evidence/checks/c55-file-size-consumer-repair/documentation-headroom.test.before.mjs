import assert from 'node:assert/strict';
import {existsSync, readFileSync} from 'node:fs';
import {dirname, resolve} from 'node:path';
import {fileURLToPath} from 'node:url';
import test from 'node:test';
import {checkListedFiles} from '../../scripts/check-file-size.mjs';

const root = fileURLToPath(new URL('../../', import.meta.url)).replace(/\/$/u, '');
test('contributor rules retain source headroom and reachable detailed guides', () => {
  const guides = ['docs/development/merge-conflict-layout.md', 'docs/development/changelog-management.md'];
  assert.deepEqual(checkListedFiles(root, new Set(['CONTRIBUTING.md', ...guides])),
    {warnings: [], violations: [], embeddedDataViolations: []});
  const entry = readFileSync(resolve(root, 'CONTRIBUTING.md'), 'utf8');
  for (const guide of guides) {
    assert(entry.includes('](' + guide + ')'), 'contributor entry must link the detailed rule: ' + guide);
    const file = resolve(root, guide);
    const text = readFileSync(file, 'utf8');
    for (const link of text.matchAll(/\]\(([^)]+)\)/gu)) {
      if (/^(?:https?:|#)/u.test(link[1])) continue;
      assert(existsSync(resolve(dirname(file), link[1].split('#')[0])), 'broken guide link: ' + link[1]);
    }
  }
  const layout = readFileSync(resolve(root, guides[0]), 'utf8');
  assert(layout.includes('Every union-merged file has a verifier.'));
  assert(layout.includes('scripts/regenerate-derived-artifacts.sh'));
  const changelog = readFileSync(resolve(root, guides[1]), 'utf8');
  assert(changelog.includes('### Creating a Fragment'));
  assert(changelog.includes('### Fragment Categories'));
  assert(changelog.includes('### During Release'));
});
