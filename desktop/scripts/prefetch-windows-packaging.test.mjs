import test from "node:test";
import assert from "node:assert/strict";
import {prefetchWindowsPackaging,validateSelectedDesktop} from "./prefetch-windows-packaging.mjs";
test("all locked runtime and NSIS downloads start concurrently with exact options", async () => {
 const calls=[], resolvers=[]; const fn=name=>(...args)=>{calls.push({name,args});return new Promise(resolve=>resolvers.push(resolve));};
 const config={toolsets:{nsis:"3.12"},nsis:{customNsisBinary:{url:"owned"},customNsisResources:{checksum:"owned"}},electronDownload:{mirror:"owned"}};
 const pending=prefetchWindowsPackaging({version:"44.7.0",config,cacheDir:"cache",downloadElectronArtifactZip:fn("electron"),getMakeNsisPath:fn("nsis"),getNsisPluginsPath:fn("resources"),prefetchTools:fn("7zip")});
 assert.deepEqual(calls.map(c=>c.name),["electron","nsis","resources","7zip"]);
 assert.deepEqual(calls[0].args,[{version:"44.7.0",platformName:"win32",arch:"arm64",artifactName:"electron",cacheDir:"cache",electronDownload:config.electronDownload}]);
 assert.deepEqual(calls[1].args,["3.12",config.nsis.customNsisBinary]);assert.deepEqual(calls[2].args,["3.12",config.nsis.customNsisResources]);
 resolvers.forEach(r=>r("verified"));assert.deepEqual(await pending,["verified","verified","verified","verified"]);
});
test("unlocked version refuses before downloads; upstream checksum errors remain failures",async()=>{
 let calls=0; const fn=()=>{calls++;return Promise.resolve();};
 await assert.rejects(prefetchWindowsPackaging({version:undefined,downloadElectronArtifactZip:fn}),/locked/);assert.equal(calls,0);
 await assert.rejects(prefetchWindowsPackaging({version:"44.7.0",downloadElectronArtifactZip:()=>Promise.reject(new Error("checksum mismatch")),getMakeNsisPath:fn,getNsisPluginsPath:fn,prefetchTools:fn}),/checksum mismatch/);
});

import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import {createHash} from "node:crypto";
import {execFileSync} from "node:child_process";
test("captured helper binds selected source and lock even when selected release has no helper",()=>{
 const root=fs.mkdtempSync(path.join(os.tmpdir(),"packaging-protocol-test-"));
 try {
  const desktop=path.join(root,"desktop"),protocol=path.join(root,"protocol");fs.mkdirSync(desktop);fs.mkdirSync(protocol);
  fs.writeFileSync(path.join(desktop,"package.json"),"{}\n");fs.writeFileSync(path.join(desktop,"package-lock.json"),"{}\n");
  const git=args=>execFileSync("git",args,{cwd:root,encoding:"utf8"}).trim();git(["init","-q"]);git(["add","desktop"]);git(["-c","user.name=Fixture","-c","user.email=fixture@example.invalid","commit","-qm","selected release"]);
  const source={source_commit:git(["rev-parse","HEAD"]),source_tree:git(["rev-parse","HEAD^{tree}"])};
  const files=["prefetch-builder-toolsets.mjs","prefetch-windows-packaging.mjs"].map(name=>{const bytes=Buffer.from(name);fs.writeFileSync(path.join(protocol,name),bytes);return{name,bytes:bytes.length,sha256:createHash("sha256").update(bytes).digest("hex")};});
  fs.writeFileSync(path.join(protocol,"packaging-protocol-selection.json"),JSON.stringify({commit:"trusted-workflow",files}));
  assert.equal(fs.existsSync(path.join(desktop,"scripts/prefetch-windows-packaging.mjs")),false);
  validateSelectedDesktop(desktop,source,protocol,"trusted-workflow");
  assert.throws(()=>validateSelectedDesktop(desktop,{...source,source_tree:"foreign"},protocol,"trusted-workflow"));
  assert.throws(()=>validateSelectedDesktop(desktop,source,protocol,"foreign-workflow"));
  fs.writeFileSync(path.join(desktop,"package-lock.json"),"changed");assert.throws(()=>validateSelectedDesktop(desktop,source,protocol,"trusted-workflow"));
  fs.writeFileSync(path.join(desktop,"package-lock.json"),"{}\n");fs.writeFileSync(path.join(protocol,files[0].name),"drift");assert.throws(()=>validateSelectedDesktop(desktop,source,protocol,"trusted-workflow"));
 } finally {fs.rmSync(root,{recursive:true,force:true});}
});
