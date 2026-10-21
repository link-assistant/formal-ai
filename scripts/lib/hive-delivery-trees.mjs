// Fresh bounded GraphQL Git trees, independently authenticated by canonical Git object bytes.
import assert from 'node:assert/strict';
import {createHash} from 'node:crypto';
const objectId=value=>assert.match(value??'',/^[a-f0-9]{40}$/u);
const modes=new Map([[16384,['040000','tree']],[33188,['100644','blob']],
 [33261,['100755','blob']],[40960,['120000','blob']],[57344,['160000','commit']]]);
export function verifiedGraphTree(value, expected) {
  objectId(expected);assert.equal(value?.oid,expected);
  assert.ok(Array.isArray(value.entries)&&value.entries.length<=100000,'complete bounded Git tree entries');
  const names=new Set(),encoded=[];
  const tree=value.entries.map(entry=>{
    assert.equal(typeof entry.name,'string');
    assert.ok(entry.name.length>0&&entry.name.length<=1024&&!/[\u0000-\u001f\u007f/]/u.test(entry.name)
      &&entry.name!=='.'&&entry.name!=='..','canonical Git tree name');
    assert.ok(!names.has(entry.name),'duplicate Git tree name');names.add(entry.name);
    assert.equal(typeof entry.nameRaw,'string');assert.ok(entry.nameRaw.length<=8192);
    const raw=Buffer.from(entry.nameRaw,'base64');
    assert.equal(raw.toString('base64'),entry.nameRaw,'canonical raw name encoding');
    assert.equal(new TextDecoder('utf-8',{fatal:true,ignoreBOM:true}).decode(raw),entry.name,'raw Git name identity');
    assert.ok(Number.isSafeInteger(entry.mode)&&modes.has(entry.mode),'supported Git tree mode');
    const [mode,type]=modes.get(entry.mode);assert.equal(entry.type,type);objectId(entry.oid);
    if(type==='blob'&&Object.hasOwn(entry,'size'))assert.ok(Number.isSafeInteger(entry.size)&&entry.size>=0,'Git blob size metadata');
    encoded.push({sort:Buffer.concat([raw,type==='tree'?Buffer.from('/'):Buffer.alloc(0)]),
      bytes:Buffer.concat([Buffer.from(entry.mode.toString(8)+' '),raw,Buffer.from([0]),Buffer.from(entry.oid,'hex')])});
    return {path:entry.name,mode,type,sha:entry.oid,...(type==='blob'&&Object.hasOwn(entry,'size')?{size:entry.size}:{})};
  });
  encoded.sort((left,right)=>Buffer.compare(left.sort,right.sort));
  const bytes=Buffer.concat(encoded.map(entry=>entry.bytes));
  assert.equal(createHash('sha1').update('tree '+bytes.length+'\0').update(bytes).digest('hex'),expected,
    'complete canonical Git tree identity');
  return {sha:expected,truncated:false,tree};
}
export function treeQuery(repository, ids) {
  assert.match(repository,/^[A-Za-z0-9_.-]+\/[A-Za-z0-9_.-]+$/u);
  assert.ok(Array.isArray(ids)&&ids.length>0&&ids.length<=4&&new Set(ids).size===ids.length,'one to four unique trees');
  ids.forEach(objectId);const [owner,name]=repository.split('/');
  const fields=ids.map((id,index)=>'tree'+index+':object(oid:'+JSON.stringify(id)
    +'){... on Tree{oid entries{name nameRaw mode oid type}}}').join(' ');
  return 'query{rateLimit{cost} repository(owner:'+JSON.stringify(owner)+',name:'+JSON.stringify(name)
    +'){nameWithOwner '+fields+'}}';
}
export function createTreeReader(repository, query) {
  const cache=new Map(),pending=[];let active=0,scheduled=false,points=0,queries=0,failure=null;
  const fail=error=>{failure??=error;for(const item of pending.splice(0))item.reject(failure);};
  const schedule=()=>{
    if(scheduled)return;scheduled=true;
    queueMicrotask(()=>{
      scheduled=false;
      for(let action=0;action<4&&active<4&&pending.length&&!failure;action++){
        const batch=pending.splice(0,4);active++;
        void (async()=>{
          try{
            assert.ok(++queries<=128&&points<128,'GraphQL point budget');
            const response=await query(treeQuery(repository,batch.map(item=>item.id)));
            assert.ok(response&&(!response.errors||response.errors.length===0),'complete GraphQL response');
            const cost=response.data?.rateLimit?.cost;
            assert.ok(Number.isSafeInteger(cost)&&cost>0,'observed GraphQL point cost');
            points+=cost;assert.ok(points<=128,'GraphQL point budget');
            assert.equal(cost,1,'fixed non-connection Git tree query cost');
            assert.equal(response.data.repository?.nameWithOwner,repository);
            const values=batch.map((item,index)=>verifiedGraphTree(response.data.repository['tree'+index],item.id));
            for(let index=0;index<batch.length;index++)batch[index].resolve(values[index]);
          }catch(error){for(const item of batch)item.reject(error);fail(error);}
          finally{active--;if(pending.length&&!failure)schedule();}
        })();
      }
    });
  };
  const readTree=id=>{
    objectId(id);if(failure)return Promise.reject(failure);
    if(cache.has(id))return cache.get(id);
    assert.ok(cache.size<512,'bounded unique Git tree observations');
    const result=new Promise((resolve,reject)=>pending.push({id,resolve,reject}));
    cache.set(id,result);schedule();return result;
  };
  return {readTree,statistics:()=>({points,queries})};
}
