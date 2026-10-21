import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import {tmpdir} from 'node:os';
import {spawnSync} from 'node:child_process';
import {fileURLToPath} from 'node:url';
const root=fileURLToPath(new URL('../../../',import.meta.url));
const checker='rust/tests/e2e/scripts/check-language-test-coverage.mjs';
const behavioralTest=fs.readFileSync(path.join(root,'rust/tests/web/source-authoring-document-root.test.mjs'),'utf8');
function runFixture(destination,source){
 const directory=fs.mkdtempSync(path.join(tmpdir(),'formal-ai-language-coverage-'));
 const write=(name,bytes)=>{const target=path.join(directory,name);fs.mkdirSync(path.dirname(target),{recursive:true});fs.writeFileSync(target,bytes);};
 const git=arguments_=>{
  const result=spawnSync('git',arguments_,{cwd:directory,encoding:'utf8',timeout:5000,maxBuffer:65536});
  assert.equal(result.status,0,result.stderr);return result.stdout.trim();
 };
 try{
  for(const name of [checker,'rust/tests/e2e/scripts/lino-seed-parser.mjs','data/seed/languages.lino']){
   write(name,fs.readFileSync(path.join(root,name)));
  }
  write('data/seed/source-authoring-grammar.lino','source-authoring-grammar\n');
  git(['init','-q']);git(['add','.']);
  git(['-c','user.name=Coverage Fixture','-c','user.email=coverage@example.invalid','commit','-qm','baseline']);
  const base=git(['rev-parse','HEAD']);
  write('data/seed/source-authoring-grammar.lino','source-authoring-grammar\n  kind float-division\n');
  write(destination,source);git(['add','.']);
  return spawnSync(process.execPath,[path.join(directory,checker)],{
   cwd:directory,encoding:'utf8',timeout:10000,maxBuffer:262144,
   env:{...process.env,LANGUAGE_TEST_COVERAGE_BASE_REF:base},
  });
 }finally{fs.rmSync(directory,{recursive:true,force:true});}
}
test('registered flat web suite contributes actual five-language behavioral cases',()=>{
 const result=runFixture('rust/tests/web/owned.test.mjs',behavioralTest);
 assert.equal(result.status,0,result.stdout+result.stderr);
 assert.match(result.stdout,/en, ru, hi, zh, es/u);
});
test('registered web suite missing Spanish still refuses',()=>{
 const source=behavioralTest.replace(/  \{language:'es',[\s\S]*?registration:[^\n]+\n/u,'');
 assert.notEqual(source,behavioralTest);
 const result=runFixture('rust/tests/web/owned.test.mjs',source);
 assert.equal(result.status,1,result.stdout+result.stderr);
 assert.match(result.stderr,/Missing: es/u);
});
for(const destination of ['rust/tests/web/coverage-helper.mjs','rust/tests/web/coverage.test.mjs.md',
 'rust/tests/web/nested/owned.test.mjs'])test('unregistered helper or document cannot grant coverage: '+destination,()=>{
 const result=runFixture(destination,behavioralTest);
 assert.equal(result.status,1,result.stdout+result.stderr);
 assert.match(result.stderr,/No changed test files/u);
});
