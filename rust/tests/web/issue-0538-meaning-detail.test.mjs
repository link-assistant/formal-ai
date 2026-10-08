// Issue #538 (R375, R377, R386): detailed meanings and words, checked over the
// committed seed and case study. The Rust twins are rust/tests/unit/word_surface_grammar.rs
// (each tomato surface reads its grammatical number, part of speech and
// denotation through the one `SemanticFacet` path) and
// rust/tests/unit/issue_538_agentic.rs; the JavaScript meaning-detail recipe
// that writes those surfaces is pinned by rust/tests/web/agentic-recipes-a.test.mjs.

import assert from 'node:assert/strict';
import { existsSync, readdirSync, readFileSync } from 'node:fs';
import { describe, it } from 'node:test';

const ROOT = new URL('../../../', import.meta.url);
const read = (path) => readFileSync(new URL(path, ROOT), 'utf8');
const exists = (path) => existsSync(new URL(path, ROOT));

/** Every `data/seed/*.lino` file as `[name, lines]`. */
function seedFiles() {
  return readdirSync(new URL('data/seed/', ROOT))
    .filter((name) => name.endsWith('.lino'))
    .sort()
    .map((name) => [name, read(`data/seed/${name}`).split('\n')]);
}

describe('grammatical number has one normalized representation (R375)', () => {
  it('no seed file uses the legacy `facet grammatical_number` wrapper', () => {
    for (const [name, lines] of seedFiles()) {
      for (const line of lines) {
        assert.ok(!/^\s*facet grammatical_number\b/.test(line), `${name}: ${line.trim()}`);
      }
    }
  });

  it('every surface carries at most one grammatical_number line, and it names singular or plural', () => {
    let surfaces = 0;
    for (const [name, lines] of seedFiles()) {
      let surface = null;
      let seen = 0;
      for (const line of lines) {
        const indent = line.length - line.trimStart().length;
        if (/^\s*surface\b/.test(line)) {
          surface = { indent, label: line.trim() };
          seen = 0;
          continue;
        }
        if (surface && line.trim() && indent <= surface.indent) surface = null;
        const value = /^\s+grammatical_number (\S+)$/.exec(line.replace(/\s+#.*$/, ''));
        if (!value || !surface) continue;
        seen += 1;
        if (seen === 1) surfaces += 1;
        assert.equal(seen, 1, `${name}: ${surface.label} repeats grammatical_number`);
        assert.ok(['singular', 'plural'].includes(value[1]), `${name}: ${surface.label} -> ${value[1]}`);
      }
    }
    assert.ok(surfaces >= 8, `the tomato surfaces alone carry eight, found ${surfaces}`);
  });

  it('the tomato surfaces pair a singular and a plural in each language, as the issue asked', () => {
    const lines = read('data/seed/meanings-translation.lino').split('\n');
    const start = lines.indexOf('  tomato');
    const block = [];
    for (let index = start + 1; index < lines.length && lines[index].startsWith('    '); index += 1) block.push(lines[index]);
    const surfaces = [];
    let current = null;
    for (const line of block) {
      if (line.startsWith('    surface ')) {
        current = { text: '', number: '' };
        surfaces.push(current);
      } else if (current && line.startsWith('      text ')) {
        current.text = line.slice('      text '.length);
      } else if (current && line.startsWith('      grammatical_number ')) {
        current.number = line.slice('      grammatical_number '.length);
      }
    }
    const numbers = Object.fromEntries(surfaces.filter((entry) => entry.number).map((entry) => [entry.text, entry.number]));
    assert.deepEqual(numbers, {
      tomato: 'singular',
      tomatoes: 'plural',
      помидор: 'singular',
      помидоры: 'plural',
      томат: 'singular',
      томаты: 'plural',
    });
  });
});

describe('the issue #538 case study (R377, R386)', () => {
  it('preserves the issue data, decomposition, plans and research', () => {
    for (const path of [
      'docs/case-studies/issue-538/README.md',
      'docs/case-studies/issue-538/requirements.md',
      'docs/case-studies/issue-538/solution-plan.md',
      'docs/case-studies/issue-538/raw-data/issue-538.json',
      'docs/case-studies/issue-538/raw-data/issue-538-comments.json',
      'docs/case-studies/issue-538/raw-data/online-research.md',
      'docs/case-studies/issue-538/raw-data/pr-601.json',
      'docs/case-studies/issue-538/raw-data/pr-601-review-comments.json',
    ]) {
      assert.ok(exists(path), path);
    }
  });

  it('records the single prepared pull request and its branch', () => {
    const pull = JSON.parse(read('docs/case-studies/issue-538/raw-data/pr-601.json'));
    assert.equal(pull.number, 601);
    assert.equal(pull.headRefName, 'issue-538-eca4a11c39c6');
    assert.ok(read('docs/case-studies/issue-538/README.md').includes('https://github.com/link-assistant/formal-ai/pull/601'));
  });
});
