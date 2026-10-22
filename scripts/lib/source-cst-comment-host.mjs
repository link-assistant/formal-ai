// Synchronous installed seed context for the existing documentary formalizer in standalone tools.
import fs from 'node:fs';
import path from 'node:path';
import {fileURLToPath} from 'node:url';
import {hasHost,host,installHost} from '../../js/agentic/host.mjs';
import {parseLino} from '../../js/server/lino.mjs';
const installedRoot=path.resolve(path.dirname(fileURLToPath(import.meta.url)),'../..');
/** No worker or inference producer is invented; the actual maintained seed parser reads installed data. */
export function withSourceCstCommentContext(action){
 const previous=hasHost()?host():null;
 if(previous!==null)return action();
 const readText=relative=>{
  if(typeof relative!=='string'||!relative.startsWith('data/')||relative.split('/').some(part=>!part||part==='.'||part==='..'))throw Error('InstalledCommentSeedPathRequired');
  const file=path.join(installedRoot,relative),metadata=fs.lstatSync(file);
  if(!metadata.isFile()||metadata.isSymbolicLink()||metadata.size>4*1024*1024||fs.realpathSync(file)!==file)throw Error('InstalledCommentSeedIdentityRequired');
  return fs.readFileSync(file,'utf8');
 };
 installHost({readText,parseLino,realm:{}});
 try{
  const result=action();
  if(result&&typeof result.then==='function')throw Error('SynchronousCommentContextRequired');
  return result;
 }finally{installHost(previous);}
}
