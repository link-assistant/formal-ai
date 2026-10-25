// Build-time client setup. Every command runs once within one shared deadline.
import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import {pathToFileURL} from 'node:url';
import {boundedCommand} from '../js/server/bounded-command.mjs';
import {makeRedactor} from '../js/server/structured-log.mjs';
import {verifyContainerRuntimeTools} from './verify-container-runtime-tools.mjs';

const clients={claude:'@anthropic-ai/claude-code',codex:'@openai/codex','playwright-mcp':'@playwright/mcp'};
const metadata=file=>JSON.parse(fs.readFileSync(file,'utf8'));
function project(directory){const home=fs.realpathSync(directory);verifyContainerRuntimeTools(home);return home;}
function member(home,name,relative){
 assert(typeof relative==='string'&&relative.split('/').every(part=>/^[A-Za-z0-9_.-]+$/u.test(part)&&part!=='.'&&part!=='..'));
 return path.join(home,'node_modules',name,relative);
}
function ensureLink(link,target,type){
 fs.mkdirSync(path.dirname(link),{recursive:true});let existing=false;
 try{fs.lstatSync(link);existing=true;}catch(error){if(error.code!=='ENOENT')throw error;}
 if(existing){
  const actual=fs.statSync(link),expected=fs.statSync(target);
  assert(fs.realpathSync(link)===fs.realpathSync(target)||
   actual.isFile()&&expected.isFile()&&actual.dev===expected.dev&&actual.ino===expected.ino,
  'unexpected existing runtime link');
 }
 else fs.symlinkSync(path.relative(path.dirname(link),target),link,type);
 return link;
}
export async function configureContainerClientTools(directory,{environment=process.env,transport=boundedCommand}={}){
 const home=project(directory),deadline=Date.now()+180000,redact=makeRedactor(environment),versions={};
 async function run(command,args,maximum=30000,cwd=home){
  const remaining=deadline-Date.now();assert(remaining>0,'container client setup deadline');
  const result=await transport(command,args,{cwd,env:environment,timeout:Math.min(maximum,remaining),maxBuffer:262144});
  if(result.error||result.status!==0||result.signal!==null||result.timed_out||result.cleanup_complete!==true){
   const detail=redact({command:path.basename(command),
    status:result.status??null,signal:result.signal??null,timedOut:result.timed_out===true,cleanupComplete:result.cleanup_complete===true,
    stdout:result.stdout?.toString()??'',stderr:result.stderr?.toString()??''});
   detail.stdout=detail.stdout.slice(-4000);detail.stderr=detail.stderr.slice(-4000);
   throw Error('Container client setup failed '+JSON.stringify(detail));
  }
  return result.stdout.toString();
 }
 const binaries={};
 for(const [command,name]of Object.entries(clients)){
  const packageHome=path.join(home,'node_modules',name),installed=metadata(path.join(packageHome,'package.json'));
  const target=member(home,name,installed.bin?.[command]);
  if(command==='claude'){
   // Bun intentionally does not trust arbitrary lifecycle scripts. This pinned
   // official package ships a placeholder and needs its reviewed native linker;
   // run only that idempotent linker, even when the placeholder already exists.
   assert.equal(installed.scripts?.postinstall,'node install.cjs','review Claude native linker changes');
   await run(process.execPath,[path.join(packageHome,'install.cjs')],60000,packageHome);
  }
  assert(fs.statSync(target).isFile(),'installed client executable missing: '+command);
  binaries[command]=ensureLink(path.join(home,'node_modules/.bin',command),target,'file');
  const output=await run(binaries[command],[command==='playwright-mcp'?'--help':'--version']);
  if(command==='playwright-mcp')assert.match(output,/--headless/u);else{
   assert(output.includes(installed.version),'client version differs from pinned metadata: '+command);
   versions[command]=installed.version;
  }
 }
 const server=[binaries['playwright-mcp'],'--isolated','--headless','--no-sandbox',
  '--timeout-action=600000','--viewport-size','1920x1080'];
 for(const command of ['claude','codex']){
  await run(binaries[command],['mcp','add','playwright',...(command==='claude'?['-s','user']:[]),'--',...server]);
  const listing=await run(binaries[command],['mcp','list']);
  const rows=listing.split('\n').filter(line=>/\bplaywright\b/iu.test(line));
  assert(rows.some(row=>/connected|enabled|running|[✓✔]/iu.test(row)&&
   !/failed|disconnected|disabled|not connected/iu.test(row)),'Playwright MCP not healthy for '+command);
 }
 return {verified:true,versions,mcp:'local-pinned-stdio',browserNavigationTested:false};
}

/** One physical package, also visible to Hive's npm-global availability probe. */
export function linkContainerMcp(directory,globalDirectory){
 const home=project(directory),source=path.join(home,'node_modules/@playwright/mcp');
 assert(path.isAbsolute(globalDirectory));
 const target=path.join(globalDirectory,'@playwright/mcp');ensureLink(target,source,'dir');
 return {linked:true,package:'@playwright/mcp',version:metadata(path.join(source,'package.json')).version};
}
if(import.meta.url===pathToFileURL(process.argv[1]??'').href){
 try{
  if(process.argv[2]==='--global-mcp'){
   const result=await boundedCommand('npm',['root','--global'],{timeout:15000,maxBuffer:65536});
   assert(!result.error&&result.status===0&&result.signal===null&&result.cleanup_complete===true,'npm global directory discovery failed');
   console.log(JSON.stringify(linkContainerMcp(process.argv[3],result.stdout.toString().trim())));
  }
  else console.log(JSON.stringify(await configureContainerClientTools(process.argv[2])));
 }catch(error){console.error(String(error.stack??error));process.exitCode=1;}
}
