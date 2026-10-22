import {before, test} from 'node:test';
import assert from 'node:assert/strict';
import {WorkerHost} from '../../../js/server/worker-host.mjs';
import {installNodeHost} from '../../../js/agentic/node-host.mjs';
import {repeatedActionStop} from '../../../js/agentic/repetition_guard.mjs';

before(async () => { await installNodeHost(new WorkerHost()); });
// Issue1364's original Codex rollout: two identical empty web_run calls with
// structured INVALID_ARGUMENT receipts. No account/session metadata is retained.
const tool = 'mcp__codex_apps__search_service___web_run';
const failure = JSON.stringify({isError:true, structuredContent:{
  error:'InvalidActionArgumentsError', error_code:'INVALID_ARGUMENT',
  error_data:{reason:'missing_required_argument', failure_stage:'argument_binding'},
}, content:[{type:'text', text:'InvalidActionArgumentsError: Provide either an input string or top-level Search commands, without combining them.'}]});
function transcript(results, options = {}) {
  const messages = [{role:'user', content:'Fix the failing coding task.'}];
  results.forEach((content, index) => {
    const id = options.unique ? String(index) : 'same-client-id';
    messages.push({role:'assistant', content:'', tool_calls:[{id, function:{
      name:index % 2 ? tool.toUpperCase() : tool,
      arguments:index % 2 ? ' { } ' : '{}',
    }}]});
    if (content !== null) messages.push({role:'tool', name:tool, tool_call_id:id, content});
  });
  return messages;
}
test('original structured invalid-argument pair stops after second failure, not third', () => {
  assert.equal(repeatedActionStop(transcript([failure])), null);
  assert.match(repeatedActionStop(transcript([failure, failure])), /after 2 attempts/);
});
test('successful and unreported results retain the separate three-action cap', () => {
  for (const result of [JSON.stringify({exit_code:0}), 'unrecognized failure', null]) {
    assert.equal(repeatedActionStop(transcript([result, result])), null);
    assert.match(repeatedActionStop(transcript([result, result, result])), /after 3 attempts/);
  }
});
test('a duplicate or unbound tool receipt cannot spend a second failure', () => {
  const messages = transcript([failure]);
  messages.push({...messages.at(-1)}, {...messages.at(-1), tool_call_id:'unbound'});
  assert.equal(repeatedActionStop(messages), null);
});
test('mismatched tool owner and duplicate simultaneous IDs cannot certify failures', () => {
  const messages = transcript([failure, failure]);
  messages[2].name = 'different';
  assert.equal(repeatedActionStop(messages), null);
  const duplicate = transcript([failure, failure]);
  duplicate[1].tool_calls.push({...duplicate[1].tool_calls[0]});
  // Three actual calls still trigger the independent overall cap.
  assert.match(repeatedActionStop(duplicate), /after 3 attempts/);
});
test('two distinct failed operands do not combine and JSON key order does', () => {
  const messages = transcript([failure, failure]);
  messages[1].tool_calls[0].function.arguments = '{"a":1,"b":2}';
  messages[3].tool_calls[0].function.arguments = '{"b":2,"a":1}';
  assert.match(repeatedActionStop(messages), /after 2 attempts/);
  messages[3].tool_calls[0].function.arguments = '{"a":2,"b":2}';
  assert.equal(repeatedActionStop(messages), null);
});

test('empty or missing call IDs cannot make failure receipts authoritative', () => {
  for (const identifier of ['', undefined]) {
    const messages = transcript([failure, failure]);
    for (const message of messages) {
      if (message.role === 'assistant') message.tool_calls[0].id = identifier;
      if (message.role === 'tool') message.tool_call_id = identifier;
    }
    assert.equal(repeatedActionStop(messages), null);
  }
});
test('an explicitly empty tool owner cannot impersonate the pending call', () => {
  const messages = transcript([failure, failure]);
  messages[2].name = '';
  assert.equal(repeatedActionStop(messages), null);
});

test('authoritative message failure flags honor both supported protocol spellings', () => {
  for (const flag of ['is_error', 'isError']) {
    const messages = transcript(['provider detail', 'provider detail']);
    messages[2][flag] = true;
    messages[4][flag] = true;
    assert.match(repeatedActionStop(messages), /after 2 attempts/, flag);
  }
});
test('nonboolean and false message flags do not manufacture authoritative failures', () => {
  for (const flag of [false, 'true', 1]) {
    const messages = transcript(['provider detail', 'provider detail']);
    messages[2].isError = flag;
    messages[4].isError = flag;
    assert.equal(repeatedActionStop(messages), null);
  }
});


import {chatCompletionFromPlan, responseFromPlan} from '../../../js/server/agentic.mjs';
test('two failed operands do not suppress a distinct verification candidate', () => {
  const messages = transcript([failure, failure]);
  assert.equal(repeatedActionStop(messages, []), null);
  assert.equal(repeatedActionStop(messages, [{tool:'bash', arguments:'{"command":"cat result.txt"}'}]), null);
  assert.match(repeatedActionStop(messages, [{tool, arguments:' { } '}]), /after 2 attempts/);
  assert.match(repeatedActionStop(messages), /after 2 attempts/);
});
test('candidate filtering never resets the independent global three-action or objective cap', () => {
  for (const candidates of [[], [{tool:'different', arguments:'{}'}]]) {
    assert.match(repeatedActionStop(transcript([null, null, null]), candidates), /after 3 attempts/);
    const messages = [{role:'user', content:'Fix the failing coding task.'}];
    for (let index=0;index<128;index+=1) messages.push({role:'assistant',tool_calls:[{
      id:String(index),function:{name:'unknown_'+index,arguments:'{}'},
    }]});
    assert.match(repeatedActionStop(messages,candidates), /128/);
  }
});
test('both wire surfaces guard actual schema-bound aliases after command projection', () => {
  const messages = transcript([JSON.stringify({exit_code:1}), JSON.stringify({exit_code:1})]);
  for (const message of messages) {
    if (message.role==='assistant') message.tool_calls[0].function={name:'bash',arguments:'{"cmd":"same"}'};
    if (message.role==='tool') message.name='bash';
  }
  const request={messages,tools:[{type:'function',function:{name:'bash',parameters:{
    type:'object',properties:{cmd:{type:'string'}},required:['cmd'],additionalProperties:false,
  }}}]};
  const makePlan=command=>({kind:'tool_calls',calls:[{tool:'bash',arguments:JSON.stringify({command})}]});
  const chat=chatCompletionFromPlan({},request,'Fix the failing coding task.',makePlan('same'),0);
  const response=responseFromPlan({},request,request,'Fix the failing coding task.',makePlan('same'),0);
  assert.deepEqual(chat.choices[0].message.tool_calls,[]);
  assert.match(chat.choices[0].message.content,/after 2 attempts/);
  assert.ok(!response.output.some(item=>item.type==='function_call'));
  const distinct=chatCompletionFromPlan({},request,'Fix the failing coding task.',makePlan('cat result.txt'),0);
  assert.equal(distinct.choices[0].message.tool_calls.length,1);
  assert.deepEqual(JSON.parse(distinct.choices[0].message.tool_calls[0].function.arguments),{cmd:'cat result.txt'});
});
