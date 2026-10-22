import test,{before} from 'node:test';
import assert from 'node:assert/strict';
import {WorkerHost} from '../../../js/server/worker-host.mjs';
import {installNodeHost} from '../../../js/agentic/node-host.mjs';
import {host,installHost} from '../../../js/agentic/host.mjs';
import {planSourceCstTransformStep} from '../../../js/agentic/source_cst_transform.mjs';
before(async()=>{await installNodeHost(new WorkerHost());});
const request='Rename parameter count of function combine to quantity in example.mjs';
const stop=()=>{throw Error('refusal dispatched a command');};
function refusal(messages,tools){const result=planSourceCstTransformStep(request,messages,tools,stop);assert.equal(result.kind,'final');return result.answer;}
function read(argumentsValue,failed=false){return [{role:'assistant',tool_calls:[{id:'read',function:{name:'read_file',arguments:argumentsValue}}]},
 {role:'tool',tool_call_id:'read',content:'function combine(count){return count;}',is_error:failed}];}
test('seed migration preserves all six actual CST refusal messages exactly',()=>{
 const trusted=host();
 try{
  installHost({...trusted,sourceCstTransformCommand:()=>[process.execPath,'/private/tmp/trusted-transform.mjs']});
  const tools=['read_file','run_shell_command'];
  assert.equal(refusal(read('{'),tools),'Read arguments are not structured data.');
  assert.equal(refusal(read('{"path":"other.mjs"}'),tools),'Read observation belongs to a different source owner.');
  assert.equal(refusal(read('{"path":"example.mjs"}',true),tools),'Read observation failed.');
  assert.equal(refusal([],['read_file']),'The complete CST transformation requires an authorized command tool.');
  assert.equal(refusal([],['run_shell_command']),'The source preimage requires an authorized read tool.');
  installHost({...trusted,sourceCstTransformCommand:undefined});
  assert.equal(refusal([],tools),'The installed host does not provide the complete CST transformation command.');
 }finally{installHost(trusted);}
});
