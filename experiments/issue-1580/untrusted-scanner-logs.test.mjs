import test from 'node:test';
import assert from 'node:assert/strict';
import {mkdtempSync,mkdirSync,writeFileSync,rmSync} from 'node:fs';
import {tmpdir} from 'node:os';
import {join,resolve} from 'node:path';
import {spawnSync} from 'node:child_process';

// The runner's V2 parser requires a leading marker after trimming spaces; its
// legacy parser finds ##[ anywhere in a line. See actions/runner ActionCommand.cs.
function commands(output) {
  return output.split(/\r?\n/u).flatMap(line=> {
    const modern=/^::([a-z-]+)(?: [^\r\n]*?)?::/u.exec(line.trimStart());
    const legacy=/##\[([a-z-]+)(?: [^\]]*)?\]/u.exec(line);
    return modern?[modern[1]]:legacy?[legacy[1]]:[];
  });
}

test('real size scanner preserves intended warnings without executing contributor filenames', {timeout:10000},()=> {
  for(const [file,count,expected] of [
    ['legacy-##[error]forged.rs',901,['warning']],
    ['row\n::add-mask::forged.rs',1001,[]],
    ['ordinary.rs',901,['warning']]
  ]) {
    const directory=mkdtempSync(join(tmpdir(),'formal-ai-log-data-'));
    try {
      writeFileSync(join(directory,file),'x\n'.repeat(count));
      const result=spawnSync(process.execPath,['--max-old-space-size=128',resolve('scripts/check-file-size.mjs')],
        {cwd:directory,encoding:'utf8',env:{...process.env,GITHUB_ACTIONS:'true'}});
      assert.equal(result.status,count>1000?1:0,result.stderr);
      assert.deepEqual(commands(result.stdout),expected,file);
      assert.equal(result.stderr,'');
      if(count<1000)assert(result.stdout.includes('::warning file='),'retain intentional machine annotation');
    }finally{rmSync(directory,{recursive:true,force:true});}
  }
});

test('diagnostic encoding round trips exact contributor data and leaves ordinary text unchanged',async()=> {
  const {safeLogText}=await import('../../scripts/lib/github-actions-log.mjs');
  for(const value of ['normal/path.rs','ordinary Unicode é 🙂','count: 3','a#b'])assert.equal(safeLogText(value),value);
  for(const value of ['::error::fake','prefix##[add-mask]fake','a\n::warning::fake','quote"\\##[error]é🙂',
    Array.from({length:32},(_,i)=>String.fromCharCode(i)).join(''),'\u007f\u0080\u009f']) {
    const actual=safeLogText(value);assert.equal(JSON.parse(actual),value);
    assert.deepEqual(commands(actual),[]);assert.doesNotMatch(actual,/[\r\n]/u);
  }
});

test('human-readable fixture commands are rejected independently of indentation and quoting',async()=> {
  const {safeLogText}=await import('../../scripts/lib/github-actions-log.mjs');
  for(const payload of ['##[error]x','##[add-mask]secret','::error::x','::stop-commands::attacker']) {
    assert.equal(commands(payload).length,1);
    assert.deepEqual(commands('  '+safeLogText(payload)),[]);
    assert.deepEqual(commands('WARNING: '+safeLogText(payload)),[]);
  }
});


test('hardcoded-language scanner treats contributor paths and prose as data', {timeout:10000},async()=> {
  const {renderAllowlist}=await import('../../scripts/check-hardcoded-language.mjs');
  for(const [file,text] of [
    ['legacy-##[add-mask]file.rs','Please wait'],
    ['ordinary.rs','Please wait. ##[warning]forged'],
    ['row\n::add-mask::file.rs','Please wait'],
    ['ordinary.rs','Please wait']
  ]) {
    const directory=mkdtempSync(join(tmpdir(),'formal-ai-prose-log-'));
    try {
      mkdirSync(join(directory,'rust/src'),{recursive:true});
      mkdirSync(join(directory,'scripts'));
      writeFileSync(join(directory,'rust/src',file),'fn reply() -> &\'static str { return "'+text+'"; }\n');
      writeFileSync(join(directory,'scripts/hardcoded-language-allowlist.txt'),renderAllowlist([]));
      const result=spawnSync(process.execPath,['--max-old-space-size=128',resolve('scripts/check-hardcoded-language.mjs')],
        {cwd:directory,encoding:'utf8',env:{...process.env,GITHUB_ACTIONS:'true'}});
      assert.equal(result.status,1,result.stderr);
      assert.deepEqual(commands(result.stdout),['error'],JSON.stringify({file,text,output:result.stdout}));
      assert.equal(result.stderr,'');
      assert(result.stdout.includes('::error file='),'retain intentional machine annotation');
    }finally{rmSync(directory,{recursive:true,force:true});}
  }
});
