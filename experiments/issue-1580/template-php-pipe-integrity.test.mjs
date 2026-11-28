// Finite observations of the actual pinned PHP helper and a disposable repair.
import assert from 'node:assert/strict';
import {spawn} from 'node:child_process';
import {createHash} from 'node:crypto';
import {mkdtempSync,readFileSync,rmSync,writeFileSync} from 'node:fs';
import {tmpdir} from 'node:os';
import {join} from 'node:path';
import {fileURLToPath} from 'node:url';
import test from 'node:test';

const fixture=fileURLToPath(new URL('./fixtures/php-template/',import.meta.url));
const source=readFileSync(join(fixture,'scripts/src/Process.php'),'utf8');
const metadata=JSON.parse(readFileSync(join(fixture,'source.json'),'utf8'));
const identity=metadata.files.find(item=>item.path==='scripts/src/Process.php');
assert.equal(createHash('sha256').update(source).digest('hex'),identity.sha256);
assert.equal(createHash('sha1').update(`blob ${Buffer.byteLength(source)}\0`).update(source).digest('hex'),identity.gitBlob);
const sequential=String.raw`        $stdout = stream_get_contents($pipes[1]) ?: '';
        $stderr = stream_get_contents($pipes[2]) ?: '';
        fclose($pipes[1]);
        fclose($pipes[2]);`;
assert.ok(source.includes(sequential));
const candidate=source.replace(sequential,String.raw`        $stdout = '';
        $stderr = '';
        stream_set_blocking($pipes[1], false);
        stream_set_blocking($pipes[2], false);
        $readers = [1 => $pipes[1], 2 => $pipes[2]];
        while ($readers !== []) {
            $ready = array_values($readers);
            $write = null;
            $except = null;
            if (stream_select($ready, $write, $except, null) === false) {
                throw new \RuntimeException('Failed to select command pipes.');
            }
            foreach ($ready as $stream) {
                $chunk = fread($stream, 8192);
                if ($chunk === false) {
                    throw new \RuntimeException('Failed to read command pipe.');
                }
                $index = array_search($stream, $readers, true);
                if ($index === 1) {
                    $stdout .= $chunk;
                } else {
                    $stderr .= $chunk;
                }
                if (feof($stream)) {
                    fclose($stream);
                    unset($readers[$index]);
                }
            }
        }`);

async function observe(t,{repaired=false,kind='large',exitCode=0}){
  const cwd=mkdtempSync(join(tmpdir(),'issue-1580-php-pipes-'));
  t.after(()=>rmSync(cwd,{recursive:true,force:true}));
  writeFileSync(join(cwd,'Process.php'),repaired?candidate:source);
  writeFileSync(join(cwd,'ProcessResult.php'),readFileSync(join(fixture,'scripts/src/ProcessResult.php')));
  const child=kind==='zero'?'echo "0";fwrite(STDERR,"0");exit((int)$argv[1]);':
    `fwrite(STDERR,str_repeat("E",${kind==='large'?131072:1024}));`+
    `fwrite(STDOUT,str_repeat("O",${kind==='large'?131072:1024}));exit((int)$argv[1]);`;
  const expression=String.raw`require $argv[1];require $argv[2];
    $r=\LinkFoundation\Template\Pipeline\Process::run([PHP_BINARY,'-d','memory_limit=32M','-r',$argv[3],$argv[4]]);
    echo json_encode([$r->exitCode,$r->stdout,$r->stderr],JSON_THROW_ON_ERROR);`;
  return await new Promise((resolve,reject)=>{
    const process=spawn('php',['-d','memory_limit=32M','-r',expression,
      join(cwd,'ProcessResult.php'),join(cwd,'Process.php'),child,String(exitCode)],
    {detached:true,stdio:['ignore','pipe','pipe']});
    const chunks=[];const errors=[];let bytes=0;let deadline=false;let overflow=false;
    const kill=()=>{try{globalThis.process.kill(-process.pid,'SIGKILL');}catch(error){if(error.code!=='ESRCH')reject(error);}};
    const timer=setTimeout(()=>{deadline=true;kill();},2000);
    process.stdout.on('data',chunk=>{bytes+=chunk.length;chunks.push(chunk);if(bytes>524288){overflow=true;kill();}});
    process.stderr.on('data',chunk=>{bytes+=chunk.length;errors.push(chunk);if(bytes>524288){overflow=true;kill();}});
    process.once('error',error=>{clearTimeout(timer);reject(error);});
    process.once('close',(status,signal)=>{clearTimeout(timer);resolve({status,signal,deadline,overflow,
      stdout:Buffer.concat(chunks).toString(),stderr:Buffer.concat(errors).toString()});});
  });
}

test('PHP Process stalls on finite stderr-first output; concurrent readers complete',{timeout:10000},async t=>{
  const original=await observe(t,{});
  assert.equal(original.deadline,true);
  assert.equal(original.signal,'SIGKILL');
  assert.equal(original.overflow,false);
  const repaired=await observe(t,{repaired:true});
  assert.equal(repaired.deadline,false);
  assert.equal(repaired.status,0,repaired.stderr);
  assert.deepEqual(JSON.parse(repaired.stdout),[0,'O'.repeat(131072),'E'.repeat(131072)]);
});

test('PHP Process loses literal zero output; the candidate retains both streams',{timeout:10000},async t=>{
  for(const repaired of [false,true]){
    const result=await observe(t,{repaired,kind:'zero',exitCode:7});
    assert.equal(result.deadline,false);
    assert.equal(result.status,0,result.stderr);
    assert.deepEqual(JSON.parse(result.stdout),repaired?[7,'0','0']:[7,'','']);
  }
});

test('PHP Process candidate preserves small outputs and failing exit codes',{timeout:15000},async t=>{
  for(const repaired of [false,true])for(const exitCode of [0,7]){
    const result=await observe(t,{repaired,kind:'small',exitCode});
    assert.equal(result.deadline,false);
    assert.equal(result.status,0,result.stderr);
    assert.deepEqual(JSON.parse(result.stdout),[exitCode,'O'.repeat(1024),'E'.repeat(1024)]);
  }
});
