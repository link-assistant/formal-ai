// Recheck the complete seed migration without compiling or executing Rust.
// Supplied implementation and conclusions receive no independent authorship credit.
import assert from 'node:assert/strict';
import { isDeepStrictEqual } from 'node:util';
import { createHash } from 'node:crypto';
import { execFileSync } from 'node:child_process';
import { readFileSync } from 'node:fs';
import { parseLino } from '../../js/server/lino.mjs';
import { WorkerHost, evaluate } from '../../js/server/worker-host.mjs';

const oldRef = process.argv[2] ?? 'e8c35985a';
const before = file => execFileSync('git', ['show', `${oldRef}:${file}`], { encoding: 'utf8' });
const hash = text => createHash('sha256').update(text).digest('hex');
const programsFile = 'data/seed/hello-world-programs.lino';
const oldPrograms = before(programsFile);
const programs = readFileSync(programsFile, 'utf8');
assert(isDeepStrictEqual(parseLino(programs), parseLino(oldPrograms)), 'Every decoded program seed value must survive');

const rulesFile = 'data/seed/handler-rules.lino';
const oldRules = before(rulesFile);
const rules = readFileSync(rulesFile, 'utf8');
const context = await new WorkerHost().boot();
const value = expression => JSON.parse(evaluate(context, `JSON.stringify(${expression})`));
const oldTree = value(`handlerRulesParseTree(${JSON.stringify(oldRules)})`);
const tree = value(`handlerRulesParseTree(${JSON.stringify(rules)})`);
const restored = structuredClone(tree);
const changes = [];
function review(oldNodes, nodes, path = '') {
  assert.equal(oldNodes.length, nodes.length);
  oldNodes.forEach((old, index) => {
    const node = nodes[index];
    assert.equal(node.name, old.name);
    const location = `${path}/${old.name}:${old.args[0] ?? ''}`;
    assert.equal(node.args.length, old.args.length, location);
    old.args.forEach((operand, argument) => {
      if (operand === node.args[argument]) return;
      assert(operand.includes('|'), location);
      assert(node.args[argument].startsWith('(') && node.args[argument].endsWith(')'), location);
      const references = value(`crateModule("crate/seed.mjs").splitPipeList(${JSON.stringify(node.args[argument])})`);
      assert.deepEqual(references, operand.split('|'), location);
      changes.push({ location, argument, references });
      node.args[argument] = operand;
    });
    review(old.children, node.children, location);
  });
}
review(oldTree, restored);
assert.equal(changes.length, 5);
assert(isDeepStrictEqual(restored, oldTree), 'No rule, policy or table delta outside the five reviewed representations');
console.log(JSON.stringify({ oldRef, completeProgramValuesEqual: true,
  completeRulesCompared: true, changes, programs: { before: hash(oldPrograms), after: hash(programs) },
  rules: { before: hash(oldRules), after: hash(rules) },
  nativeCompilationOrExecution: false, independentAuthorshipCredit: 0 }, null, 2));
