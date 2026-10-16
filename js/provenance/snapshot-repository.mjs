// Snapshot tracked working bytes with explicit resource and inventory custody bounds.
import fs from 'node:fs';
import path from 'node:path';
import {spawnSync} from 'node:child_process';
import {blake3Hex} from '../crypto/blake3.mjs';
import {languageExtensions,sourceSnapshot,merkleRoot} from './snapshot.mjs';
export const snapshotLimits=Object.freeze({maxFiles:50000,maxFileBytes:4*1024*1024,maxTotalBytes:32*1024*1024});
function boundedRead(file,expected){
 const fd=fs.openSync(file,fs.constants.O_RDONLY|fs.constants.O_NOFOLLOW);
 try{const stat=fs.fstatSync(fd);if(!stat.isFile()||stat.dev!==expected.dev||stat.ino!==expected.ino||stat.size!==expected.size||stat.mtimeMs!==expected.mtimeMs)throw Error('snapshot_source_drift');
 const bytes=Buffer.alloc(stat.size);let offset=0;while(offset<bytes.length){const count=fs.readSync(fd,bytes,offset,bytes.length-offset,offset);if(!count)throw Error('snapshot_source_short_read');offset+=count;}
 const after=fs.fstatSync(fd);if(after.size!==stat.size||after.mtimeMs!==stat.mtimeMs)throw Error('snapshot_source_drift');return bytes;
 }finally{fs.closeSync(fd);}
}
export function snapshotRepository(directory,{maxFiles=snapshotLimits.maxFiles,maxFileBytes=snapshotLimits.maxFileBytes,maxTotalBytes=snapshotLimits.maxTotalBytes,readFile=boundedRead}={}){
 for(const limit of [maxFiles,maxFileBytes,maxTotalBytes])if(!Number.isSafeInteger(limit)||limit<0)throw Error('invalid_snapshot_budget');
 if(typeof readFile!=='function')throw Error('snapshot_reader_required');
 const root=fs.realpathSync(directory);
 const top=spawnSync('git',['rev-parse','--show-toplevel'],{cwd:root,encoding:'utf8'});
 if(top.status!==0||fs.realpathSync(top.stdout.trim())!==root)throw Error('repository_root_required');
 const result=spawnSync('git',['ls-files','--cached','--stage','-z'],{cwd:root,encoding:'buffer',maxBuffer:4*1024*1024});
 if(result.error||result.status!==0)throw Error('tracked_source_inventory_failed_or_budget');
 const text=result.stdout.toString('utf8');if(!Buffer.from(text).equals(result.stdout))throw Error('non_utf8_path');
 const rows=text.split('\0');if(rows.pop()!=='')throw Error('incomplete_inventory');if(rows.length>maxFiles)throw Error('snapshot_file_count_budget');
 const seen=new Set();let totalBytes=0;
 const entries=rows.map(row=>{const match=/^([0-7]{6}) ([0-9a-f]{40}|[0-9a-f]{64}) ([0-3])\t([\s\S]+)$/u.exec(row);if(!match||match[3]!=='0')throw Error('unresolved_source_inventory');
 const [,mode,blob,,p]=match;if(!['100644','100755'].includes(mode))throw Error('source_symlink_or_gitlink_refused');
 if(!p||p.includes('\\')||path.isAbsolute(p)||p.split('/').some(s=>!s||s==='.'||s==='..')||seen.has(p))throw Error('source_path_escape_or_duplicate');seen.add(p);
 let full=root;for(const part of p.split('/')){full=path.join(full,part);if(fs.lstatSync(full).isSymbolicLink())throw Error('source_symlink_refused');}
 const stat=fs.statSync(full);if(!stat.isFile())throw Error('source_not_regular');if(!Number.isSafeInteger(stat.size)||stat.size>maxFileBytes)throw Error('snapshot_individual_byte_budget');totalBytes+=stat.size;if(!Number.isSafeInteger(totalBytes)||totalBytes>maxTotalBytes)throw Error('snapshot_total_byte_budget');
 return {path:p,mode,blob,full,stat};});
 // All inventory/size refusals precede the first payload read. The iterator retains one file only.
 function* observedFiles(){for(const entry of entries){const content=readFile(entry.full,entry.stat);if(!(content instanceof Uint8Array)||content.length!==entry.stat.size||content.length>maxFileBytes)throw Error('snapshot_reader_size_drift');yield {path:entry.path,content};}}
 const seed=fs.readFileSync(new URL('../../data/seed/page-formalization-rules.lino',import.meta.url),'utf8');
 const snapshot=sourceSnapshot(observedFiles(),languageExtensions(seed));
 const inventoryRoot=merkleRoot(entries.map(entry=>blake3Hex(Buffer.from(entry.path+'\0'+entry.mode+'\0'+entry.blob))));
 return {...snapshot,inventory:{kind:'git-index-tracked-working-bytes',file_count:entries.length,total_bytes:totalBytes,root:inventoryRoot,exclusions:[],symlinks:'refused',gitlinks:'refused',unmerged:'refused'},limits:{maxFiles,maxFileBytes,maxTotalBytes}};
}
