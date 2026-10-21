import assert from 'node:assert/strict';
import {readFileSync} from 'node:fs';
import {fileURLToPath} from 'node:url';
import {test} from 'node:test';
const source=readFileSync(fileURLToPath(new URL('../../../Dockerfile',import.meta.url)),'utf8');
function contract(text){
 assert.match(text,/^FROM node:26\.12\.0-bookworm-slim AS node-runtime$/m);
 assert(text.includes('ENV PATH="${BUN_INSTALL}/bin:/usr/local/bin:${PATH}"'));
 assert.doesNotMatch(text,/FROM --platform=\S+ node:/);
 const copy=text.indexOf('COPY --from=node-runtime /usr/local/bin/node /usr/local/bin/node');
 const libraries=text.indexOf('--no-install-recommends libstdc++6 libatomic1');
 const capability=text.indexOf("new RegExp('', 'v')");
 const install=text.indexOf('RUN /usr/local/lib/formal-ai-runtime-tools/install-container-runtime-tools.sh /home/box/.formal-ai-runtime-tools');
 assert(libraries>text.indexOf('FROM konard/box-dind:2.10.2')&&copy>libraries&&capability>copy&&install>capability);
 assert(text.includes("process.versions.node.split('.')[0] !== '26'"));
 assert.doesNotMatch(text,/apt-get install[^\n]*\bnodejs\b/);
 for(const check of ['"$" --version','agent --version','start-agent --help','COPY scripts/verify-docker-runtime.sh','ENTRYPOINT ["/usr/local/bin/dind-entrypoint.sh"]'])assert(text.includes(check));
}
test('target-native Node26 donor and shared libraries precede agent capability checks',()=>contract(source));
test('Node18 regression is rejected',()=>assert.throws(()=>contract(source.replace('node:26.12.0-bookworm-slim','node:18.19.1-bookworm-slim'))));
test('architecture pin and missing capability fail closed',()=>{
 assert.throws(()=>contract(source.replace('FROM node:','FROM --platform=linux/amd64 node:')));
 assert.throws(()=>contract(source.replace("new RegExp('', 'v')",'true')));
 assert.throws(()=>contract(source.replace('libstdc++6 libatomic1','nodejs')));
});
