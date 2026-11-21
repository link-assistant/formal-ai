// Replay the literal native source witness without compiling Rust.
import assert from 'node:assert/strict';
import {readFileSync} from 'node:fs';
import {resolve, dirname} from 'node:path';
import {fileURLToPath} from 'node:url';
import test from 'node:test';

const root = fileURLToPath(new URL('../../', import.meta.url));
const owner = 'rust/tests/unit/docs_requirements/agent_cli_shell.rs';
function replayWitness(overrides = new Map()) {
  const read = file => overrides.get(file) ?? readFileSync(resolve(root, file), 'utf8');
  const witness = read(owner);
  const captures = [...witness.matchAll(/let (router|classification) =\s*read\(root\.join\(\s*"([^"]+)"[,]?\s*\)\);\s*assert_contains_all\(\s*"[^"]+",\s*&(router|classification),\s*&\[([\s\S]*?)\],\s*\);/gu)];
  assert(captures.length, 'the real native witness must read and assert its capability sources');
  for (const [, binding, file, asserted, literals] of captures) {
    assert.equal(asserted, binding);
    const needles = [...literals.matchAll(/"(?:\\.|[^"\\])*"/gu)].map(match => JSON.parse(match[0]));
    assert(needles.length);
    for (const needle of needles) assert(read(file).includes(needle), file + ' lacks ' + needle);
  }
  const parent = captures.find(match => match[1] === 'router');
  const child = captures.find(match => match[1] === 'classification');
  assert(parent && child, 'the native witness must trace both owners');
  const declaration = /#\[path = "([^"]+)"\]\s*mod tool_classification;/u.exec(read(parent[2]));
  assert(declaration, 'the parent must actually compile the asserted child');
  assert.equal(resolve(root, child[2]), resolve(root, dirname(parent[2]), declaration[1]));
}

test('the native shell source witness follows the compiled capability child', () => replayWitness());
test('the native witness rejects disconnected modules and missing seed lookup', () => {
  const parent = 'rust/src/agentic_coding/capability_router.rs';
  const child = 'rust/src/agentic_coding/capability_router/tool_classification.rs';
  assert.throws(() => replayWitness(new Map([[parent, readFileSync(resolve(root, parent), 'utf8')
    .replace('mod tool_classification;', 'mod disconnected;')]])), /lacks|compile/u);
  assert.throws(() => replayWitness(new Map([[child, readFileSync(resolve(root, child), 'utf8')
    .replaceAll('seed::agentic_tool_capabilities()', 'disconnected_registry()')]])), /lacks/u);
});
