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

// GitHub annotated every job of main run 7837d50 (95 notices) with "The
// ubuntu-latest label will migrate to Ubuntu 26 beginning October 19, 2026"
// (actions/runner-images#14748). A floating label changes the OS under a
// green build, so every job names an image version.
test('every job runs on a pinned runner image, never a floating *-latest label',()=>{
 const floating=/\b(?:ubuntu|windows|macos)-latest\b/u,unpinned=[];
 for(const {name,text} of workflows)for(const match of text.matchAll(/^\s*runs-on:\s*(.+)$/gmu)){
  const value=match[1].trim(),where=`${name}:${text.slice(0,match.index).split('\n').length}`;
  if(!value.includes('${{')){if(floating.test(value))unpinned.push(`${where} ${value}`);continue;}
  // A matrix-driven runner: every value of that matrix key must be pinned,
  // or the expression must map the floating name to a pinned image.
  for(const [,key] of value.matchAll(/matrix\.([\w-]+)/gu))
   for(const [,label] of text.matchAll(new RegExp(`(?:^|[{,])\\s*-?\\s*${key}:\\s*["']?([\\w.-]+)`,'gmu')))
    if(floating.test(label)&&!value.includes(`matrix.${key} == '${label}' && '`))unpinned.push(`${where} ${key}=${label}`);
 }
 assert.deepEqual(unpinned,[]);
 // Lookup tables that name the runner a build ran on use pinned images too.
 const tables=workflows.flatMap(({name,text})=>[...text.matchAll(/^\s*\[[\w-]+\]="([^"]+)"$/gmu)]
  .filter(([,label])=>floating.test(label)).map(([line])=>`${name}: ${line.trim()}`));
 assert.deepEqual(tables,[]);
});

// The daily dependency job moved web-capture to 2.0.2, whose engines say
// node >=26.10.0, and npm only warned (EBADENGINE). Electron 44 embeds Node 24,
// and VS Code runs the extension on Node 24, so the 2.x line cannot load there.
// Every package these runtimes load must accept a Node 24 floor.
test('every runtime package in the desktop and VS Code locks accepts the Node 24 that Electron 44 embeds',()=>{
 const embedded=[24,21,0],floorOf=alternative=>{
  const match=/^(?:>=|\^|~|=)?\s*v?(\d+)(?:\.(\d+))?(?:\.(\d+))?/u.exec(alternative.trim());
  return match?match.slice(1).map(part=>Number(part??0)):[0,0,0];
 },accepts=range=>range.split('||').some(alternative=>{
  const floor=floorOf(alternative);
  for(let index=0;index<3;index+=1)if(floor[index]!==embedded[index])return floor[index]<embedded[index];
  return true;
 });
 const rejected=['desktop','vscode'].flatMap(directory=>{
  const lock=JSON.parse(readFileSync(new URL(`${directory}/package-lock.json`,root),'utf8'));
  return Object.entries(lock.packages??{}).filter(([,entry])=>!entry.dev&&entry.engines?.node&&!accepts(entry.engines.node))
   .map(([path,entry])=>`${directory}/${path}: ${entry.engines.node}`);
 });
 assert.deepEqual(rejected,[]);
 // The pin that holds web-capture on 1.x names the upstream report.
 for(const directory of ['desktop','vscode']){
  const manifest=JSON.parse(readFileSync(new URL(`${directory}/package.json`,root),'utf8'));
  assert.match(manifest.dependencies['@link-assistant/web-capture'],/^\^1\./u);
  assert.match(manifest['@link-assistant/web-capture//'],/^blocked: https:\/\/github\.com\/link-assistant\/web-capture\/issues\/191 /u);
 }
});
