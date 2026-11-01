import test, { before } from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { createHash } from 'node:crypto';
import { WorkerHost } from '../../../js/server/worker-host.mjs';
import { installNodeHost } from '../../../js/agentic/node-host.mjs';
import { planWorkItemEvidence, workItemEvidencePaths } from '../../../js/agentic/work_item_evidence.mjs';
import { planChatStep } from '../../../js/agentic/planner.mjs';
const fixture = JSON.parse(readFileSync('data/benchmarks/fixtures/work-item-1455/failure.json', 'utf8'));
before(async () => installNodeHost(new WorkerHost({
  WebAssembly: {
    compile() {
      throw Error('LocalProductWasmForbidden');
    },
    instantiate() {
      throw Error('LocalProductWasmForbidden');
    }
  }
})));
function observe(messages, tool, args, content, extra = {}) {
  const id = 'observation-' + messages.length;
  messages.push({
    role: 'assistant',
    content: '',
    tool_calls: [{
      id,
      type: 'function',
      function: {
        name: tool,
        arguments: JSON.stringify(args)
      }
    }]
  });
  messages.push({
    role: 'tool',
    tool_call_id: id,
    name: tool,
    content,
    ...extra
  });
}
test('the evidence collector retains original task sources without claiming an implementation', async () => {
  const paths = workItemEvidencePaths(fixture.objective);
  assert.deepEqual(paths, ['docs/associative-tech-stack.md', 'rust/Cargo.toml', 'package.json', 'js/web-search-component-entry.js']);
  const messages = [{
    role: 'user',
    content: 'Implement ' + fixture.issue
  }];
  const tools = ['fetch_url', 'read', 'write'];
  const retrieval = await planChatStep(messages, tools);
  assert.equal(retrieval.kind, 'tool_calls');
  assert.equal(retrieval.calls[0].tool, 'fetch_url');
  assert.equal(JSON.parse(retrieval.calls[0].arguments).url, fixture.issue);
  observe(messages, retrieval.calls[0].tool, JSON.parse(retrieval.calls[0].arguments), fixture.title + '\n\n' + fixture.objective);
  for (const path of paths) {
    const step = planWorkItemEvidence(fixture.title + '\n\n' + fixture.objective, messages, tools, fixture.issue);
    assert.equal(step.kind, 'tool_calls');
    assert.equal(step.calls.length, 1);
    const call = step.calls[0],
      args = JSON.parse(call.arguments);
    assert.equal(call.tool, 'read');
    assert.equal(args.path, path);
    observe(messages, 'read', args, readFileSync(path, 'utf8'), {
      source_read: {
        path,
        success: true,
        complete: true,
        format: 'raw'
      }
    });
  }
  const result = planWorkItemEvidence(fixture.title + '\n\n' + fixture.objective, messages, tools, fixture.issue);
  assert.equal(result.kind, 'final');
  const evidence = JSON.parse(result.answer);
  assert.equal(evidence.reason, 'EvidenceCollected');
  assert.equal(evidence.workItem, fixture.issue);
  assert.equal(evidence.authored, false);
  assert.equal(evidence.verified, false);
  assert.equal(evidence.decomposition[2].status, 'not_executed');
  assert.ok(evidence.requirements.length > 0);
  for (const requirement of evidence.requirements) {
    assert.equal(requirement.status, 'unbound');
    assert.equal(Buffer.from(fixture.title + '\n\n' + fixture.objective).subarray(...requirement.span).toString(), requirement.text);
  }
  for (const observation of evidence.observations) {
    assert.equal(observation.complete, true);
    assert.equal(observation.sourceIdentity, createHash('sha256').update(readFileSync(observation.path)).digest('hex'));
    assert.ok(Array.from(observation.excerpt).length <= 1024);
    if (/\.(?:json|toml)$/u.test(observation.path)) {
      assert.ok(observation.manifestFacts.length > 0);
      assert.equal(observation.factScope, 'literal-declarations-only; executable use remains unverified');
      for (const fact of observation.manifestFacts) assert.equal(fact.sourceIdentity, observation.sourceIdentity);
    }
  }
});
test('missing tool, failed and truncated reads terminate without repeating an action or writing', () => {
  const task = 'Investigate package.json before implementing the requested integration.';
  assert.equal(JSON.parse(planWorkItemEvidence(task, [], []).answer).reason, 'MissingReadTool');
  for (const extra of [{
    is_error: true
  }, {
    source_read: {
      path: 'package.json',
      success: true,
      complete: false,
      format: 'raw'
    }
  }]) {
    const messages = [{
      role: 'user',
      content: task
    }];
    observe(messages, 'read', {
      path: 'package.json'
    }, 'untrusted incomplete source', extra);
    const result = planWorkItemEvidence(task, messages, ['read', 'write']);
    assert.equal(result.kind, 'final');
    const report = JSON.parse(result.answer);
    assert.equal(report.reason, 'IncompleteSourceObservation');
    assert.equal(report.observations[0].sourceIdentity, null);
  }
});
test('paths are bounded, deduplicated and cannot become shell or traversal instructions', () => {
  assert.deepEqual(workItemEvidencePaths('Read ../private.json /etc/secrets.json https://example.com/page.md package.json package.json.'), ['package.json']);
  assert.equal(workItemEvidencePaths(Array.from({
    length: 20
  }, (_, i) => 'source' + i + '.js').join(' ')).length, 8);
  assert.deepEqual(workItemEvidencePaths('x'.repeat(32769) + ' package.json'), []);
  assert.equal(planWorkItemEvidence('An unsupported obligation with no repository paths', [], ['read']), null);
});
test('read prohibitions and oversized observations remain unverified', () => {
  assert.equal(JSON.parse(planWorkItemEvidence('Do not read package.json. Investigate package.json.', [], ['read']).answer).reason, 'ReadPolicyRefused');
  const task = 'Investigate package.json.',
    messages = [{
      role: 'user',
      content: task
    }];
  observe(messages, 'read', {
    path: 'package.json'
  }, 'x'.repeat(262145), {
    source_read: {
      path: 'package.json',
      success: true,
      complete: true,
      format: 'raw'
    }
  });
  assert.equal(JSON.parse(planWorkItemEvidence(task, messages, ['read']).answer).reason, 'IncompleteSourceObservation');
});
test('source references preserve native Unicode separator semantics', () => {
  assert.deepEqual(workItemEvidencePaths('Inspect\u0085Cargo.toml.'), ['Cargo.toml']);
  assert.deepEqual(workItemEvidencePaths('Inspect\uFEFFCargo.toml.'), []);
});

test('quoted documentation examples remain literal edit payloads',async()=>{
  const owner='tests/request.mjs',before='const note = "before";',after='const note = "Document docs/runtime.md using package.json and js/client.mjs.";';
  const task='In '+owner+', replace «'+before+'» with «'+after+'». Preserve all other content.';
  const messages=[{role:'user',content:task}],tools=['read_file','edit_file'];
  const read=await planChatStep(messages,tools);
  assert.equal(read.kind,'tool_calls');assert.equal(read.calls.length,1);
  assert.equal(read.calls[0].tool,'read_file');const args=JSON.parse(read.calls[0].arguments);
  assert.equal(args.path,owner);
  observe(messages,'read_file',args,before,{source_read:{path:owner,success:true,complete:true,format:'raw'}});
  const edit=await planChatStep(messages,tools);
  assert.equal(edit.kind,'tool_calls');assert.equal(edit.calls.length,1);
  assert.equal(edit.calls[0].tool,'edit_file');const replacement=JSON.parse(edit.calls[0].arguments);
  assert.equal(replacement.path,owner);assert.equal(replacement.old_string,before);assert.equal(replacement.new_string,after);
});

test('captured original issue with invalid evidence terminates without effects',async()=>{
  const messages=[{role:'user',content:'Implement '+fixture.issue}],tools=['fetch_url','read_file','write_file'];
  const retrieval=await planChatStep(messages,tools);
  assert.equal(retrieval.kind,'tool_calls');assert.equal(retrieval.calls.length,1);
  assert.equal(retrieval.calls[0].tool,'fetch_url');
  assert.equal(JSON.parse(retrieval.calls[0].arguments).url,fixture.issue);
  observe(messages,'fetch_url',JSON.parse(retrieval.calls[0].arguments),fixture.title+'\n\n'+fixture.objective);
  const paths=['rust/Cargo.toml','package.json','js/web-search-component-entry.js','docs/associative-tech-stack.md'];
  for(const path of paths){
    const step=await planChatStep(messages,tools);
    assert.equal(step.kind,'tool_calls');assert.equal(step.calls.length,1);
    const call=step.calls[0],args=JSON.parse(call.arguments);
    assert.equal(call.tool,'read_file');assert.equal(args.path,path);
    assert.equal(args.filePath,path);assert.equal(args.file_path,path);
    observe(messages,call.tool,args,'observed source\n',{source_read:{path,success:true,complete:true,format:'raw'}});
  }
  const result=await planChatStep(messages,tools);
  assert.equal(result.kind,'final');const state=JSON.parse(result.answer);
  assert.equal(state.schema,'repository-work-item-controller/v1');
  assert.equal(state.reason,'UnsupportedManifestDocumentationSource');
  for(const field of ['authored','verified','complete'])assert.equal(state[field],false);
  for(const field of ['candidateEdits','checks','effects'])assert.deepEqual(state[field],[]);
  assert.equal(state.workItem,fixture.issue);assert.deepEqual(state.observations.map(row=>row.path),paths);
  assert.ok(state.requirements.length>0);assert.ok(state.requirements.every(row=>row.status==='unverified'));
  for(const row of state.observations){assert.equal(row.complete,true);assert.equal(row.sourceIdentity,createHash('sha256').update('observed source\n').digest('hex'));}
});
