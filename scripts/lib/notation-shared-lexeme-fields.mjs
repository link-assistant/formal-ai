import assert from 'node:assert/strict';
const depth=line=>/^ */u.exec(line)[0].length;
function scopes(lines){
 const result=[];
 for(let i=0;i<lines.length;i++){
  if(depth(lines[i])!==2||!lines[i].trim()||lines[i].trim().startsWith('#'))continue;
  let end=i+1;while(end<lines.length&&(!lines[end].trim()||depth(lines[end])>2))end++;
  result.push({start:i,end});i=end-1;
 }
 return result;
}
export function factorSharedLexemeFields(source){
 assert.ok(source.endsWith('\n'),'terminal LF required');
 if(source.includes('shared-lexeme-fields lexical')||source.includes('use-lexeme-fields lexical')){expandSharedLexemeFields(source);return {text:source,converted:0};}
 const lines=source.split('\n'),replacements=[];let converted=0;
 for(const scope of scopes(lines)){
  const surfaces=[];
  for(let i=scope.start+1;i<scope.end;i++){
   if(lines[i]!=='      surface')continue;
   let end=i+1;while(end<scope.end&&depth(lines[end])>6)end++;
   const children=lines.slice(i+1,end);
   if(children.length<2||!/^        text /u.test(children[0]))continue;
   const suffix=children.slice(1);
   if(!suffix.every(line=>/^        [a-z][a-z_-]* [^#]+$/u.test(line)))continue;
   surfaces.push({start:i+2,end,fields:suffix});
  }
  const lexemes=lines.slice(scope.start+1,scope.end).filter(line=>/^    lexeme /u.test(line));
  if(lexemes.length<2||surfaces.length!==lexemes.length)continue;
  const fields=surfaces[0].fields;
  if(!surfaces.every(surface=>JSON.stringify(surface.fields)===JSON.stringify(fields)))continue;
  const keys=fields.map(line=>line.trim().split(' ')[0]);
  if(new Set(keys).size!==keys.length||keys.some(key=>!['part_of_speech','grammatical_number'].includes(key)))continue;
  replacements.push({start:scope.start+1,end:scope.start+1,lines:['    shared-lexeme-fields lexical',...fields.map(line=>line.slice(2))]});
  for(const surface of surfaces)replacements.push({...surface,lines:['        use-lexeme-fields lexical']});
  converted++;
 }
 for(const replacement of replacements.sort((a,b)=>b.start-a.start))lines.splice(replacement.start,replacement.end-replacement.start,...replacement.lines);
 const text=lines.join('\n');assert.equal(expandSharedLexemeFields(text),source,'exact lossless inverse');return {text,converted};
}
export function expandSharedLexemeFields(source){
 const lines=source.split('\n'),replacements=[];
 for(const scope of scopes(lines)){
  const declarations=[];
  for(let i=scope.start+1;i<scope.end;i++)if(lines[i]==='    shared-lexeme-fields lexical')declarations.push(i);
  if(!declarations.length){assert.ok(!lines.slice(scope.start,scope.end).some(line=>line.includes('use-lexeme-fields lexical')),'unresolved reference');continue;}
  assert.equal(declarations.length,1,'duplicate shared declaration');const start=declarations[0];
  let end=start+1;while(end<scope.end&&depth(lines[end])>4)end++;
  const fields=lines.slice(start+1,end);assert.ok(fields.length&&fields.every(line=>/^      [a-z][a-z_-]* [^#]+$/u.test(line)),'non-leaf shared fields refused');
  const keys=fields.map(line=>line.trim().split(' ')[0]);assert.equal(new Set(keys).size,keys.length,'duplicate field');
  assert.ok(!keys.includes('text')&&!keys.includes('use-lexeme-fields lexical'),'recursive or text shared field refused');
  replacements.push({start,end,lines:[]});let uses=0;
  for(let i=scope.start+1;i<scope.end;i++)if(lines[i].includes('use-lexeme-fields lexical')){
   assert.equal(lines[i],'        use-lexeme-fields lexical','invalid reference scope');
   let surface=i-1;while(surface>scope.start&&depth(lines[surface])>6)surface--;
   assert.equal(lines[surface],'      surface','reference requires surface');
   let surfaceEnd=i+1;while(surfaceEnd<scope.end&&depth(lines[surfaceEnd])>6)surfaceEnd++;
   const siblings=lines.slice(surface+1,surfaceEnd).filter(line=>line!=='        use-lexeme-fields lexical').map(line=>line.trim().split(' ')[0]);
   assert.ok(keys.every(key=>!siblings.includes(key)),'field collision');
   replacements.push({start:i,end:i+1,lines:fields.map(line=>'  '+line)});uses++;
  }
  assert.ok(uses>=2,'shared fields require multiple references');
 }
 for(const replacement of replacements.sort((a,b)=>b.start-a.start))lines.splice(replacement.start,replacement.end-replacement.start,...replacement.lines);
 return lines.join('\n');
}
