/** Split complete contiguous intent families while preserving every record byte and order. */
export function splitResponseFamilies(source,maximumLines) {
  if(typeof source!=='string'||!source.endsWith('\n')||!Number.isSafeInteger(maximumLines)||maximumLines<2)throw Error('complete source and positive line budget required');
  const lines=source.split('\n');lines.pop();
  const root=lines.findIndex(line=>line.trim()&&!line.trimStart().startsWith('#'));
  if(root<0||!/^multilingual[_-]responses\s*$/.test(lines[root]))throw Error('one response root required');
  const header=lines.slice(0,root+1).join('\n')+'\n';
  const records=[];let current=null;
  for(const line of lines.slice(root+1)){
    if(/^  response\s+\S+\s*$/.test(line)){
      if(current)records.push(current);current={lines:[line]};
    }else{
      if(!current||line.trim()&&!/^    /.test(line))throw Error('unknown root child or partial response');
      current.lines.push(line);
    }
  }
  if(current)records.push(current);
  if(!records.length)throw Error('empty response family');
  const ids=new Set(),keys=new Set(),families=[];
  for(const record of records){
    const id=record.lines[0].trim().split(/\s+/)[1];
    const intents=record.lines.filter(l=>/^    intent\s+\S+\s*$/.test(l));
    const languages=record.lines.filter(l=>/^    language\s+\S+\s*$/.test(l));
    if(intents.length!==1||languages.length!==1||!record.lines.some(l=>/^    text\s/.test(l)))throw Error('incomplete response record');
    const intent=intents[0].trim().split(/\s+/)[1],language=languages[0].trim().split(/\s+/)[1],key=intent+'\0'+language;
    if(ids.has(id)||keys.has(key))throw Error('ambiguous response id or lookup key');ids.add(id);keys.add(key);
    record.intent=intent;record.bytes=record.lines.join('\n')+'\n';
    if(families.at(-1)?.intent===intent)families.at(-1).records.push(record);
    else{if(families.some(f=>f.intent===intent))throw Error('noncontiguous intent family');families.push({intent,records:[record]})}
  }
  const headerLines=header.split('\n').length-1,totalLines=lines.length;
  const shardCount=Math.ceil(totalLines/maximumLines),targetFamilies=Math.ceil(families.length/shardCount);
  const shards=[];let pending=[];
  const emit=()=>{if(pending.length){shards.push(header+pending.flatMap(f=>f.records.map(r=>r.bytes)).join(''));pending=[]}};
  for(const family of families){
    const familyLines=family.records.reduce((n,r)=>n+r.lines.length,0);
    if(headerLines+familyLines>maximumLines)throw Error('complete family exceeds unchanged line budget');
    const used=headerLines+pending.reduce((n,f)=>n+f.records.reduce((m,r)=>m+r.lines.length,0),0);
    if(pending.length>=targetFamilies||used+familyLines>maximumLines)emit();pending.push(family);
  }
  emit();
  if(shards.some(s=>s.split('\n').length-1>maximumLines))throw Error('line budget exceeded');
  if(shards.map(s=>s.slice(header.length)).join('')!==source.slice(header.length))throw Error('record bytes or order changed');
  return {header,shards,records:records.length,families:families.length,maximumLines};
}
