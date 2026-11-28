// Run with SECRETLINT_TEST_BIN pointing to the pinned Secretlint 14 bin directory.
// Finite synthetic credentials exercise the actual engine without printing tokens.
import assert from 'node:assert/strict';
import {mkdtempSync,writeFileSync,rmSync} from 'node:fs';
import {tmpdir} from 'node:os';
import {join,delimiter} from 'node:path';
import {fileURLToPath} from 'node:url';
import {gzipSync} from 'node:zlib';
import {spawnSync} from 'node:child_process';
import {createScanner,checkFiles} from '../../scripts/check-secrets-files.mjs';

const bin=process.env.SECRETLINT_TEST_BIN;
assert.ok(bin,'SECRETLINT_TEST_BIN is required; install only secretlint@14.1.0');
const engine=await createScanner('14.1.0',bin);
const directory=mkdtempSync(join(tmpdir(),'secretlint14-live-'));
const github='gh'+'p_'+'A'.repeat(36);
const slack='xo'+'xb-'+'1'.repeat(12)+'-'+'2'.repeat(13)+'-'+'A'.repeat(24);
const cases=[['clean.txt','ordinary content',true],['github.txt',github,false],
  ['slack.txt',slack,false],['github.log.gz',github,false],['slack.log.gz',slack,false]];
try {
  for(const [name,content,ok] of cases) {
    const file=join(directory,name);
    writeFileSync(file,name.endsWith('.gz')?gzipSync(content):content);
    let output='';
    const result=await checkFiles([file],engine,value=>{output+=value;});
    assert.equal(result.ok,ok,name+' scan status');
    if(!ok)assert.ok(output.includes('***'),name+' finding must be masked');
    assert.equal(output.includes(github),false,'GitHub fixture leaked');
    assert.equal(output.includes(slack),false,'Slack fixture leaked');
    console.log(JSON.stringify({fixture:name,ok:result.ok,masked:!ok}));
  }
  const filename=join(directory,'::error:: forged annotation.txt');
  writeFileSync(filename,github);
  const cli=spawnSync(process.execPath,[fileURLToPath(new URL('../../scripts/check-secrets-files.mjs',import.meta.url))],{
    env:{...process.env,SECRETLINT_VERSION:'14.1.0',PATH:bin+delimiter+(process.env.PATH??'')},
    input:filename+'\0',encoding:'utf8',maxBuffer:1024*1024});
  assert.equal(cli.error,undefined);
  assert.equal(cli.status,1,'a real finding must fail the CLI');
  assert.ok(cli.stdout.includes('***'),'CLI finding must be masked');
  assert.ok(cli.stdout.includes('\"files\":1'),'CLI must scan the actual selected file');
  assert.equal(cli.stderr,'','CLI must not fail from malformed inventory or file errors');
  assert.equal(cli.stdout.includes(github),false,'CLI fixture leaked');
  const first=cli.stdout.split('\n')[0];
  assert.match(first,/^::stop-commands::secret-scan-[a-f0-9-]+$/u);
  const marker=first.slice('::stop-commands::'.length);
  assert.ok(cli.stdout.trimEnd().endsWith('::'+marker+'::'),'workflow commands must be restored');
  assert.ok(cli.stdout.indexOf('::error:: forged annotation.txt')>first.length,
    'untrusted filename must occur only while workflow commands are stopped');
  console.log(JSON.stringify({cliFindingFailed:true,masked:true,workflowCommandBoundary:true}));
} finally {rmSync(directory,{recursive:true,force:true});}
