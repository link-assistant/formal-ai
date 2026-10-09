import assert from 'node:assert/strict';
import { execFileSync } from 'node:child_process';
import test from 'node:test';

const probe = (line) => JSON.parse(execFileSync('python3', ['-c',
  "import importlib.util,json,sys; spec=importlib.util.spec_from_file_location('closure','scripts/audit-total-closure.py'); m=importlib.util.module_from_spec(spec); spec.loader.exec_module(m); print(json.dumps(m._line_tokens(sys.argv[1])))",
  line], { encoding: 'utf8' }));

test('escaped scalar contents never become semantic closure edges', () => {
  const body = JSON.stringify({ title: 'Arbitrary sample', extract: 'a new literal concept', markup: '<span class="sample">value</span>' });
  assert.deepEqual(probe('body ' + JSON.stringify(body)), ['body']);
  assert.deepEqual(probe('role verified ' + JSON.stringify('quoted "invented_edge" # content') + ' actual_reference # ignored'), ['role', 'verified', 'actual_reference']);
});

test('even and odd backslash runs preserve quote boundaries and trailing references', () => {
  for (const value of ['ends with slash\\', 'quoted "value"', '\\\\"', 'Unicode α😀', 'contains # marker']) {
    assert.deepEqual(probe('effect ' + JSON.stringify(value) + ' known_relation'), ['effect', 'known_relation']);
  }
  assert.deepEqual(probe("role 'one \\' inside' outside"), ['role', 'outside']);
  const tick = String.fromCharCode(96), slash = String.fromCharCode(92);
  assert.deepEqual(probe('role ' + tick + 'quoted ' + slash + tick + ' inside' + tick + ' outside'), ['role', 'outside']);
});

test('bare semantic references remain visible beside literals and comments', () => {
  assert.deepEqual(probe('role invented_reference + ungrounded_relation # comment'), ['role', 'invented_reference', '+', 'ungrounded_relation']);
  assert.deepEqual(probe('role "complete scalar" undefined_target'), ['role', 'undefined_target']);
});
