import assert from 'node:assert/strict';
import {execFileSync} from 'node:child_process';
import {readFileSync} from 'node:fs';
import {test} from 'node:test';

const before = '33bac27018f08e171424321344b930cdbfe29bfb';
const planner = 'rust/src/agentic_coding/planner.rs';
const requestRoutes = 'rust/src/agentic_coding/planner/request_routes.rs';
const [requestSignature, settledSignature] = ['fn plan_chat_step_routes(', 'fn plan_settled_routes('];
const oldSource = path => execFileSync('git', ['show', `${before}:${path}`], {encoding: 'utf8'});
const currentSource = path => readFileSync(path, 'utf8');

// Mirror the Rust gate's brace scan for the two source functions, without compiling Rust.
function functionBody(source, signature) {
  const start = source.indexOf(signature);
  assert.ok(start >= 0, signature);
  const open = source.indexOf('{', start);
  assert.ok(open >= 0, signature);
  let depth = 1;
  for (let index = open + 1; index < source.length; index += 1) {
    if (source[index] === '{') depth += 1;
    if (source[index] === '}' && --depth === 0) return source.slice(open + 1, index);
  }
  assert.fail(`Incomplete body for ${signature}`);
}

const count = (source, signature) => functionBody(source, signature).split('\n')
  .filter(line => line === '    if' || line.startsWith('    if ')).length;

test('measure both planner functions and enforce the reviewed ceiling', () => {
  const beforeCount = count(oldSource(requestRoutes), requestSignature)
    + count(oldSource(planner), settledSignature);
  const afterCount = count(currentSource(requestRoutes), requestSignature)
    + count(currentSource(planner), settledSignature);
  const ledger = currentSource('data/meta/capability-routing-ratchet.lino');
  const ceiling = Number(ledger.match(/measure planner-route-arms\n    value (\d+)/u)?.[1]);
  assert.equal(beforeCount, 61);
  assert.equal(afterCount, 50);
  assert.equal(afterCount, ceiling);
});

test('all twelve existing recipe predicates and steps retain their order', () => {
  const oldPlanner = functionBody(oldSource(planner), settledSignature);
  const oldRecipes = [...oldPlanner.matchAll(/if (\w+::is_\w+_task)\(task\) \{\n        return Some\((plan_\w+_step)\(/gu)]
    .map(([, predicate, step]) => [predicate, step]);
  const recipes = currentSource('rust/src/agentic_coding/document_recipe.rs')
    .split('const SELF_REFERENTIAL_RECIPES:')[1].split('];')[0];
  const table = [...recipes.matchAll(/(\w+::is_\w+_task),[\s\S]*?(plan_\w+_step)/gu)]
    .map(([, predicate, step]) => [predicate, step]);
  assert.equal(table.length, 12);
  const first = oldRecipes.findIndex(([predicate]) => predicate === table[0][0]);
  assert.ok(first >= 0);
  assert.deepEqual(table, oldRecipes.slice(first, first + table.length));
});
