// Issue #1483: print the Claude and Codex adapter sessions for the boundary-repair
// work item (first run and repeat). The harness is shared with the regression test.
// Usage: node experiments/issue-1483/adapter-sessions.mjs [claude|codex]
import {readFileSync} from 'node:fs';
import {join} from 'node:path';
import {adapterSession,boundaryWorkspace,boundaryTask} from '../../rust/tests/web/helpers/work-item-adapter-session.mjs';

for(const protocol of process.argv.slice(2).length?process.argv.slice(2):['claude','codex']){
  const workspace=boundaryWorkspace();
  try{
    for(const run of ['first','repeat']){
      const session=await adapterSession(protocol,workspace.directory,boundaryTask);
      console.log(`== ${protocol} ${run}`);
      for(const row of session.trail)console.log(' ',row.tool,row.status,JSON.stringify(row.args).slice(0,200));
      console.log(String(session.final).slice(0,600));
    }
    console.log(readFileSync(join(workspace.directory,'src/ceiling.mjs'),'utf8'));
  }finally{workspace.dispose();}
}
