// Explicit input text owns its summary; conversation labels inside it cannot claim history.
import test from 'node:test';
import assert from 'node:assert/strict';
import {createWorkerContext,evaluate} from './support/browser-runtime.mjs';
const worker=createWorkerContext();const ready=evaluate(worker,'loadSeed()');
const history=[{role:'user',content:'你好'},{role:'assistant',content:'你好!'}];
async function solve(prompt,turns=[]){await ready;return worker.solve(prompt,turns,{}, {},[],{});}
test('original Chinese text-transform case keeps exact declared input with real prior history',async()=>{
 const prompt='总结一下：解析器读取文件。它构建一棵树。树被检查。';
 for(const turns of [[],history]){const result=await solve(prompt,turns);assert.equal(result.intent,'summarization_free_text');assert.equal(result.content,'解析器读取文件。');}
});
test('conversation vocabulary in supplied text does not change its owner',async()=>{
 await ready;const prompt='总结一下：这段对话记录在文件中。文件保存对话。对话已保存。';
 assert.equal(evaluate(worker,`isSummarizePrompt(normalizePrompt(${JSON.stringify(prompt)}),${JSON.stringify(prompt)})`),false);
 assert.equal(evaluate(worker,'isSummarizePrompt(normalizePrompt("Summarize: A conversation is recorded. The record is saved."),"Summarize: A conversation is recorded. The record is saved.")'),false);
});
test('bare and explicit conversation requests retain their original contextual owner',async()=>{
 for(const prompt of ['总结','总结对话','Summarize this conversation']){
  const result=await solve(prompt,history);assert.equal(result.intent,'summarize_conversation');assert.ok(result.content.includes('你好'));
 }
});
test('English declared source keeps the original exact dependency summary',async()=>{
 const result=await solve('Summarize: The parser reads the file. It builds a tree. The tree is checked.');assert.equal(result.intent,'summarization_free_text');assert.equal(result.content,'The parser reads the file.');
});
