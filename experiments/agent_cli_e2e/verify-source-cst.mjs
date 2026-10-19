import fs from 'node:fs';
import assert from 'node:assert/strict';
import path from 'node:path';
import {fileURLToPath,pathToFileURL} from 'node:url';
import {createHash} from 'node:crypto';
import {readSourceLinks,sourceFromLinks,sourceOwner,verifySourceLinks} from '../../js/meta/readable-source.js';
import {sourceCstFrontends} from '../../scripts/lib/source-cst-frontends.mjs';
import {TARGET_MODULE_PATH} from '../../js/agentic/self_ast.mjs';
const root=fileURLToPath(new URL('../../',import.meta.url));
const digest=value=>createHash('sha256').update(value).digest('hex');

/** Reconstruct persisted content before comparing it with independently selected source. */
export function verifySourceCst(document,expectedSource,options){
  const model=readSourceLinks(document);
  assert.equal(sourceOwner(model),TARGET_MODULE_PATH,'self-inspection source owner differs');
  const reconstructed=sourceFromLinks(model);
  assert.equal(reconstructed,expectedSource,'complete source content differs, including comments and whitespace');
  verifySourceLinks(model,options);
  return Object.freeze({schema:'formal-ai-self-source-cst-verification/v1',owner:TARGET_MODULE_PATH,
    sourceSha256:digest(reconstructed),modelSha256:digest(document),fidelity:'readable-recursive-cst',
    wholeSourceEqual:true,freshGrammar:true,publicationAuthority:false,autonomousAuthorshipCredit:0});
}
function regularFile(filename,limit){
  const stat=fs.lstatSync(filename);
  assert(stat.isFile()&&!stat.isSymbolicLink(),'regular source CST file required');
  assert(stat.size>0&&stat.size<=limit,'source CST verification size limit');
  return new TextDecoder('utf-8',{fatal:true,ignoreBOM:true}).decode(fs.readFileSync(filename));
}
if(process.argv[1]&&pathToFileURL(path.resolve(process.argv[1])).href===import.meta.url){
  assert.equal(process.argv.length,4,'usage: verify-source-cst.mjs model.lino receipt.json');
  const document=regularFile(process.argv[2],8*1024*1024);
  const expected=regularFile(path.join(root,'rust',TARGET_MODULE_PATH),4*1024*1024);
  // Rust-only verification uses the vendored grammar; no JavaScript parser install is needed.
  const runtime=await sourceCstFrontends(undefined,['Rust']);
  try{
    const options=runtime.forSource(TARGET_MODULE_PATH,'Rust');
    const receipt={...verifySourceCst(document,expected,options),grammar:options.provenance};
    fs.writeFileSync(process.argv[3],JSON.stringify(receipt,null,2)+'\n',{flag:'wx',mode:0o600});
    console.log(JSON.stringify(receipt));
  }finally{runtime.close();}
}
