import assert from 'node:assert/strict';
import {mkdtempSync,mkdirSync,readFileSync,rmSync,symlinkSync,writeFileSync} from 'node:fs';
import {tmpdir} from 'node:os';
import {join} from 'node:path';
import test from 'node:test';
import {fileURLToPath} from 'node:url';
import {WEB_FILES,sealWeb,importWeb} from '../../../scripts/release-web.mjs';

function fixture(run) {
  const directory=mkdtempSync(join(tmpdir(),'formal-ai-release-web-'));
  const root=join(directory,'source'),target=join(directory,'target');
  const payload=join(directory,'payload');
  mkdirSync(join(root,'js'),{recursive:true});
  mkdirSync(join(target,'js'),{recursive:true});
  const context={commit:'a'.repeat(40),tree:'b'.repeat(40),run:'123'};
  const selection=Buffer.from(JSON.stringify({version:1,source_commit:context.commit,
    source_tree:context.tree,producer_run:context.run,package_version:'1.2.3'}));
  // Data-only fixture bytes have zero product/build or renderer-readiness credit.
  for(const name of WEB_FILES)writeFileSync(join(root,'js',name),'fixture '+name);
  try {
    sealWeb(root,payload,selection,context);
    run({root,target,payload,selection,context});
  } finally {
    rmSync(directory,{recursive:true,force:true});
  }
}

test('generated renderer payload round trips every exact allowlisted byte',()=>{
  fixture(({root,target,payload,selection,context})=>{
    const receipt=importWeb(target,payload,selection,context);
    assert.equal(receipt.files.length,9);
    for(const name of WEB_FILES) {
      assert.deepEqual(readFileSync(join(target,'js',name)),readFileSync(join(root,'js',name)));
    }
  });
});

for(const fault of ['corrupt','missing','extra','cross-source','cross-run','receipt','symlink']) {
  test('renderer import refuses '+fault+' before any output',()=>{
    fixture(({target,payload,selection,context})=>{
      let supplied=selection,actual={...context};
      if(fault==='corrupt')writeFileSync(join(payload,WEB_FILES[8]),'changed');
      if(fault==='missing')rmSync(join(payload,WEB_FILES[8]));
      if(fault==='extra')writeFileSync(join(payload,'unexpected.js'),'extra');
      if(fault==='cross-source')actual.commit='c'.repeat(40);
      if(fault==='cross-run')actual.run='456';
      if(fault==='receipt') {
        const path=join(payload,'web-receipt.json');
        const receipt=JSON.parse(readFileSync(path));
        receipt.source_tree='c'.repeat(40);
        writeFileSync(path,JSON.stringify(receipt));
      }
      if(fault==='symlink') {
        rmSync(join(payload,WEB_FILES[8]));
        symlinkSync(join(payload,WEB_FILES[0]),join(payload,WEB_FILES[8]));
      }
      assert.throws(()=>importWeb(target,payload,supplied,actual));
      for(const name of WEB_FILES) {
        assert.throws(()=>readFileSync(join(target,'js',name)),{code:'ENOENT'});
      }
    });
  });
}

test('Desktop producer and both package consumers use the same source-bound handoff',()=>{
  const root=fileURLToPath(new URL('../../../',import.meta.url));
  const workflow=readFileSync(join(root,'.github/workflows/desktop-release.yml'),'utf8');
  const action=readFileSync(join(root,'.github/actions/release-web/action.yml'),'utf8');
  assert.equal(workflow.split('uses: ./.github/actions/release-web').length-1,3);
  assert.equal(workflow.split('with: {mode: build}').length-1,1);
  assert.equal(workflow.split('with: {mode: import}').length-1,2);
  assert(workflow.indexOf('with: {mode: build}')>workflow.indexOf('build-selected-wasm.mjs" build'));
  assert(workflow.indexOf('with: {mode: import}')<workflow.indexOf('run: node scripts/prepare-resources.mjs'));
  assert.match(action,/bun install --frozen-lockfile --ignore-scripts/u);
  assert.match(action,/bun-version-file: \.bun-version/u);
  assert.match(action,/bun run build:web/u);
  assert.match(action,/NODE_OPTIONS: --disable-warning=DEP0005/u);
  assert.doesNotMatch(action,/run-id:|github-token:|actions\/cache/u);
  assert.match(action,/name: native-release-web/u);
  assert(workflow.split('\n').length-1<=1500);
});
