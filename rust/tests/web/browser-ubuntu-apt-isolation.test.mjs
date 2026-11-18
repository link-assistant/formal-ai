import test from 'node:test';
import assert from 'node:assert/strict';
import {mkdtempSync, mkdirSync, readFileSync, writeFileSync, rmSync, existsSync, readdirSync, realpathSync, symlinkSync} from 'node:fs';
import {tmpdir} from 'node:os';
import {join} from 'node:path';
import {spawnSync} from 'node:child_process';
import {fileURLToPath} from 'node:url';
import {createRequire} from 'node:module';
const root=fileURLToPath(new URL('../../../',import.meta.url));
const helper=join(root,'scripts/with-ubuntu-apt-sources.sh');
const captured=readFileSync(new URL('../fixtures/apt/browser-microsoft-403.log',import.meta.url),'utf8');
const YAML=createRequire(import.meta.url)('yaml');

function fixture(t){
  const directory=realpathSync(mkdtempSync(join(tmpdir(),'browser-apt-isolation-')));
  t.after(()=>rmSync(directory,{recursive:true,force:true}));
  const bin=join(directory,'bin'),sources=join(directory,'sources.list.d');
  const home=join(directory,'home'),work=join(directory,'work');
  for(const path of [bin,sources,home,work])mkdirSync(path);
  const mirror=join(directory,'apt-mirrors.txt');
  writeFileSync(mirror,'http://azure.archive.ubuntu.com/ubuntu/\tpriority:1\n');
  const ubuntu=join(sources,'ubuntu.sources'),foreign=join(sources,'microsoft.sources');
  const ubuntuText='# Distribution managed: preserve comments, suites, components and keyring.\nTypes: deb\nURIs: mirror+file:/etc/apt/apt-mirrors.txt\nSuites: noble noble-updates noble-backports\nComponents: main restricted universe multiverse\nSigned-By: /usr/share/keyrings/ubuntu-archive-keyring.gpg\n\nTypes: deb\nURIs: https://security.ubuntu.com/ubuntu/\nSuites: noble-security\nComponents: main restricted universe multiverse\nSigned-By: /usr/share/keyrings/ubuntu-archive-keyring.gpg\n';
  const foreignText='Types: deb\nURIs: https://packages.microsoft.com/ubuntu/24.04/prod\nSuites: noble\nComponents: main\nSigned-By: /usr/share/keyrings/microsoft-prod.gpg\n';
  writeFileSync(ubuntu,ubuntuText);writeFileSync(foreign,foreignText);
  const cached=join(directory,'existing-third-party-index');writeFileSync(cached,'retained runner index\n');
  const calls=join(directory,'calls.jsonl'),client=join(directory,'client');
  const sudo=join(bin,'real-sudo');
  writeFileSync(sudo,`#!/usr/bin/env bash
set -eu
# Model sudo env_reset; an ordinary inherited APT_CONFIG is insufficient.
if [ "\${1:-}" = --preserve-env=APT_CONFIG ]; then shift; else unset APT_CONFIG; fi
if [ "\${1:-}" = -- ]; then shift; fi
exec "$@"
`,{mode:0o755});
  writeFileSync(join(bin,'sudo'),`#!/usr/bin/env bash\nexec "$FAKE_REAL_SUDO" "$@"\n`,{mode:0o755});
  writeFileSync(join(bin,'apt-get'),`#!/usr/bin/env node
const fs=require('node:fs'),path=require('node:path');
const config=process.env.APT_CONFIG;
const text=config?fs.readFileSync(config,'utf8'):'';
const selected=config?fs.readdirSync(/Dir::Etc::sourceparts "([^"\\n]+)";/.exec(text)[1]).map(name=>path.join(/Dir::Etc::sourceparts "([^"\\n]+)";/.exec(text)[1],name)):fs.readdirSync(process.env.FAKE_SOURCES).map(name=>path.join(process.env.FAKE_SOURCES,name));
const sourceTexts=selected.map(file=>fs.readFileSync(file,'utf8'));
fs.appendFileSync(process.env.FAKE_CALLS,JSON.stringify({args:process.argv.slice(2),config,text,sourceTexts,home:process.env.HOME})+'\\n');
if(sourceTexts.some(value=>value.includes('packages.microsoft.com'))){process.stderr.write(process.env.FAKE_CAPTURED);process.exit(100);}
if(process.env.FAKE_SIGNATURE_FAILURE==='1'){process.stderr.write('E: Ubuntu repository signature could not be verified\\n');process.exit(100);}
`,{mode:0o755});
  writeFileSync(client,`#!/usr/bin/env bash
set -eu
printf '%s\\n' "$@" > "$HOME/browser-arguments"
if [ "\${FAKE_DIRECT_ROOT:-}" = 1 ]; then
  sh -c 'apt-get update && apt-get install -y --no-install-recommends libexample'
else
  # Playwright 1.64.0 transformCommandsForRoot uses exactly this boundary.
  sudo -- sh -c 'apt-get update && apt-get install -y --no-install-recommends libexample'
fi
[ "\${FAKE_BROWSER_FAILURE:-}" != 1 ] || exit 71
printf 'browser downloaded by original user\\n' > "$HOME/browser-cache"
`,{mode:0o755});
  const env={...process.env,PATH:bin+':'+process.env.PATH,HOME:home,TMPDIR:work,
    FORMAL_AI_UBUNTU_APT_SOURCES:ubuntu,FORMAL_AI_UBUNTU_APT_SUDO:sudo,
    FORMAL_AI_APT_MIRROR_LIST:mirror,FAKE_REAL_SUDO:sudo,FAKE_SOURCES:sources,
    FAKE_CALLS:calls,FAKE_CAPTURED:captured};
  delete env.APT_CONFIG;
  function run({wrapped=true,environment={}}={}){
    const args=[client,'install','chromium','--with-deps'];
    const result=spawnSync('bash',wrapped?[helper,...args]:args,{cwd:root,env:{...env,...environment},encoding:'utf8',timeout:10000,maxBuffer:1048576});
    assert.ifError(result.error);assert.equal(result.signal,null);
    assert.equal(readFileSync(ubuntu,'utf8'),ubuntuText);
    assert.equal(readFileSync(foreign,'utf8'),foreignText);
    assert.equal(readFileSync(cached,'utf8'),'retained runner index\n');
    const observed=existsSync(calls)?readFileSync(calls,'utf8').trim().split('\n').map(JSON.parse):[];
    assert.deepEqual(readdirSync(work),[],'all command-scoped APT files are removed');
    for(const call of observed)if(call.config)assert.equal(existsSync(call.config),false);
    return {...result,calls:observed};
  }
  return {directory,home,work,mirror,ubuntu,ubuntuText,sudo,env,calls,client,run};
}

test('captured Microsoft HTTP403 blocks the original Playwright privilege boundary',t=>{
  const f=fixture(t),result=f.run();
  // Even after mirror preparation, the original unscoped command still fails.
  assert.equal(result.status,0,result.stderr);
  rmSync(f.calls);rmSync(join(f.home,'browser-cache'));
  const original=f.run({wrapped:false,environment:{APT_CONFIG:join(f.directory,'unused.conf')}});
  assert.equal(original.status,100);assert.equal(original.calls.length,1);
  assert.equal(original.stderr,captured,'sudo clears an unpreserved configuration');
  assert.equal(existsSync(join(f.home,'browser-cache')),false);
});

for(const direct of [false,true])test('only byte-exact signed Ubuntu sources reach update/install; browser keeps its user '+direct,t=>{
  const f=fixture(t),result=f.run({environment:{FAKE_DIRECT_ROOT:direct?'1':'0'}});
  assert.equal(result.status,0,result.stderr);assert.equal(result.calls.length,2);
  assert.deepEqual(result.calls.map(value=>value.args),[['update'],['install','-y','--no-install-recommends','libexample']]);
  for(const call of result.calls){
    assert.deepEqual(call.sourceTexts,[f.ubuntuText]);assert.equal(call.home,f.home);
    assert.match(call.text,/APT::Get::List-Cleanup "false";/u);
    assert.doesNotMatch(call.text,/allow-unauthenticated|trusted|AllowInsecure|AllowUnauthenticated/u);
  }
  assert.equal(readFileSync(join(f.home,'browser-arguments'),'utf8'),'install\nchromium\n--with-deps\n');
  assert.equal(readFileSync(join(f.home,'browser-cache'),'utf8'),'browser downloaded by original user\n');
  assert.equal(readFileSync(f.mirror,'utf8'),'https://archive.ubuntu.com/ubuntu/\tpriority:1\n');
});

test('Ubuntu signature failure remains fatal without another attempt or browser download',t=>{
  const f=fixture(t),result=f.run({environment:{FAKE_SIGNATURE_FAILURE:'1'}});
  assert.equal(result.status,100);assert.equal(result.calls.length,1);
  assert.match(result.stderr,/signature could not be verified/u);
  assert.equal(existsSync(join(f.home,'browser-cache')),false);
});

test('a later browser download failure propagates after scoped APT cleanup',t=>{
  const f=fixture(t),result=f.run({environment:{FAKE_BROWSER_FAILURE:'1'}});
  assert.equal(result.status,71);assert.equal(result.calls.length,2);
  assert.equal(existsSync(join(f.home,'browser-cache')),false);
});

for(const shape of ['missing','symlink','oversized'])test('unsafe Ubuntu source input fails before any dependency command '+shape,t=>{
  const f=fixture(t),input=join(f.directory,'unsupported.sources');
  if(shape==='symlink')symlinkSync(f.ubuntu,input);
  if(shape==='oversized')writeFileSync(input,'#'.repeat(65537));
  const result=f.run({environment:{FORMAL_AI_UBUNTU_APT_SOURCES:input}});
  assert.equal(result.status,2);assert.equal(result.calls.length,0);
  assert.equal(existsSync(join(f.home,'browser-cache')),false);
  assert.equal(readFileSync(f.mirror,'utf8'),'http://azure.archive.ubuntu.com/ubuntu/\tpriority:1\n');
});

test('all Ubuntu browser CI paths use the shared wrapper with their original deadlines and checks',()=>{
  const load=name=>YAML.parse(readFileSync(join(root,'.github/workflows',name),'utf8'));
  const producer=load('publish-engine.yml');
  for(const path of ['scripts/prepare-ubuntu-apt-mirror.sh','scripts/with-ubuntu-apt-sources.sh']){
    assert(producer.on.pull_request.paths.includes(path),'a helper-only edit must run the real browser checks');
  }
  const consumer=load('verify-release-downloads.yml').jobs['browser-public-package'];
  assert.equal(consumer['timeout-minutes'],15);
  assert.equal(consumer.steps.find(step=>step.name==='Install test browser without compiling the product').run,
    'bash scripts/with-ubuntu-apt-sources.sh node _source/vscode/node_modules/playwright/cli.js install chromium --with-deps');
  assert(consumer.steps.some(step=>step.name==='Verify original engine bindings and fresh public package in a real Worker'));
  for(const name of ['release.yml','e2e-local.yml']){
    const jobs=Object.values(load(name).jobs);
    const step=jobs.flatMap(job=>job.steps??[]).find(value=>value.name==='Install Playwright system dependencies (best effort)');
    assert.equal(step['timeout-minutes'],6);
    assert.match(step.run,/bash \.\.\/\.\.\/\.\.\/scripts\/with-ubuntu-apt-sources\.sh npx playwright install-deps chromium/u);
  }
});
