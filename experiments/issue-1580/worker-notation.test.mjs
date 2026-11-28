// Issue #1580: keep worker notation sources below the warning threshold.
import assert from 'node:assert/strict';
import {readFileSync} from 'node:fs';
import test from 'node:test';
import {createWorkerContext, plain, REPO_ROOT} from '../../rust/tests/web/support/browser-runtime.mjs';

test('worker notation sources preserve file-size headroom', () => {
  for (const file of [
    'js/worker/formal_ai_worker_meaning_lexicon.js',
    'ts/worker/formal_ai_worker_meaning_lexicon.ts',
    'js/worker/formal_ai_worker_notation.js',
    'ts/worker/formal_ai_worker_notation.ts',
  ]) {
    const source = readFileSync(REPO_ROOT + '/' + file, 'utf8');
    const lines = source.split('\n').length - Number(source.endsWith('\n'));
    assert.ok(lines <= 1400, file + ' has ' + lines + ' lines; warning threshold is 1400');
  }
});

test('worker notation preserves quoting, tree structure and shared variable bindings in either load order', () => {
  for (const reverse of [false, true]) {
    const worker = createWorkerContext({}, reverse ? {reorderModules: modules => modules.reverse()} : {});
    const tree = plain(worker.parseLinoTree([
      'root',
      '  phrase: "quoted # hash: value" # discarded comment',
      "  single: 'one\\x27two'",
      '  point: codepoints 0x41 66',
    ].join('\n')));
    assert.deepEqual(tree.children[0].children.map(node => [node.name, node.value]), [
      ['phrase', 'quoted # hash: value'], ['single', "one'two"], ['point', 'AB'],
    ]);
    const rule = plain(worker.parseSubstitutionRules([
      'substitution_rules', '  id test', '  rule join', '    order 2',
      '    event update', '    when $left -> $middle',
      '    when $middle -> $right', '    replace $left -> $middle',
      '      with $left -> $right',
    ].join('\n'))).rules[0];
    const links = [{from: 'a', to: 'b'}, {from: 'b', to: 'c'}];
    const bindings = worker.findBindings(links, rule.conditions, 0, {});
    assert.deepEqual(plain(bindings), {left: 'a', middle: 'b', right: 'c'});
    assert.deepEqual(plain(worker.instantiatePattern(rule.actions[0].add[0], bindings)), {from: 'a', to: 'c'});
    assert.equal(worker.findBindings([{from: 'a', to: 'b'}, {from: 'd', to: 'c'}], rule.conditions, 0, {}), null);
    assert.equal(worker.instantiatePattern(worker.parseLinkPattern('$missing -> c'), {}), null);
    assert.throws(() => worker.parseLinkPattern('missing arrow'), /expected/);
    assert.throws(() => worker.parseCrudEvent('unsupported'), /invalid CRUD event/);
  }
});
