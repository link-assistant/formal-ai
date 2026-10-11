import fs from 'node:fs';
import path from 'node:path';
import os from 'node:os';
import assert from 'node:assert/strict';
import { spawnSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';
import { test } from 'node:test';
import { WorkerHost } from '../../../js/server/worker-host.mjs';
import { installNodeHost } from '../../../js/agentic/node-host.mjs';
import { host, installHost } from '../../../js/agentic/host.mjs';
import { drive } from '../../../experiments/js_dogfood/drive.mjs';
import { createNodeSourceSessionHost, registeredSourceReferenceReadPlan }
  from '../../../js/server/node-source-session-host.mjs';
import { prepareTrackedRustSourceTransaction, executePreparedTrackedRustSource }
  from '../../../js/server/tracked-source-transaction.mjs';

const original = [
  "Atomic task L31: In the file rust/src/learning_adoption_ledger.rs, rename the constant UNK",
  "NOWN_INTENT to UNKNOWN_INTENT_NAME. Change only that file and keep it valid Rust.\n\nThis is",
  " recursive binary-tree node 2.2.2.2.1 at depth 5. Solve only this node's task in this fres",
  "h temporary repository. Its harness-evaluated completion criterion is: tracked_source_chan",
  "ge. Apply the change to the tracked file `rust/src/learning_adoption_ledger.rs` itself -- ",
  "the file has to end up modified in the Git worktree, and nothing else may change. Then cre",
  "ate `agent-ladder-effects/node-2.2.2.2.1.lino` with these exact field lines: `node_path=2.",
  "2.2.2.1`, `node_depth=5`, `node_kind=leaf`, and `result=` followed by at least four words ",
  "that state the change you made and that contain the exact text UNKNOWN_INTENT_NAME. Leave ",
  "supporting evidence in .agent-ladder/node-2.2.2.2.1-proof.md. The first line must be exact",
  "ly node_path=2.2.2.2.1 and the body must state the concrete result. The harness rejects pr",
  "oof without the separate Git effect. Use web research when it materially improves factual ",
  "accuracy. Do not claim success without evidence.\n",
].join('');
async function controls() {
  await installNodeHost(new WorkerHost());
  const root = fs.realpathSync(new URL('../../..', import.meta.url));
  const workspace = fs.realpathSync(fs.mkdtempSync(path.join(os.tmpdir(), 'tracked-platform-')));
  const operand = 'rust/src/learning_adoption_ledger.rs';
  const source = fs.readFileSync(root + '/' + operand);
  fs.mkdirSync(path.dirname(workspace + '/' + operand), { recursive: true });
  fs.writeFileSync(workspace + '/' + operand, source);
  const git = (...values) => {
    const result = spawnSync('/usr/bin/git', ['-C', workspace, ...values], { encoding: 'utf8' });
    assert.equal(result.status, 0, result.stderr);
    return result.stdout;
  };
  let phase = 0, observed = null;
  try {
    git('init', '--quiet');
    git('add', '--', operand);
    git('-c', 'user.name=Fixture', '-c', 'user.email=fixture@example.invalid',
      '-c', 'commit.gpgsign=false', 'commit', '--quiet', '--no-verify', '-m', 'fixture');
    const session = createNodeSourceSessionHost(relative => fs.readFileSync(root + '/' + relative, 'utf8'));
    installHost({ ...host(), sourceSession: session, sourceOperation: session.sourceOperation });
    const tools = ['read', 'write', 'bash'];
    const planner = async messages => {
      if (phase++ === 0) {
        const prerequisite = registeredSourceReferenceReadPlan(original, workspace, tools);
        assert.equal(prerequisite.kind, 'OnlySourceReferenceReadPrerequisite');
        assert.equal(prerequisite.mutationAuthority, false);
        assert.equal(prerequisite.pendingNeeds.length, 12);
        return prerequisite.plan;
      }
      const result = messages.findLast(value => value.role === 'tool');
      const assistant = messages.findLast(value => value.role === 'assistant' && value.tool_calls);
      const declaration = assistant.tool_calls.find(value => value.id === result.tool_call_id);
      const exact = { id: declaration.id, name: declaration.function.name,
        arguments: declaration.function.arguments };
      const prepared = await prepareTrackedRustSourceTransaction(
        result.owned_source_read, original, workspace, exact);
      observed = await executePreparedTrackedRustSource(prepared);
      assert.equal(observed.kind, 'ObservedTrackedSourceChange');
      assert.equal(observed.sourceChangeObserved, true);
      assert.equal(observed.verified, false);
      assert.equal(observed.unresolvedNeeds.length, 12);
      assert.equal(observed.nativeValidity, 'Unknown');
      return { kind: 'final', text: 'Gap: native validity and evidence obligations remain unresolved.' };
    };
    const result = await drive(planner, workspace, original, { tools, steps: 6 });
    assert.equal(phase, 2, JSON.stringify(result));
    assert.equal(result.transcript.length, 1);
    assert.equal(result.transcript[0].tool, 'read');
    const expected = source.toString('utf8').replaceAll('UNKNOWN_INTENT', 'UNKNOWN_INTENT_NAME');
    assert.equal(fs.readFileSync(workspace + '/' + operand, 'utf8'), expected);
    assert.equal(git('status', '--porcelain=v1', '-z', '--untracked-files=all'), ' M ' + operand + '\0');
    const directories = fs.readdirSync(workspace + '/.git').filter(value => value.startsWith('formal-ai-inverse-'));
    assert.equal(directories.length, 1);
    const inverse = JSON.parse(fs.readFileSync(workspace + '/.git/' + directories[0] + '/inverse.json', 'utf8'));
    assert.equal(Buffer.from(inverse.before, 'base64').equals(source), true);
    assert.equal(Buffer.from(inverse.after, 'base64').toString('utf8'), expected);
    assert.equal(inverse.request, original);
    assert.equal(inverse.pendingNeeds.length, 12);
    assert.equal(fs.existsSync(workspace + '/agent-ladder-effects'), false);
    assert.equal(fs.existsSync(workspace + '/.agent-ladder'), false);
    console.log(JSON.stringify({ actualTrackedEffect: true, actualDurableInverse: true,
      independentReadback: true, nativeValidity: 'Unknown', wholeL31Delivered: false,
      customTestPlanner: true, autonomousAuthorshipCredit: false, inverse, observed }));
  } finally { fs.rmSync(workspace, { recursive: true, force: true }); }
}
if (process.env.NODE_TEST_CONTEXT && !process.argv.includes('--tracked-platform-child')) {
  test('actual driver-bound tracked platform effect with durable inverse', () => {
    const env = { ...process.env }; delete env.NODE_TEST_CONTEXT;
    const result = spawnSync(process.execPath, [fileURLToPath(import.meta.url), '--tracked-platform-child'],
      { env, encoding: 'utf8' });
    console.log(result.stdout);
    assert.equal(result.status, 0, result.stderr + result.stdout);
  });
} else await controls();
