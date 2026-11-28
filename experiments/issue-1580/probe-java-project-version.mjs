// Execute the unchanged pinned release CLI against four disposable local Git
// repositories. No Maven, hosted service or network is used.
import assert from 'node:assert/strict';
import {spawnSync} from 'node:child_process';
import {createHash} from 'node:crypto';
import {cpSync,existsSync,mkdirSync,mkdtempSync,readFileSync,rmSync,writeFileSync} from 'node:fs';
import {tmpdir} from 'node:os';
import {join} from 'node:path';
import {fileURLToPath,pathToFileURL} from 'node:url';

const fixture=fileURLToPath(new URL('./fixtures/java-template/',import.meta.url));
const provenance=JSON.parse(readFileSync(join(fixture,'source.json'),'utf8'));
for(const item of provenance.files){
  const bytes=readFileSync(join(fixture,item.path));
  assert.equal(createHash('sha1').update(`blob ${bytes.length}\0`).update(bytes).digest('hex'),item.gitBlob);
  assert.equal(createHash('sha256').update(bytes).digest('hex'),item.sha256);
}
const parent=version=>`<parent><groupId>example</groupId><artifactId>parent</artifactId><version>${version}</version></parent>`;
const cases=[
  {name:'plain-project',prefix:''},
  {name:'different-parent-version',prefix:parent('9.8.7')},
  {name:'equal-parent-version',prefix:parent('1.2.3')},
  {name:'comment-before-project-version',prefix:'<!-- Example: <version>1.2.3</version> -->'},
];
export function observeJavaProjectVersions({candidate=false}={}){
const observations=[];
for(const entry of cases){
  const temporary=mkdtempSync(join(tmpdir(),'issue-1580-java-project-version-'));
  try{
    const cwd=join(temporary,'checkout');mkdirSync(cwd);
    const remote=join(temporary,'remote.git');
    const hooks=join(temporary,'hooks');mkdirSync(hooks);
    const environment={...process.env};
    for(const name of Object.keys(environment))if(name.startsWith('GIT_'))delete environment[name];
    Object.assign(environment,{GIT_CONFIG_NOSYSTEM:'1',GIT_CONFIG_GLOBAL:'/dev/null',GIT_TERMINAL_PROMPT:'0',
      GITHUB_OUTPUT:join(temporary,'outputs'),GITHUB_ACTIONS:'',GITHUB_REPOSITORY:''});
    const git=(...arguments_)=>{
      const result=spawnSync('git',arguments_,{cwd,env:environment,encoding:'utf8',maxBuffer:131072});
      assert.equal(result.error,undefined,String(result.error));
      assert.equal(result.status,0,result.stderr);
      return result.stdout.trim();
    };
    cpSync(join(fixture,'scripts'),join(cwd,'scripts'),{recursive:true});
    if(candidate){
      const filename=join(cwd,'scripts/version-and-commit.mjs');
      const source=readFileSync(filename,'utf8');
      const start=source.indexOf('function updatePomVersion(');
      const end=source.indexOf('\n/**',start);
      assert(start>=0&&end>start,'one complete upstream POM writer');
      const replacement=String.raw`function updatePomVersion(pomPath, oldVersion, newVersion) {
  const content = readFileSync(pomPath, 'utf-8');
  const masked = content.replace(/<!--[\s\S]*?-->/g, value => ' '.repeat(value.length))
    .replace(/<parent>[\s\S]*?<\/parent>/g, value => ' '.repeat(value.length));
  const match = masked.match(/(<project[^>]*>[\s\S]*?<version>)([^<]+)(<\/version>)/);
  if (!match || match[2].trim() !== oldVersion) throw new Error('Could not locate project version');
  const offset = match.index + match[1].length;
  writeFileSync(pomPath, content.slice(0, offset) + newVersion + content.slice(offset + match[2].length));
}
`;
      writeFileSync(filename,source.slice(0,start)+replacement+source.slice(end));
    }
    mkdirSync(join(cwd,'.changeset'));
    const javaDirectory=join(cwd,'src/main/java/com/linkfoundation/mypackage');mkdirSync(javaDirectory,{recursive:true});
    const pom=`<project>${entry.prefix}<groupId>example</groupId><artifactId>my-package</artifactId><version>1.2.3</version></project>\n`;
    writeFileSync(join(cwd,'pom.xml'),pom);
    writeFileSync(join(javaDirectory,'MyPackage.java'),'class MyPackage { static final String VERSION = "1.2.3"; }\n');
    writeFileSync(join(cwd,'CHANGELOG.md'),'# Changelog\n\nPrevious release.\n');
    writeFileSync(join(cwd,'.changeset','fix.md'),"---\n'my-package': patch\n---\n\nProject version fixture.\n");
    git('init','--quiet','--initial-branch=fixture');
    git('config','user.email','fixture@example.invalid');git('config','user.name','Version fixture');
    git('config','commit.gpgsign','false');git('config','tag.gpgsign','false');git('config','core.autocrlf','false');git('config','core.hooksPath',hooks);
    git('add','.');git('commit','--quiet','-m','version input');
    git('init','--quiet','--bare',remote);git('remote','add','origin',remote);git('push','--quiet','--set-upstream','origin','fixture');
    const result=spawnSync(process.execPath,['--max-old-space-size=64',join(cwd,'scripts/version-and-commit.mjs'),'--mode','instant','--bump-type','patch'],
      {cwd,env:environment,encoding:'utf8',maxBuffer:131072});
    assert.equal(result.error,undefined,String(result.error));
    observations.push({name:entry.name,initialPom:pom,exitCode:result.status,stdout:result.stdout,stderr:result.stderr,
      outputs:existsSync(environment.GITHUB_OUTPUT)?readFileSync(environment.GITHUB_OUTPUT,'utf8'):'',
      taggedPom:git('--git-dir',remote,'show','v1.2.4:pom.xml'),
      taggedJava:git('--git-dir',remote,'show','v1.2.4:src/main/java/com/linkfoundation/mypackage/MyPackage.java')});
  }finally{rmSync(temporary,{recursive:true,force:true});}
}
return {fixtureProvenance:provenance,candidateApplied:candidate,observations,nativeExecutionObserved:false,mavenExecuted:false,hostedPublicationExecuted:false};
}
if(process.argv[1]&&pathToFileURL(process.argv[1]).href===import.meta.url){
  const destination=process.argv[2];
  assert(destination,'Usage: node experiments/issue-1580/probe-java-project-version.mjs OUTPUT_JSON [--candidate]');
  assert(process.argv[3]===undefined||process.argv[3]==='--candidate','unknown option');
  const report=observeJavaProjectVersions({candidate:process.argv[3]==='--candidate'});
  writeFileSync(destination,JSON.stringify(report,null,2)+'\n');
  console.log(JSON.stringify({observations:report.observations.length,output:destination,candidate:report.candidateApplied}));
}
