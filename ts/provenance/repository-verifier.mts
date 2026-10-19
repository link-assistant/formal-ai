// Authenticated change attribution is separate from pinned replay and autonomous authorship.
import {provenancePrimitives} from './session.mjs';
import {verifyInstallationSeal} from './installation-key.mjs';
import {encodeTyped,decodeTyped} from '../logging/typed-codec.mjs';
import {repositoryGit,repositoryText,repositoryObjectIdentity,repositoryPath,commitPrefix,
 observeRepositoryCommit,repositoryBlobMatches} from './git-evidence.mjs';
const primitives=provenancePrimitives({encodeTyped,decodeTyped});
const same=(left,right)=>encodeTyped(left)===encodeTyped(right);
function causalPath(directory,change,edits,recordsByHash,anchorIndex){
 if(!['000000','100644','100755'].includes(change.oldMode)||!['000000','100644','100755'].includes(change.newMode))return 'unsupported-file-mode';
 if(change.oldMode!==change.newMode&&change.oldMode!=='000000'&&change.newMode!=='000000')return 'unobserved-file-mode-change';
 if(!edits.length)return 'missing-causal-edit';
 let previous=edits[0].payload.preimage;
 if(!repositoryBlobMatches(directory,change.oldBlob,previous))return 'preimage-mismatch';
 for(const edit of edits){
  const output=recordsByHash.get(edit.payload.messageOut);
  if(edit.index>anchorIndex||!output||output.kind!=='message_out'||output.index>=edit.index)return 'edit-message-binding';
  if(!same(previous,edit.payload.preimage))return 'edit-state-discontinuity';
  previous=edit.payload.postimage;
 }
 return repositoryBlobMatches(directory,change.newBlob,previous)?null:'postimage-mismatch';
}
export function verifyRepositoryEvidence(directory,options){
 if(options===null||typeof options!=='object'||Array.isArray(options))throw Error('repository_verification_request_required');
 const {lines,expectedAnchor,trustedPublicKey,seal,base,head,minShare=0}=options;
 if(!Number.isFinite(minShare)||minShare<0||minShare>1)throw Error('repository_min_share_required');
 primitives.verifyChain(lines,expectedAnchor);
 const installation=verifyInstallationSeal(lines,expectedAnchor,seal,trustedPublicKey);
 repositoryObjectIdentity(base);repositoryObjectIdentity(head);
 repositoryGit(directory,['merge-base','--is-ancestor',base,head]);
 const commits=repositoryText(repositoryGit(directory,['rev-list','--reverse','--topo-order',base+'..'+head])).trim().split('\n').filter(Boolean);
 if(!commits.length||commits.length>1000)throw Error('repository_commit_range_budget');
 const records=[],byHash=new Map();
 for(const line of lines){const record=decodeTyped(line);records.push(record);byHash.set(record.hash,record);}
 const observations=new Map();
 for(const record of records){
  if(record.kind==='edit')repositoryPath(record.payload.path);
  if(record.kind!=='commit')continue;
  const payload=record.payload;
  repositoryObjectIdentity(payload.commit);
  if(observations.has(payload.commit))throw Error('repository_duplicate_commit_observation');
  observations.set(payload.commit,record);
 }
 let totalLines=0,provenLines=0,unknownChurn=false,lastCommitIndex=-1;
 const files=[],commitReports=[];
 for(const commit of commits){
  const record=observations.get(commit),payload=record?.payload;
  let observed,bindingReason=null,edits=[];
  if(!record)bindingReason='missing-commit-observation';
  else{
   const anchor=byHash.get(payload.precommitAnchor?.hash),output=byHash.get(payload.messageOut);
   if(record.index<=lastCommitIndex||!anchor||anchor.index!==payload.precommitAnchor.index||anchor.sessionId!==payload.precommitAnchor.sessionId
    ||anchor.index>=record.index||!output||output.kind!=='message_out'||output.index>anchor.index)bindingReason='commit-causal-binding';
   else{
    observed=observeRepositoryCommit(directory,commit,payload);
    if(!same(observed,payload)||payload.trailers.length!==1||payload.trailers[0]!==commitPrefix(anchor))bindingReason='commit-object-or-trailer-mismatch';
    edits=payload.editHashes.map(hash=>byHash.get(hash));
    if(edits.some(edit=>!edit||edit.kind!=='edit'||edit.index>anchor.index))bindingReason='commit-edit-binding';
    else if(edits.some(edit=>edit.index<=lastCommitIndex||edit.index>=output.index))bindingReason='commit-edit-window';
   }
  }
  // Missing observations still enumerate the actual changed blobs; none disappear from the denominator.
  if(!observed){
   const fallback=records[0];
   observed=observeRepositoryCommit(directory,commit,{messageOut:fallback.hash,editHashes:[],precommitAnchor:fallback});
  }
  const actualPaths=new Set(observed.changes.map(change=>change.path));
  if(edits.some(edit=>edit&&(!actualPaths.has(edit.payload.path))))bindingReason='extra-unmatched-edit';
  for(const change of observed.changes){
   const pathEdits=edits.filter(edit=>edit?.payload.path===change.path).sort((left,right)=>left.index-right.index);
   const reason=bindingReason??causalPath(directory,change,pathEdits,byHash,payload.precommitAnchor.index);
   const churn=change.added===null||change.deleted===null?null:change.added+change.deleted;
   if(churn===null)unknownChurn=true;else{totalLines+=churn;if(reason===null)provenLines+=churn;}
   files.push({commit,path:change.path,status:reason===null?'causally-observed':'foreign',reason,
    oldBlob:change.oldBlob,newBlob:change.newBlob,changedLines:churn});
  }
  commitReports.push({commit,status:bindingReason===null?'bound':'foreign',reason:bindingReason});
  if(record)lastCommitIndex=Math.max(lastCommitIndex,record.index);
 }
 const share=unknownChurn||totalLines===0?null:provenLines/totalLines;
 return {schema:'formal-ai-repository-evidence-report-v1',integrity:'verified',installation,
  base,head,commits:commitReports,files,totalChangedLines:totalLines,provenChangedLines:provenLines,
  share,minimumShare:minShare,meetsMinimum:share!==null&&share>=minShare,
  allBlobsAttributed:files.every(file=>file.status==='causally-observed'),
  workingTree:'not-checked',replay:'not-performed',autonomousAuthorship:'not-established'};
}
