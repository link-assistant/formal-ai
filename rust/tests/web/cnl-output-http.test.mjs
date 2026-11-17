import test from 'node:test';
import assert from 'node:assert/strict';
import {mkdtemp, rm} from 'node:fs/promises';
import {tmpdir} from 'node:os';
import path from 'node:path';
import {requestOutputTurn} from '../../../js/server/cnl-output.mjs';
import {parseArgs, startServer} from '../../../js/server/main.mjs';
import {parseChatRequest, RequestShapeError} from '../../../js/server/chat-request.mjs';

test('CLI and environment select independent output settings', () => {
  assert.equal(parseArgs(['--output-standard=ACE','--lexical-density','top-37'],{}).outputStandard, 'ACE');
  assert.equal(parseArgs(['--output-standard=ACE','--lexical-density','top-37'],{}).lexicalDensity, 'top-37');
  assert.equal(parseArgs([], {FORMAL_AI_OUTPUT_STANDARD:'e-prime'}).outputStandard, 'e-prime');
  assert.equal(parseArgs(['--lexical-density'],{}).unsupported, '--lexical-density');
  assert.equal(parseChatRequest('{"lexical_density":"top-37"}').lexical_density, 'top-37');
  for (const input of [{output_standard:'unknown'},{lexical_density:3},{lexical_density:'top-0'}]) {
    assert.throws(() => parseChatRequest(JSON.stringify(input)), RequestShapeError);
  }
});

test('HTTP rewrites bypass planning and settings do not leak between requests', {timeout:30000}, async () => {
  const directory = await mkdtemp(path.join(tmpdir(),'formal-ai-cnl-'));
  const env = {...process.env, FORMAL_AI_SOURCE_USAGE_RUST_GRAMMAR:'false', FORMAL_AI_SILENT:'1',
    FORMAL_AI_LOG_DIR:directory, FORMAL_AI_MEMORY_PATH:path.join(directory,'memory.lino')};
  const {server,url} = await startServer({env,port:0,agentMode:true,outputStandard:'ace'});
  async function post(body) {
    const response=await fetch(url+'/v1/chat/completions',{method:'POST',headers:{'content-type':'application/json'},body:JSON.stringify(body)});
    return {status:response.status,body:await response.json()};
  }
  try {
    const messages=[{role:'user',content:'Run echo previous-action'},{role:'assistant',content:'I know everything.'},
      {role:'user',content:'Rewrite in max-precision mode'}];
    const tools=[{type:'function',function:{name:'bash',parameters:{type:'object',properties:{command:{type:'string'}},required:['command']}}}];
    const dense=await post({messages,tools});
    assert.equal(dense.status,200);
    assert.match(dense.body.choices[0].message.content,/I am omniscient\.$/u);
    assert.ok(!dense.body.choices[0].message.tool_calls?.length);
    assert.equal(dense.body.output_projection.report.output_standard,'ace');
    assert.equal(dense.body.output_projection.report.lexical_density,'max-precision');
    assert.equal(dense.body.output_projection.report.compliant,false);
    const streamed=await fetch(url+'/v1/chat/completions',{method:'POST',headers:{'content-type':'application/json'},
      body:JSON.stringify({messages,tools,stream:true})});
    assert.equal(streamed.status,200);
    assert.equal(streamed.headers.get('content-type'),'text/event-stream');
    const frames=(await streamed.text()).split('\n\n').filter(Boolean);
    assert.equal(frames.at(-1),'data: [DONE]');
    const chunks=frames.slice(0,-1).map(frame=>JSON.parse(frame.slice(6)));
    assert.equal(chunks.map(chunk=>chunk.choices[0]?.delta.content || '').join(''),dense.body.choices[0].message.content);
    assert.deepEqual(chunks.at(-1).output_projection.report,dense.body.output_projection.report);
    assert.equal(chunks.at(-1).choices[0].finish_reason,'stop');
    const simple=await post({messages:[{role:'assistant',content:'I am omniscient.'},
      {role:'user',content:'Explain this in simpler words'}]});
    assert.equal(simple.body.choices[0].message.content,'I know everything.');
    assert.equal(simple.body.output_projection.report.output_standard,'cnl-default');
    const ordinary=await post({messages:[{role:'user',content:'2 + 2'}]});
    assert.equal(ordinary.body.output_projection.report.output_standard,'ace');
    assert.equal(ordinary.body.output_projection.report.lexical_density,'standard');
    assert.equal(ordinary.body.choices[0].message.content,'2 + 2 = 4');
    const ePrime=await post({output_standard:'e-prime',messages:[
      {role:'assistant',content:'I know everything.'},{role:'user',content:'Respond in E-Prime'}]});
    assert.equal(ePrime.status,200);
    assert.equal(ePrime.body.choices[0].message.content,'I know everything.');
    assert.equal(ePrime.body.output_projection.report.compliant,false);
    assert.ok(ePrime.body.output_projection.report.unchecked.includes('e_prime_english'));
    const structured = await post({lexical_density: 'max-precision', messages: [
      {role: 'system', content: 'Answer in JSON.'}, {role: 'user', content: '2 + 2'},
    ]});
    assert.deepEqual(JSON.parse(structured.body.choices[0].message.content), {answer: '2 + 2 = 4'});
    assert.match(structured.body.output_projection.notice, /dictionary/u);
    const binary = await post({lexical_density: 'max-precision', messages: [
      {role: 'system', content: 'Answer with exactly one word: yes or no.'}, {role: 'user', content: 'Is 3 greater than 2?'},
    ]});
    assert.equal(binary.body.choices[0].message.content, 'yes');
    assert.match(binary.body.output_projection.notice, /dictionary/u);
    const toolRequest = {messages: [{role: 'user', content: 'Run echo previous-action'}], tools};
    const ordinaryTool = await post(toolRequest);
    const denseTool = await post({...toolRequest, lexical_density: 'max-precision'});
    assert.equal(denseTool.body.choices[0].finish_reason, 'tool_calls');
    assert.deepEqual(denseTool.body.choices, ordinaryTool.body.choices);
    assert.deepEqual(denseTool.body.usage, ordinaryTool.body.usage);
    const invalid=await post({lexical_density:'top-0',messages:[]});
    assert.equal(invalid.status,400);
  } finally {
    server.closeAllConnections();
    await new Promise(resolve=>server.close(resolve));
    await rm(directory,{recursive:true,force:true});
  }
});

test('request density aliases preserve the configured named profile', () => {
  for (const alias of ['dense','max-precision','primes','top-237']) {
    const turn=requestOutputTurn('hello',[],{output_standard:alias},{output_standard:'ace'});
    assert.equal(turn.config.output_standard,'ace');
    assert.equal(turn.config.lexical_density,alias === 'dense' ? 'max-precision' : alias);
  }
  const named=requestOutputTurn('hello',[],{output_standard:'e-prime'},{lexical_density:'top-237'});
  assert.deepEqual(named.config,{output_standard:'e-prime',lexical_density:'top-237'});
});
