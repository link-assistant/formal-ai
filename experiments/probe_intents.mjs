import fs from "node:fs";import path from "node:path";import vm from "node:vm";import {fileURLToPath} from "node:url";import {TextEncoder,TextDecoder} from "node:util";
const root=path.resolve(path.dirname(fileURLToPath(import.meta.url)),"..");const webDir=path.join(root,"src","web");
const sandbox={};sandbox.self=sandbox;sandbox.globalThis=sandbox;sandbox.console=console;sandbox.postMessage=()=>{};sandbox.fetch=async(url)=>{const c=String(url).split("?")[0];return{ok:true,status:200,async text(){return fs.readFileSync(path.join(webDir,c),"utf8");}};};sandbox.WebAssembly={instantiate:async()=>{throw new Error("x");}};sandbox.location={search:""};sandbox.setTimeout=setTimeout;sandbox.clearTimeout=clearTimeout;sandbox.TextEncoder=TextEncoder;sandbox.TextDecoder=TextDecoder;sandbox.URL=URL;sandbox.URLSearchParams=URLSearchParams;
vm.createContext(sandbox);const run=(f)=>vm.runInContext(fs.readFileSync(f,"utf8"),sandbox,{filename:f});
run(path.join(webDir,"seed_loader.js"));for(let i=0;i<=20;i++)run(path.join(webDir,"worker",`formal_ai_worker_${String(i).padStart(2,"0")}.js`));
await vm.runInContext("loadSeed()",sandbox);const g=sandbox;
const prompts=["what are you","who are you","what can you do","define recursion","what is formal-ai","how do you work","2+2","what is a link"];
for(const p of prompts){const a=await g.solve(p,[],{},{},[],{});const forced=await g.solve(p,[],{},{},[],{forcedResponseLanguage:"ru"});console.log(`${JSON.stringify(p).padEnd(22)} intent=${String(a.intent).padEnd(22)} forced_ru_intent=${forced.intent}`);}

console.log("\n-- follow-up generalization across intent families --");
for(const [q,fu,lang] of [
  ["what can you do","я не понимаю по английски, напиши по русски","ru"],
  ["what are you","用中文回答","zh"],
  ["2+2","हिंदी में लिखें","hi"],
]){
  const hist=[{role:"user",content:q}];
  const first=await g.solve(q,[],{},{},[],{});
  hist.push({role:"assistant",content:String(first.content||"")});
  const second=await g.solve(fu,hist,{},{},[],{});
  const ev=Array.isArray(second.evidence)?second.evidence:[];
  console.log(`\n[${q}] -> [${fu}]`);
  console.log(`  first=${first.intent}  second=${second.intent}`);
  console.log(`  target:${ev.includes(`response_language_followup:target:${lang}`)} language_to:${ev.includes(`language_to:${lang}`)} handler:${ev.find(e=>e.startsWith("response_language_followup:handler:"))}`);
  console.log(`  content: ${String(second.content||"").slice(0,90).replace(/\n/g," ")}`);
}
