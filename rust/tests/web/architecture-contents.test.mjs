// R1188-U2: ARCHITECTURE.md is a table of contents whose links and anchors
// name the topic files under docs/architecture/ (the "Small Public
// Documentation" principle), checked by scripts/check-architecture-contents.mjs.

import { test } from 'node:test';
import assert from 'node:assert/strict';

import {
  contentsProblems,
  documentAnchors,
  headingAnchor,
  topicLinks,
} from '../../../scripts/check-architecture-contents.mjs';

test('a heading anchor follows the GitHub rule', () => {
  assert.equal(headingAnchor('4.4 Default native link-cli / doublets-web store'), '44-default-native-link-cli--doublets-web-store');
  assert.equal(headingAnchor('4.5 Fact-query reasoning pipeline (Issue #127)'), '45-fact-query-reasoning-pipeline-issue-127');
  assert.equal(headingAnchor('Память и кэш'), 'память-и-кэш');
});

test('a repeated heading is numbered and fenced headings are not headings', () => {
  const anchors = documentAnchors(['# Notes', '```', '# not a heading', '```', '## Notes'].join('\n'));
  assert.deepEqual([...anchors], ['notes', 'notes-1']);
});

test('the links into the topic directory are read with their anchors', () => {
  assert.deepEqual(topicLinks('- [A](docs/architecture/a.md)\n  - [B](docs/architecture/a.md#b-part) [C](README.md)'), [
    { file: 'docs/architecture/a.md', anchor: '' },
    { file: 'docs/architecture/a.md', anchor: 'b-part' },
  ]);
});

test('an unlinked topic, a missing topic and a stale anchor are each reported', () => {
  const topics = new Map([
    ['docs/architecture/a.md', '# A\n## B part\n'],
    ['docs/architecture/orphan.md', '# Orphan\n'],
  ]);
  const contents = [
    '- [A](docs/architecture/a.md#b-part)',
    '- [Gone](docs/architecture/gone.md)',
    '- [Stale](docs/architecture/a.md#renamed-part)',
  ].join('\n');
  assert.deepEqual(contentsProblems(contents, topics), [
    'docs/architecture/orphan.md is not linked from ARCHITECTURE.md',
    'ARCHITECTURE.md links docs/architecture/gone.md, which does not exist',
    'ARCHITECTURE.md links docs/architecture/a.md#renamed-part, which names no heading of that file',
  ]);
});

test('a complete contents document has no problems', () => {
  const topics = new Map([['docs/architecture/a.md', '# A\n## B part\n']]);
  assert.deepEqual(contentsProblems('- [A](docs/architecture/a.md)\n  - [B](docs/architecture/a.md#b-part)', topics), []);
});
