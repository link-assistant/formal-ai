import test from 'node:test';
import assert from 'node:assert/strict';
import React from 'react';
import {renderToStaticMarkup} from 'react-dom/server';
import {createI18n} from 'lino-i18n';
import {fileURLToPath} from 'node:url';
import {createRequire} from 'node:module';
import {readFileSync} from 'node:fs';
import {ConversationSummaryDetails} from '../../../js/app/conversation-summary-details.js';
import {conversationSummaryRows} from '../../../js/conversation-summary-history.js';
import {WorkerHost} from '../../../js/server/worker-host.mjs';

test('UI renderer uses the actual root-declared locked React packages',()=>{
 const manifestUrl=new URL('../../../package.json',import.meta.url);
 const manifest=JSON.parse(readFileSync(manifestUrl,'utf8'));
 const locked=readFileSync(new URL('../../../bun.lock',import.meta.url),'utf8');
 const require=createRequire(manifestUrl);
 assert.equal(require('react'),React);
 assert.equal(require('react-dom/server').renderToStaticMarkup,renderToStaticMarkup);
 for(const name of ['react','react-dom','lino-i18n']) {
  const installedUrl=new URL('../../../node_modules/'+name+'/package.json',import.meta.url);
  const version=JSON.parse(readFileSync(installedUrl,'utf8')).version;
  assert.equal(version,manifest.dependencies[name],name);
  assert.ok(locked.includes('"'+name+'@'+version+'"'),name+' locked package');
 }
});

const i18n=createI18n({locale:'en'});await i18n.loadLocaleFile(fileURLToPath(new URL('../../../js/i18n-catalog-messages.lino',import.meta.url)));
const host=new WorkerHost();
const reports=[];
async function report(prompts,summaryPrompt) {
 const messages=[];
 for(const [index,prompt] of prompts.entries()) {
  const answer=await host.solve(prompt,messages);
  messages.push({id:'observed-user-'+index,role:'user',content:prompt},
   {id:'observed-assistant-'+index,role:'assistant',content:answer.content,intent:answer.intent,evidence:Array.from(answer.evidence??[])});
 }
 const answer=await host.solve(summaryPrompt,messages);
 const message={id:'observed-summary',role:'assistant',intent:answer.intent,content:answer.content,evidence:Array.from(answer.evidence??[])};
 messages.push({id:'observed-summary-request',role:'user',content:summaryPrompt},message);
 return {answer,message,messages};
}
function render(item,locale='en') {return renderToStaticMarkup(React.createElement(ConversationSummaryDetails,{message:item.message,messages:item.messages,t:key=>i18n.t(key,{}, {locale})}));}
for(const [language,prompts,summaryPrompt] of [['en',['Hi','What is 2 + 2?'],'Summarize this conversation'],['ru',['Привет'],'Резюме беседы'],['zh',['你好'],'总结']]) {
 test('unchanged '+language+' production summary gains genuine UI details with UI locale en',async()=>{
  const item=await report(prompts,summaryPrompt);const canonical=item.answer.content;
  assert.equal(item.answer.intent,'summarize_conversation');
  const html=render(item);assert.match(html,/Conversation summary/u);assert.match(html,/user/u);assert.match(html,/assistant/u);assert.match(html,/greeting/u);
  if(language==='en'){assert.match(html,/calculation/u);assert.match(html,/2 \+ 2 = 4/u);}
  if(language==='ru')assert.match(canonical,/^Резюме разговора:/u);
  if(language==='zh')assert.match(canonical,/^对话摘要/u);
  assert.equal(item.answer.content,canonical);reports.push({language,canonical,html});
 });
}
test('UI heading follows four actual checked catalog locales independently from solver language',()=>{
 const message={id:'summary',role:'assistant',intent:'summarize_conversation',content:'canonical body',evidence:[]};
 const item={message,messages:[{id:'u',role:'user',content:'actual text'},message]};
 for(const [locale,expected] of [['en','Conversation summary'],['ru','Резюме разговора'],['zh','对话摘要'],['hi','वार्तालाप सारांश']])assert.ok(render(item,locale).includes(expected),locale);
 assert.equal(message.content,'canonical body');
});
test('undeclared intent is reported absent even when prose resembles a calculation',()=>{
 const message={role:'assistant',intent:'summarize_conversation',evidence:[],content:'canonical'};
 const previous={role:'assistant',content:'2 + 2 = 4 and greeting',id:'not-declared'};
 const rows=conversationSummaryRows(message,[previous,message]);assert.equal(rows[0].intent,null);
 const html=render({message,messages:[previous,message]});assert.match(html,/Not recorded/u);assert.ok(!html.includes('<td>calculation</td>'));
});
test('actual declared arbitrary intents and exact source content are preserved',()=>{
 const message={role:'assistant',intent:'summarize_conversation',evidence:[]};
 const previous={id:'actual-source',role:'assistant',intent:'heldout_source_analysis',content:'  <script>real text</script>\n'};
 assert.deepEqual(conversationSummaryRows(message,[previous,message]),[{id:'actual-source',role:'assistant',intent:'heldout_source_analysis',content:'  <script>real text</script>\n'}]);
 const html=render({message,messages:[previous,message]});assert.ok(html.includes('heldout_source_analysis'));assert.ok(html.includes('&lt;script&gt;real text&lt;/script&gt;'));
});
test('later turns and system records cannot leak into an earlier summary view',()=>{
 const message={role:'assistant',intent:'summarize_conversation',evidence:[]};
 const early={id:'early',role:'user',content:'known prior message'};
 const messages=[early,{role:'system',content:'hidden system source'},message,{role:'user',content:'later unknown turn'}];
 assert.deepEqual(conversationSummaryRows(message,messages),[{id:'early',role:'user',intent:null,content:'known prior message'}]);
 const html=render({message,messages});assert.ok(!html.includes('later unknown turn'));assert.ok(!html.includes('hidden system source'));
});
test('plain recap, foreign message identity and malformed metadata cannot acquire rich-history view',()=>{
 const summary={role:'assistant',intent:'summarize_conversation',evidence:[],id:'same'};
 const prior={role:'user',content:'actual prior'};
 assert.equal(conversationSummaryRows({...summary},[prior,summary]),null);
 const plain={...summary,evidence:['summarization:format:plain']};assert.equal(render({message:plain,messages:[prior,plain]}),'');
 const malformed={...summary,evidence:{}};assert.equal(conversationSummaryRows(malformed,[prior,malformed]),null);
});
process.on('beforeExit',()=>{if(reports.length)console.log(JSON.stringify({actualWorkerReports:reports}));});
