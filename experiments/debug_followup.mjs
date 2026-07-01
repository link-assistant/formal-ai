import fs from "node:fs";
import path from "node:path";
import vm from "node:vm";
import { fileURLToPath } from "node:url";
import { TextEncoder, TextDecoder } from "node:util";
const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const webDir = path.join(root, "src", "web");
const seedDir = path.join(webDir, "seed");
function readSeedRaw(){const raw={};for(const f of fs.readdirSync(seedDir))if(f.endsWith(".lino"))raw[`seed/${f}`]=fs.readFileSync(path.join(seedDir,f),"utf8");return raw;}
const sandbox={};sandbox.self=sandbox;sandbox.globalThis=sandbox;sandbox.console=console;sandbox.postMessage=()=>{};sandbox.fetch=async(url)=>{const clean=String(url).split("?")[0];const file=path.join(webDir,clean);const text=fs.readFileSync(file,"utf8");return{ok:true,status:200,async text(){return text;}};};sandbox.WebAssembly={instantiate:async()=>{throw new Error("no wasm");}};sandbox.location={search:""};sandbox.setTimeout=setTimeout;sandbox.clearTimeout=clearTimeout;sandbox.TextEncoder=TextEncoder;sandbox.TextDecoder=TextDecoder;sandbox.URL=URL;sandbox.URLSearchParams=URLSearchParams;
vm.createContext(sandbox);
const run=(f)=>vm.runInContext(fs.readFileSync(f,"utf8"),sandbox,{filename:f});
run(path.join(webDir,"seed_loader.js"));
for(let i=0;i<=20;i++)run(path.join(webDir,"worker",`formal_ai_worker_${String(i).padStart(2,"0")}.js`));
sandbox.FormalAiSeed.loadAll=async()=>({raw:readSeedRaw()});
await vm.runInContext("loadSeed()",sandbox);
const g=sandbox;
const norm=g.normalizePrompt;
for(const p of ["Я не понимаю по-английски, ответь по-русски","用中文","मुझे समझ नहीं आता, हिंदी में लिखें"]){
  const n=norm(p);
  console.log(`prompt=${p}`);
  console.log(`  normalized=${n}`);
  console.log(`  detectResponseLanguage=${g.detectResponseLanguage(n)}`);
  console.log(`  detectComprehensionFailure=${g.detectComprehensionFailure(n)}`);
}
console.log("\n-- replay of previous prompts (forced) --");
for(const [p,lang] of [["What is the deep-theory repository?","ru"],["What is a formal system?","zh"],["What is deep-theory?","hi"]]){
  const ans=await g.solve(p,[],{},{},[],{forcedResponseLanguage:lang});
  console.log(`  "${p}" -> intent=${ans.intent}  lang content: ${String(ans.content||"").slice(0,60).replace(/\n/g," ")}`);
}

console.log("\n-- e2e-style project lookup then RU followup --");
const hist=[
  {role:"user",content:"ты можешь сделать кодревью https://github.com/netkeep80/anum_docs ?"},
];
const first=await g.solve(hist[0].content,[],{},{},[],{});
console.log(`first intent=${first.intent}`);
hist.push({role:"assistant",content:String(first.content||"")});
const second=await g.solve("я не понимаю по английски, напиши по русски",hist,{},{},[],{});
console.log(`second intent=${second.intent}`);
const ev=Array.isArray(second.evidence)?second.evidence:[];
console.log(`  target: ${ev.includes("response_language_followup:target:ru")}`);
console.log(`  language_to: ${ev.includes("language_to:ru")}`);
console.log(`  handler: ${ev.find(e=>e.startsWith("response_language_followup:handler:"))}`);
console.log(`  content[0:120]: ${String(second.content||"").slice(0,120).replace(/\n/g," ")}`);
