import test from 'node:test';
import assert from 'node:assert/strict';
import {
  parseNativeSession
}
from '../../../js/agentic/crate/orchestration_runner.mjs';
import {
  encodeTyped,decodeTyped,MAXIMUM_LOG_RECORD_BYTES
}
from '../../../js/server/structured-log.mjs';
const prefix='formal-ai: orchestration-session-json:';
const session={
  id:'session-α',resume_command:'formal-ai with --orchestration-resume session-α'
}
;
const envelope=(change={
}
)=>({
  schema:'formal-ai-control-v1',kind:'orchestration_session',message:prefix+JSON.stringify(session),...change
}
);
const parse=value=>parseNativeSession(encodeTyped(value)+'\n','',null);
test('typed control and legacy prefix preserve exact native session and resume fields',()=>{
  assert.deepEqual(parse(envelope()),session);
  assert.deepEqual(parseNativeSession('  '+prefix+JSON.stringify(session)+'\r\n','',null),session);
  assert.deepEqual(parse(envelope({
    message:prefix+JSON.stringify({
      ...session,additional:'legacy ignores this field'
    }
    )
  }
  )),session);
}
);
test('wrong schema, kind, field types and non-object records cannot become control evidence',()=>{
  for(const value of [null,[],true,'text',envelope({
    schema:'formal-ai-diagnostic-v1'
  }
  ),envelope({
    schema:null
  }
  ),envelope({
    kind:'other'
  }
  ),envelope({
    kind:[]
  }
  ),envelope({
    message:null
  }
  ),envelope({
    message:{
      id:'forged'
    }
  }
  ),envelope({
    message:42
  }
  ),{
    message:prefix+JSON.stringify(session)
  }
  ])assert.equal(parse(value),null);
}
);
test('malformed, noncanonical and near-prefix control lines refuse without substring promotion',()=>{
  const canonical=encodeTyped(envelope());
  for(const line of ['not links',canonical+' trailing',canonical.replace('formal-ai-control-v1','formal-ai-control-v2'),'(formal-ai-log-v1 (object (field schema (string formal-ai-control-v1))))'])assert.equal(parseNativeSession(line,'',null),null);
  for(const message of ['before '+prefix+JSON.stringify(session),prefix.replace(': orchestration',':  orchestration')+JSON.stringify(session),prefix.slice(0,-1)+JSON.stringify(session),JSON.stringify({
    embedded:prefix+JSON.stringify(session)
  }
  )])assert.equal(parse(envelope({
    message
  }
  )),null);
}
);
test('native payload validation remains unchanged for malformed JSON and missing/wrong session fields',()=>{
  for(const payload of ['not JSON','{}','[]','null','{"id":1,"resume_command":"run"}','{"id":"s","resume_command":null}','{"id":"s"}'])assert.equal(parse(envelope({
    message:prefix+payload
  }
  )),null);
}
);
test('large machine control remains intact above diagnostic budget while default diagnostic limit stays strict',()=>{
  const large={
    id:'s'.repeat(MAXIMUM_LOG_RECORD_BYTES+32),resume_command:'resume '+'r'.repeat(6000)
  }
  ;
  const line=encodeTyped(envelope({
    message:prefix+JSON.stringify(large)
  }
  ));
  assert.ok(Buffer.byteLength(line)>MAXIMUM_LOG_RECORD_BYTES);
  assert.throws(()=>decodeTyped(line),/record_size_limit/);
  assert.deepEqual(parseNativeSession(line,'',null),large);
  for(const limit of [-1,NaN,Infinity,'1024',Number.MAX_SAFE_INTEGER+1])assert.throws(()=>decodeTyped(encodeTyped({
  }
  ),{
    maximumRecordBytes:limit
  }
  ),/invalid_record_size_limit/);
}
);
test('stderr precedence and caller stdout resume fallback remain unchanged',()=>{
  const integration={
    invocation:{
      resume_command:'client resume {session_id}'
    }
  }
  ;
  const stdout='{"type":"thread.started","thread_id":"stdout-session"}\n';
  assert.deepEqual(parseNativeSession(encodeTyped(envelope()),stdout,integration),session);
  assert.deepEqual(parseNativeSession(encodeTyped(envelope({
    kind:'other'
  }
  )),stdout,integration),{
    id:'stdout-session',resume_command:'client resume stdout-session'
  }
  );
  assert.equal(parseNativeSession('',stdout,null),null);
  assert.equal(parseNativeSession('',stdout,{
    invocation:{
      resume_command:''
    }
  }
  ),null);
  assert.equal(parseNativeSession('', '{"partial":',integration),null);
}
);
test('invalid records do not hide later valid control or legacy evidence',()=>{
  const wrong=encodeTyped(envelope({
    kind:'wrong'
  }
  ));
  assert.deepEqual(parseNativeSession(wrong+'\n'+encodeTyped(envelope())+'\n','',null),session);
  assert.deepEqual(parseNativeSession(wrong+'\n'+prefix+JSON.stringify(session)+'\n','',null),session);
}
);
test('explicit record budget accepts exact boundary and refuses one extra byte without relaxing defaults',()=>{
  const base=Buffer.byteLength(encodeTyped({
    message:'x'
  }
  )),count=MAXIMUM_LOG_RECORD_BYTES-base+1;
  const atBoundary=encodeTyped({
    message:'x'.repeat(count)
  }
  ),above=encodeTyped({
    message:'x'.repeat(count+1)
  }
  );
  assert.equal(Buffer.byteLength(atBoundary),MAXIMUM_LOG_RECORD_BYTES);
  assert.equal(decodeTyped(atBoundary).message.length,count);
  assert.throws(()=>decodeTyped(above),/record_size_limit/);
  assert.equal(decodeTyped(above,{
    maximumRecordBytes:Buffer.byteLength(above)
  }
  ).message.length,count+1);
  assert.throws(()=>decodeTyped(atBoundary,{
    maximumRecordBytes:MAXIMUM_LOG_RECORD_BYTES-1
  }
  ),/record_size_limit/);
  assert.throws(()=>decodeTyped(encodeTyped({
  }
  ),{
    maximumRecordBytes:0
  }
  ),/record_size_limit/);
}
);
