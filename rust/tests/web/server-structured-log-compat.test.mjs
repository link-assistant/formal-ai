// Independent compatibility oracles retained from existing debugger/JSON contracts.
import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import {test} from 'node:test';
import {startServer} from '../../../js/server/main.mjs';
import {decodeDialogRecord} from '../../../js/server/dialog-codec.mjs';
import {recordApiExchangeIfEnabled} from '../../../js/server/dialog-log.mjs';
import {loadConversationContext,turnsInExchange} from '../../../js/server/conversations.mjs';
import {parseJson,sortedKeys,toCompactJson} from '../../../js/server/json.mjs';
import {createLogger,encodeTyped,makeRedactor} from '../../../js/server/structured-log.mjs';
import {temporaryHome} from './support/structured-log-fixture.mjs';

test('regression: debugger loopback refusal retains the exact public host',async t=>{
  await assert.rejects(startServer({host:'0.0.0.0',port:0,debugSession:true,
    env:{HOME:temporaryHome(t)}}),/^Error: debug_session_requires_loopback:0\.0\.0\.0$/u);
});

test('regression: legacy writer and reader retain float, negative zero and large integers',t=>{
  const home=temporaryHome(t),env={HOME:home,FORMAL_AI_MEMORY_PATH:path.join(home,'memory.lino'),
    FORMAL_AI_DIALOG_LOG_DIR:path.join(home,'dialogs'),FORMAL_AI_API_KEY:'env-private'};
  const request='{"model":"formal-ai","messages":[{"role":"user","content":"hi","weight":1.0,"counter":9007199254740993,"negative":-0.0,"api_key":"field-private","echo":"env-private"}]}';
  const response='{"choices":[{"message":{"role":"assistant","content":"ok","score":2.0,"counter":18446744073709551615,"password":"response-private"}}]}';
  const expected='[{"api_key":"[REDACTED]","content":"hi","counter":9007199254740993,"echo":"[REDACTED]","negative":-0.0,"role":"user","weight":1.0},{"content":"ok","counter":18446744073709551615,"password":"[REDACTED]","role":"assistant","score":2.0}]';
  const chunks=[],logger=createLogger({env,directory:path.join(home,'typed'),
    stderrWrite:bytes=>(chunks.push(Buffer.from(bytes)),bytes.length)});
  try{
    recordApiExchangeIfEnabled({method:'POST',path:'/v1/chat/completions',
      headers:[['x-formal-ai-dialog-id','numeric-fidelity']],body:request},
    {statusCode:200,contentType:'application/json',body:response},true,env,logger);
  }finally{logger.close();}
  const traceFile=path.join(home,'dialogs/numeric-fidelity.lino');
  const record=decodeDialogRecord(fs.readFileSync(traceFile,'utf8').trim());
  assert.equal(toCompactJson(sortedKeys(turnsInExchange(record.request_body,record.response_body))),expected);
  assert.equal(toCompactJson(sortedKeys(loadConversationContext('numeric-fidelity',env).messages)),expected);
  const sinks=fs.readFileSync(traceFile,'utf8')+
    fs.readFileSync(path.join(home,'dialogs/numeric-fidelity.conversation.lino'),'utf8')+
    Buffer.concat(chunks).toString();
  for(const secret of ['field-private','env-private','response-private']){
    assert.equal(sinks.includes(secret),false);
  }
});

test('legacy numeric support does not silently widen the diagnostic typed model',()=>{
  const raw='{"a":1.0,"negative":-0.0,"big":9007199254740993,"password":"private"}';
  const preserved=makeRedactor({},{preserveJsonNumbers:true})(parseJson(raw));
  assert.equal(toCompactJson(preserved),raw.replace('private','[REDACTED]'));
  assert.throws(()=>encodeTyped(preserved));
  assert.throws(()=>makeRedactor({})(parseJson(raw)));
});
