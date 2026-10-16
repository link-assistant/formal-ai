#!/usr/bin/env node
// JavaScript integrity tooling. Native CLI and full authorship/replay remain separate acceptance.
import {snapshotRepository} from './snapshot-repository.mjs';
import {verifyDriveEvidence} from './verify-drive.mjs';
import {encodeTyped} from '../server/structured-log.mjs';
const argumentsList=process.argv.slice(2);
try{
 if(argumentsList.length===2&&argumentsList[0]==='snapshot')process.stdout.write(encodeTyped(snapshotRepository(argumentsList[1]))+'\n');
 else if(argumentsList.length===3&&argumentsList[0]==='verify-drive'){const report=verifyDriveEvidence(argumentsList[1],argumentsList[2]);process.stdout.write(encodeTyped(report)+'\n');if(report.files.some(x=>x.status==='foreign'))process.exitCode=1;}
 else{console.error('usage: node js/provenance/cli.mjs snapshot <dir> | verify-drive <evidence-dir> <workspace>');process.exitCode=2;}
}catch(error){console.error(error.message);process.exitCode=1;}
