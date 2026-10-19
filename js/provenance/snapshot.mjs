import {snapshotTreeShape, snapshotTreePair, snapshotProofRecipe, snapshotSourceLanguage, snapshotVerificationStep} from './portable/snapshot.mjs';
import {wellFormedText} from '../logging/text-lines.mjs';
// Source snapshot integrity is distinct from engine authorship or replay.
import {blake3Hex} from '../crypto/blake3.mjs';
const bytes=x=>Buffer.from(x);
const order=(a,b)=>Buffer.compare(bytes(a),bytes(b));
const hash=x=>blake3Hex(bytes(x));
const digest=/^[0-9a-f]{64}$/u;
export function sourceLanguage(file, extensions){
 return snapshotSourceLanguage(file,file.split('/').at(-1),extensions.map(row=>row[0]),extensions.map(row=>row[1]));
}
export function languageExtensions(seed){
 const rows=[...seed.matchAll(/^  extension\n    suffix (\S+)\n    language (\S+)$/gmu)].map(m=>[m[1],m[2]]);
 if(!rows.length)throw Error('language_table_required');
 return rows.sort((a,b)=>b[0].length-a[0].length||order(a[0],b[0]));
}
function merkleNodes(leaves){
 const nodes=leaves.slice(),shape=snapshotTreeShape(leaves.length);
 for(let node=leaves.length;node<shape.total;node++){
  const pair=snapshotTreePair(leaves.length,node);
  if(pair.$!=='pair')throw Error('invalid_snapshot_recipe');
  nodes.push(hash(Buffer.concat([bytes('formal-ai-source-node-v1\0'),Buffer.from(nodes[pair.left],'hex'),Buffer.from(nodes[pair.right],'hex')])));
 }
 return nodes;
}
export function merkleRoot(leaves){
 if(!Array.isArray(leaves)||leaves.some(x=>typeof x!=='string'||!digest.test(x)))throw Error('invalid_leaves');
 if(!leaves.length)return hash('formal-ai-source-empty-v1\0');
 return merkleNodes(leaves).at(-1);
}
export function sourceSnapshot(files,extensions){
 if(!files||typeof files[Symbol.iterator]!=='function'||!Array.isArray(extensions)||extensions.some(row=>!Array.isArray(row)||row.length!==2||row.some(x=>typeof x!=='string'||!x||!wellFormedText(x))))throw Error('snapshot_inputs_required');
 const seen=new Set(),groups=new Map();
 for(const entry of files){const p=entry?.path;
 if(typeof p!=='string'||!wellFormedText(p)||!p||p.includes('\0')||p.includes('\\')||p.startsWith('/')||p.split('/').some(s=>!s||s==='.'||s==='..')||seen.has(p)||!(entry.content instanceof Uint8Array))throw Error('invalid_source_file');
 seen.add(p);const language=sourceLanguage(p,extensions);
 const group=groups.get(language)??[];group.push({path:p,leaf:hash(Buffer.concat([bytes(p),Buffer.from([0]),bytes(entry.content)]))});groups.set(language,group);
 }
 const languages=[...groups].sort((a,b)=>order(a[0],b[0])).map(([language,group])=>({language,file_count:group.length,root:merkleRoot(group.sort((a,b)=>order(a.path,b.path)).map(x=>x.leaf))}));
 const total=merkleRoot(languages.map(x=>hash(Buffer.concat([bytes('formal-ai-language-v1\0'+x.language+'\0'+x.file_count+'\0'),Buffer.from(x.root,'hex')]))));
 return {schema:'formal-ai-source-snapshot-v1',languages,total};
}
export function appendSourceSnapshot(session,files,extensions){return session.append('source_snapshot',sourceSnapshot(files,extensions));}

// Proofs authenticate a leaf to a root; path/content binding is checked separately.
export function merkleProof(leaves,index){
 merkleRoot(leaves);if(!Number.isSafeInteger(index)||index<0||index>=leaves.length)throw Error('invalid_leaf_index');
 const nodes=merkleNodes(leaves);
 return {index,count:leaves.length,siblings:snapshotProofRecipe(leaves.length,index).map(node=>nodes[node])};
}
export function verifyMerkleProof(leaf,proof,root){
 if(!digest.test(leaf)||!digest.test(root)||!proof||!Number.isSafeInteger(proof.index)||!Number.isSafeInteger(proof.count)||!Array.isArray(proof.siblings))return false;
 let value=leaf,index=proof.index,count=proof.count,cursor=0;
 while(true){
  const sibling=proof.siblings[cursor];
  const decision=snapshotVerificationStep(count,index,cursor,proof.siblings.length,sibling===value,value===root);
  if(decision.$==='verified')return true;
  if(decision.$==='invalid'||typeof sibling!=='string'||!digest.test(sibling))return false;
  value=merkleRoot(decision.leftSibling?[sibling,value]:[value,sibling]);
  index=decision.nextIndex;count=decision.nextCount;cursor=decision.nextCursor;
 }
}
