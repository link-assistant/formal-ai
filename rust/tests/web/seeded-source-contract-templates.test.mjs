import assert from 'node:assert/strict';
import {before,test} from 'node:test';
import {WorkerHost} from '../../../js/server/worker-host.mjs';
import {installNodeHost} from '../../../js/agentic/node-host.mjs';
import {readText,host,installHost} from '../../../js/agentic/host.mjs';
import {contractText,workspaceDiscoveryCommand,workspaceCandidateReadCommand} from '../../../js/agentic/workspace_discovery.mjs';
import {parseLinoRoot} from '../../../js/agentic/write_lino.mjs';
import {stableId} from '../../../js/agentic/crate/engine_stable_identifier.mjs';
before(async()=>{await installNodeHost(new WorkerHost());});
const seedPath='data/seed/workspace-discovery-contracts.lino';
function withSeed(text,action){const original=host();try{installHost({...original,readText:path=>path===seedPath?text:original.readText(path)});return action();}finally{installHost(original);}}
for(const need of ['Inspect the task model','Inspect the task model and record 😀 result.','node_path=1.1.1.1.1']) {
 test('exact original listing command '+need,()=>{
  const expected='# '+stableId('workspace_discovery_need',need)+"\nprintf '%s\\n' 'workspace-discovery-v1'; find . -maxdepth 4 -type f -not -path './.git/*' -print; discovery_status=$?; printf '%s\\n' 'workspace-discovery-end'; exit \"$discovery_status\"";
  assert.equal(workspaceDiscoveryCommand(need),expected);
 });
 test('exact original physical candidate command '+need,()=>{
  const path='src/task_model.rs';
  const expected='# '+stableId('workspace_candidate_need',need)+"\nprintf '%s\\n' 'workspace-source-v1'; head -c 65537 < '"+path+"'; discovery_status=$?; printf '\\n%s\\n' 'workspace-source-end'; exit \"$discovery_status\"";
  assert.equal(workspaceCandidateReadCommand(need,path),expected);
 });
}
for(const path of ['../a','a//b','./a','a;true',"a'b",''])test('unchanged unsafe physical operand refusal '+path,()=>assert.equal(workspaceCandidateReadCommand('Inspect task model',path),null));
test('exact accepted Node command operand',()=>assert.equal(contractText('accepted-node-test-command',['x.test.mjs']),'node --test x.test.mjs'));
test('template arguments are copied once rather than reparsed',()=>assert.equal(contractText('accepted-node-test-command',['{} {path}']),'node --test {} {path}'));
test('unknown template refuses',()=>assert.throws(()=>contractText('caller-forged-template',[])));
test('missing source refuses before command',()=>withSeed('',()=>assert.throws(()=>workspaceDiscoveryCommand('Inspect task model'))));
test('wrong owned root refuses',()=>withSeed('forged-root\n  template accepted-node-test-command\n    text "node --test {}"\n',()=>assert.throws(()=>contractText('accepted-node-test-command',['a']))));
test('duplicate source records refuse',()=>withSeed('workspace-discovery-contracts\n  template k\n    text "a"\n  template k\n    text "b"\n',()=>assert.throws(()=>contractText('k'))));
test('duplicate text fields refuse',()=>withSeed('workspace-discovery-contracts\n  template k\n    text "a"\n    text "b"\n',()=>assert.throws(()=>contractText('k'))));
test('absent text refuses',()=>withSeed('workspace-discovery-contracts\n  template k\n',()=>assert.throws(()=>contractText('k'))));
test('wrong positional arity refuses',()=>assert.throws(()=>contractText('accepted-node-test-command',[])));
test('extra positional operands refuse',()=>assert.throws(()=>contractText('accepted-node-test-command',['a','b'])));
test('nonstring positional operands refuse',()=>assert.throws(()=>contractText('accepted-node-test-command',[{}])));

test('canonical and compact source lexemes preserve exact original order',()=>{
 const document=parseLinoRoot(readText('data/seed/source-authoring-grammar.lino'));
 const composition=document.children.find(node=>node.name==='source-callable-composition');
 const goal=composition.children.find(node=>node.name==='goal'&&node.id==='optional-source-text');
 const english=goal.children.filter(node=>node.name==='lexeme'&&node.id==='en');
 assert.equal(english.length,1);
 const surfaces=english.flatMap(node=>node.children.filter(value=>value.name==='surface').flatMap(surface=>surface.children.filter(value=>value.name==='text').map(value=>value.id)));
 assert.deepEqual(surfaces,['canonical summary','compact summary']);
});
