// Finite real-scanner regression. Credentials are synthetic and never printed.
import assert from 'node:assert/strict';
import {spawnSync} from 'node:child_process';
import {mkdtempSync,writeFileSync,rmSync} from 'node:fs';
import {tmpdir} from 'node:os';
import {join} from 'node:path';
import {gzipSync} from 'node:zlib';
const directory=mkdtempSync(join(tmpdir(),'formal-ai-compressed-secrets-'));
const token='gh'+'p_'+'A'.repeat(36);
const cases=[['clean.txt','ordinary evidence\n',0],['credential.txt',token,1],
  ['credential.log.gz',gzipSync(token),1],['corrupt.log.gz',Buffer.from('not gzip'),1]];
let failures=0;
try {
  for(const [name,bytes,expected] of cases) {
    const file=join(directory,name);writeFileSync(file,bytes);
    const result=spawnSync('bash',['scripts/check-secrets.sh',file],{
      encoding:'utf8',timeout:30000,maxBuffer:1024*1024,
      env:{...process.env,NODE_OPTIONS:'--max-old-space-size=256'},
    });
    const actual=result.status===0?0:1;
    console.log(JSON.stringify({name,expected,actual,status:result.status,signal:result.signal,
      scannerExecuted:result.stdout.includes('Scanning 1 file(s)'),
      masked:!((result.stdout??'')+(result.stderr??'')).includes(token)}));
    try {assert.equal(result.signal,null);assert.equal(actual,expected,name);assert(result.stdout.includes('Scanning 1 file(s)'));assert(!((result.stdout??'')+(result.stderr??'')).includes(token));}
    catch {failures++;}
  }
} finally {rmSync(directory,{recursive:true,force:true});}
process.exitCode=failures?1:0;
