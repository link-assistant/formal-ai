/** Split complete ordered meaning records without rewriting any record byte. */
export function splitMeaningRecords(source,maximumLines) {
  if(typeof source!=='string'||!source.endsWith('\n')||!Number.isSafeInteger(maximumLines)||maximumLines<2)
    throw Error('complete source and positive line budget required');
  const lines=source.split('\n');lines.pop();
  const root=lines.findIndex(line=>line.trim()&&!line.trimStart().startsWith('#'));
  if(root<0||lines[root]!=='meanings')throw Error('one meanings root required');
  const prefix=lines.slice(0,root+1),records=[];let current=null;
  for(const line of lines.slice(root+1)){
    const declaration=/^  ([A-Za-z_][A-Za-z_0-9-]*)\s*$/u.exec(line);
    if(declaration){
      if(current)records.push(current);
      current={id:declaration[1],lines:[line]};
    }else if(!line.trim()||line.trimStart().startsWith('#')){
      (current?.lines??prefix).push(line);
    }else{
      if(!current||!line.startsWith('    '))throw Error('unknown root child or partial meaning');
      current.lines.push(line);
    }
  }
  if(current)records.push(current);
  if(!records.length)throw Error('empty meanings');
  const ids=new Set();
  for(const record of records){
    if(ids.has(record.id))throw Error('ambiguous meaning identity');ids.add(record.id);
    if(!record.lines.slice(1).some(line=>line.startsWith('    ')&&line.trim()&&!line.trimStart().startsWith('#')))
      throw Error('empty meaning record');
  }
  const header=prefix.join('\n')+'\n',shards=[];
  const targetRecords=Math.ceil(records.length/Math.ceil(lines.length/maximumLines));
  let pending=[],used=prefix.length;
  const emit=()=>{
    if(pending.length)shards.push(header+pending.map(record=>record.lines.join('\n')+'\n').join(''));
    pending=[];used=prefix.length;
  };
  for(const record of records){
    if(prefix.length+record.lines.length>maximumLines)throw Error('complete meaning exceeds unchanged line budget');
    if(pending.length>=targetRecords||used+record.lines.length>maximumLines)emit();
    pending.push(record);used+=record.lines.length;
  }
  emit();
  if(shards.some(shard=>shard.split('\n').length-1>maximumLines))throw Error('line budget exceeded');
  if(shards.map(shard=>shard.slice(header.length)).join('')!==source.slice(header.length))
    throw Error('meaning record bytes or order changed');
  return {header,shards,recordIds:records.map(record=>record.id),maximumLines};
}
