// This real CommonJS entry compiles after the journal preload installs its hook.
// The fixed ESM planner remains the sole planner entry; its rejection fails the process.
const plannerEntrypoint = './main.mjs';
void import(plannerEntrypoint).catch(async error=>{
 process.exitCode=1;
 try{
  const fileSystem=require('node:fs'),path=require('node:path'),url=require('node:url');
  const configuration=JSON.parse(fileSystem.readFileSync(process.env.FORMAL_AI_RUNTIME_CONFIG,'utf8'));
  const {runtimeFailureIdentity}=await import(url.pathToFileURL(path.join(configuration.repository,'js/provenance/runtime-failure-identity.mjs')).href);
  fileSystem.writeFileSync(configuration.resultFile,JSON.stringify({runtimeFailure:runtimeFailureIdentity(error,configuration.repository)}),{flag:'wx',mode:0o600});
 }catch{}
 // The launcher retains this stream privately; the public verifier emits only the identity.
 process.stderr.write((typeof error?.stack==='string'?error.stack:String(error))+'\n');
});
