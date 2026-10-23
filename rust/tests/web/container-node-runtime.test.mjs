import assert from 'node:assert/strict';
import fs,{readFileSync} from 'node:fs';
import {fileURLToPath} from 'node:url';
import {test} from 'node:test';
import os from 'node:os';
import path from 'node:path';
import {spawnSync} from 'node:child_process';
const source=readFileSync(fileURLToPath(new URL('../../../Dockerfile',import.meta.url)),'utf8');
function contract(text){
 assert.match(text,/^FROM node:26\.12\.0-bookworm-slim AS node-runtime$/m);
 assert(text.includes('ENV PATH="${BUN_INSTALL}/bin:/usr/local/bin:${PATH}"'));
 assert.doesNotMatch(text,/FROM --platform=\S+ node:/);
 const copy=text.indexOf('COPY --from=node-runtime /usr/local/bin/node /usr/local/bin/node');
 assert(text.includes('COPY --from=node-runtime /usr/local/lib/node_modules/npm/ /usr/local/lib/node_modules/npm/'));
 const libraries=text.indexOf('--no-install-recommends libstdc++6 libatomic1');
 const capability=text.indexOf("new RegExp('', 'v')");
 const install=text.indexOf('RUN /usr/local/lib/formal-ai-runtime-tools/install-container-runtime-tools.sh /home/box/.formal-ai-runtime-tools');
 assert(libraries>text.indexOf('FROM konard/box-dind:2.10.2')&&copy>libraries&&capability>copy&&install>capability);
 assert(text.includes("process.versions.node.split('.')[0] !== '26'"));
 assert.doesNotMatch(text,/apt-get install[^\n]*\bnodejs\b/);
 for(const check of ['"$" --version','agent --version','start-agent --help','COPY scripts/verify-docker-runtime.sh','ENTRYPOINT ["/usr/local/bin/dind-entrypoint.sh"]'])assert(text.includes(check));
 for(const check of ['npm --version','npx --version'])assert(text.indexOf(check)>text.indexOf('USER box',copy));
}
test('target-native Node26 donor and shared libraries precede agent capability checks',()=>contract(source));
test('Node18 regression is rejected',()=>assert.throws(()=>contract(source.replace('node:26.12.0-bookworm-slim','node:18.19.1-bookworm-slim'))));
test('architecture pin and missing capability fail closed',()=>{
 assert.throws(()=>contract(source.replace('FROM node:','FROM --platform=linux/amd64 node:')));
 assert.throws(()=>contract(source.replace("new RegExp('', 'v')",'true')));
 assert.throws(()=>contract(source.replace('libstdc++6 libatomic1','nodejs')));
});

test('captured Hive npm failure is repaired by the shipped package payload and launchers',t=>{
 const scratch=fs.mkdtempSync(path.join(os.tmpdir(),'formal-ai-npm-image-control-'));
 t.after(()=>fs.rmSync(scratch,{recursive:true,force:true}));
 const prefix=path.join(scratch,'usr','local'),bin=path.join(prefix,'bin'),project=path.join(scratch,'project');
 fs.mkdirSync(bin,{recursive:true});fs.mkdirSync(project);
 fs.symlinkSync(process.execPath,path.join(bin,'node'));
 for(const name of ['user.npmrc','global.npmrc'])fs.writeFileSync(path.join(scratch,name),'');
 const env={...Object.fromEntries(Object.entries(process.env).filter(([key])=>!/^npm_/iu.test(key))),
  PATH:bin,HOME:scratch,NPM_PROJECT:project,npm_config_userconfig:path.join(scratch,'user.npmrc'),
  npm_config_globalconfig:path.join(scratch,'global.npmrc'),npm_config_cache:path.join(scratch,'cache'),
  npm_config_update_notifier:'false'};
 const captured=readFileSync(new URL('../fixtures/container/npm-missing.log',import.meta.url),'utf8');
 const original=captured.match(/^RUN (npm .+)$/m)[1].replace('/opt/hive-e2e','"$NPM_PROJECT"');
 const invoke=command=>spawnSync('/bin/sh',['-c',command],{cwd:project,env,encoding:'utf8',timeout:15000,maxBuffer:1048576});
 const missing=invoke(original);
 assert.equal(missing.status,127);assert.match(missing.stderr,/npm: (?:command )?not found/u);
 assert.match(captured,/npm: command not found/u);
 // Emulate only the immutable donor COPY and launcher instructions, using the
 // genuine npm bundled with the test runner's Node. No image or network is used.
 const npmCli=fs.realpathSync(path.join(path.dirname(process.execPath),'npm'));
 const donor=path.dirname(path.dirname(npmCli));
 assert.equal(JSON.parse(readFileSync(path.join(donor,'package.json'))).name,'npm');
 const payload=source.match(/^COPY --from=node-runtime (\/usr\/local\/lib\/node_modules\/npm\/) (\/usr\/local\/lib\/node_modules\/npm\/)$/m);
 assert(payload,'the same official Node donor must supply npm');
 fs.cpSync(donor,path.join(scratch,payload[2]),{recursive:true});
 const launchers=[...source.matchAll(/\bln -sf (\S+) (\/usr\/local\/bin\/(?:npm|npx))/g)];
 assert.deepEqual(launchers.map(row=>path.basename(row[2])).sort(),['npm','npx']);
 for(const row of launchers){
  const link=spawnSync('/bin/ln',['-sf',row[1],path.join(bin,path.basename(row[2]))],{encoding:'utf8',timeout:5000});
  assert.equal(link.status,0,link.stderr);
 }
 const name='formal-ai-offline-package-control',version='1.0.0';
 fs.writeFileSync(path.join(project,'package.json'),JSON.stringify({name,version,private:true,scripts:{install:'exit 73'}}));
 fs.writeFileSync(path.join(project,'package-lock.json'),JSON.stringify({name,version,lockfileVersion:3,requires:true,packages:{'':{name,version,hasInstallScript:true}}}));
 const installed=invoke(original+' --offline --no-audit --no-fund');
 assert.equal(installed.error,undefined);assert.equal(installed.status,0,installed.stderr);
 for(const tool of ['npm','npx']){
  const observed=invoke(tool+' --version');assert.equal(observed.status,0,observed.stderr);
  assert.match(observed.stdout.trim(),/^\d+\.\d+\.\d+$/u);
 }
});
