// Issue #1571: CI steps that fail must show why, and caches must cache.
import test from 'node:test';
import assert from 'node:assert/strict';
import {execFileSync, spawnSync} from 'node:child_process';
import {existsSync, mkdirSync, mkdtempSync, readdirSync, readFileSync, rmSync, symlinkSync, writeFileSync} from 'node:fs';
import {tmpdir} from 'node:os';
import {join} from 'node:path';
import {fileURLToPath} from 'node:url';
import YAML from 'yaml';

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

const learningJob = YAML.parse(workflows.find(({name}) => name === 'learning-cycle.yml').text).jobs.cycle;

test('the live learning capture fails within an execution budget and keeps its diagnostics', (t) => {
  const jobSteps = learningJob.steps;
  const capture = jobSteps.find(step => step.id === 'capture');
  assert.ok(capture, 'the ongoing live capture needs its own execution deadline');
  const build = jobSteps.find(step => step.id === 'build');
  assert.match(build.run, /cargo build --manifest-path rust\/Cargo.toml --release --locked/u);
  assert.match(build.run, /--example dump_google_trends_learning --bin formal-ai/u);
  const phaseBudgets = jobSteps.flatMap(step => step.env?.TEST_BUDGET_SECONDS ?? []);
  assert.equal(phaseBudgets.length, 4);
  assert.ok(phaseBudgets.every(seconds => Number.isInteger(seconds) && seconds > 0));
  assert.ok(phaseBudgets.reduce((sum, seconds) => sum + seconds, 0) <= learningJob['timeout-minutes'] * 60 - 180);
  const diagnostics = jobSteps.find(step => step.with?.name === 'learning-cycle-failure');
  assert.equal(diagnostics.if, `failure() && hashFiles('learning-cycle/run.log') != ''`);
  assert.equal(diagnostics.with.path, 'learning-cycle/run.log');
  const proposals = jobSteps.find(step => step.with?.name === 'learning-cycle-proposals');
  assert.equal(proposals.if, undefined);
  const assertionIndex = jobSteps.findIndex(step => step.name === 'Assert the run stayed proposal-only and human-gated');
  assert.ok(assertionIndex >= 0 && assertionIndex < jobSteps.indexOf(proposals));

  const cwd = mkdtempSync(join(tmpdir(), 'learning-cycle-budget-'));
  t.after(() => rmSync(cwd, {recursive: true, force: true}));
  symlinkSync(fileURLToPath(new URL('scripts/', root)), join(cwd, 'scripts'), 'dir');
  const examples = join(cwd, 'rust', 'target', 'release', 'examples');
  mkdirSync(examples, {recursive: true});
  mkdirSync(join(cwd, 'learning-cycle'));
  const runnerTemp = join(cwd, 'runner temp');
  mkdirSync(runnerTemp);
  const example = join(examples, 'dump_google_trends_learning');
  const runCapture = () => spawnSync('bash', ['-c', capture.run], {
    cwd,
    encoding: 'utf8',
    env: {
      ...process.env,
      RUNNER_TEMP: runnerTemp,
      TEST_BUDGET_SECONDS: '1',
      TEST_BUDGET_GRACE_SECONDS: '0',
      TEST_BUDGET_POLL_SECONDS: '0.05',
    },
  });

  writeFileSync(example, '#!/bin/bash\nprintf "frontier\\n"\nprintf "%s\\n" "$FORMAL_AI_SOURCE_CACHE_DIR" >&2\nsleep 4\n', {mode: 0o755});
  const failure = runCapture();
  assert.equal(failure.status, 124, failure.stderr);
  assert.match(failure.stderr, /::error title=Capture the live Google Trends frontier exceeded its execution budget/u);
  assert.ok(failure.stderr.includes(join(runnerTemp, 'learning-cycle-source')));
  assert.equal(readFileSync(join(cwd, 'learning-cycle', 'run.log'), 'utf8'), failure.stderr);
  assert.equal(readFileSync(join(cwd, 'learning-cycle', 'frontier.lino'), 'utf8'), 'frontier\n');

  writeFileSync(example, '#!/bin/bash\nprintf "source_sha256 ready\\n"\n');
  const success = runCapture();
  assert.equal(success.status, 0, success.stderr);
  assert.equal(readFileSync(join(cwd, 'learning-cycle', 'frontier.lino'), 'utf8'), 'source_sha256 ready\n');
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

// Dependabot reported rustls 0.23.43 in experiments/issue-1017-.../Cargo.lock
// while the Cargo audit read only rust/Cargo.lock and stayed green. Which
// lockfiles the job checks out is pinned in fast-pull-request-production-profile.
test('the Cargo audit discovers every committed Cargo.lock and holds unshipped ones to vulnerabilities, unsound and yanked',()=>{
 const locks=execFileSync('git',['ls-files','-z','--','Cargo.lock','*/Cargo.lock'],{cwd:root,encoding:'utf8'}).split('\0').filter(Boolean);
 assert.ok(locks.includes('rust/Cargo.lock')&&locks.length>1,locks.join(', '));
 const script=readFileSync(new URL('scripts/check-rust-dependencies.sh',root),'utf8');
 assert.match(script,/git ls-files -z -- 'Cargo\.lock' '\*\/Cargo\.lock'/u);
 assert.match(script,/cargo audit --file "\$each" --deny unsound --deny yanked/u);
 assert.match(script,/::error::\$annotation_path is committed but not checked out/u);
});
