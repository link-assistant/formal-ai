import fs from 'node:fs';
import path from 'node:path';
import {fileURLToPath} from 'node:url';
import {subprocessBoundaryAuditor} from './lib/subprocess-boundary-audit.mjs';
const root=process.argv[2] || path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const auditor=await subprocessBoundaryAuditor(root);
let failures=0, unresolved=0, files=0;
function scan(directory,language){
  for(const entry of fs.readdirSync(directory,{withFileTypes:true})){
    if(entry.isSymbolicLink())continue;
    const filename=path.join(directory,entry.name);
    if(entry.isDirectory()){if(!['node_modules','vendor','worker','generated'].includes(entry.name))scan(filename,language);continue;}
    if(!entry.isFile() || !(language==='Rust'?filename.endsWith('.rs'):/\.(?:mjs|js)$/u.test(filename)))continue;
    const source=fs.readFileSync(filename,'utf8');
    const result=language==='Rust'?auditor.rust(source):auditor.javascript(source,filename);
    files++;unresolved+=result.unresolved.length;
    for(const failure of result.violations){failures++;console.error(JSON.stringify({file:path.relative(root,filename),...failure}));}
  }
}
try{scan(path.join(root,'rust/src'),'Rust');scan(path.join(root,'js/server'),'JavaScript');scan(path.join(root,'js/agentic'),'JavaScript');console.log(JSON.stringify({files,violations:failures,unresolved,coverage:'direct-local-process-provenance and opaque-macro-token-call-shapes; no macro-expansion/interprocedural/emitted-program/async-spawn proof'}));process.exitCode=failures?1:0;}finally{auditor.close();}
