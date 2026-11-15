// Records the ordinary Hive Mind system and user prompts of the Claude and
// Codex tools (issue #1483) as fixtures with placeholders, so the offline
// adapter test needs no hive-mind install. Usage:
//   node experiments/issue-1483/record-hive-prompts.mjs [hive-mind src directory]
import {writeFileSync,readFileSync} from 'node:fs';
import {join} from 'node:path';
const source=process.argv[2]??'/home/box/.bun/install/global/node_modules/@link-assistant/hive-mind/src/';
const version=JSON.parse(readFileSync(join(source,'..','package.json'),'utf8')).version;
const out=new URL('../../data/benchmarks/fixtures/work-item-1483/',import.meta.url).pathname;
const params={issueUrl:'{issue_url}',issueNumber:'{issue}',prNumber:'{pull}',prUrl:'{pull_url}',branchName:'{branch}',
  tempDir:'{workspace}',workspaceTmpDir:'{workspace_tmp}',isContinueMode:false,forkedRepo:null,feedbackLines:[],
  owner:'{owner}',repo:'{repo}',argv:{model:'formal-ai'},contributingGuidelines:'',claudeVersion:'2.0.0'};
for(const tool of ['claude','codex']){
  const builders=await import(join(source,tool+'.prompts.lib.mjs'));
  writeFileSync(join(out,`hive-${tool}-system.txt`),await builders.buildSystemPrompt({...params,argv:{...params.argv,tool}}));
  writeFileSync(join(out,`hive-${tool}-user.txt`),await builders.buildUserPrompt({...params,argv:{...params.argv,tool}}));
}
console.log('recorded hive-mind',version);
