// Independent check of a #1455 guide change against the manifests and source
// of the checkout it was made on. Shares no code with Formal AI: it reads the
// commit with plain `git show`, parses Cargo.toml line by line and package.json
// with JSON.parse, and looks for each quoted call verbatim in the named file.
//
//   node experiments/issue-1483/check-1455-guide-change.mjs --at 3c592625d \
//     --diff docs/case-studies/issue-1483/hive-agent-1455/workspace.diff
//
// Exit 0 only when every added claim is backed and the diff only appends.
import {readFileSync} from 'node:fs';
import {execFileSync} from 'node:child_process';

const args=process.argv.slice(2),option=name=>args[args.indexOf(name)+1];
const at=option('--at'),diff=readFileSync(option('--diff'),'utf8');
const show=path=>execFileSync('git',['show',`${at}:${path}`],{encoding:'utf8'});
const results=[];const check=(name,ok,detail)=>results.push({name,ok:Boolean(ok),detail});

const lines=diff.split('\n');
check('only docs/associative-tech-stack.md changes',lines.filter(l=>l.startsWith('+++ ')).every(l=>l==='+++ b/docs/associative-tech-stack.md'),'');
check('preimage preserved (no removed lines)',!lines.some(l=>l.startsWith('-')&&!l.startsWith('---')),'');
const added=lines.filter(l=>l.startsWith('+')&&!l.startsWith('+++')).map(l=>l.slice(1));

// Cargo.toml: dependency name -> {section, target, raw value}.
const cargo={};let section='',target='unqualified';
for(const line of show('rust/Cargo.toml').split('\n')){
  const header=line.match(/^\[(.+)\]\s*$/u);
  if(header){const t=header[1].match(/^target\.'(.+)'\.(.+)$/u);section=t?t[2]:header[1];target=t?t[1]:'unqualified';continue;}
  const entry=line.match(/^([A-Za-z0-9_-]+)\s*=\s*(.+)$/u);
  if(entry&&/dependencies$/u.test(section))cargo[entry[1]]={section,target,value:entry[2]};
}
const pkg=JSON.parse(show('package.json'));

let manifest=null;
for(const line of added){
  const start=line.match(/^<!-- formal-ai-manifest-start: (.+) -->$/u);if(start){manifest=start[1];continue;}
  if(/^<!-- formal-ai-manifest-end/u.test(line)){manifest=null;continue;}
  const row=line.match(/^\| `([^`]+)` \| `([^`]+)` \| `([^`]+)` \| `([^`]+)` \| `(.+)` \|$/u);
  if(!row||!manifest)continue;
  const [,name,version,sectionName,targetName,options]=row;
  if(manifest==='rust/Cargo.toml'){
    const found=cargo[name];
    const declared=found&&(found.value.match(/^"([^"]+)"/u)?.[1]??found.value.match(/version\s*=\s*"([^"]+)"/u)?.[1]);
    const optionsOk=Object.entries(JSON.parse(options)).every(([key,value])=>found?.value.includes(`${key} = ${JSON.stringify(value).replaceAll(',',', ')}`));
    check(`Cargo.toml declares ${name}`,found&&declared===version&&found.section===sectionName&&found.target===targetName&&optionsOk,found?.value??'missing');
  }else if(manifest==='package.json'){
    check(`package.json declares ${name}`,pkg[sectionName]?.[name]===version&&targetName==='unqualified'&&options==='{}',String(pkg[sectionName]?.[name]));
  }else check(`known manifest ${manifest}`,false,'');
}

// Each "Imported X from P is called in F" claim and its fenced excerpt.
for(let i=0;i<added.length;i++){
  const claim=added[i].match(/^Imported `([^`]+)` from `([^`]+)` is called in `([^`]+)`/u);if(!claim)continue;
  const [,symbol,from,file]=claim,source=show(file);
  const fence=added.indexOf('```javascript',i),excerpt=added[fence+1];
  const imported=new RegExp(`import\\s*\\{[^}]*\\b${symbol}\\b[^}]*\\}\\s*from\\s*["']${from.replace(/[.*+?^${}()|[\]\\/]/gu,'\\$&')}["']`,'u').test(source);
  check(`${file} imports ${symbol} from ${from} and contains the excerpt`,imported&&excerpt.startsWith(symbol+'(')&&source.includes(excerpt),excerpt);
}

// The issue names three components; each must be documented by a backed row.
for(const name of ['command-stream','web-capture','web-search'])
  check(`issue component ${name} documented`,results.some(r=>r.ok&&r.name.endsWith(`declares ${name}`)),'');

for(const r of results)console.log(`${r.ok?'pass':'FAIL'}  ${r.name}${r.detail?`  (${r.detail})`:''}`);
const failed=results.filter(r=>!r.ok).length;
console.log(`${results.length-failed}/${results.length} checks passed at ${at}`);
process.exit(failed?1:0);
