// Preserve optional local runtimes, but never count hosted skips as observations.
import assert from 'node:assert/strict';
import {spawnSync} from 'node:child_process';

export function templatePythonAvailable(context){
  const unavailable=detail=>{
    assert.notEqual(process.env.GITHUB_ACTIONS,'true','Python 3.14+ is required for hosted template observations: '+detail);
    context.skip(detail);return false;
  };
  const runtime=spawnSync('python3',['--version'],{encoding:'utf8',timeout:5000,maxBuffer:4096});
  if(runtime.error?.code==='ENOENT')return unavailable('python3 is not installed');
  assert.equal(runtime.error,undefined,String(runtime.error));
  assert.equal(runtime.status,0,runtime.stderr);
  const version=/Python (\d+)\.(\d+)/u.exec(runtime.stdout);
  assert.ok(version,runtime.stdout);
  if(Number(version[1])<3||(Number(version[1])===3&&Number(version[2])<14))
    return unavailable('Pinned upstream syntax requires Python 3.14+');
  return true;
}
