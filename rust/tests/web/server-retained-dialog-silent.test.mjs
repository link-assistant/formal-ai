import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
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
