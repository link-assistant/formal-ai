import path from 'node:path';
import {publicEvidenceValue} from './issue-1364-public-evidence.mjs';

/** Retain both diagnostic streams without repeating an uncertain command. */
export function requireSuccessfulReplayCommand(result,{program,environment=process.env}){
 if(result.error===undefined&&result.cleanup_complete===true&&result.signal===null&&
  result.status===0&&result.timed_out!==true)return;
 const detail=publicEvidenceValue({program:path.basename(program),exitCode:result.status??null,
  signal:result.signal??null,timedOut:result.timed_out===true,cleanupComplete:result.cleanup_complete===true,
  errorCode:result.error?.code??null,stdout:result.stdout?.toString()??'',stderr:result.stderr?.toString()??''},environment);
 detail.stdout=detail.stdout.slice(-4000);detail.stderr=detail.stderr.slice(-4000);
 throw Object.assign(Error('Replay command failed '+JSON.stringify(detail)),{code:'ISSUE1364_COMMAND_FAILED'});
}
