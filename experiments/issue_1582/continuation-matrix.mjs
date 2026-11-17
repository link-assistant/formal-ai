// Supplied complete registry census: configuration evidence, never vendor execution.
import fs from 'node:fs';
import assert from 'node:assert/strict';
import {installDefaultNodeSourceHost} from '../../js/server/default-node-source-bootstrap.mjs';
import {WorkerHost} from '../../js/server/worker-host.mjs';
await installDefaultNodeSourceHost(new WorkerHost());
import {parseRoot,findChildValue} from '../../js/agentic/crate/seed_parser.mjs';
const registry=parseRoot(fs.readFileSync('data/seed/client-integrations.lino','utf8')).children.find(row=>row.name==='client_integrations');
const tools=registry.children.filter(row=>row.name==='tool');
assert.equal(tools.length,12);
const rows=tools.map(tool=>({
 tool:tool.id,aliases:findChildValue(tool,'aliases'),protocols:findChildValue(tool,'supported_protocols'),
 scope:tool.id==='cursor'?'MCP tool; host model context does not belong to this adapter':'model provider',
 inputContext:tool.id==='gemini'?'BLOCKED: upstream tokenLimit fallback ignores custom model metadata':tool.id==='cursor'?'not applicable':'configuration declared; native execution pending',
 modes:tool.children.filter(row=>['ephemeral','global'].includes(row.name)).map(mode=>({
  mode:mode.name,protocol:mode.id||'default',path:findChildValue(mode,'path')||findChildValue(mode,'temp_home_config_path'),
  contextSettings:mode.children.filter(row=>/context_window_tokens|max_output_tokens|model_catalog_path/u.test(row.id+' '+row.name)).map(row=>({kind:row.name,value:row.id})),
 })),
}));
const languages=parseRoot(fs.readFileSync('data/seed/languages.lino','utf8')).children.find(row=>row.name==='languages').children.filter(row=>row.name==='language').map(row=>row.id);
assert.deepEqual(languages,['en','ru','hi','zh','es']);
fs.writeFileSync('docs/case-studies/issue-1582/raw-data/local/continuation-support-matrix.json',JSON.stringify({kind:'source configuration census; not installed-client execution',tools:rows,naturalLanguages:languages,programmingRoots:{JavaScript:'executable Node/worker regressions',TypeScript:'generated projection and executable permission recognizer',Rust:'source twin and regression drafts; production type/lint only on PR CI, native tests unexecuted'}},null,2)+'\n');
console.log(JSON.stringify({tools:rows.map(row=>row.tool),naturalLanguages:languages,modeCount:rows.reduce((n,row)=>n+row.modes.length,0)}));
