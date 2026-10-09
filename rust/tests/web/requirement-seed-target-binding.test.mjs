import assert from 'node:assert/strict';
import { readFileSync, mkdirSync, mkdtempSync, writeFileSync, rmSync } from 'node:fs';
import path from 'node:path';
import { tmpdir } from 'node:os';
import { before, test } from 'node:test';
import { WorkerHost, REPO_ROOT } from '../../../js/server/worker-host.mjs';
import { installNodeHost, censusDocuments } from '../../../js/agentic/node-host.mjs';
import { stableId } from '../../../js/agentic/crate/engine_stable_identifier.mjs';
import { resolveRequirementTarget, resolveSeedTarget, resolveIn } from '../../../js/agentic/requirement_resolution.mjs';
before(async () => { await installNodeHost(new WorkerHost()); });
const read = file => readFileSync(REPO_ROOT+'/'+file,'utf8');
const census = {modules:[{path:'src/a.rs',symbols:[{kind:'const',name:'ALPHA_LINO'}]},{path:'src/b.rs',symbols:[{kind:'const',name:'BETA_LINO'}]}]};
const source = 'meanings\n  arbitrary\n    role request_signal\n    lexeme en "azure dawn"\n';
const expected = {module_path:'src/a.rs',symbol:'ALPHA_LINO',kind:'const'};

test('every unchanged ladder leaf resolves to its actual source or embedded seed declaration',()=>{
  const rows=read('experiments/issue_1028_agent_cli_ladder/leaves.tsv').trimEnd().split('\n').map(line=>line.split('\t'));
  assert.equal(rows.length,32);
  for(const [leaf,originalTask,path,,,requirement] of rows){
    // Native declarations retain the actual original file scope; seed surfaces retain their semantic requirement.
    const target=resolveRequirementTarget(path.startsWith('rust/') ? originalTask : requirement);
    assert.ok(target,`${leaf}: ${requirement}`);
    if(path.startsWith('data/seed/')){
      assert.equal(target.module_path,'src/seed/embedded_registry.rs',leaf);
      const registry=read('rust/'+target.module_path);
      const declaration=registry.slice(registry.indexOf('pub const '+target.symbol)).split('\n').slice(0,3).join('\n');
      assert.ok(declaration.includes('../../embedded/'+path),`${leaf}: actual generated include`);
    }else assert.equal(target.module_path,path.replace(/^rust\//u,''),leaf);
  }
});

test('arbitrary canonical surface and role evidence bind declared owners without guessing',()=>{
  assert.deepEqual(resolveSeedTarget(census,'Change signal surface from azure dawn to cobalt sun',[['data/seed/alpha.lino',source]]),expected);
  assert.equal(resolveSeedTarget(census,'Improve the code',[['data/seed/alpha.lino',source]]),null);
  assert.equal(resolveSeedTarget(census,'Change signal surface from azure dawnish',[['data/seed/alpha.lino',source]]),null);
  assert.equal(resolveSeedTarget(census,'Change unrelated surface from azure dawn',[['data/seed/alpha.lino',source]]),null);
  assert.equal(resolveSeedTarget(census,'Change signal surface from azure dawn',[['data/seed/missing.lino',source]]),null);
  assert.deepEqual(resolveIn(census,'Rename ALPHA_LINO after changing the signal'),expected);
});

test('ambiguous seed owners remain unresolved and stronger actual surfaces win',()=>{
  const sources=[['data/seed/alpha.lino',source],['data/seed/beta.lino',source]];
  assert.equal(resolveSeedTarget(census,'Change signal surface from azure dawn',sources),null);
  const longer=source.replace('azure dawn','azure dawn bright');
  assert.deepEqual(resolveSeedTarget(census,'Change signal surface from azure dawn bright',[sources[0],['data/seed/beta.lino',longer]]),{module_path:'src/b.rs',symbol:'BETA_LINO',kind:'const'});
  assert.deepEqual(resolveSeedTarget(census,'Change signal surface from azure dawn',[['invalid.txt',source],sources[0]]),expected);
});

test('real census document reads reject changed source identities',()=>{
  const directory=mkdtempSync(path.join(tmpdir(),'formal-ai-census-identity-'));
  try{
    mkdirSync(path.join(directory,'rust/src'),{recursive:true});
    mkdirSync(path.join(directory,'data/meta/self-ast/src'),{recursive:true});
    const source='pub const ARBITRARY: &str = "azure";\n';
    const contentId=stableId('source_module',source);
    const document='self_ast_census\n  target src/arbitrary.rs\n  content_id '+contentId+'\n  byte_len '+Buffer.byteLength(source)+'\n  symbols\n    const ARBITRARY 1 1\n';
    writeFileSync(path.join(directory,'rust/src/arbitrary.rs'),source);
    writeFileSync(path.join(directory,'data/meta/self-ast/src/arbitrary.lino'),document);
    assert.deepEqual(censusDocuments(directory),[{path:'data/meta/self-ast/src/arbitrary.lino',text:document,sourceIdentity:{path:'src/arbitrary.rs',content_id:contentId,byte_len:Buffer.byteLength(source)}}]);
    writeFileSync(path.join(directory,'rust/src/arbitrary.rs'),source.replace('azure','cobalt'));
    assert.deepEqual(censusDocuments(directory),[]);
  }finally{rmSync(directory,{recursive:true,force:true});}
});

test('the unchanged L07 requirement binds the actual canonical conversation seed owner',()=>{
  const row=read('experiments/issue_1028_agent_cli_ladder/leaves.tsv').split('\n').map(line=>line.split('\t')).find(fields=>fields[0]==='L07');
  const target=resolveRequirementTarget(row[5]);
  assert.deepEqual(target,{module_path:'src/seed/embedded_registry.rs',symbol:'MEANINGS_CONVERSATION_LINO',kind:'const'});
  const registry=read('rust/'+target.module_path);
  const declaration=registry.slice(registry.indexOf('pub const '+target.symbol)).split('\n').slice(0,3).join('\n');
  assert.ok(declaration.includes('../../embedded/'+row[2]));
});


test('a seed surface cannot supply its own independent role evidence',()=>{
  for (const [surface,role,negative,positive] of [
    ['signal','request_signal','Improve signal.','Change request surface from signal.'],
    ['code','script_or_code_artifact','Improve the code.','Change artifact surface from code.'],
    ['request signal','request_signal','Improve request signal.','Change signal surface from request signal.'],
    ['返回结果','request_signal','Improve 返回结果.','Change request surface from 返回结果.'],
  ]) {
    const seed=`meanings\n  arbitrary\n    role ${role}\n    lexeme en "${surface}"\n`;
    assert.equal(resolveSeedTarget(census,negative,[['data/seed/alpha.lino',seed]]),null,negative);
    assert.deepEqual(resolveSeedTarget(census,positive,[['data/seed/alpha.lino',seed]]),expected,positive);
  }
});
