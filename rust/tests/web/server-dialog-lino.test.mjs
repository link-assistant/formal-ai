import assert from 'node:assert/strict';
import fs from 'node:fs';import os from 'node:os';import path from 'node:path';
import {writeDialogExchange} from '../../../js/server/dialog-log.mjs';
import {loadConversationContext} from '../../../js/server/conversations.mjs';
import {encodeDialogRecord,decodeDialogRecord} from '../../../js/server/dialog-codec.mjs';
import {runLogsCli} from '../../../js/server/logs-cli.mjs';
import {f64,toCompactJson} from '../../../js/server/json.mjs';
const dir=fs.mkdtempSync(path.join(os.tmpdir(),'formal-ai-lino-test-'));
try{
const record={integer:9007199254740993n,float:f64(-0),nested:['x\n%()中文',false,null]};
assert.deepEqual(decodeDialogRecord(encodeDialogRecord(record)),record);
const file=writeDialogExchange(dir,'POST','/v1/chat/completions',[['x-formal-ai-dialog-id','test']],'{"messages":[{"role":"user","content":"hello"}]}',200,'application/json','{"choices":[{"message":{"role":"assistant","content":"world"}}]}');
assert.equal(file,path.join(dir,'test.lino'));assert(!fs.existsSync(path.join(dir,'test.jsonl')));
let context=loadConversationContext('test',{FORMAL_AI_DIALOG_LOG_DIR:dir});assert.equal(context.messages.length,2);assert.equal(context.metadata.exchange_count,1);
const original=fs.readFileSync(file,'utf8');const prior={...decodeDialogRecord(original.trimEnd())};prior.request_id='legacy';fs.writeFileSync(path.join(dir,'test.jsonl'),toCompactJson(prior)+'\n');
context=loadConversationContext('test',{FORMAL_AI_DIALOG_LOG_DIR:dir});assert.equal(context.metadata.exchange_count,2);assert.equal(context.server_logs[0].request_id,'legacy');
const json=runLogsCli([file,'--format','json']);assert.equal(json.code,0);assert.equal(JSON.parse(json.stdout).length,1);
assert.equal(runLogsCli([file,'--format','lino']).stdout,original);
fs.appendFileSync(file,'(');assert.throws(()=>loadConversationContext('test',{FORMAL_AI_DIALOG_LOG_DIR:dir}),/unterminated_dialog_log/);
assert.equal(runLogsCli([file,'--format','json']).code,2);
console.log('DIALOG_LINO_CONSUMER_PASS');
}finally{fs.rmSync(dir,{recursive:true,force:true});}
