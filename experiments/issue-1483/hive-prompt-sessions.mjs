// Issue #1483: drive the Claude and Codex adapters with the ordinary Hive
// prompts (recorded by record-hive-prompts.mjs). The user prompt names only
// the issue URL; a `gh` shim serves the issue text offline, so the controller
// fetches the task itself. Prints the trail and final report of a first run
// and a repeat for each adapter.
import {join} from 'node:path';
import {adapterSession,boundaryWorkspace,boundaryTask,hivePrompts,ghShim} from '../../rust/tests/web/helpers/work-item-adapter-session.mjs';
for(const protocol of ['claude','codex']){
  const workspace=boundaryWorkspace();
  try{
    const gh=ghShim(workspace.root,{title:'Ceiling check excludes the maximum',body:boundaryTask});
    const prompts=hivePrompts(protocol,{issue_url:'https://github.com/example/ceiling/issues/7',issue:7,pull_url:'https://github.com/example/ceiling/pull/8',
      pull:8,branch:'issue-7-0123456789ab',workspace:workspace.directory,workspace_tmp:join(workspace.root,'tmp'),owner:'example',repo:'ceiling'});
    for(const round of ['first','repeat']){
      const session=await adapterSession(protocol,workspace.directory,prompts.user,{system:prompts.system,env:gh.env});
      console.log(`== ${protocol} ${round}`);
      for(const row of session.trail)console.log('  ',row.tool,row.status,JSON.stringify(row.args).slice(0,120));
      console.log(session.final);
    }
  }finally{workspace.dispose();}
}
