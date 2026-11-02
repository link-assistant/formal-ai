// Exact loaded-source custody; diagnostics contain identities, never source bytes.
import path from 'node:path';
import {fileURLToPath,pathToFileURL} from 'node:url';
export function requirePinnedLoadedSources(expectedSources,actualSources,context){
 const key=entry=>entry.url+'\0'+entry.format+'\0'+entry.blake3;
 const actualKeys=new Set(actualSources.map(key)),missing=new Map();
 for(const entry of expectedSources){
  const filename=fileURLToPath(entry.url),relative=path.relative(context.originalRuntime,filename);
  const owned=context.bootstrapArtifacts.some(artifact=>artifact.file===relative);
  const address=owned?pathToFileURL(path.join(context.freshRuntime,relative)).href:entry.url;
  if(actualKeys.has(key({...entry,url:address})))continue;
  const source=path.relative(context.repository,filename);
  const safeSource=owned?'bootstrap/'+relative:source!==''&&!path.isAbsolute(source)
   &&source!=='..'&&!source.startsWith('..'+path.sep)?source.split(path.sep).join('/'):'external-source';
  missing.set(key(entry),{source:safeSource,format:/^[a-z-]{1,40}$/u.test(entry.format)?entry.format:'unknown',
   blake3:/^[0-9a-f]{64}$/u.test(entry.blake3)?entry.blake3:'invalid'});
 }
 if(missing.size)throw Object.assign(Error('PinnedReplayLoadedSourceMismatch'),{
  missingRuntimeInputCount:missing.size,missingRuntimeInputs:[...missing.values()].slice(0,16)});
}
