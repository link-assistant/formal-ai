import assert from 'node:assert/strict';
import fs from 'node:fs';
import { before, test } from 'node:test';
import { WorkerHost, REPO_ROOT } from '../../../js/server/worker-host.mjs';
import { installNodeHost } from '../../../js/agentic/node-host.mjs';
import { host, installHost } from '../../../js/agentic/host.mjs';
import { stableId } from '../../../js/agentic/crate/engine_stable_identifier.mjs';
import { resolveIn, resolveRequirementTarget } from '../../../js/agentic/requirement_resolution.mjs';
before(async () => { await installNodeHost(new WorkerHost()); });
const census = { modules: [
  { path: 'src/目录/alpha-beta.rs', symbols: [{ kind: 'const', name: 'SHARED' }] },
  { path: 'src/other_alpha.rs', symbols: [{ kind: 'const', name: 'SHARED' }] },
] };

test('explicit complete module paths scope duplicate names without token splitting guesses', () => {
  for (const path of ['src/目录/alpha-beta.rs', 'src/other_alpha.rs']) {
    for (const reference of [path, 'rust/' + path, './rust/' + path]) {
      for (const request of ['In ' + reference + ', rename SHARED.', 'In ' + String.fromCharCode(96) + reference + String.fromCharCode(96) + ', rename SHARED.']) {
        assert.deepEqual(resolveIn(census, request), { module_path: path, symbol: 'SHARED', kind: 'const' });
      }
    }
  }
  assert.deepEqual(resolveIn(census, 'In src/other_alpha.rs and rust/src/other_alpha.rs rename SHARED.'),
    { module_path: 'src/other_alpha.rs', symbol: 'SHARED', kind: 'const' });
});

test('ambiguous, missing, conflicting, traversal and token-embedded module operands refuse', () => {
  for (const request of ['Rename SHARED.', 'In src/missing.rs rename SHARED.',
    'In src/other_alpha.rs and src/目录/alpha-beta.rs rename SHARED.',
    'In ../src/other_alpha.rs rename SHARED.', 'In prefixsrc/other_alpha.rs rename SHARED.',
    'In src/other_alpha.rs.backup rename SHARED.', 'In src/other_alpha.rs rename MISSING.']) {
    assert.equal(resolveIn(census, request), null, request);
  }
});

test('the exact original L17 file-qualified task resolves while its unqualified requirement refuses', () => {
  const row = fs.readFileSync(REPO_ROOT + '/experiments/issue_1028_agent_cli_ladder/leaves.tsv', 'utf8')
    .split('\n').map(line => line.split('\t')).find(row => row[0] === 'L17');
  assert.deepEqual(resolveRequirementTarget(row[1]),
    { module_path: 'src/web_search_fusion_core.rs', symbol: 'NEGATION_ROLE', kind: 'const' });
  assert.equal(resolveRequirementTarget(row[5]), null);
});

test('literal owners require exact source identity, unique complete initializer and current bytes', () => {
  const originalHost = host();
  const path = 'src/literal_owner.rs';
  let source = 'pub const VALUE: &str = "azure dawn";\n';
  let identitySource = source;
  const module = { path, symbols: [{ kind: 'const', name: 'VALUE', start_line: 1, end_line: 1 }] };
  const literalCensus = { modules: [module] };
  const request = 'In the file rust/' + path + ', replace "azure dawn" with "cobalt sun".';
  installHost({ ...originalHost,
    readText: name => name === 'rust/' + path ? source : originalHost.readText(name),
    censusDocuments: () => [{ text: 'self_ast_census\n  target ' + path + '\n  symbols\n    const VALUE 1 1\n',
      sourceIdentity: { path, content_id: stableId('source_module', identitySource), byte_len: Buffer.byteLength(identitySource) } }],
  });
  try {
    assert.deepEqual(resolveIn(literalCensus, request), { module_path: path, symbol: 'VALUE', kind: 'const' });
    source = source.replace('azure dawn', 'other words');
    assert.equal(resolveIn(literalCensus, request), null, 'changed source identity refuses');
    source = 'pub const VALUE: &str = "azure dawn";\npub const SECOND: &str = "azure dawn";\n';
    identitySource = source;
    module.symbols.push({ kind: 'const', name: 'SECOND', start_line: 2, end_line: 2 });
    assert.equal(resolveIn(literalCensus, request), null, 'two matching owners refuse');
    module.symbols.pop();
    for (const value of ['/* "azure dawn" */ "other"', '"azure dawn".trim()', '["azure dawn"]', '"azure dawn bright"']) {
      source = 'pub const VALUE: &str = ' + value + ';\n';
      identitySource = source;
      assert.equal(resolveIn(literalCensus, request), null, value);
    }
    source = 'pub const VALUE: &str = "azure dawn";\n';
    identitySource = source;
    assert.equal(resolveIn(literalCensus, 'Inspect rust/' + path + ' and the words "azure dawn".'), null, 'no edit operand');
  } finally { installHost(originalHost); }
});

test('missing source host refuses literal ownership without breaking pure named scope', () => {
  const originalHost = host();
  installHost(null);
  try {
    assert.equal(resolveIn(census, 'In src/other_alpha.rs replace "old" with "new".'), null);
    assert.deepEqual(resolveIn(census, 'In src/other_alpha.rs rename SHARED.'),
      { module_path: 'src/other_alpha.rs', symbol: 'SHARED', kind: 'const' });
    assert.equal(resolveIn(census, 'Rename SHARED.'), null);
  } finally { installHost(originalHost); }
});
