import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import * as pure from '../../../js/logging/typed-codec.mjs';
import * as numbers from '../../../js/logging/json-numbers.mjs';
import * as server from '../../../js/server/structured-log.mjs';
import * as json from '../../../js/server/json.mjs';
test('server public codec and numeric facades expose identical shared function objects',()=>{
  for(const name of ['encodeTyped','decodeTyped','MAXIMUM_LOG_RECORD_BYTES','MAX_LOG_RECORD_BYTES'])assert.equal(server[name],pure[name]);
  for(const name of ['f64','f32','isFloat','renderFloat'])assert.equal(json[name],numbers[name]);
}
);
test('one shared FLOAT marker preserves serde numeric rendering and typed custody',()=>{
  for(const value of [1.0,-0.0,1e-7,1e16,Math.fround(0.1)]){
    for(const bits of [32,64]){
      const number=bits===32?json.f32(value):numbers.f64(value);
      assert.equal(numbers.isFloat(number),true);
      const encoded=pure.encodeTyped({
        number
      }
      ,{
        preserveJsonNumbers:true
      }
      );
      const decoded=server.decodeTyped(encoded,{
        preserveJsonNumbers:true
      }
      );
      assert.equal(json.isFloat(decoded.number),true);
      assert.equal(decoded.number.bits,bits);
      assert.equal(json.toCompactJson(decoded),json.toCompactJson({
        number
      }
      ));
    }
  }
  const value={
    integer:9007199254740993n
  }
  ;
  assert.deepEqual(pure.decodeTyped(pure.encodeTyped(value,{
    preserveJsonNumbers:true
  }
  ),{
    preserveJsonNumbers:true
  }
  ),value);
}
);
test('ordinary numeric/depth/canonical and default size refusals remain strict',()=>{
  for(const value of [NaN,Infinity,-0,Number.MAX_SAFE_INTEGER+1,json.f64(1)])assert.throws(()=>pure.encodeTyped(value));
  let deep=null;
  for(let i=0;
  i<66;
  i++)deep=[deep];
  assert.throws(()=>pure.encodeTyped(deep),/depth_limit/);
  assert.throws(()=>pure.decodeTyped(pure.encodeTyped({
    value:1
  }
  )+' '),/noncanonical|invalid/);
  const line=pure.encodeTyped({
    text:'x'.repeat(pure.MAXIMUM_LOG_RECORD_BYTES+1)
  }
  );
  assert.throws(()=>server.decodeTyped(line),/record_size_limit/);
  assert.deepEqual(pure.decodeTyped(line,{
    maximumRecordBytes:Buffer.byteLength(line)
  }
  ),{
    text:'x'.repeat(pure.MAXIMUM_LOG_RECORD_BYTES+1)
  }
  );
}
);
test('neutral implementations have no surface or host IO imports and orchestration imports the neutral codec',()=>{
  for(const name of ['typed-codec','json-numbers']){
    const source=fs.readFileSync(new URL('../../../js/logging/'+name+'.mjs',import.meta.url),'utf8');
    assert.ok(!/from ['"](?:node:|[^'"]*server\/)/u.test(source));
  }
  const source=fs.readFileSync(new URL('../../../js/agentic/crate/orchestration_runner.mjs',import.meta.url),'utf8');
  assert.ok(source.includes("from '../../logging/typed-codec.mjs'"));
  assert.ok(!source.includes("from '../../server/structured-log.mjs'"));
}
);
