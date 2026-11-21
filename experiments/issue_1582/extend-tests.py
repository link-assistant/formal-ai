"""Supplied regression tests and focused investigation, zero authorship credit."""
from pathlib import Path
p=Path('rust/tests/web/issue-1582-defaults.test.mjs')
s=p.read_text()+'''
test('serving worker performs real HTTP reads and preserves explicit network denies',{timeout:20000},async t=>{
  const {createServer}=await import('node:http');
  const {createReadOnlyFetch}=await import('../../../js/runtime/read-access.mjs');
  const requests=[];
  const source=createServer((request,response)=>{requests.push(request.method);response.end('captured-source');});
  await new Promise(resolve=>source.listen(0,'127.0.0.1',resolve));
  t.after(()=>new Promise(resolve=>source.close(resolve)));
  const url='http://127.0.0.1:'+source.address().port+'/source';
  const networkFetch=createReadOnlyFetch({env:{}});
  const host=new WorkerHost({}, {networkFetch});
  await host.boot();
  const text=await host.run('fetch(__url).then(response=>response.text())',{__url:url});
  assert.equal(text,'captured-source');
  await assert.rejects(()=>networkFetch(url,{method:'POST'}),/read_network_method/u);
  await assert.rejects(()=>createReadOnlyFetch({env:{FORMAL_AI_NETWORK:'denied'}})(url),/read_network_denied/u);
  await assert.rejects(()=>createReadOnlyFetch({env:{FORMAL_AI_OFFLINE:'1'}})(url),/read_network_denied/u);
  assert.deepEqual(requests,['GET']);
});
'''
p.write_text(s)
