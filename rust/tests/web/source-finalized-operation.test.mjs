import test from 'node:test';
import assert from 'node:assert/strict';
import {installHost} from '../../../js/agentic/host.mjs';
import {acceptedOperationCandidateStatus} from '../../../js/agentic/module_function/source-operation-port.mjs';
const operands=[Object.freeze({}),'request','command','workspace','destination','identity'];
test('raw process status cannot become finalized delivery authority',()=>{
  installHost({sourceOperation:{candidateStatus:()=>0}});
  assert.throws(()=>acceptedOperationCandidateStatus(...operands),/MissingCandidateOperationHost/u);
});
test('private completion refusal remains refused after a successful process',()=>{
  installHost({sourceOperation:{candidateStatus:()=>0,finalizedCandidateStatus:()=>null}});
  assert.equal(acceptedOperationCandidateStatus(...operands),null);
});
test('finalized host receives every source and candidate binding unchanged',()=>{
  let observed;
  installHost({sourceOperation:{finalizedCandidateStatus:(...actual)=>{observed=actual;return 0;}}});
  assert.equal(acceptedOperationCandidateStatus(...operands),0);
  assert.deepEqual(observed,operands);
});
