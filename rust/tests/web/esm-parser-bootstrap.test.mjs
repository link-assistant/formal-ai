import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import os from 'node:os';
import {pathToFileURL,fileURLToPath} from 'node:url';
import {generateWorkerPrerequisites} from '../../../scripts/generate-worker-prerequisites.mjs';
const root=path.resolve(fileURLToPath(new URL('../../../',import.meta.url)));
function workspace(t){
 const directory=fs.realpathSync(fs.mkdtempSync(path.join(os.tmpdir(),'formal-ai-esm-parser-')));
 t.after(()=>fs.rmSync(directory,{recursive:true,force:true}));return directory;
}
test('portable compiler bootstrap awaits an ESM parser concurrently loading with top-level await',async t=>{
 const directory=workspace(t),folder='scripts/self-translation';
 fs.mkdirSync(path.join(directory,folder),{recursive:true});
 for(const name of ['portable-string.mjs','ir.mjs','lino.mjs','constructs.mjs'])
  fs.copyFileSync(path.join(root,folder,name),path.join(directory,folder,name));
 const provider=path.join(directory,'node_modules/@babel/parser');fs.mkdirSync(provider,{recursive:true});
 fs.writeFileSync(path.join(provider,'package.json'),'{"name":"@babel/parser","type":"module","exports":"./index.mjs"}\n');
 fs.writeFileSync(path.join(provider,'index.mjs'),
  'await new Promise(resolve=>setTimeout(resolve,20));export const ready=true;export function parse(){return {program:{body:[]}};}\n');
 const [compiler,parser]=await Promise.all([
  import(pathToFileURL(path.join(directory,folder,'portable-string.mjs')).href),
  import(pathToFileURL(path.join(provider,'index.mjs')).href)]);
 assert.equal(parser.ready,true);assert.equal(compiler.PROFILE,'portable-string-v1');
});
test('synchronous worker generation still resolves the parser owned by the selected target root',t=>{
 const directory=workspace(t),provider=path.join(directory,'node_modules/@babel/parser');
 fs.mkdirSync(provider,{recursive:true});fs.mkdirSync(path.join(directory,'js/worker'),{recursive:true});
 fs.writeFileSync(path.join(provider,'package.json'),'{"name":"@babel/parser","type":"commonjs","exports":"./index.cjs"}\n');
 fs.writeFileSync(path.join(provider,'index.cjs'),'module.exports={parse(){throw Error("SelectedRootParserWasUsed");}};\n');
 for(const file of ['parser-cache-primordials.js','seed_loader.js','worker/one.js'])fs.writeFileSync(path.join(directory,'js',file),'// fixture\n');
 assert.throws(()=>generateWorkerPrerequisites({root:directory}),/SelectedRootParserWasUsed/);
});
