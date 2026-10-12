import test from 'node:test';
import assert from 'node:assert/strict';
import {readFileSync} from 'node:fs';
import {parse} from '@babel/parser';
import {WorkerHost} from '../../../js/server/worker-host.mjs';
import {installNodeHost} from '../../../js/agentic/node-host.mjs';
import {host, installHost} from '../../../js/agentic/host.mjs';
import {matchSourceDescription} from '../../../js/agentic/code_task/source_contract/matcher.mjs';

await installNodeHost(new WorkerHost());
const maintainedHost = host();
const document = maintainedHost.readText('data/seed/source-authoring-grammar.lino');
const source = readFileSync(new URL('./issue-0848-coding-ladder.test.mjs', import.meta.url), 'utf8');
const originalCases = [];
function sourceCases(node) {
  if (!node || typeof node !== 'object') return;
  if (node.type === 'ArrayExpression' && node.elements.length === 3
      && node.elements.every(element => element?.type === 'StringLiteral')) {
    const [task, path, content] = node.elements.map(element => element.value);
    if (task.startsWith('Create ') && path.endsWith('.rs')) {
      originalCases.push({task, artifact: {path, content}});
    }
  }
  for (const [key, value] of Object.entries(node)) {
    if (['loc', 'start', 'end'].includes(key)) continue;
    if (Array.isArray(value)) value.forEach(sourceCases);
    else if (value && typeof value === 'object') sourceCases(value);
  }
}
sourceCases(parse(source, {sourceType: 'module'}));
assert.equal(originalCases.length, 4);
const containers = document.split(/\n(?=[^\s#])/u);
const grammar = containers.find(container => container.startsWith('source-authoring-grammar\n'));
assert.ok(grammar);
function withDocument(text, run) {
  installHost({...maintainedHost, readText: path => path === 'data/seed/source-authoring-grammar.lino'
    ? text : maintainedHost.readText(path)});
  try { run(); } finally { installHost(maintainedHost); }
}
function acceptedCases() {
  for (const {task, artifact} of originalCases) {
    const result = matchSourceDescription(task, artifact);
    assert.ok(result, task);
    assert.equal(result.wholeRequestConsumed, true);
    assert.equal(result.output.content, artifact.content);
    assert.equal(result.output.path, artifact.path);
    assert.deepEqual(result.full, [0, task.length]);
    assert.equal(result.compilation, 'pending');
    assert.deepEqual(result.unknownEffects, ['module_initialization', 'tool_execution']);
  }
}
test('source grammar binds all four unchanged original declarations in a shared document', () => {
  withDocument(document, acceptedCases);
});
test('source grammar ownership is independent of document container order', () => {
  withDocument([...containers].reverse().join('\n'), acceptedCases);
});
test('single named source grammar retains the same supported ownership', () => {
  withDocument(grammar, acceptedCases);
});
const malformed = [
  ['missing', containers.filter(container => container !== grammar).join('\n')],
  ['duplicate', document + '\n' + grammar],
  ['wrong root', grammar.replace(/^source-authoring-grammar/u, 'unrelated-grammar')],
  ['invalid pattern', 'source-authoring-grammar\n  form broken\n    language en\n'
    + '    kind equality-test\n    pattern "^("\n'],
];
for (const [label, text] of malformed) {
  test('source grammar refuses ' + label + ' container ownership', () => {
    withDocument(text, () => {
      for (const {task, artifact} of originalCases) {
        assert.equal(matchSourceDescription(task, artifact), null);
      }
    });
  });
}
for (const suffix of [' Deploy it.', ' Do not write.', ' Read another.rs first.', ' and unknown task.']) {
  test('source grammar does not consume an independent tail: ' + suffix, () => {
    withDocument(document, () => {
      for (const {task, artifact} of originalCases) {
        assert.equal(matchSourceDescription(task + suffix, artifact), null);
      }
    });
  });
}
test('source grammar retains exact technical type, name and destination splices', () => {
  const {task, artifact} = originalCases[0];
  withDocument(document, () => {
    assert.equal(matchSourceDescription(task.replace('f64', 'F64'), artifact), null);
    assert.equal(matchSourceDescription(task, {...artifact, path: artifact.path.toUpperCase()}), null);
    assert.equal(matchSourceDescription(task, {...artifact,
      content: artifact.content.replace('millimetres_to_metres', 'other_function')}), null);
  });
});


import {rustSourceForTask} from '../../../js/agentic/code_task.mjs';
import {sourceRegistrationContract} from '../../../js/agentic/code_task/source_contract/registration_contract.mjs';
const registrationTasks = new Set();
function registrationCases(node) {
  if (!node || typeof node !== 'object') return;
  if (node.type === 'StringLiteral' && node.value.startsWith('Create ')
      && node.value.includes('and register the module in ')) registrationTasks.add(node.value);
  for (const [key, value] of Object.entries(node)) {
    if (['loc', 'start', 'end'].includes(key)) continue;
    if (Array.isArray(value)) value.forEach(registrationCases);
    else if (value && typeof value === 'object') registrationCases(value);
  }
}
registrationCases(parse(source, {sourceType: 'module'}));
assert.equal(registrationTasks.size, 1);
test('both declaration and registration consume the uniquely owned grammar container', () => {
  for (const text of [document, [...containers].reverse().join('\n'), grammar]) {
    withDocument(text, () => {
      for (const task of registrationTasks) {
        const result = sourceRegistrationContract(task, rustSourceForTask);
        assert.ok(result);
        assert.equal(result.wholeRequestConsumed, true);
        assert.deepEqual(result.full, [0, task.length]);
        assert.equal(result.declaration.wholeRequestConsumed, true);
        assert.equal(result.compilation, 'pending');
      }
    });
  }
});
test('registration refuses missing, duplicate, wrong and malformed grammar ownership', () => {
  for (const [, text] of malformed) {
    withDocument(text, () => {
      for (const task of registrationTasks) {
        assert.equal(sourceRegistrationContract(task, rustSourceForTask), null);
      }
    });
  }
});
test('registration retains an unconsumed independent Need rather than certifying the whole task', () => {
  withDocument(document, () => {
    for (const task of registrationTasks) {
      const additional = ' Deploy it.';
      const result = sourceRegistrationContract(task + additional, rustSourceForTask);
      assert.ok(result);
      assert.equal(result.wholeRequestConsumed, false);
      assert.deepEqual(result.remainingSpan, [task.length + 1, task.length + additional.length]);
      assert.equal((task + additional).slice(...result.remainingSpan), 'Deploy it.');
    }
  });
});
