// Supplied evidence correction: include companion-file context settings.
import fs from 'node:fs';
import assert from 'node:assert/strict';
const file='experiments/issue_1582/continuation-matrix.mjs';
let source=fs.readFileSync(file,'utf8');
const previous="contextSettings:mode.children.filter(row=>/context_window_tokens|max_output_tokens|model_catalog_path/u.test(row.id+' '+row.name)).map(row=>({kind:row.name,value:row.id})),";
assert(source.includes(previous));
source=source.replace('const rows=tools.map(tool=>({',`function contextSettings(node, path=''){
 return node.children.flatMap(row=>{
  const currentPath=row.name==='companion'?row.id:path;
  const current=/context_window_tokens|max_output_tokens|model_catalog_path/u.test(row.id+' '+row.name)
   ?[{kind:row.name,value:row.id,...(currentPath?{path:currentPath}:{})}]:[];
  return [...current,...contextSettings(row,currentPath)];
 });
}
const rows=tools.map(tool=>({`).replace(previous,'contextSettings:contextSettings(mode),');
source=source.replace("fs.writeFileSync('docs/case-studies/issue-1582/raw-data/local/continuation-support-matrix.json',JSON.stringify({kind:","for(const row of rows)for(const mode of row.modes)assert(row.tool==='cursor'||mode.contextSettings.length>0, row.tool+' '+mode.mode+' has no context metadata');\nconst programmingTargets=JSON.parse(fs.readFileSync('docs/case-studies/issue-1582/raw-data/local/continuation-support-matrix.json','utf8')).programmingTargets;\nfs.writeFileSync('docs/case-studies/issue-1582/raw-data/local/continuation-support-matrix.json',JSON.stringify({programmingTargets,kind:");
fs.writeFileSync(file,source);
console.log('The old matrix omitted nested companion settings in four global modes; regeneration now walks descendants and asserts each non-MCP mode has context metadata.');
