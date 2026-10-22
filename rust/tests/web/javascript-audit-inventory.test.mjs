import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import {fileURLToPath} from 'node:url';
import {execFileSync,spawnSync} from 'node:child_process';
const root=fileURLToPath(new URL('../../../',import.meta.url));
const inventory=execFileSync('git',['ls-files','-z','--','bun.lock','**/bun.lock','package-lock.json','**/package-lock.json'],{cwd:root,encoding:'utf8',timeout:10000,maxBuffer:1048576})
  .split('\0').filter(name=>['bun.lock','package-lock.json'].includes(path.basename(name))
    &&fs.existsSync(path.join(root,name))).sort();
assert(inventory.length>0,'the committed lockfile inventory must be nonempty');
function audit(mode,failures,attempts) {
  const directory=fs.mkdtempSync(path.join(os.tmpdir(),'formal-ai-audit-inventory-'));
  try {
    const standIn=`#!/usr/bin/env bash
set -eu
log="$FAKE_AUDIT_CALLS.$(printf '%s' "$PWD" | tr -c 'A-Za-z0-9' '_')"
printf '%s\\n' "$*" >> "$log"
case "$(basename "$0")" in
  npm)
    [ "$*" = 'audit --package-lock-only --audit-level=moderate --fetch-timeout=30000 --fetch-retries=0' ] || {
      echo 'unexpected npm audit request policy' >&2; exit 64;
    } ;;
  bun)
    [ "$*" = 'audit --audit-level=moderate' ] || {
      echo 'unexpected Bun audit arguments' >&2; exit 64;
    } ;;
esac
if [ "$(wc -l < "$log")" -le "$FAKE_AUDIT_FAILURES" ]; then
  case "$FAKE_AUDIT_MODE" in
    advisory) echo '1 vulnerability found (1 moderate)'; exit 1 ;;
    npm-outage) echo 'npm error audit endpoint returned an error' >&2; exit 1 ;;
    request-timeout) echo 'npm error code ETIMEDOUT' >&2; exit 1 ;;
    *) echo 'error: POST https://registry.npmjs.org/-/npm/v1/security/advisories/bulk - 503' >&2; exit 1 ;;
  esac
fi
echo 'No vulnerabilities found'
`;
    for(const name of ['bun','npm'])fs.writeFileSync(path.join(directory,name),standIn,{mode:0o755});
    const result=spawnSync('bash',['scripts/check-javascript-dependencies.sh'],{
      cwd:root,encoding:'utf8',timeout:30000,maxBuffer:1048576,
      env:{...process.env,PATH:directory+path.delimiter+process.env.PATH,
        FAKE_AUDIT_CALLS:path.join(directory,'calls'),FAKE_AUDIT_MODE:mode,
        FAKE_AUDIT_FAILURES:String(failures),FORMAL_AI_AUDIT_ATTEMPTS:String(attempts),
        FORMAL_AI_AUDIT_ATTEMPT_SECONDS:'5',FORMAL_AI_AUDIT_BUDGET_SECONDS:'20',
        FORMAL_AI_AUDIT_RETRY_DELAY_SECONDS:'0'}});
    assert.ifError(result.error);
    assert.equal(result.signal,null);
    const reported=result.stdout.split('\n').filter(line=>line.startsWith('Auditing '))
      .map(line=>line.slice('Auditing '.length)).sort();
    assert.deepEqual(reported,inventory,'every tracked lockfile must actually be audited');
    const calls=fs.readdirSync(directory).filter(name=>name.startsWith('calls.'))
      .map(name=>fs.readFileSync(path.join(directory,name),'utf8').trim().split('\n').length);
    assert.equal(calls.length,inventory.length);
    return {...result,calls};
  } finally {fs.rmSync(directory,{recursive:true,force:true});}
}
for(const mode of ['outage','npm-outage'])test(`${mode}: each committed lockfile retries twice and obtains an answer`,()=>{
  const result=audit(mode,2,3);assert.equal(result.status,0,result.stdout+result.stderr);
  assert(result.calls.every(count=>count===3));
  for(const lock of inventory)assert.equal(result.stderr.split(
    `::warning title=advisory registry unreachable for ${lock}::`).length-1,2);
});
test('an advisory ends each committed lockfile audit on the first attempt',()=>{
  const result=audit('advisory',99,3);assert.notEqual(result.status,0);
  assert(result.calls.every(count=>count===1));
  assert.match(result.stdout,/1 vulnerability found/);
  assert(!result.stderr.includes('registry unreachable'));
});
test('exhausting unanswered attempts refuses every committed lockfile',()=>{
  const result=audit('outage',99,2);assert.notEqual(result.status,0);
  assert(result.calls.every(count=>count===2));
  for(const lock of inventory)assert(result.stderr.includes(
    `::error title=advisory registry unreachable for ${lock}::`));
});

test('request timeouts recover only after every lock receives a real verdict',()=>{
  const result=audit('request-timeout',2,3);
  assert.equal(result.status,0,result.stdout+result.stderr);
  assert(result.calls.every(count=>count===3));
  const records=result.stderr.split('\n').filter(line=>line.startsWith('audit-attempt '));
  assert.equal(records.length,inventory.length*6);
  for(const line of records)assert.match(line,
    /^audit-attempt phase=(start|finish) tool=(bun|npm) lock=[A-Za-z0-9_./-]+ attempt=[1-3] status=(pending|0|1) elapsedSeconds=[0-9]+$/);
  assert.equal(records.filter(line=>line.includes('phase=finish')&&line.includes(' status=0 ')).length,inventory.length);
});
test('persistent request timeouts never substitute a clean audit verdict',()=>{
  const result=audit('request-timeout',99,3);
  assert.notEqual(result.status,0);
  assert(result.calls.every(count=>count===3));
  assert(!result.stdout.includes('No vulnerabilities found'));
  for(const lock of inventory)assert(result.stderr.includes(
    `::error title=advisory registry unreachable for ${lock}::`));
});
