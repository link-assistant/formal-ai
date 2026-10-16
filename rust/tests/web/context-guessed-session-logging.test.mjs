import assert from 'node:assert/strict';
import fs from 'node:fs';import os from 'node:os';import path from 'node:path';
import {test} from 'node:test';
import {agentInfo} from '../../../js/server/seed.mjs';
import {createLogger,createServeLogger,decodeTyped} from '../../../js/server/structured-log.mjs';
const selected='conversation-rc6',variable='FORMAL_AI_DIALOG_ID';
const notice=agentInfo().get('context_session_guessed_notice').replace('{recorded}',selected).replace('{variable}',variable);
test('shared typed warning boundary retains the seeded guessed-session disclosure at debug/info/warn',()=>{
 for(const level of ['debug','info','warn']){
  const chunks=[],logger=createLogger({level,env:{},stderrWrite:bytes=>(chunks.push(Buffer.from(bytes)),bytes.length)});
  try{
   const result=logger.log('warn',notice,{component:'context',dialog_id:selected});assert.equal(result.emitted,true);
   const text=Buffer.concat(chunks).toString();assert.equal(text.split('\n').length,2);
   const record=decodeTyped(text.trimEnd());assert.equal(record.level,'warn');assert.equal(record.message,notice);
   assert.equal(record.dialog_id,selected);assert(record.message.includes('('+selected+')'));assert(record.message.includes(variable));assert(record.message.includes('--session <id>'));
   assert(!record.message.includes('{recorded}'));assert(!record.message.includes('{variable}'));
  }finally{logger.close();}
 }
});
test('explicit error-only and silent settings suppress the warning through the shared policy',()=>{
 for(const level of ['error','silent']){
  const chunks=[],logger=createLogger({level,env:{},stderrWrite:bytes=>(chunks.push(Buffer.from(bytes)),bytes.length)});
  try{assert.equal(logger.log('warn',notice,{component:'context',dialog_id:selected}).emitted,false);assert.equal(chunks.length,0);}finally{logger.close();}
 }
 const directory=fs.mkdtempSync(path.join(os.tmpdir(),'formal-ai-rc6-silent-'));
 try{for(const env of [{FORMAL_AI_SILENT:'1'},{FORMAL_AI_LOG_LEVEL:'silent'}]){
  const logs=path.join(directory,'logs'),logger=createServeLogger({env,logDirectory:logs});
  try{assert.equal(logger.log('warn',notice,{component:'context',dialog_id:selected}).emitted,false);assert.equal(fs.existsSync(logs),false);}finally{logger.close();}
 }}finally{fs.rmSync(directory,{recursive:true,force:true});}
});
