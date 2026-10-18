#!/usr/bin/env node
// Inspection is not trusted verification, replay, or autonomous authorship.
import fileSystem from 'node:fs';
import path from 'node:path';
import {readBoundedFileBytes}from './bounded-file.mjs';
import {verifyFullRepositoryProof}from './full-verifier.mjs';
import {snapshotRepository} from './snapshot-repository.mjs';
import {verifyDriveEvidence} from './verify-drive.mjs';
import {encodeTyped} from '../server/structured-log.mjs';
const argumentsList=process.argv.slice(2);
try{
 if(argumentsList.length===2&&argumentsList[0]==='snapshot'){
  process.stdout.write(encodeTyped(snapshotRepository(argumentsList[1]))+'\n');
 }else if(argumentsList.length===3&&['inspect-drive','verify-drive'].includes(argumentsList[0])){
  const report=verifyDriveEvidence(argumentsList[1],argumentsList[2],undefined);
  const inspection=argumentsList[0]==='inspect-drive';
  process.stdout.write(encodeTyped({...report,
   verification:inspection?'inspection-only':'Unknown',trustedVerification:false,
   fullReplay:{verified:false,reason:'PinnedEngineReplayUnavailable'},
   verificationGaps:['IndependentExpectedAnchorRequired','TrustedRuntimeSignerUnavailable',
    'AuthoritativeLoadedEngineGenesisUnavailable','ActualRepositoryChangeEvidenceUnavailable']})+'\n');
  process.exitCode=inspection?(report.files.some(file=>file.status==='foreign')?1:0):2;
 }else if(argumentsList.length===3&&argumentsList[0]==='verify-provenance'&&argumentsList[1]==='--request'){
  const filename=fileSystem.realpathSync(argumentsList[2]);
  const request=JSON.parse(readBoundedFileBytes(path.dirname(filename),path.basename(filename),256*1024).toString('utf8'));
  const evidence=fileSystem.realpathSync(request.directory),workspace=fileSystem.realpathSync(request.workspace);
  if(filename===evidence||filename.startsWith(evidence+path.sep)
   ||filename===workspace||filename.startsWith(workspace+path.sep))throw Error('FullProofIndependentRequestRequired');
  try{process.stdout.write(encodeTyped(verifyFullRepositoryProof(request))+'\n');}
  catch(error){process.stdout.write(encodeTyped({verification:'Unknown',trustedVerification:false,
   fullReplay:{verified:false},verificationGaps:[String(error.message)]})+'\n');process.exitCode=2;}
 }else if(argumentsList[0]==='verify-provenance'){
  process.stdout.write(encodeTyped({verification:'Unknown',trustedVerification:false,
   scope:'repository-proof-and-pinned-replay-unavailable',fullReplay:{verified:false},
   verificationGaps:['IndependentExpectedAnchorRequired','TrustedRuntimeSignerUnavailable',
    'ActualRepositoryChangeEvidenceUnavailable','PinnedEngineReplayUnavailable']})+'\n');
  process.exitCode=2;
 }else{
  console.error('usage: node js/provenance/cli.mjs snapshot <dir> | inspect-drive <evidence-dir> <workspace> | verify-drive <evidence-dir> <workspace> | verify-provenance <repo> [<pr>|<range>]');
  process.exitCode=2;
 }
}catch(error){console.error(error.message);process.exitCode=1;}
