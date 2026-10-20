// Diagnostic codec, redaction and transport contracts; no native runtime required.
import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import {createHash} from 'node:crypto';
import {test} from 'node:test';
import {
  encodeTyped,decodeTyped,createLogger,makeRedactor,convertLog,
} from '../../../js/server/structured-log.mjs';
import {parseLinks} from '../../../js/logging/lino-codec.mjs';
import {temporaryHome} from './support/structured-log-fixture.mjs';

const digest=text=>createHash('sha256').update(text).digest('hex');

test('typed Lino has golden bytes and preserves scalar/container distinctions',()=>{
  assert.equal(encodeTyped({b:'1',a:1}),
    '(formal-ai-log-v1 (object (field a (number 1)) (field b (string 1))))');
  const value={number:123,text:'123',nil:null,array:[],object:{},boolean:false,
    unicode:'héllo\n🙂\\n',ordered:[2,'1',true]};
  const line=encodeTyped(value);
  assert.equal(parseLinks(line).length,1);
  assert.equal(line.includes('\n'),false);
  assert.deepEqual(decodeTyped(line),value);
  assert.equal(encodeTyped({'é':1,z:2,a:3}),encodeTyped({a:3,z:2,'é':1}));
  assert.match(encodeTyped({z:1,'é':2}),/field z.*field %C3%A9/u);
});

test('typed decoder rejects malformed, duplicate, noncanonical and oversized input',()=>{
  for(const line of [
    '(formal-ai-log-v1 (object (field a (null)) (field a (null))))',
    '(formal-ai-log-v1 (number 01))','(formal-ai-log-v1 (string %GG))',
    '(formal-ai-log-v1 (null)) trailing','(formal-ai-log-v1 (null)',
  ])assert.throws(()=>decodeTyped(line));
  assert.throws(()=>decodeTyped('('.repeat(1024*1024+1)),/record_size_limit/u);
  for(const value of [NaN,Infinity,-0,2**53,undefined,1n,new Date()]){
    assert.throws(()=>encodeTyped(value));
  }
});

test('nested credentials and Error causes are redacted before digest, size or trace',()=>{
  const chunks=[],secret='multiline\nprivate-value';
  const logger=createLogger({level:'trace',env:{FORMAL_AI_API_KEY:secret,GH_TOKEN:'gh-private'},
    stderrWrite:bytes=>(chunks.push(Buffer.from(bytes)),bytes.length)});
  const error=new Error('failure '+secret,{cause:new Error('cause gh-private')});
  logger.log('error','failed',{Authorization:'Bearer unknown'},()=>({
    nested:{password:'field-password',api_key:'field-api',message:secret},error,
  }));
  const text=Buffer.concat(chunks).toString(),record=decodeTyped(text.trimEnd());
  for(const value of [secret,'gh-private','Bearer unknown','field-password','field-api']){
    assert.equal(text.includes(value),false);
  }
  const expected={nested:{password:'[REDACTED]',api_key:'[REDACTED]',message:'[REDACTED]'},
    error:{type:'Error',stack_omitted:true,message:'failure [REDACTED]',
      cause:{type:'Error',stack_omitted:true,message:'cause [REDACTED]'}}};
  assert.deepEqual(record.payload.content,expected);
  assert.equal(record.metadata.Authorization,'[REDACTED]');
  assert.equal(record.payload.digest,digest(encodeTyped(expected)));
  assert.equal(record.payload.bytes,Buffer.byteLength(encodeTyped(expected)));
});

test('disabled suppliers are lazy and silent never creates a directory',t=>{
  const directory=path.join(temporaryHome(t),'silent');let called=0;
  const logger=createLogger({level:'silent',directory,env:{},stderrWrite:()=>assert.fail()});
  assert.deepEqual(logger.log('debug','unused',{},()=>{called++;}),
    {emitted:false,reason:'disabled'});
  logger.close();assert.equal(called,0);assert.equal(fs.existsSync(directory),false);
  assert.throws(()=>createLogger({level:'verbose'}),/invalid_log_level/u);
});

test('stderr and PID file receive ordered identical bytes through short writes',t=>{
  const chunks=[],logger=createLogger({directory:temporaryHome(t),pid:42,env:{},
    stderrWrite:bytes=>{
      const size=Math.min(bytes.length,3);chunks.push(Buffer.from(bytes.subarray(0,size)));
      return size;
    }});
  logger.log('debug','begin',{value:'1'});logger.log('error','terminal',{reason:'failure'});
  logger.close();const text=Buffer.concat(chunks).toString();
  assert.equal(fs.readFileSync(logger.file,'utf8'),text);
  assert.equal(path.basename(logger.file),'serve-42.lino');
  assert.deepEqual(text.trimEnd().split('\n').map(decodeTyped).map(x=>x.sequence),[1,2]);
  assert.equal(fs.statSync(logger.file).mode&0o777,0o600);
});

test('sink failures remain fixed data and never disable a surviving sink',t=>{
  const home=temporaryHome(t),chunks=[];
  const open=createLogger({directory:path.join(home,'open'),env:{},
    stderrWrite:bytes=>(chunks.push(Buffer.from(bytes)),bytes.length),
    openFile:()=>{throw Error('private path');}});
  assert.equal(open.log('error','open').emitted,true);
  assert.deepEqual(open.failures,[{sink:'file',reason:'open_failed'}]);
  let writes=0;
  const zero=createLogger({directory:path.join(home,'zero'),env:{},
    stderrWrite:()=>{writes++;return 0;}});
  assert.deepEqual(zero.log('error','first').delivered,['file']);
  assert.deepEqual(zero.log('error','second').delivered,['file']);zero.close();
  assert.equal(writes,1);assert.deepEqual(zero.failures,[{sink:'stderr',reason:'write_failed'}]);
  let partialWrites=0;
  const partial=createLogger({directory:path.join(home,'partial'),env:{},
    stderrWrite:bytes=>bytes.length,fileWrite:(fd,bytes)=>{
      if(partialWrites++===0)return fs.writeSync(fd,bytes.subarray(0,9));
      throw Error('private disk error');
    },closeFile:fd=>{fs.closeSync(fd);throw Error('private close error');}});
  assert.deepEqual(partial.log('error','partial').delivered,['stderr']);partial.close();
  assert.throws(()=>decodeTyped(fs.readFileSync(partial.file,'utf8')));
  assert.deepEqual(partial.failures,[{sink:'file',reason:'write_failed'},
    {sink:'file',reason:'close_failed'}]);
});

test('accessors, unsupported keys and redacted key collisions fail without leakage',()=>{
  let reads=0,writes=0;
  const logger=createLogger({env:{},stderrWrite:()=>{writes++;}}),error=new Error('private');
  Object.defineProperty(error,'message',{get(){reads++;return 'private';}});
  assert.equal(logger.log('error','getter',{error}).reason,'serialization_failed');
  assert.equal(reads,0);assert.equal(writes,0);
  const symbol={};symbol[Symbol('secret')]='private';const array=[];array.extra='private';
  for(const metadata of [symbol,{array}]){
    assert.equal(logger.log('error','unsupported',metadata).reason,'serialization_failed');
  }
  assert.throws(()=>makeRedactor({GH_TOKEN:'private-key'})({'private-key':1,'[REDACTED]':2}),
    /redacted_key_collision/u);
  const counters={prompt_tokens:4,completion_tokens:5,total_tokens:9,token_count:9};
  assert.deepEqual(makeRedactor({})(counters),counters);
});

test('explicit legacy/Lino converters preserve supported values and reject truncation',()=>{
  const legacy='{"value":"123","nil":null,"empty":[],"token":"private"}\n{"value":123}\n';
  const lino=convertLog(legacy,{from:'jsonl',env:{}}),records=lino.trimEnd().split('\n').map(decodeTyped);
  assert.deepEqual(records,[{value:'123',nil:null,empty:[],token:'[REDACTED]'},{value:123}]);
  for(const to of ['json','pretty'])assert.deepEqual(JSON.parse(convertLog(lino,{to,env:{}})),records);
  assert.equal(convertLog(lino,{to:'jsonl',env:{}}),records.map(x=>JSON.stringify(x)+'\n').join(''));
  assert.throws(()=>convertLog(legacy.slice(0,-1),{from:'jsonl'}),/unterminated_log/u);
  assert.throws(()=>convertLog(lino.slice(0,-2)));
  assert.throws(()=>convertLog('',{to:'yaml'}),/invalid_format/u);
});

test('oversized trace content is explicit while digest/size remain post-redaction',()=>{
  const chunks=[],logger=createLogger({level:'trace',env:{},
    stderrWrite:bytes=>(chunks.push(Buffer.from(bytes)),bytes.length)});
  const payload='x'.repeat(1024*1024),canonical=encodeTyped(payload);
  assert.equal(logger.log('debug','large',{},()=>payload).emitted,true);
  const record=decodeTyped(Buffer.concat(chunks).toString().trimEnd());
  assert.equal(record.payload.content_omitted,'record_size_limit');
  assert.equal(Object.hasOwn(record.payload,'content'),false);
  assert.equal(record.payload.digest,digest(canonical));
  assert.equal(record.payload.bytes,Buffer.byteLength(canonical));
  const count=chunks.length;
  assert.equal(logger.log('debug','large-metadata',{payload}).emitted,true);
  assert.equal(chunks.length,count+1);
  const diagnostic=decodeTyped(chunks.at(-1).toString().trimEnd());
  assert.equal(diagnostic.metadata.payload.bytes,Buffer.byteLength(payload));
  assert.equal(diagnostic.metadata.payload.sha256,digest(payload));
  assert.equal(diagnostic.metadata.payload.preview_truncated,true);
});
