// The all-mode ladder keeps the full corpus while exchanging verified leaf evidence.
// Timings in these small fixtures are synthetic; actual CI timings remain mandatory.
import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { spawnSync } from 'node:child_process';
import { mkdtempSync, mkdirSync, readFileSync, writeFileSync, rmSync, cpSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join, resolve } from 'node:path';
import { test } from 'node:test';
import { workflowJobs } from '../../../scripts/lib/ci-speed-workflows.mjs';
const root = resolve(import.meta.dirname, '../../..');
const scripts = join(root, 'experiments/issue_1028_agent_cli_ladder');
function python(name, args = [], input) {
  return spawnSync('python3', [join(scripts, name), ...args], { encoding: 'utf8', input, env: { ...process.env, PYTHONDONTWRITEBYTECODE: '1',
    LADDER_SOURCE_COMMIT: 'a'.repeat(40), LADDER_SOURCE_TREE: 'b'.repeat(40) } });
}
function planned(mode = 'all', filter = '') {
  const process = python('plan-legs.py', [mode, filter]);
  assert.equal(process.status, 0, process.stderr);
  return JSON.parse(process.stdout);
}
function hash(text) { return createHash('sha256').update(text).digest('hex'); }
function fixture(mode = 'all', filter = '', rules = true) {
  const folder = mkdtempSync(join(tmpdir(), 'formal-ai-ladder-waves-'));
  const plan = planned(mode, filter);
  const planFile = join(folder, 'plan.json'); writeFileSync(planFile, JSON.stringify(plan));
  const treePath = join(folder, 'tree.tsv');
  const leaves = readFileSync(join(scripts, 'leaves.tsv'), 'utf8');
  const source = readFileSync(join(scripts, 'run.sh'), 'utf8');
  const generator = source.split('python3 - "$OUT/leaves.tsv" "$NODES" <<\'PY\'\n')[1].split('\nPY\n')[0];
  const generate = spawnSync('python3', ['-', join(scripts, 'leaves.tsv'), treePath], { input: generator, encoding: 'utf8' });
  assert.equal(generate.status, 0, generate.stderr);
  const tree = readFileSync(treePath, 'utf8');
  const rows = new Map(tree.trimEnd().split('\n').map(row => [row.split('\t')[0], row]));
  const shards = join(folder, 'shards');mkdirSync(shards);
  const legs = [...plan.legs, ...plan.composites];
  for (const leg of legs) {
    const dir = join(shards, leg.id);mkdirSync(dir);
    writeFileSync(join(dir, 'tree.tsv'), tree);writeFileSync(join(dir, 'leaves.tsv'), leaves);
    writeFileSync(join(dir, 'selected.tsv'), leg.nodes.map(node => rows.get(node)).join('\n') + '\n');
    writeFileSync(join(dir, 'run.log'), leg.nodes.map(node => `${node}\tPASS\tdepth=${leg.depth}`).join('\n') + '\n');
    writeFileSync(join(dir, 'durations.tsv'), leg.nodes.map(node => `${node}\t${leg.depth}\t1`).join('\n') + '\n');
    for (const node of leg.nodes) {
      const nodeDir = join(dir, node);mkdirSync(nodeDir);
      writeFileSync(join(nodeDir, 'effect.lino'), `node_path=${node}\nnode_kind=${leg.depth === 5 ? 'leaf' : 'requirement'}\nresult=synthetic fixture verified node effect\n`);
      writeFileSync(join(nodeDir, 'change.diff'), 'diff --git a/fixture b/fixture\n+synthetic fixture\n');
    }
    const selectedLeaves = leg.depth === 5 ? leg.nodes.length : 0;
    const values = { requested_depth: String(leg.depth), node_filter: leg.filter,
      authored_rules_enabled: String(rules), source_commit: 'a'.repeat(40), source_tree: 'b'.repeat(40),
      tree_sha256: hash(tree), leaves_sha256: hash(leaves), selected_nodes: String(leg.nodes.length), failures: '0',
      deepest_passing_level: String(leg.depth), leaf_nodes_selected: String(selectedLeaves), leaf_nodes_passing: String(selectedLeaves) };
    if (!rules) values.leaf_nodes_passing_without_authored_rules = String(selectedLeaves);
    writeFileSync(join(dir, 'ladder-result.lino'), 'ladder_result\n' + Object.entries(values).map(([key,value]) => `  ${key} "${value}"\n`).join(''));
  }
  return { folder, plan, planFile, shards, legs, output: join(folder, 'joined'), rows, tree, leaves };
}
function combine(f) { return python('combine-shards.py', [f.shards, f.output, String(f.legs.length), f.planFile]); }
function editResult(dir, key, value) {
  const file = join(dir, 'ladder-result.lino');
  writeFileSync(file, readFileSync(file, 'utf8').replace(new RegExp(`^  ${key} ".*"$`, 'm'), `  ${key} "${value}"`));
}
function withFixture(fn, mode = 'all', filter = '', rules = true) {
  const f = fixture(mode, filter, rules);try { fn(f); } finally { rmSync(f.folder, { recursive: true, force: true }); }
}

test('all mode plans exactly 63 original corpus nodes in 31 independent and 16 dependent legs', () => {
  const plan = planned();assert.equal(plan.expected_nodes.length, 63);
  assert.equal(plan.legs.length, 31);assert.equal(plan.composites.length, 16);
  assert.equal(new Set([...plan.legs, ...plan.composites].flatMap(leg => leg.nodes)).size, 63);
  assert.ok(plan.legs.every(leg => leg.nodes.length <= 2 && leg.depth !== 4));
  assert.ok(plan.composites.every(leg => leg.depth === 4 && leg.nodes.length === 1));
});
test('every binary subtree preserves its exact all-mode count and leg membership', () => {
  for (const prefix of ['R','1','2.1','1.2.1','2.2.1.2','1.1.1.1.1']) {
    const plan = planned('all', prefix);const depth = prefix === 'R' ? 0 : prefix.split('.').length;
    assert.equal(plan.expected_nodes.length, 2 ** (6-depth) - 1);
    assert.deepEqual(new Set([...plan.legs,...plan.composites].flatMap(leg => leg.nodes)),new Set(plan.expected_nodes));
  }
});
test('depth-five plan preserves 32 leaves in 16 pairs and a filtered single leaf', () => {
  assert.equal(planned('5').legs.length,16);const p = planned('5','1.1.1.1.1');
  assert.deepEqual(p.legs[0].nodes,['1.1.1.1.1']);assert.equal(p.legs[0].filter,'1.1.1.1.1');
});
test('invalid depths, filters and empty selections fail rather than validate no work', () => {
  for (const args of [['6'],['all','../escape'],['3','1.1.1.1']]) assert.notEqual(python('plan-legs.py',args).status,0);
});
for (const rules of [true,false]) test(`the 47-leg join preserves all 63 nodes and actual corpus bytes with rules=${rules}`, () => withFixture(f => {
  const result = combine(f);assert.equal(result.status,0,result.stderr);
  assert.equal(readFileSync(join(f.output,'tree.tsv'),'utf8'),f.tree);
  assert.equal(readFileSync(join(f.output,'leaves.tsv'),'utf8'),f.leaves);
  assert.equal(readFileSync(join(f.output,'selected.tsv'),'utf8').trimEnd().split('\n').length,63);
  assert.equal(readFileSync(join(f.output,'durations.tsv'),'utf8').trimEnd().split('\n').length,63);
  assert.match(result.stdout,/selected_nodes "63"/);assert.match(result.stdout,/leaf_nodes_passing "32"/);
  assert.match(result.stdout,/deepest_passing_level "0"/); // Preserve the existing summary definition.
}, 'all', '', rules));
const invalid = {
  'missing leg': f => rmSync(join(f.shards,f.legs[0].id),{recursive:true}),
  'repeated depth/subtree': f => cpSync(join(f.shards,f.legs[0].id),join(f.shards,'duplicate'),{recursive:true}),
  'missing selected node': f => writeFileSync(join(f.shards,f.legs[0].id,'selected.tsv'),''),
  'foreign node disposition': f => writeFileSync(join(f.shards,f.legs[0].id,'run.log'),'foreign\tPASS\tdepth=5\n'),
  'duplicate node disposition': f => {const p=join(f.shards,f.legs[0].id,'run.log');writeFileSync(p,readFileSync(p,'utf8').repeat(2));},
  'foreign source tree': f => editResult(join(f.shards,f.legs[0].id),'source_tree','c'.repeat(40)),
  'foreign source commit': f => editResult(join(f.shards,f.legs[0].id),'source_commit','c'.repeat(40)),
  'consistently foreign source tree': f => f.legs.forEach(leg => editResult(join(f.shards,leg.id),'source_tree','c'.repeat(40))),
  'repeated planned node': f => { f.plan.expected_nodes.push(f.plan.expected_nodes[0]);writeFileSync(f.planFile,JSON.stringify(f.plan)); },
  'extra planned node': f => { f.plan.expected_nodes.push('outside');writeFileSync(f.planFile,JSON.stringify(f.plan)); },
  'mixed rule mode': f => editResult(join(f.shards,f.legs[0].id),'authored_rules_enabled','false'),
  'conflicting corpus bytes': f => writeFileSync(join(f.shards,f.legs[0].id,'leaves.tsv'),'changed corpus\n'),
  'false result counts': f => editResult(join(f.shards,f.legs[0].id),'leaf_nodes_passing','0'),
  'missing actual elapsed rows': f => writeFileSync(join(f.shards,f.legs[0].id,'durations.tsv'),''),
  'a lossy or repeated plan': f => {f.plan.expected_nodes.pop();writeFileSync(f.planFile,JSON.stringify(f.plan));},
  'passing node without effect': f => rmSync(join(f.shards,f.legs[0].id,f.legs[0].nodes[0],'effect.lino')),
};
for (const [name,mutate] of Object.entries(invalid)) test(`join rejects ${name}`,()=>withFixture(f=>{mutate(f);assert.notEqual(combine(f).status,0);}));
test('a filtered all-mode join retains the complete requested subtree',()=>withFixture(f=>assert.equal(combine(f).status,0),'all','2.1'));
test('child import copies only the two declared leaves and rejects foreign source identity',()=>withFixture(f=>{
  assert.equal(combine(f).status,0);const parent=f.legs[0].filter;const target=join(f.folder,'composite');mkdirSync(target);
  writeFileSync(join(target,'tree.tsv'),f.tree);writeFileSync(join(target,'leaves.tsv'),f.leaves);
  writeFileSync(join(target,'selected.tsv'),f.rows.get(parent)+'\n');
  const args=[f.output,target,'a'.repeat(40),'b'.repeat(40),'true'];
  const imported=python('import-children.py',args);assert.equal(imported.status,0,imported.stderr);
  assert.equal(imported.stdout.trimEnd().split('\n').length,2);
  args[3]='c'.repeat(40);assert.notEqual(python('import-children.py',args).status,0);
},'5'));

function importTarget(f) {
  const target=join(f.folder,'import-target');mkdirSync(target);
  writeFileSync(join(target,'tree.tsv'),f.tree);writeFileSync(join(target,'leaves.tsv'),f.leaves);
  writeFileSync(join(target,'selected.tsv'),f.rows.get(f.legs[0].filter)+'\n');
  return [f.output,target,'a'.repeat(40),'b'.repeat(40),'true'];
}
test('failed leaf evidence remains a measured failure and is never imported as verified',()=>withFixture(f=>{
  const leg=f.legs[0], dir=join(f.shards,leg.id), failed=leg.nodes[0];
  const log=join(dir,'run.log');writeFileSync(log,readFileSync(log,'utf8').replace(failed+'\tPASS',failed+'\tFAIL'));
  editResult(dir,'failures','1');editResult(dir,'leaf_nodes_passing','1');editResult(dir,'deepest_passing_level','none');
  const joined=combine(f);assert.equal(joined.status,0,joined.stderr);assert.match(joined.stdout,/leaf_nodes_passing "31"/);
  const imported=python('import-children.py',importTarget(f));assert.equal(imported.status,0,imported.stderr);
  assert.equal(imported.stdout.trimEnd().split('\n').length,1);assert.ok(!imported.stdout.includes(failed));
},'5'));
for(const invalidChild of ['wrong node effect','empty diff','missing diff'])test('child import rejects '+invalidChild,()=>withFixture(f=>{
  assert.equal(combine(f).status,0);const node=f.legs[0].nodes[0], dir=join(f.output,node);
  if(invalidChild==='wrong node effect')writeFileSync(join(dir,'effect.lino'),'node_path=foreign\nnode_kind=leaf\n');
  if(invalidChild==='empty diff')writeFileSync(join(dir,'change.diff'),'');
  if(invalidChild==='missing diff')rmSync(join(dir,'change.diff'));
  assert.notEqual(python('import-children.py',importTarget(f)).status,0);
},'5'));

test('the real verdict shell requires all declared waves and allows an all-mode single-leaf selection',()=>{
  const workflow=readFileSync(join(root,'.github/workflows/issue-1028-agent-ladder.yml'),'utf8');
  const step=workflow.split('- name: Every ladder leg finished\n')[1].split('\n      - name:')[0];
  const shell=step.split('        run: |\n')[1].split('\n').map(line=>line.replace(/^          /,'')).join('\n');
  const verdict=(composites,leaf='success',composite='success')=>spawnSync('bash',['-c',shell],{
    cwd:root,encoding:'utf8',env:{...process.env,COMPOSITE_LEGS:composites,JOBS:'plan ladder',CHECK:'fixture',
      NEEDS_JSON:JSON.stringify({plan:{result:'success'},ladder:{result:'success'},'leaf-evidence':{result:leaf},composites:{result:composite}})}}).status;
  assert.equal(verdict('[]','skipped','skipped'),0);
  assert.equal(verdict('[{}]'),0);
  assert.notEqual(verdict('[{}]','failure'),0);
  assert.notEqual(verdict('[{}]','success','skipped'),0);
});

test('measured ladder retirement caps each wave and records the new static DAG',()=>{
  const workflow=readFileSync(join(root,'.github/workflows/issue-1028-agent-ladder.yml'),'utf8');
  const jobs=new Map(workflowJobs(workflow).map(job=>[job.id,job]));
  const longest=id=>Math.max(0,...jobs.get(id).needs.map(longest))+Math.max(...jobs.get(id).timeoutValues);
  assert.equal(longest('compare'),120);
  assert.equal(jobs.get('ladder').timeoutValues[0],30);
  assert.equal(jobs.get('composites').timeoutValues[0],30);
  assert.match(jobs.get('ladder').body,/TEST_BUDGET_SECONDS: 1200/);
  assert.match(jobs.get('ladder').body,/run-with-budget-warning\.sh/);
  const policy=readFileSync(join(root,'data/meta/ci-speed.lino'),'utf8');
  assert.doesNotMatch(policy,/workflow "\.github\/workflows\/issue-1028-agent-ladder\.yml"/);
  const record=readFileSync(join(root,'data/meta/ci-wall-clock.lino'),'utf8');
  assert.match(record,/measured_minutes 120/);assert.match(record,/ceiling_minutes 225/);
  assert.match(record,/workflow "\.github\/workflows\/issue-1028-agent-ladder\.yml"/);
});


function expressionValue(template, context) {
  return template.replace(/\$\{\{([\s\S]*?)\}\}/g, (_, expression) =>
    Function('inputs', 'github', 'runner', 'steps', `return (${expression});`)(context.inputs, context.github, context.runner, context.steps));
}
function ladderCheck(job, inputs) {
  const workflow = readFileSync(join(root, '.github/workflows/issue-1028-agent-ladder.yml'), 'utf8');
  const body = workflowJobs(workflow).find(candidate => candidate.id === job).body;
  const check = body.match(/^\s+check: (.+)$/m)?.[1];
  assert.ok(check, `${job} declares its real green-ledger check`);
  return expressionValue(check, { inputs, github: { event_name: 'workflow_dispatch' } });
}
function cacheIdentity(check, digest) {
  const action = readFileSync(join(root, '.github/actions/green-ledger/action.yml'), 'utf8');
  const keys = [...action.matchAll(/^\s+key: (.+)$/gm)].map(match => match[1]);
  assert.equal(new Set(keys).size, 1, 'lookup-only and recording cache actions use the same identity');
  return expressionValue(keys[0], { inputs: { check }, runner: { os: 'Linux' }, steps: { key: { outputs: { digest } } } });
}
function actualLedgerDecision(check, hit) {
  const folder = mkdtempSync(join(tmpdir(), 'formal-ai-ladder-cache-'));
  try {
    const action = readFileSync(join(root, '.github/actions/green-ledger/action.yml'), 'utf8');
    const shell = action.split('    - name: Decide, and say so\n')[1].split('      run: |\n')[1]
      .split('\n').map(line => line.replace(/^        /, '')).join('\n');
    const output = join(folder, 'outputs');
    const proc = spawnSync('bash', ['-c', shell], { cwd: folder, encoding: 'utf8', env: {
      ...process.env, CHECK: check, ENABLED: 'true', HIT: String(hit), RECORD: 'false', DIGEST: 'same-source',
      GITHUB_OUTPUT: output, GITHUB_STEP_SUMMARY: join(folder, 'summary'),
    } });
    assert.equal(proc.status, 0, proc.stderr);
    return readFileSync(output, 'utf8').includes('already-green=true');
  } finally { rmSync(folder, { recursive: true, force: true }); }
}

test('a filtered all-mode green cannot skip the complete63-node measurement', () => {
  const inputs = { depth: 'all', authored_rules: 'enabled', node_filter: '1.2' };
  const filtered = ladderCheck('compare', inputs);
  assert.equal(ladderCheck('plan', inputs), filtered);
  const cache = new Set([cacheIdentity(filtered, 'same-source')]);
  const accepts = candidate => {
    const check = ladderCheck('plan', candidate);
    return actualLedgerDecision(check, cache.has(cacheIdentity(check, 'same-source')));
  };
  assert.equal(accepts(inputs), true, 'identical measured inputs retain their valid skip');
  assert.equal(accepts({ ...inputs, node_filter: '' }), false, 'the complete corpus must actually run');
  assert.equal(accepts({ ...inputs, node_filter: '1.1' }), false);
  assert.equal(accepts({ ...inputs, authored_rules: 'disabled' }), false);
  assert.equal(accepts({ ...inputs, depth: '5' }), false);
});

test('full-corpus cache preserves exact source and never proves a differently filtered run', () => {
  const inputs = { depth: 'all', authored_rules: 'disabled', node_filter: '' };
  const check = ladderCheck('compare', inputs);
  assert.equal(ladderCheck('plan', inputs), check);
  const cache = new Set([cacheIdentity(check, 'measured-source')]);
  assert.equal(actualLedgerDecision(check, cache.has(cacheIdentity(check, 'measured-source'))), true);
  const filtered = ladderCheck('plan', { ...inputs, node_filter: '2.2' });
  assert.equal(actualLedgerDecision(filtered, cache.has(cacheIdentity(filtered, 'measured-source'))), false);
  assert.equal(actualLedgerDecision(check, cache.has(cacheIdentity(check, 'changed-source'))), false);
});
