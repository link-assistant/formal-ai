// Supplied regression draft, executed by the Formal AI driver.
import fs from 'node:fs';
const fixtures = [
  ['en','How do I enable internet access?',true],
  ['en','Allow internet access',true],
  ['en','What should I do to give you access to the internet?',true],
  ['ru','Как включить доступ к интернету?',true],
  ['ru','Что мне сделать, чтобы дать тебе доступ в интернет?',true],
  ['ru','Как мне разрешить доступ к интернету?',true],
  ['hi','इंटरनेट की अनुमति कैसे दें?',true],
  ['hi','मैं आपको इंटरनेट का उपयोग करने की अनुमति कैसे दूँ?',true],
  ['zh','如何启用互联网访问？',true],
  ['zh','我怎样才能允许你访问互联网？',true],
  ['es','¿Cómo habilitar el acceso a internet?',true],
  ['es','¿Cómo puedo darte acceso a internet?',true],
  ['en','Translate "enable internet access" into Russian.',false],
  ['en','Write a program that displays "enable internet access".',false],
  ['ru','Переведи фразу «как включить интернет» на английский.',false],
  ['hi','"इंटरनेट की अनुमति दें" का अनुवाद करें।',false],
  ['zh','请翻译“如何启用互联网访问”。',false],
  ['es','Traduce "permitir acceso a internet" al inglés.',false],
].map(([language,prompt,guidance])=>({language,prompt,guidance}));
fs.writeFileSync('data/parity/issue-1582-permissions.json',JSON.stringify(fixtures,null,2)+'\n');
fs.writeFileSync('rust/tests/web/issue-1582-permissions.test.mjs',`// Shared language regressions for issue #1582; supplied, zero authorship credit.
import assert from 'node:assert/strict';
import {readFileSync} from 'node:fs';
import {test} from 'node:test';
import {WorkerHost} from '../../../js/server/worker-host.mjs';
const fixtures=JSON.parse(readFileSync(new URL('../../../data/parity/issue-1582-permissions.json',import.meta.url),'utf8'));
const host=new WorkerHost({WebAssembly:{instantiate(){throw Error('LocalWasmExecutionForbidden');},compile(){throw Error('LocalWasmExecutionForbidden');}}});
for(const row of fixtures)test(row.language+': '+row.prompt,{timeout:15000},async()=>{
  if(row.guidance){
    const result=await host.solve(row.prompt);
    assert.equal(result.intent,'capabilities');
    assert(result.evidence.includes('language:'+row.language),'detect the language without forcing it');
    assert(result.evidence.includes('permission:guidance'));
    assert.match(result.content,/FORMAL_AI_OFFLINE/u);
    assert.match(result.content,/\\/permissions/u);
  }else{
    await host.boot();
    const answer=await host.run('tryFeatureCapabilityStatus(__prompt, normalizePrompt(__prompt), __language, {})',{__prompt:row.prompt,__language:row.language});
    assert.equal(answer,null,'a quoted permission phrase does not own the request');
  }
});
`);
console.log('Drafted 18 shared language fixtures and their executable regression suite');
