#!/usr/bin/env node
// JavaScript integrity tooling. Native CLI and full authorship/replay remain separate acceptance.
import {snapshotRepository} from './snapshot-repository.mjs';
import {verifyDriveEvidence} from './verify-drive.mjs';
import {encodeTyped} from '../server/structured-log.mjs';
const args=process.argv.slice(2);
try{
 if(args.length===2&&args[0]==='snapshot')process.stdout.write(encodeTyped(snapshotRepository(args[1]))+'\n');
 else if(args.length===3&&args[0]==='verify-drive'){const report=verifyDriveEvidence(args[1],args[2]);process.stdout.write(encodeTyped(report)+'\n');if(report.files.some(x=>x.status==='foreign'))process.exitCode=1;}
 else{console.error('usage: node js/provenance/cli.mjs snapshot <dir> | verify-drive <evidence-dir> <workspace>');process.exitCode=2;}
}catch(error){console.error(error.message);process.exitCode=1;}
