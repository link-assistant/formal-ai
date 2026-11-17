// Unmodified upstream scripts, controlled subprocess failures, finite child budgets.
import test from 'node:test';
import assert from 'node:assert/strict';
import {readFileSync,writeFileSync,mkdtempSync,mkdirSync,cpSync,rmSync} from 'node:fs';
import {tmpdir} from 'node:os';
import {join} from 'node:path';
import {fileURLToPath} from 'node:url';
import {createHash} from 'node:crypto';
import {spawnSync} from 'node:child_process';
import {templatePythonAvailable} from './template-python-runtime.mjs';

const cases=[
  {language:'java',command:process.execPath,args:['--max-old-space-size=64','scripts/detect-code-changes.mjs'],code:'src/Main.java',initial:false},
  {language:'python',command:'python3',args:['-B','scripts/detect_code_changes.py'],code:'src/main.py',initial:false},
  {language:'php',command:'php',args:['-d','memory_limit=64M','scripts/detect-code-changes.php'],code:'src/Main.php',initial:true},
];

for(const item of cases)test(`${item.language}: failed Git is reported as a successful no-code observation`,{timeout:15000},context=>{
  if(item.language==='python'){
    if(!templatePythonAvailable(context))return;
  }else{
    const runtime=spawnSync(item.command,['--version'],{encoding:'utf8',timeout:5000,maxBuffer:4096});
    if(runtime.error?.code==='ENOENT'){context.skip(item.command+' is not installed');return;}
    assert.equal(runtime.status,0,runtime.stderr);
  }
  const fixture=fileURLToPath(new URL(`./fixtures/${item.language}-template/`,import.meta.url));
  const provenance=JSON.parse(readFileSync(join(fixture,'source.json'),'utf8'));
  for(const entry of provenance.files){
    const bytes=readFileSync(join(fixture,entry.path));
    assert.equal(createHash('sha1').update(`blob ${bytes.length}\0`).update(bytes).digest('hex'),entry.gitBlob);
    assert.equal(createHash('sha256').update(bytes).digest('hex'),entry.sha256);
  }
  const directory=mkdtempSync(join(tmpdir(),`formal-ai-${item.language}-git-`));
  try{
    cpSync(join(fixture,'scripts'),join(directory,'scripts'),{recursive:true});
    const bin=join(directory,'bin');mkdirSync(bin);
    writeFileSync(join(bin,'git'),`#!${process.execPath}\n`+String.raw`
const command=process.argv[2],mode=process.env.FAKE_GIT_MODE;
if(command==='cat-file'||command==='fetch')process.exit(0);
if(command==='diff'){
  if(mode==='code'){process.stdout.write(process.env.FAKE_SOURCE+'\n');process.exit(0);}
  if(mode==='docs'||(mode==='unknown-pr'&&process.argv.includes('HEAD^'))){process.stdout.write('docs/guide.md\n');process.exit(0);}
  if(mode==='empty')process.exit(0);
}
if(command==='ls-tree'&&mode==='initial'){process.stdout.write(process.env.FAKE_SOURCE+'\n');process.exit(0);}
process.stderr.write('fatal: unavailable comparison/repository\n');process.exit(128);
`,{mode:0o755});
    const outputFile=join(directory,'outputs');
    const invoke=(mode,event='push')=>{
      writeFileSync(outputFile,'');
      const result=spawnSync(item.command,item.args,{
      cwd:directory,encoding:'utf8',timeout:5000,maxBuffer:65536,
      env:{...process.env,PATH:bin+':'+process.env.PATH,FAKE_GIT_MODE:mode,FAKE_SOURCE:item.code,
        GITHUB_EVENT_NAME:event,GITHUB_BASE_SHA:'a'.repeat(40),GITHUB_HEAD_SHA:'b'.repeat(40),GITHUB_OUTPUT:outputFile,GITHUB_ACTIONS:'false'},
      });
      return {...result,outputs:readFileSync(outputFile,'utf8')};
    };
    for(const event of ['push','pull_request']){
      const result=invoke('unavailable',event);
      assert.equal(result.status,0,result.stderr);
      assert.match(result.outputs,/^any-code-changed=false$/mu);
    }
    for(const mode of ['code','docs','empty','initial','unknown-pr']){
      const result=invoke(mode,mode==='unknown-pr'?'pull_request':'push');
      assert.equal(result.status,0,result.stderr);
      const expected=mode==='code'||mode==='initial'&&item.initial;
      assert.match(result.outputs,new RegExp('^any-code-changed='+expected+'$','mu'),mode);
    }
  }finally{rmSync(directory,{recursive:true,force:true});}
});
