// Finite scheduling-only probe. No functional suites, child compilers or writes.
// Usage: node experiments/issue-1580/probe-javascript-duration-reserves.mjs <floor.lino> <observations.jsonl>
import {readFileSync,readdirSync} from 'node:fs';
import {parseTestDurations} from '../../scripts/lib/ci-speed-durations.mjs';
import {durationLookup,planShards} from '../../scripts/lib/ci-speed-shards.mjs';
const [floorPath,observationsPath]=process.argv.slice(2);
if(!floorPath||!observationsPath)throw Error('Provide the previous scheduling floor and current observations');
const floorText=readFileSync(floorPath,'utf8');
const floors=parseTestDurations(floorText);
const defaultSeconds=Number(floorText.match(/^ {2}default-seconds (\S+)$/mu)[1]);
const current=readFileSync(observationsPath,'utf8').trim().split('\n').map(line=>JSON.parse(line));
const files=readdirSync(new URL('../../rust/tests/web/',import.meta.url)).filter(file=>file.endsWith('.test.mjs')).map(file=>'rust/tests/web/'+file);
const captures=readdirSync(new URL('./fixtures/',import.meta.url)).filter(name=>name.startsWith('hosted-javascript-'));
const cohorts=captures.map(name=>{
  const directory=new URL('./fixtures/'+name+'/',import.meta.url);
  const identity=JSON.parse(readFileSync(new URL('identity.json',directory),'utf8'));
  const measured=new Map(readFileSync(new URL('durations.jsonl',directory),'utf8').trim().split('\n').map(line=>{const row=JSON.parse(line);return[row.file,row.seconds];}));
  const splitParent=name==='hosted-javascript-de79e6ec';
  if(splitParent)measured.delete('rust/tests/web/full-provenance-cli.test.mjs');
  return{name,measured,allowance:identity.runnerOverheadAllowanceSeconds??0,fallback:splitParent?0:identity.unobservedFallbackSeconds};
});
for(let step=0;step<=12;step++){
  const factor=(140+step*5)/100;
  const weights=new Map(floors);
  for(const row of current)weights.set(row.file,Math.max(weights.get(row.file)??defaultSeconds,Math.ceil(row.seconds*factor*10)/10));
  const {shards}=planShards(files,8,durationLookup(weights,defaultSeconds));
  const results=cohorts.map(({name,measured,allowance,fallback})=>{
    const costs=durationLookup(measured,fallback);
    const seconds=shards.map(shard=>{
      const workers=Array(4).fill(0);
      for(const file of shard)workers[workers.indexOf(Math.min(...workers))]+=costs(file);
      return Math.max(...workers)+allowance;
    });
    return{name,max:Math.max(...seconds),fits:seconds.every(seconds=>seconds<=270)};
  });
  console.log(JSON.stringify({factor,allFit:results.every(row=>row.fits),results}));
}
