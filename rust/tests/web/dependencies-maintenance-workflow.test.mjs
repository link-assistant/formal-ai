import assert from 'node:assert/strict';
import {execFileSync} from 'node:child_process';
import {mkdtempSync,mkdirSync,readFileSync,realpathSync,rmSync} from 'node:fs';
import {tmpdir} from 'node:os';
import path from 'node:path';
import test from 'node:test';
const workflow=readFileSync(new URL('../../../.github/workflows/dependencies-latest.yml',import.meta.url),'utf8');

function observedLocks(source) {
  const loop=source.match(/          while IFS= read -r lock; do\n([\s\S]*?)          done < <\(git ls-files -- '\*bun.lock' '\*package-lock.json'\)/u);
  assert(loop,'maintained lock re-resolution loop missing');
  const directory=realpathSync(mkdtempSync(path.join(tmpdir(),'formal-ai-maintenance-locks-')));
  mkdirSync(path.join(directory,'nested'));
  const script='set -euo pipefail\n'+
    'bun() { printf "bun:%s\\n" "$PWD"; }\n'+
    'npm() { printf "npm:%s\\n" "$PWD"; }\n'+
    'while IFS= read -r lock; do\n'+loop[1]+
    'done <<\'LOCKS\'\nbun.lock\nnested/bun.lock\npackage-lock.json\nnested/package-lock.json\nignored.txt\nLOCKS\n';
  try {
    // Only shell fixture functions run; no dependency resolver or install executes.
    return execFileSync('bash',['-c',script],{cwd:directory,encoding:'utf8',timeout:3000})
      .trim().split('\n').map(line=>line.replace(directory,'ROOT'));
  } finally {rmSync(directory,{recursive:true,force:true});}
}

test('maintenance resolves root and nested locks with their actual owning tools',()=>{
  assert.deepEqual(observedLocks(workflow),['bun:ROOT','bun:ROOT/nested','npm:ROOT','npm:ROOT/nested']);
});

test('original nested-only case pattern demonstrably loses both root locks',()=>{
  const original=workflow.replace('bun.lock|*/bun.lock)','*/bun.lock)')
    .replace('package-lock.json|*/package-lock.json)','*/package-lock.json)');
  assert.deepEqual(observedLocks(original),['bun:ROOT/nested','npm:ROOT/nested']);
});

test('history-dependent maintenance gates receive full checkout history',()=>{
  assert.match(workflow,/- uses: actions\/checkout@v7\.0\.1\n        with:\n          fetch-depth: 0/u);
  assert.match(workflow,/Run registered gates \(stage rust\)/u);
});
