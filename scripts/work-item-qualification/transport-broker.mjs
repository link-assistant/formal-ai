// DRAFT: host-only GitHub capability broker; requires explicit owned resource receipts.
import fs from 'node:fs';
import path from 'node:path';
import assert from 'node:assert/strict';
import {spawnSync} from 'node:child_process';
import {randomUUID} from 'node:crypto';
export function startBroker({directory,policy,environment=process.env,observe,execute=null,capture=null,timeoutFor=requested=>requested}){
  const seen=new Set(), root='repos/'+policy.repository, knownPulls=new Set();
  let active=true, mutations=0, requests=0;
  const run=(file,args)=>{
    const timeout=timeoutFor(45000,file);
    assert(Number.isSafeInteger(timeout)&&timeout>0&&timeout<=45000,'invalid broker command timeout');
    if(execute)return execute(file,args,{timeout});
    const value=spawnSync(file,args,{env:environment,encoding:null,timeout,maxBuffer:16*1024*1024});
    capture?.({program:file,args,status:value.status,signal:value.signal,
      error:value.error?{code:value.error.code??null,message:value.error.message}:null},
      value.stdout??Buffer.alloc(0),value.stderr??Buffer.alloc(0));
    return {status:value.status??1,stdout:value.stdout??Buffer.alloc(0),stderr:value.stderr??Buffer.from(value.error?.message??'')};
  };
  function gh(args){
    assert(Array.isArray(args)&&args.length<=80&&args.every(value=>typeof value==='string'&&value.length<=65536));
    const text=args.join(' ');assert(!text.includes('\0'));
    assert(!args.some(value=>value==='--hostname'||/^--[A-Za-z-]+=/u.test(value)),'canonical GitHub-host options required');
    assert(!args.some(value=>['--body-file','--input','--template','-t','--web','--browser','--recover',
      '--add-reviewer','--reviewer','--add-assignee','--assignee','--editor','--add-project'].includes(value)),'unowned host file/tool/recipient option refused');
    // Evaluating caller jq under GH_TOKEN would expose host env. Support only bounded dotted JSON field projection.
    let projection=null;
    const jqIndex=args.findIndex(value=>value==='--jq'||value==='-q');
    if(jqIndex>=0){projection=args[jqIndex+1];assert.match(projection,/^\.[A-Za-z_][A-Za-z0-9_]*(?:\.[A-Za-z_][A-Za-z0-9_]*)*$/u);args=[...args.slice(0,jqIndex),...args.slice(jqIndex+2)];}
    const project=result=>{
      if(result.status===0&&projection){let value=JSON.parse(result.stdout);for(const key of projection.slice(1).split('.'))value=value?.[key];
        result.stdout=Buffer.from((typeof value==='string'?value:JSON.stringify(value??null))+'\n');}
      return result;
    };
    if(args[0]==='auth'){
      assert(['status','setup-git'].includes(args[1]));
      return {status:0,stdout:Buffer.from(''),stderr:Buffer.from('Hosted transport credential preflight passed.\n')};
    }
    if(args[0]==='api'){
      if(args.includes('user')||args.includes('/user')){
        assert.equal(args.length,2,'only a literal read-only identity probe');
        return project(run('gh',args)); // Actual host identity/403 observation; no token is returned.
      }
      const endpoint=args.find(value=>value.startsWith('repos/'));
      assert(endpoint&&(endpoint===root||endpoint.startsWith(root+'/')),'owned endpoint required');
      assert(!args.includes('--hostname')&&!args.includes('--paginate'),'bounded one-page transport');
      const method=args.includes('--method')?args[args.indexOf('--method')+1]:args.includes('-X')?args[args.indexOf('-X')+1]:'GET';
      assert.equal(method,'GET','API writes must use owned high-level commands');
      assert(!args.some(value=>['-f','-F','--field','--raw-field','--input'].includes(value)),'read-only API cannot imply POST through fields');
      const relative=endpoint.slice(root.length);
      assert(!/\.{2}|%2e|%2f|%5c|\\/iu.test(relative),'canonical endpoint path required');
      const allowed=relative===''||relative==='/issues/'+policy.issueNumber
        ||relative==='/issues/'+policy.issueNumber+'/comments'
        ||/^\/pulls(?:\?|$)/u.test(relative)
        ||[...knownPulls].some(number=>new RegExp('^/(?:pulls|issues)/'+number+'(?:[/?]|$)').test(relative))
        ||relative.startsWith('/contents/')||relative.startsWith('/git/trees/'+policy.baseCommit)
        ||relative.startsWith('/commits/'+policy.baseCommit);
      assert(allowed,'history/source-answer endpoint refused');
      const rewritten=args.map(value=>value===endpoint
        ? relative.startsWith('/contents/') ? endpoint.split('?')[0]+'?ref='+policy.baseCommit
          : /^\/pulls(?:\?|$)/u.test(relative) ? root+'/pulls?state=all&base='+encodeURIComponent(policy.baseBranch)+'&per_page=100'
          : value
        : value);
      const result=run('gh',rewritten);
      if(result.status===0&&relative===''){
        const value=JSON.parse(result.stdout);value.default_branch=policy.baseBranch;
        result.stdout=Buffer.from(JSON.stringify(value));
      }
      return project(result);
    }
    assert(['issue','pr','repo'].includes(args[0]),'no search, extension, run, release or arbitrary gh command');
    const valueOptions=new Set(['--repo','-R','--json','--title','--body','--base','--head','--state','--limit','--search','--interval']);
    const flagOptions=new Set(['--draft','--comments','--patch','--watch','--required']);
    const permitted=args[0]==='issue'?new Set(['--repo','-R','--json','--comments']):args[0]==='repo'?new Set(['--repo','-R','--json'])
      :args[1]==='create'?new Set(['--repo','-R','--title','--body','--base','--head','--draft'])
      :args[1]==='edit'||args[1]==='comment'?new Set(['--repo','-R','--title','--body'])
      :new Set(['--repo','-R','--json','--base','--head','--state','--limit','--search','--comments','--patch','--watch','--required','--interval']);
    const seenOptions=new Set();
    for(let index=2;index<args.length;index++){
      const option=args[index];if(!option.startsWith('-'))continue;
      assert(permitted.has(option)&&!seenOptions.has(option),'unknown or duplicate high-level gh option');seenOptions.add(option);
      if(valueOptions.has(option)){assert(index+1<args.length);index++;}else assert(flagOptions.has(option));
    }
    assert(!(seenOptions.has('--repo')&&seenOptions.has('-R')),'single repository selector required');
    const repoIndex=Math.max(args.indexOf('--repo'),args.indexOf('-R'));
    const url=args.find(value=>value.startsWith('https://github.com/'));
    assert(repoIndex>=0&&args[repoIndex+1]===policy.repository||url?.startsWith('https://github.com/'+policy.repository+'/')
      ||args[0]==='repo'&&args.includes(policy.repository),'exact owned repository required');
    if(args[0]==='issue'){
      assert.equal(args[1],'view');
      assert(args.includes(String(policy.issueNumber))||url?.endsWith('/issues/'+policy.issueNumber));
    }else if(args[0]==='pr'){
      assert(['list','view','create','edit','comment','ready','checks','diff'].includes(args[1]));
      if(args[1]==='list')assert(args.includes('--base')&&args[args.indexOf('--base')+1]===policy.baseBranch,'PR lists must bind the task base');
      else if(args[1]==='create'){
        assert(args.includes('--base')&&args[args.indexOf('--base')+1]===policy.baseBranch);
        const head=args[args.indexOf('--head')+1];assert(new RegExp('^issue-'+policy.issueNumber+'-[A-Za-z0-9-]+$').test(head));
        assert(++mutations<=20,'finite hosted mutations');
      }else{
        const number=Number(args[2])||Number(url?.split('/').at(-1));
        assert(knownPulls.has(number),'only independently discovered owned PR');
        if(['edit','comment','ready'].includes(args[1]))assert(++mutations<=20);
      }
    }else {assert.equal(args[1],'view');assert(args.includes('--json'),'repo reads are metadata-only');}
    const result=run('gh',args);
    if(result.status===0&&args[0]==='repo'&&args[1]==='view'&&args.includes('--json')){
      const value=JSON.parse(result.stdout);
      if(value.defaultBranchRef)value.defaultBranchRef.name=policy.baseBranch;
      result.stdout=Buffer.from(JSON.stringify(value));
    }
    // Creation output identifies the candidate; corroborate its base/head before granting subsequent access.
    if(result.status===0&&args[0]==='pr'&&args[1]==='create'){
      const match=/\/pull\/(\d+)/u.exec(result.stdout.toString());assert(match);
      const number=Number(match[1]), detail=run('gh',['api',root+'/pulls/'+number]);
      assert.equal(detail.status,0);const pr=JSON.parse(detail.stdout);
      assert.equal(pr.base.ref,policy.baseBranch);assert.equal(pr.head.repo.full_name,policy.repository);
      assert(new RegExp('^issue-'+policy.issueNumber+'-').test(pr.head.ref));knownPulls.add(number);
    }
    return project(result);
  }
  const timer=setInterval(()=>{
    if(!active)return;
    const directoryStat=fs.lstatSync(directory);assert(directoryStat.isDirectory()&&!directoryStat.isSymbolicLink(),'stable RPC directory');
    for(const name of fs.readdirSync(directory)){
      if(!/^[a-f0-9-]+\.request\.json$/u.test(name)||seen.has(name))continue;seen.add(name);
      let response;
      try{
        assert(++requests<=512,'finite hosted transport request budget');
        const file=path.join(directory,name),requestFd=fs.openSync(file,fs.constants.O_RDONLY|fs.constants.O_NOFOLLOW);
        let value;try{const stat=fs.fstatSync(requestFd);assert(stat.isFile()&&stat.size<=131072);value=JSON.parse(fs.readFileSync(requestFd));}finally{fs.closeSync(requestFd);}
        observe({kind:'request',request:name,value});
        if(value.kind==='gh')response=gh(value.args);
        else{
          assert.equal(value.kind,'push-bundle');assert(++mutations<=20);
          assert(new RegExp('^issue-'+policy.issueNumber+'-[A-Za-z0-9-]+$').test(value.branch));
          assert.match(value.bundle,/^push-[a-f0-9-]+\.bundle$/u);
          const input=path.join(path.dirname(directory),'bundles',value.bundle),descriptor=fs.openSync(input,fs.constants.O_RDONLY|fs.constants.O_NOFOLLOW);
          let bytes;try{const stat=fs.fstatSync(descriptor);assert(stat.isFile()&&stat.size<=64*1024*1024);bytes=fs.readFileSync(descriptor);}finally{fs.closeSync(descriptor);}
          const trusted=path.join(policy.privateDirectory,value.bundle);fs.writeFileSync(trusted,bytes,{flag:'wx',mode:0o600});
          const ref='refs/heads/'+value.branch;
          const imported=run('git',['--git-dir',policy.privateMirror,'fetch','--no-tags',trusted,ref+':'+ref]);
          assert.equal(imported.status,0,'owned object bundle import');
          const ancestry=run('git',['--git-dir',policy.privateMirror,'merge-base','--is-ancestor',policy.baseCommit,ref]);
          assert.equal(ancestry.status,0,'task branch must retain frozen snapshot ancestry');
          response=run('git',['--git-dir',policy.privateMirror,'-c','credential.helper=',
            '-c','credential.helper=!gh auth git-credential','push','--no-verify','--no-follow-tags',
            'https://github.com/'+policy.repository+'.git',ref+':'+ref]);
          if(response.status===0){
            const mirrorUpdate=run('git',['--git-dir',policy.mirror,'fetch','--no-tags',trusted,ref+':'+ref]);
            assert.equal(mirrorUpdate.status,0,'published candidate mirror update');
          }
        }
        observe({kind:value.kind,status:response.status,request:name});
      }catch(error){const reason=error instanceof SyntaxError?'invalid request JSON':error.code==='ELOOP'?'symlink request refused':error.message;response={status:1,stdout:Buffer.alloc(0),stderr:Buffer.from('Hosted transport refused: '+reason+'\n')};observe({kind:'refusal',message:reason,request:name});}
      const destination=path.join(directory,name.replace('.request.json','.response.json'));
      const temporary=destination+'.host-'+randomUUID();
      fs.writeFileSync(temporary,JSON.stringify({status:response.status,stdout:response.stdout.toString('base64'),stderr:response.stderr.toString('base64')}),{flag:'wx',mode:0o666});
      fs.renameSync(temporary,destination);
    }
  },20);
  return {stop(){active=false;clearInterval(timer);},statistics(){return {requests,mutations};}};
}
