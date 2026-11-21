#!/usr/bin/env node
// Finite investigation only. No runtime, native or authorship qualification.
const started=performance.now();
const record=stage=>console.error(JSON.stringify({stage,seconds:(performance.now()-started)/1000,memory:process.memoryUsage()}));
const deadline=setTimeout(()=>{record('bounded probe deadline');process.exit(1);},45000);
const heartbeat=setInterval(()=>record('pending bootstrap'),5000);heartbeat.unref();
try{
  process.env.FORMAL_AI_OFFLINE='1';
  record('before module import');
  const {WorkerHost}=await import('../../js/server/worker-host.mjs');
  record('before worker boot');
  const worker=new WorkerHost();await worker.boot();
  record('after worker boot');
  const {installNodeHost}=await import('../../js/agentic/node-host.mjs');
  record('before host installation');await installNodeHost(worker);
  record('after host installation');
  const {forGeneration,baselineVersionSet}=await import('../../js/agentic/crate/version_resolution.mjs');
  const assert=(await import('node:assert/strict')).default;
  record('before offline version resolution');const actual=forGeneration();record('after resolution before baseline');const expected=baselineVersionSet();record('after baseline before comparison');assert.equal(JSON.stringify(actual),JSON.stringify(expected));
  record('after offline version resolution');
}finally{clearTimeout(deadline);clearInterval(heartbeat);}
