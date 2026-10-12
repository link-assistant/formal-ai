// Native pins: issue_1066_agent_ladder tree anchors and issue_1069_ladder_change_tasks whole-node obligations.
import { before, test } from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { createHash } from 'node:crypto';
import { WorkerHost, REPO_ROOT } from '../../../js/server/worker-host.mjs';
import { installNodeHost } from '../../../js/agentic/node-host.mjs';
import { parseRuleDocument } from '../../../js/agentic/link_edit_rules.mjs';
import { planChatStep } from '../../../js/agentic/planner.mjs';
before(async () => { await installNodeHost(new WorkerHost()); });
const read = (path) => readFileSync(REPO_ROOT + '/' + path, 'utf8');
const leaf = read('experiments/issue_1028_agent_cli_ladder/leaves.tsv').trimEnd().split('\n')[6].split('\t');
const [id, task, path, marker, guard] = leaf;
const node = '1.1.2.2.1';
const source = read(path);
const changed = source.replace('good evening', 'good day');
const quote = String.fromCharCode(96);
const prompt = 'Atomic task ' + id + ': ' + task + '\n\nThis is recursive binary-tree node ' + node
  + ' at depth 5. Solve only this node\'s task in this fresh temporary repository. Its harness-evaluated completion criterion is: tracked_source_change. Apply the change to the tracked file ' + quote
  + path + quote + ' itself -- the file has to end up modified in the Git worktree, and nothing else may change. Then create ' + quote + 'agent-ladder-effects/node-'
  + node + '.lino' + quote + ' with these exact field lines: ' + quote + 'node_path=' + node + quote
  + ', ' + quote + 'node_depth=5' + quote + ', ' + quote + 'node_kind=leaf' + quote + ', and ' + quote + 'result=' + quote + ' followed by at least four words that state the change you made and that contain the exact text '
  + marker + '. Leave supporting evidence in .agent-ladder/node-' + node
  + '-proof.md. The first line must be exactly node_path=' + node
  + ' and the body must state the concrete result. The harness rejects proof without the separate Git effect. Use web research when it materially improves factual accuracy. Do not claim success without evidence.\n';

test('L07 derives a real canonical greeting edit and fulfills every whole-node obligation', async () => {
  assert.equal(id, 'L07');
  assert.equal(path, 'data/seed/meanings-conversation.lino');
  assert.equal(source.includes(marker), false);
  assert.equal(source.includes(guard), true);
  assert.deepEqual(parseRuleDocument(read('experiments/issue_1028_agent_cli_ladder/rules/L07.lino')).ok, {
    leaf: 'L07', path, language: 'lino', rule: { kind: 'replace_literal', old: 'good evening', new: 'good day' }, expect: ['good day'],
  });
  const workspace = new Map([[path, source]]);
  const messages = [{ role: 'user', content: prompt }];
  let finished = false;
  const recordReceipts = [];
  for (let step = 0; step < 16; step += 1) {
    const plan = await planChatStep(messages, ['read_file', 'write_file', 'edit_file', 'run_shell_command']);
    assert.ok(plan);
    if (plan.kind === 'final') { finished = true; break; }
    const calls = plan.calls.map((call, index) => ({ id: 'step_' + step + '_' + index, type: 'function', function: { name: call.tool, arguments: call.arguments } }));
    messages.push({ role: 'assistant', content: '', tool_calls: calls });
    for (const [index, call] of plan.calls.entries()) {
      const args = JSON.parse(call.arguments);
      let result = '';
      if (call.tool === 'read_file') result = workspace.get(args.path) ?? 'Error: File not found: ' + args.path;
      else if (call.tool === 'edit_file') workspace.set(args.path, workspace.get(args.path).replace(args.old_string, args.new_string));
      else if (call.tool === 'write_file') workspace.set(args.path, args.content);
      else {
        assert.equal(call.tool, 'run_shell_command');
        if (args.command.startsWith('cat ')) {
          const target = args.command.slice(4);
          assert.ok(['agent-ladder-effects/node-' + node + '.lino', '.agent-ladder/node-' + node + '-proof.md'].includes(target));
          assert.ok(workspace.has(target));
          const bytes = workspace.get(target);
          recordReceipts.push([target, bytes]);
          result = JSON.stringify({ stdout: bytes, exit_code: 0 });
        } else {
          assert.equal(args.command, 'sha256sum -- ' + path);
          result = createHash('sha256').update(workspace.get(path)).digest('hex') + '  ' + path + '\n';
        }
      }
      messages.push({ role: 'tool', tool_call_id: calls[index].id, name: call.tool, content: result });
    }
  }
  assert.equal(finished, true);
  assert.equal(workspace.get(path), changed);
  const effect = workspace.get('agent-ladder-effects/node-' + node + '.lino');
  assert.deepEqual(effect.split('\n').slice(0, 3), ['node_path=' + node, 'node_depth=5', 'node_kind=leaf']);
  const result = effect.split('\n').find((line) => line.startsWith('result=')).slice(7);
  assert.ok(result.includes(marker));
  assert.ok(result.split(/\s+/u).length >= 4);
  assert.equal(workspace.get('.agent-ladder/node-' + node + '-proof.md').split('\n')[0], 'node_path=' + node);
  assert.equal(workspace.size, 3);
  for (const target of ['agent-ladder-effects/node-' + node + '.lino', '.agent-ladder/node-' + node + '-proof.md']) {
    assert.ok(recordReceipts.some(([path, bytes]) => path === target && bytes === workspace.get(target)));
  }
});

test('the changed Links Notation greeting lexeme reaches the production greeting caller', async () => {
  const worker = new WorkerHost({ fetch: async (url) => {
    const relative = new URL(String(url), 'http://localhost/').pathname.replace(/^\/+/, '');
    const text = relative === 'seed/meanings-conversation.lino' ? changed : read((relative.startsWith('seed/') ? 'data/' : 'js/') + relative);
    return { ok: true, status: 200, text: async () => text };
  } });
  const realm = await worker.boot();
  const words = realm.wordsForRole('social_greeting');
  assert.equal(words.includes('good day'), true);
  assert.equal(words.includes('good evening'), false);
  assert.equal((await worker.solve('Good day')).intent, 'greeting');
});
