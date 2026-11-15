// Issue #1571: CI steps that fail must show why, and caches must cache.
// Workflow text is read directly so the check needs no installed package.
import test from 'node:test';
import assert from 'node:assert/strict';
import {existsSync,readdirSync,readFileSync} from 'node:fs';

const root=new URL('../../../',import.meta.url);
const workflows=readdirSync(new URL('.github/workflows/',root)).filter(name=>/\.ya?ml$/u.test(name))
 .map(name=>({name,text:readFileSync(new URL(`.github/workflows/${name}`,root),'utf8')}));

// The `with:` of the step whose `uses:` line starts at `index`: an inline
// `with: {...}` or the block indented under the step.
function stepWith(text,index){
 const lines=text.slice(index).split('\n'),indent=/^\s*(?:-\s*)?/u.exec(lines[0])[0].length,body=[];
 for(const line of lines.slice(1)){
  if(line.trim()&&!line.trim().startsWith('#')&&(/^\s*/u.exec(line)[0].length<indent||/^\s*-\s/u.test(line)&&/^\s*/u.exec(line)[0].length<=indent))break;
  body.push(line);
 }
 return body.join('\n');
}

test('every rust-cache step names the rust/ workspace, because the repository root has no Cargo.toml',()=>{
 assert.equal(existsSync(new URL('Cargo.toml',root)),false);
 assert.equal(existsSync(new URL('rust/Cargo.toml',root)),true);
 const steps=workflows.flatMap(({name,text})=>[...text.matchAll(/^[ \t-]*uses:\s*Swatinem\/rust-cache@\S+/gmu)]
  .map(match=>({where:`${name}:${text.slice(0,match.index).split('\n').length}`,with:stepWith(text,match.index)})));
 assert.ok(steps.length>0);
 // Without `workspaces` the action runs `cargo metadata` at the root, which
 // fails, and restores and saves an empty cache (learning-cycle run 43706083454).
 const unbound=steps.filter(step=>!/workspaces:\s*["']?rust\s*->/u.test(step.with)).map(step=>step.where);
 assert.deepEqual(unbound,[]);
});

test('the learning cycle prints its captured stderr on every exit, so a failed cargo run is not a bare exit 101',()=>{
 const {text}=workflows.find(({name})=>name==='learning-cycle.yml');
 const step=text.slice(text.indexOf('- name: Run one learning cycle'));
 const run=step.slice(0,step.indexOf('\n      - name:'));
 const trap=run.search(/^\s*trap '[^']*cat learning-cycle\/run\.log[^']*' EXIT$/mu);
 assert.ok(trap>=0,run);
 assert.ok(trap<run.indexOf('cargo run'),'the trap is installed before the first cargo run');
});
