import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import {dispatch} from '../../../js/server/dispatch.mjs';
import {decodeDialogRecord} from '../../../js/server/dialog-codec.mjs';
import {writeDialogExchange,recordApiExchangeIfEnabled} from '../../../js/server/dialog-log.mjs';
import {loadConversationContext,configuredDialogLogDirectory} from '../../../js/server/conversations.mjs';

for (const explicit of [false,true]) {
  for (const silent of ['true','yes','1','on']) {
    test('silent retained conversation read without new capture: explicit='+explicit+' silent='+silent,()=>{
      const root=fs.mkdtempSync(path.join(os.tmpdir(),'retained-dialog-silent-'));
      try {
        const logs=path.join(root,explicit?'explicit':'dialog-logs');
        const env={FORMAL_AI_MEMORY_PATH:path.join(root,'memory.lino'),FORMAL_AI_SILENT:silent};
        if(explicit)env.FORMAL_AI_DIALOG_LOG_DIR=logs;
        const request={method:'POST',path:'/v1/chat/completions',headers:[['x-formal-ai-dialog-id','retained']],body:'{"messages":[{"role":"user","content":"Retained user turn"}]}'};
        const response={statusCode:200,contentType:'application/json',body:'{"choices":[{"message":{"role":"assistant","content":"Retained assistant turn"}}]}'};
        writeDialogExchange(logs,request.method,request.path,request.headers,request.body,response.statusCode,response.contentType,response.body);
        const snapshot=()=>fs.readdirSync(logs).sort().map(name=>[name,fs.readFileSync(path.join(logs,name)).toString('base64')]);
        const before=snapshot();
        const context=loadConversationContext('retained',env);
        assert.equal(context.metadata.exchange_count,1);
        assert.deepEqual(context.messages.map(row=>row.content),['Retained user turn','Retained assistant turn']);
        assert.equal(configuredDialogLogDirectory(env),null);
        recordApiExchangeIfEnabled({...request,headers:[['x-formal-ai-dialog-id','must-not-capture']]},response,true,env);
        assert.deepEqual(snapshot(),before);
      } finally { fs.rmSync(root,{recursive:true,force:true}); }
    });
  }
}

// Reproduce the native fixture's health probe before its captured conversation.
test('an explicitly captured health probe adds one transport file without conversation turns',async()=>{
  const root=fs.mkdtempSync(path.join(os.tmpdir(),'health-dialog-capture-'));
  try {
    const env={FORMAL_AI_DIALOG_LOG_DIR:root,FORMAL_AI_MEMORY_PATH:path.join(root,'absent-memory.lino')};
    const response=await dispatch({env,bearerToken:'private-token'},{method:'GET',path:'/health',headers:[],body:''});
    assert.equal(response.statusCode,200);
    const healthFiles=fs.readdirSync(root);
    assert.equal(healthFiles.length,1);
    assert.ok(!healthFiles[0].endsWith('.conversation.lino'));
    const health=decodeDialogRecord(fs.readFileSync(path.join(root,healthFiles[0]),'utf8').trim());
    assert.equal(health.path,'/health');
    writeDialogExchange(root,'POST','/v1/chat/completions',[['x-formal-ai-dialog-id','captured']],
      '{"messages":[{"role":"user","content":"CAPTURED_DIALOG_MARKER"}]}',200,'application/json',
      '{"choices":[{"message":{"role":"assistant","content":"Acknowledged"}}]}');
    const files=fs.readdirSync(root);
    assert.equal(files.length,3);
    const dialogFiles=files.filter(name=>!healthFiles.includes(name));
    assert.equal(dialogFiles.length,2);
    assert.equal(dialogFiles.filter(name=>name.endsWith('.conversation.lino')).length,1);
    for(const name of dialogFiles)assert.ok(fs.readFileSync(path.join(root,name),'utf8').includes('CAPTURED_DIALOG_MARKER'));
    assert.ok(!fs.readFileSync(path.join(root,healthFiles[0]),'utf8').includes('CAPTURED_DIALOG_MARKER'));
  } finally {fs.rmSync(root,{recursive:true,force:true});}
});
