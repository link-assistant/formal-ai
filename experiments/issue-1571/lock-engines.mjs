// Issue #1571: list package-lock entries whose engines.node floor is above a runtime.
// Usage: node experiments/issue-1571/lock-engines.mjs <package-lock.json> <node-version>
import {readFileSync} from 'node:fs';
const [lockPath,runtime='22.0.0']=process.argv.slice(2);
const parse=v=>v.split('.').map(Number);
const less=(a,b)=>{for(let i=0;i<3;i++){if((a[i]??0)!==(b[i]??0))return (a[i]??0)<(b[i]??0);}return false;};
const lock=JSON.parse(readFileSync(lockPath,'utf8'));
for(const [path,entry] of Object.entries(lock.packages??{})){
 const range=entry.engines?.node;if(!range||entry.dev)continue;
 // Every alternative's lower bound (>=X or ^X) must be satisfied; report when none accepts the runtime floor.
 const alternatives=range.split('||').map(s=>s.trim());
 const ok=alternatives.some(alt=>{const m=/(?:>=|\^|~)?\s*v?(\d+(?:\.\d+){0,2})/u.exec(alt);if(!m)return true;
  if(/^\^/u.test(alt)){const lo=parse(m[1]);const rt=parse(runtime);return !less(rt,lo)&&rt[0]===lo[0];}
  return !less(parse(runtime),parse(m[1]));});
 if(!ok)console.log(`${path}\t${range}`);
}
