#!/usr/bin/env node
// Hosted scanner qualification. Inventories expose debt; they never certify strict zero.
import fs from 'node:fs';
import path from 'node:path';
import {spawnSync,execFileSync} from 'node:child_process';
import {createHash} from 'node:crypto';
import {gzipSync} from 'node:zlib';
const phase=process.argv[2];
if(process.argv.length!==3||!['js','native'].includes(phase))throw Error('phase must be exactly js or native');
const out=process.env.SCANNER_REPORT_DIRECTORY;
if(!out)throw Error('SCANNER_REPORT_DIRECTORY is required');
fs.mkdirSync(out,{recursive:true});
const sha=bytes=>createHash('sha256').update(bytes).digest('hex');
const git=args=>execFileSync('git',args,{encoding:'utf8'}).trim();
const head=git(['rev-parse','HEAD']),tree=git(['rev-parse','HEAD^{tree}']);
const listed=execFileSync('git',['ls-files','-z'],{encoding:'utf8'}).split('\0').filter(Boolean);
const sources=listed.filter(file=>/^(js\/|ts\/|rust\/src\/)/u.test(file)||[
 'scripts/check-source-duplication.mjs','scripts/check-source-duplication.rs',
 'scripts/lib/source-duplication.mjs','rust/tests/web/source-duplication.test.mjs',
 'scripts/qualify-source-scanners.mjs','package.json','bun.lock',
 '.github/workflows/source-scanner-qualification.yml',
].includes(file)).map(file=>({path:file,sha256:sha(fs.readFileSync(file))}));
const record={schema:'source-scanner-hosted-qualification/v1',phase,head,tree,
 run:process.env.GITHUB_RUN_ID,attempt:process.env.GITHUB_RUN_ATTEMPT,
 strictZeroAcceptance:'Pending',inventoryIsAcceptance:false,sources,operations:[]};
let failed=false;
function run(name,command,args,inventory=false){
 const result=spawnSync(command,args,{encoding:'utf8',timeout:300000,maxBuffer:128*1024*1024});
 const stdout=Buffer.from(result.stdout??''),stderr=Buffer.from(result.stderr??'');
 const operation={name,command,args,status:result.status,signal:result.signal,
 error:result.error?.message??null,stdoutBytes:stdout.length,stdoutSha256:sha(stdout),
 stderrBytes:stderr.length,stderrSha256:sha(stderr),complete:!result.error&&result.status===0};
 fs.writeFileSync(path.join(out,name+'.stdout.gz'),gzipSync(stdout));
 fs.writeFileSync(path.join(out,name+'.stderr.gz'),gzipSync(stderr));
 if(inventory&&operation.complete){
  try{
   const report=JSON.parse(stdout.toString());
   operation.accepted=name==='javascript-inventory'?report.accepted:
    report.files>0&&report.clones.length===0&&report.gaps.length===0;
   operation.coverageGaps=report.gaps?.length??null;
   operation.cloneGroups=report.independentCloneGroups??report.clones?.length;
   operation.files=report.files;
   // Preserve native raw fields and make strict acceptance explicit without altering evidence.
   if(name==='native-inventory')fs.writeFileSync(path.join(out,'native-assessment.json'),
    JSON.stringify({accepted:operation.accepted,inventoryMode:true,gaps:report.gaps,
     cloneGroups:report.clones.length,strictZeroAcceptance:'Pending'})+'\n');
  }catch(error){operation.complete=false;operation.error=error.message;}
 }
 if(!operation.complete)failed=true;
 record.operations.push(operation);
 console.log(JSON.stringify(operation));
}
record.runtime={node:process.version};
if(Number(process.versions.node.split('.')[0])!==24)throw Error('qualification requires Node24');
if(phase==='js'){
 const parser=JSON.parse(fs.readFileSync('node_modules/@babel/parser/package.json'));
 const traverse=JSON.parse(fs.readFileSync('node_modules/@babel/traverse/package.json'));
 record.runtime.babelParser=parser.version;record.runtime.babelTraverse=traverse.version;
 if(parser.version!=='8.0.7'||traverse.version!=='8.0.7')throw Error('locked Babel8.0.7 required');
 run('javascript-controls',process.execPath,['--test','rust/tests/web/source-duplication.test.mjs']);
 run('javascript-inventory',process.execPath,['scripts/check-source-duplication.mjs','--inventory'],true);
}else{
 run('native-controls','rust-script',['--test','scripts/check-source-duplication.rs']);
 run('native-inventory','rust-script',['scripts/check-source-duplication.rs','--inventory'],true);
}
const expected=phase==='js'?'javascript-inventory':'native-inventory';
const inventories=record.operations.filter(op=>op.name.endsWith('-inventory'));
record.qualificationComplete=!failed&&record.operations.length===2
 &&inventories.length===1&&inventories[0].name===expected;
record.strictZeroAccepted=record.qualificationComplete&&inventories[0].complete
 &&inventories[0].accepted===true;
if(!record.qualificationComplete)failed=true;
fs.writeFileSync(path.join(out,'qualification.json'),JSON.stringify(record,null,2)+'\n');
console.log('Strict zero duplication acceptance remains '+(record.strictZeroAccepted?'observed clean; review required':'Pending'));
if(failed)process.exitCode=1;
