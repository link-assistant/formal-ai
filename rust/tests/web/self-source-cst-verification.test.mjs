import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import {fileURLToPath} from 'node:url';
import {spawnSync} from 'node:child_process';
import {createHash} from 'node:crypto';
import {sourceCstFrontends} from '../../../scripts/lib/source-cst-frontends.mjs';
import {sourceToLinks,ownedSourceLinks,writeSourceLinks} from '../../../js/meta/readable-source.js';
import {TARGET_MODULE_PATH,AST_PATH} from '../../../js/agentic/self_ast.mjs';
import {verifySourceCst} from '../../../experiments/agent_cli_e2e/verify-source-cst.mjs';

test('source CST verifier preserves all text and requires owner and fresh concrete grammar without Babel',async()=>{
  const runtime=await sourceCstFrontends('/nonexistent/no-babel-dependencies',['Rust']);
  try{
    const options=runtime.forSource(TARGET_MODULE_PATH,'Rust');
    const source='// Unicode 计划 🙂\r\npub fn value() -> &\'static str { "exact" }\r\n\r\n';
    const content=sourceToLinks(source,'Rust',options);
    const owned=ownedSourceLinks(TARGET_MODULE_PATH,content),document=writeSourceLinks(owned);
    const receipt=verifySourceCst(document,source,options);
    assert.equal(receipt.wholeSourceEqual,true);assert.equal(receipt.freshGrammar,true);
    assert.equal(receipt.owner,TARGET_MODULE_PATH);assert.equal(receipt.publicationAuthority,false);
    assert.equal(AST_PATH,'self-cst.lino');
    assert.throws(()=>runtime.forSource('foreign.mjs','JavaScript'),/UnsupportedSourceCstLanguage/);
    assert.throws(()=>verifySourceCst(document,source.replace('Unicode','changed'),options));
    assert.throws(()=>verifySourceCst(document,source.replaceAll('\r\n','\n'),options));
    const foreign=writeSourceLinks(ownedSourceLinks('src/foreign.rs',content));
    assert.throws(()=>verifySourceCst(foreign,source,options),/source owner differs/);
    const altered=document.replace('function_item','struct_item');assert.notEqual(altered,document);
    assert.throws(()=>verifySourceCst(altered,source,options),/SourceStructureMismatch/);
    for(const diagnostic of ['source_census_diagnostic\n  named_node_count 42\n',
      '(source (language "Rust") (named-node-count "42"))']){
      assert.throws(()=>verifySourceCst(diagnostic,source,options));
    }
  }finally{runtime.close();}
});
test('actual source CST CLI independently verifies current planner and refuses symlink and foreign ownership',async()=>{
  const directory=fs.mkdtempSync(path.join(os.tmpdir(),'self-source-cst-verifier-'));
  const runtime=await sourceCstFrontends('/nonexistent/no-babel-dependencies',['Rust']);
  try{
    const options=runtime.forSource(TARGET_MODULE_PATH,'Rust');
    const source=fs.readFileSync(new URL('../../../rust/'+TARGET_MODULE_PATH,import.meta.url),'utf8');
    const content=sourceToLinks(source,'Rust',options);
    const model=path.join(directory,'source.lino'),receipt=path.join(directory,'receipt.json');
    fs.writeFileSync(model,'\uFEFF'+writeSourceLinks(ownedSourceLinks(TARGET_MODULE_PATH,content)));
    const verifier=fileURLToPath(new URL('../../../experiments/agent_cli_e2e/verify-source-cst.mjs',import.meta.url));
    const run=filename=>spawnSync(process.execPath,[verifier,filename,receipt],
      {encoding:'utf8',timeout:30000,maxBuffer:32768});
    const result=run(model);assert.equal(result.status,0,result.stdout+result.stderr);
    const observed=JSON.parse(fs.readFileSync(receipt));
    assert.equal(observed.freshGrammar,true);assert.equal(observed.wholeSourceEqual,true);
    assert.equal(observed.modelSha256,createHash('sha256').update(fs.readFileSync(model)).digest('hex'));
    fs.unlinkSync(receipt);const alias=path.join(directory,'alias.lino');fs.symlinkSync(model,alias);
    assert.notEqual(run(alias).status,0);assert.equal(fs.existsSync(receipt),false);
    fs.writeFileSync(model,writeSourceLinks(ownedSourceLinks('src/foreign.rs',content)));
    assert.notEqual(run(model).status,0);assert.equal(fs.existsSync(receipt),false);
  }finally{runtime.close();fs.rmSync(directory,{recursive:true,force:true});}
});
