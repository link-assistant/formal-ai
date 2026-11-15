import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import {fileURLToPath} from 'node:url';
import {execFileSync} from 'node:child_process';
import {collect, render, stripComment} from '../../../scripts/generate-role-registry.mjs';

const repository=fileURLToPath(new URL('../../../',import.meta.url));
function fixture(source){
  const root=fs.mkdtempSync(path.join(os.tmpdir(),'formal-ai-role-twin-'));
  fs.mkdirSync(path.join(root,'rust/src/seed'),{recursive:true});
  fs.mkdirSync(path.join(root,'data/seed'),{recursive:true});
  fs.writeFileSync(path.join(root,'rust/src/seed/embedded_registry.rs'),
    'pub const ONE_LINO: &str = include_str!("../../embedded/data/seed/meanings-one.lino");\n'
    +'pub const UNUSED_LINO: &str = include_str!("../../embedded/data/seed/meanings-unused.lino");\n'
    +'pub const MEANING_FILES: &[&str] = &[ONE_LINO];\n');
  fs.writeFileSync(path.join(root,'data/seed/meanings-one.lino'),source);
  fs.writeFileSync(path.join(root,'data/seed/meanings-unused.lino'),'meanings\n  unseen\n    role unseen\n');
  return root;
}
function dispose(root){
  assert.ok(path.basename(root).startsWith('formal-ai-role-twin-'));
  fs.rmSync(root,{recursive:true,force:true});
}
const sample='meanings\n  shared-role\n    role shared-role\n  helper:\n    role zeta_1 # trailing\n    nested\n      role alpha\n    role zeta_1\n    role Uppercase\n    role invalid/path\n    lexeme en "text # retained"\n';

test('registry follows MEANING_FILES and classifies sorted unique meaning and predicate roles',()=>{
  const root=fixture(sample);
  try{
    const facts=collect(root);
    assert.deepEqual([...facts.defined].sort(),['helper','shared-role']);
    assert.deepEqual([...facts.roles].sort(),['alpha','shared-role','zeta_1']);
    const text=render(facts);
    assert.ok(text.endsWith('roles\n  alpha\n    kind predicate\n  shared-role\n    kind meaning\n  zeta_1\n    kind predicate\n'));
    assert.ok(!text.includes('unseen'));
  }finally{dispose(root);}
});

test('comments retain quoted hashes and canonical indentation counts only ASCII spaces',()=>{
  assert.equal(stripComment('role alpha # discarded'),'role alpha');
  assert.equal(stripComment('lexeme en "a # b" # discarded'),'lexeme en "a # b"');
  const root=fixture('meanings\n\tinner\n  role inner\n');
  try{
    const facts=collect(root);
    assert.equal(facts.defined.has('inner'),false);
    assert.equal(facts.roles.has('inner'),true);
    assert.ok(render(facts).endsWith('roles\n  inner\n    kind predicate\n'));
  }finally{dispose(root);}
});

test('JavaScript CLI checks without writing stale projections',()=>{
  const root=fixture(sample);
  try{
    const script=path.join(repository,'scripts/generate-role-registry.mjs');
    const options={encoding:'utf8',timeout:5000,maxBuffer:16384};
    execFileSync(process.execPath,[script,root],options);
    const target=path.join(root,'data/seed/roles.lino');
    assert.equal(fs.readFileSync(target,'utf8'),render(collect(root)));
    execFileSync(process.execPath,[script,root,'--check'],options);
    fs.appendFileSync(target,'# stale\n');
    assert.throws(()=>execFileSync(process.execPath,[script,root,'--check'],options),error=>error.status===1);
    assert.ok(fs.readFileSync(target,'utf8').endsWith('# stale\n'));
  }finally{dispose(root);}
});

test('canonical Python and JavaScript projections agree on held-out and actual repository seeds',
  {skip:process.env.GITHUB_ACTIONS!=='true'?'Canonical Python parity runs in CI; local tests remain JavaScript only.':false},()=>{
    const root=fixture(sample+'\tinner\n  role inner\n');
    try{
      const script=path.join(repository,'scripts/generate-role-registry.py');
      for(const [index,input]of [root,repository].entries()){
        const target=path.join(root,'canonical-'+index+'.lino');
        execFileSync('python3',[script,input,'--output',target],{encoding:'utf8',timeout:10000,maxBuffer:16384});
        assert.equal(fs.readFileSync(target,'utf8'),render(collect(input)));
      }
    }finally{dispose(root);}
  });
