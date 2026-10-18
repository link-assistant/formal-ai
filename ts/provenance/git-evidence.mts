// Actual bounded Git objects; these observations do not establish autonomous authorship.
import {spawnSync} from 'node:child_process';
import {blake3Hex} from '../crypto/blake3.mjs';
const objectIdentity=value=>typeof value==='string'&&/^(?:[a-f0-9]{40}|[a-f0-9]{64})$/u.test(value);
export function repositoryGit(directory,args,options={input:undefined}){
 const input=options===undefined?undefined:options.input;
 const result=spawnSync('git',['--no-replace-objects','-c','core.fsmonitor=false',...args],{
  cwd:directory,input:typeof input==='string'?Buffer.from(input,'utf8'):input,encoding:null,timeout:15000,maxBuffer:8*1024*1024,
  env:{...process.env,GIT_NO_REPLACE_OBJECTS:'1',GIT_OPTIONAL_LOCKS:'0'}});
 if(result.error||result.status!==0)throw Error('repository_git_observation_failed');
 return result.stdout;
}
export function repositoryText(bytes){
 const text=bytes.toString('utf8');
 if(!Buffer.from(text,'utf8').equals(bytes))throw Error('repository_non_utf8_identity');
 return text;
}
export function repositoryPath(value){
 if(typeof value!=='string'||!value||value.startsWith('/')||value.includes('\\')
  ||/[\x00-\x1f\x7f]/u.test(value)||value.split('/').some(part=>!part||part==='.'||part==='..'))throw Error('repository_unsafe_path');
 return value;
}
export function repositoryObjectIdentity(value){if(!objectIdentity(value))throw Error('repository_object_id_required');return value;}
export function commitPrefix(record){
 if(!record||typeof record.sessionId!=='string'||!record.sessionId||!Number.isSafeInteger(record.index)||record.index<0
  ||typeof record.hash!=='string'||!/^[a-f0-9]{64}$/u.test(record.hash)
  ||/[\s@]/u.test(record.sessionId))throw Error('repository_prefix_required');
 return record.sessionId+'@'+record.hash;
}
function commitChanges(directory,commit){
 const chunks=repositoryText(repositoryGit(directory,['diff-tree','--root','--no-commit-id','--raw','-r','-z','--no-renames','--no-abbrev',commit])).split('\0');
 if(chunks.pop()!=='')throw Error('repository_incomplete_diff');
 const changes=[];
 for(let index=0;index<chunks.length;index+=2){
  const matched=/^:([0-7]{6}) ([0-7]{6}) ([a-f0-9]{40}|[a-f0-9]{64}) ([a-f0-9]{40}|[a-f0-9]{64}) ([AMDMT])$/u.exec(chunks[index]);
  if(!matched||chunks[index+1]===undefined)throw Error('repository_unsupported_diff');
  const [,oldMode,newMode,oldObject,newObject,status]=matched;
  changes.push({path:repositoryPath(chunks[index+1]),oldMode,newMode,
   oldBlob:/^0+$/u.test(oldObject)?null:oldObject,newBlob:/^0+$/u.test(newObject)?null:newObject,status});
 }
 const statistics=repositoryText(repositoryGit(directory,['diff-tree','--root','--no-commit-id','--numstat','-r','-z','--no-renames','--no-ext-diff','--no-textconv',commit])).split('\0');
 if(statistics.pop()!=='')throw Error('repository_incomplete_numstat');
 const byPath=new Map(changes.map(change=>[change.path,change]));
 for(const row of statistics){
  const matched=/^(\d+|-)\t(\d+|-)\t([\s\S]+)$/u.exec(row);
  if(!matched||!byPath.has(matched[3]))throw Error('repository_numstat_binding');
  const change=byPath.get(matched[3]);
  if(Object.hasOwn(change,'added'))throw Error('repository_duplicate_numstat');
  change.added=matched[1]==='-'?null:Number(matched[1]);
  change.deleted=matched[2]==='-'?null:Number(matched[2]);
 }
 if(changes.some(change=>!Object.hasOwn(change,'added')))throw Error('repository_missing_numstat');
 return changes;
}
export function observeRepositoryCommit(directory,commit,metadata){
 if(metadata===null||typeof metadata!=='object'||Array.isArray(metadata))throw Error('repository_causal_binding_required');
 const {messageOut,editHashes,precommitAnchor}=metadata;
 repositoryObjectIdentity(commit);
 if(typeof messageOut!=='string'||!/^[a-f0-9]{64}$/u.test(messageOut)
  ||!Array.isArray(editHashes)||editHashes.length>10000||new Set(editHashes).size!==editHashes.length
  ||editHashes.some(value=>typeof value!=='string'||!/^[a-f0-9]{64}$/u.test(value)))throw Error('repository_causal_binding_required');
 commitPrefix(precommitAnchor);
 const fields=repositoryText(repositoryGit(directory,['show','--no-patch','--no-show-signature','--format=%H%x00%T%x00%P%x00%B',commit])).split('\0');
 if(fields.length!==4||fields[0]!==commit||!objectIdentity(fields[1]))throw Error('repository_commit_identity');
 const parents=fields[2]?fields[2].split(' '):[];
 if(parents.length>1||parents.some(parent=>!objectIdentity(parent)))throw Error('repository_merge_commit_requires_parent_policy');
 const trailers=repositoryText(repositoryGit(directory,['interpret-trailers','--parse'],{input:fields[3]})).split('\n')
  .filter(line=>/^Formal-AI-Chain:/iu.test(line)).map(line=>line.slice(line.indexOf(':')+1).trim());
 return {schema:'formal-ai-observed-git-commit-v1',commit,tree:fields[1],parents,
  messageOut,editHashes:[...editHashes],precommitAnchor:{sessionId:precommitAnchor.sessionId,hash:precommitAnchor.hash,index:precommitAnchor.index},
  trailers,changes:commitChanges(directory,commit)};
}
export function repositoryBlob(directory,object){
 if(object===null)return null;
 return repositoryGit(directory,['cat-file','blob',repositoryObjectIdentity(object)]);
}
export function repositoryBlobMatches(directory,object,observation){
 if(object===null)return observation===null;
 const field=object.length===40?'gitBlobSha1':'gitBlobSha256';
 if(!observation||observation[field]!==object||!Number.isSafeInteger(observation.size)||observation.size<0)return false;
 const bytes=repositoryBlob(directory,object);
 return bytes.length===observation.size&&blake3Hex(bytes)===observation.blake3;
}
