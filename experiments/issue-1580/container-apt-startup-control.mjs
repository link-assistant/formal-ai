// Reproduce host-shell contamination without touching the host or real APT.
import assert from 'node:assert/strict';
import {readFileSync,writeFileSync,mkdtempSync,rmSync} from 'node:fs';
import {join} from 'node:path';
import {spawnSync} from 'node:child_process';
const source=readFileSync('rust/tests/web/container-apt-retry.test.mjs','utf8');
const guarded="['--noprofile','--norc','-c',command]";
assert.equal(source.split(guarded).length,2);
const directory=mkdtempSync('experiments/issue-1580/container-startup-control-');
try{
 const file=join(directory,'unisolated.test.mjs');
 writeFileSync(file,source.replace(guarded,"['-c',command]"));
 const result=spawnSync(process.execPath,['--test','--test-name-pattern=actual Docker apt',file],{encoding:'utf8'});
 process.stdout.write(JSON.stringify({status:result.status,signal:result.signal,stdout:result.stdout,stderr:result.stderr},null,2)+'\n');
 assert.equal(result.signal,null);assert.equal(result.status,1);
 assert.match(result.stdout+result.stderr,/host startup must not run/);
}finally{rmSync(directory,{recursive:true,force:true});}
